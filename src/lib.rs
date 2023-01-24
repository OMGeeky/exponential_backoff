#![feature(async_closure)]
use rand::Rng;

const EXTRA_BUFFER_TIME: u64 = 100;

pub enum Api {
    Youtube,
    Twitch,
    Bigquery,
}

pub mod errors;
pub mod youtube;
pub mod twitch;
pub mod bigquery;


async fn sleep_for_backoff_time(backoff_time: u64, with_extra_buffer_time: bool) {
    let extra_buffer_time = match with_extra_buffer_time {
        true => EXTRA_BUFFER_TIME,
        false => 0
    };
    let backoff_time = backoff_time * 1000 as u64;

    // let random_extra = rand::thread_rng().gen_range(0..100);
    let random_extra = rand::thread_rng().gen_range(0..100);
    tokio::time::sleep(std::time::Duration::from_millis(backoff_time + extra_buffer_time + random_extra)).await;
}
