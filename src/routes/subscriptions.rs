//! src/routes/subscriptions.rs
use crate::domain::{NewSubscriber, SubscriberEmail, SubscriberName};

use actix_web::{web, HttpResponse};
use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(serde::Deserialize)]
pub struct FormData {
    email: String,
    name: String,
}

pub fn parse_subscriber(form: FormData) -> Result<NewSubscriber, String> {
    let name = SubscriberName::parse(form.0.name)?;
    let email = SubscriberEmail::parse(form.0.email)?;
    Ok(NewSubscriber { email, name })
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
        subscriber_email = %form.email,
        subscriber_name= %form.name
    )
)]
pub async fn subscribe(
    form: web::Form<FormData>,
    pool: web::Data<PgPool>
) -> HttpResponse {
    let new_subscriber = match parse_subscriber(form.0) {
        Ok(subscriber) => subscriber,
        Err(_) => return HttpResponse::BadRequest().finish(),
    };
    match insert_subscriber(&pool, &new_subscriber).await {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

// `insert_subscriber` takes care of the
// database logic and it has no awareness of
// the surrounding web framework - i.e.
// we are not passing `web::Form` or `web::Data` wrappers as input types
#[tracing::instrument(
    name = "Saving new subscriber details in the database",
    skip(new_subscriber, pool)
)]
pub async fn insert_subscriber(
    pool: &PgPool,
    new_subscriber: &NewSubscriber,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
    INSERT INTO subscriptions (id, email, name, subscribed_at)
    VALUES ($1, $2, $3, $4)
        "#,
        Uuid::new_v4(),
        // Using `as_ref` now!
        new_subscriber.email.as_ref(),
        new_subscriber.name.as_ref(),
        Utc::now()
    )
    .execute(pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        e
    })?;
    Ok(())
}
