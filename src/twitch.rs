use std::error::Error;

use chrono::NaiveDateTime;
use reqwest::{Body, Client, IntoUrl, Request, Response};
use reqwest::header::HeaderMap;

use log::{info, trace};

use crate::errors::BackoffError;
use crate::sleep_for_backoff_time;

//region reqwest
//region convenience functions

//region GET

pub async fn check_backoff_twitch_get<T: IntoUrl>(url: T) -> Result<Response, Box<dyn Error>> {
    check_backoff_twitch(Request::new(reqwest::Method::GET, url.into_url()?)).await
}

pub async fn check_backoff_twitch_get_with_client<T: IntoUrl>(
    url: T,
    client: &Client,
) -> Result<Response, Box<dyn Error>> {
    check_backoff_twitch_with_client(Request::new(reqwest::Method::GET, url.into_url()?), client)
        .await
}

//endregion

//region POST

pub async fn check_backoff_twitch_post<T: IntoUrl, B: Into<Body>>(
    url: T,
    headers: Option<HeaderMap>,
    body: Option<B>,
) -> Result<Response, Box<dyn Error>> {
    let client = Client::new();
    check_backoff_twitch_post_with_client(url, headers, body, &client).await
}

pub async fn check_backoff_twitch_post_with_client<T: IntoUrl, B: Into<Body>>(
    url: T,
    headers: Option<HeaderMap>,
    body: Option<B>,
    client: &Client,
) -> Result<Response, Box<dyn Error>> {
    let url = url.into_url()?;
    trace!("check_backoff_twitch_post_with_client {:?}", url);
    let mut request = client.post(url);

    if let Some(headers) = headers {
        request = request.headers(headers);
    }
    if let Some(body) = body {
        request = request.body(body);
    }

    let request = request.build()?;
    check_backoff_twitch_with_client(request, client).await
}
//endregion

pub async fn check_backoff_twitch(request: Request) -> Result<Response, Box<dyn Error>> {
    let client = Client::new();
    check_backoff_twitch_with_client(request, &client).await
}

//endregion

pub async fn check_backoff_twitch_with_client(
    request: Request,
    client: &Client,
) -> Result<Response, Box<dyn Error>> {
    trace!("check_backoff_twitch_with_client {:?}", request);
    let mut counter = 0;
    loop {
        trace!("check_backoff_twitch_with_client loop ({})", counter);
        let r: Request = request
            .try_clone()
            .ok_or::<BackoffError>("Request is None".into())?;
        // Some(v) => Ok(v),
        // None => Err("Request is None".into()),
        // }?;
        let response = client.execute(r).await?;

        let status_code = response.status();
        match status_code.as_u16() {
            200 => return Ok(response),
            429 => {
                let x = &request
                    .headers()
                    .get("Ratelimit-Reset")
                    .ok_or(BackoffError::new("No rate limit reset given"))?;
                let value: String = x.to_str()?.to_string();
                handle_e429(value).await?;
            }

            _ => return Ok(response),
            // _ => todo!("Handle other errors or "),
        }

        counter += 1;
    }
}

async fn handle_e429(value: String) -> Result<(), Box<dyn Error>> {
    trace!("handle_e429 {}", value);
    let value = value.parse::<i64>()?;
    let timestamp = NaiveDateTime::from_timestamp_opt(value, 0).ok_or(BackoffError::new(
        format!("Could not convert the provided timestamp: {}", value),
    ))?;
    let now = chrono::Local::now().naive_local();
    if timestamp < now {
        sleep_for_backoff_time(1, true).await;
        return Ok(());
    }
    let duration = timestamp - now;
    let duration = duration.num_seconds() as u64;
    info!("Sleeping for {} seconds", duration);
    sleep_for_backoff_time(duration, true).await;
    //TODO: test this somehow
    Ok(())
}

//endregion
