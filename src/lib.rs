use rand::Rng;

use log::{trace, info};

const EXTRA_BUFFER_TIME: u64 = 100;

pub enum Api {
    Youtube,
    Twitch,
    Bigquery,
}

pub mod bigquery;
pub mod errors;
pub mod twitch;
pub mod youtube;

/// Sleeps for the given backoff time, plus some extra buffer time, plus some random extra time.
/// backoff_time is in seconds.
/// with_extra_buffer_time is a bool that determines whether or not to add the extra buffer time.
async fn sleep_for_backoff_time(backoff_time: u64, with_extra_buffer_time: bool) {
    let extra_buffer_time = match with_extra_buffer_time {
        true => EXTRA_BUFFER_TIME,
        false => 0,
    };
    trace!(
        "sleep_for_backoff_time->backoff_time: {}, extra_buffer_time: {}",
        backoff_time,
        extra_buffer_time
    );

    //convert to milliseconds
    let backoff_time = backoff_time * 1000;

    //add some random extra time for good measure (in milliseconds)
    let random_extra = rand::thread_rng().gen_range(0..100);
    let total_millis = backoff_time + extra_buffer_time + random_extra;
    info!(
        "sleep_for_backoff_time->Sleeping for {} milliseconds",
        total_millis
    );
    tokio::time::sleep(std::time::Duration::from_millis(total_millis)).await;
}
