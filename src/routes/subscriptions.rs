//! src/routes/subscriptions.rs
use actix_web::{web, HttpResponse};
use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;

#[allow(dead_code)]
#[derive(serde::Deserialize)]
pub struct FormData {
    email: String,
    name: String,
}

// `#[tracing::instrument]` creates a span
// at the beginning of the function invocation and
// automatically attaches all arguments
// passed to the function to the context of the span - in our case,
// `form` and `pool`.
// Often function arguments won’t be displayable on log records
// (e.g. `pool`) or we’d like to specify more explicitly
// what should/how they should be captured
// (e.g. naming each field of `form`) -
// we can explicitly tell `tracing` to ignore them using the `skip` directive.
// subscribe orchestrates the work to be done
// by calling the required routines and translates
// their outcome into the proper response
// according to the rules and conventions of the HTTP protocol.
#[tracing::instrument(
    // name can be used to specify
    // the message associated to the function span
    // - if omitted, it defaults to the function name.
    name = "Adding a new subscriber",
    skip(form, pool),
    fields(
        request_id = %Uuid::new_v4(),
        subscriber_email = %form.email,
        subscriber_name= %form.name
    )
)]
pub async fn subscribe(
    form: web::Form<FormData>,
    pool: web::Data<PgPool>,
) -> HttpResponse {
    match insert_subscriber(&pool, &form).await
    {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(_) => HttpResponse::InternalServerError().finish()
    }
}

// `insert_subscriber` takes care of the
// database logic and it has no awareness of
// the surrounding web framework - i.e.
// we are not passing `web::Form` or `web::Data` wrappers as input types
#[tracing::instrument(
name = "Saving new subscriber details in the database",
skip(form, pool)
)]
pub async fn insert_subscriber(
    pool: &PgPool,
    form: &FormData,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
    INSERT INTO subscriptions (id, email, name, subscribed_at)
    VALUES ($1, $2, $3, $4)
            "#,
        Uuid::new_v4(),
        form.email,
        form.name,
        Utc::now()
    )
    .execute(pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        e
    // Using the `?` operator to return early
    // if the function failed, returning a sqlx::Error
    // We will talk about error handling in depth later!
    })?;
    Ok(())
}
