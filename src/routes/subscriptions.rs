//! src/routes/subscriptions.rs
use actix_web::{web, HttpResponse};
use chrono::Utc;
// No longer importing PgConnection!
use sqlx::PgPool;
use tracing::Instrument;
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
    let query_span = tracing::info_span!(
        "Saving new subscriber details in the database"
    );

    match sqlx::query!(
        r#"
        INSERT INTO subscriptions (id, email, name, subscribed_at)
        VALUES ($1, $2, $3, $4)
        "#,
        Uuid::new_v4(),
        form.email,
        form.name,
        Utc::now()
    )
    .execute(pool.as_ref())
    // First we attach the instrumentation, then we `.await` it
    .instrument(query_span)
    .await
    {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(e) => {
            // Yes, this error log falls outside of `query_span`
            // We'll rectify it later, pinky swear!
            tracing::error!("Failed to execute query: {:?}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}
