use std::error::Error;
use std::future::Future;

use crate::prelude::*;

use google_youtube3::Error::BadRequest;
use google_youtube3::hyper::{Body, Response};
use google_youtube3::hyper::client::HttpConnector;
use google_youtube3::hyper_rustls::HttpsConnector;
use google_youtube3::YouTube;

use crate::sleep_for_backoff_time;

/// the base number for the backoff
///
/// gets used as base^n where n is the amount of backoffs
const YOUTUBE_DEFAULT_BACKOFF_TIME_S: u64 = 2;
/// the max amount a single backoff can be
const YOUTUBE_MAX_BACKOFF_TIME_S: u64 = 3600;
/// the max amount of backoffs that can be done
///
/// after this amount of backoffs, the method will return Err()
const YOUTUBE_MAX_TRIES: u64 = 50;//should result in ~39 hours of maximum backoff time


pub async fn generic_check_backoff_youtube<'a, 'b, 'c,
    T,
    Para,
    Fut: Future<Output=Result<(Response<Body>, T), google_youtube3::Error>>,
>
(
    client: &'a YouTube<HttpsConnector<HttpConnector>>,
    para: &'b Para,
    function: impl Fn(&'a YouTube<HttpsConnector<HttpConnector>>, &'b Para) -> Fut,
)
    -> Result<google_youtube3::Result<(Response<Body>, T)>, Box<dyn Error>>
{
    trace!("generic_check_backoff_youtube");
    let mut backoff = 0;
    let mut res: google_youtube3::Result<(Response<Body>, T)>;
    'try_upload: loop {
        trace!("generic_check_backoff_youtube loop ({})", backoff);
        res = function(&client, para).await;
        match res {
            Ok(_) => break 'try_upload,
            Err(e) => {
                warn!("Error: {}", e);
                if let BadRequest(e1) = &e {
                    let is_quota_error = get_is_quota_error(&e1);

                    if is_quota_error {
                        info!("quota_error: {}", e);
                        backoff += 1;
                        if !wait_for_backoff(backoff).await {
                            return Err(e.into());
                        }
                    } else {
                        return Err(e.into());
                    }
                } else {
                    return Err(e.into());
                }
            }
        }
    }

    let res: google_youtube3::Result<(Response<Body>, T)> = res;
    Ok(res)
}

async fn wait_for_backoff<'a>(backoff: u32) -> bool {
    trace!("wait_for_backoff ({})", backoff);
    let mut backoff_time = YOUTUBE_DEFAULT_BACKOFF_TIME_S.pow(backoff);
    info!("backoff_time: {}", backoff_time);
    if backoff as u64 > YOUTUBE_MAX_TRIES {
        return false;
    }
    if backoff_time > YOUTUBE_MAX_BACKOFF_TIME_S {
        backoff_time = YOUTUBE_MAX_BACKOFF_TIME_S;
    }
    sleep_for_backoff_time(backoff_time, false).await;
    true
}

// 
// pub async fn check_backoff_youtube_upload(client: &YouTube<HttpsConnector<HttpConnector>>,
//                                           video: Video,
//                                           path: impl AsRef<Path>,
//                                           mime_type: mime::Mime)
//                                           -> Result<google_youtube3::Result<(Response<Body>, Video)>, Box<dyn Error>>
// {
//     struct UploadParameters {
//         video: Video,
//         path: PathBuf,
//         mime_type: mime::Mime
//     }
//
//     let params = UploadParameters {
//         video: video.clone(),
//         path: path.as_ref().into(),
//         mime_type: mime_type.clone()
//     };
//
//     async fn function(client: &YouTube<HttpsConnector<HttpConnector>>, para: &UploadParameters)
//                       -> Result<(Response<Body>, Video), google_youtube3::Error> {
//         let stream = tokio::fs::File::open(&para.path).await?;
//         let stream = stream.into_std().await;
//         client.videos().insert(para.video.clone()).upload(stream, para.mime_type.clone()).await
//     }
//     let res = generic_check_backoff_youtube::<Video, UploadParameters, _>
//         (client, &params, function).await??;
//
//     let res: google_youtube3::Result<(Response<Body>, Video)> = Ok(res);
//     Ok(res)
// }

fn get_is_quota_error(e: &serde_json::value::Value) -> bool {
    trace!("get_is_quota_error");
    let is_quota_error = e.get("error")
        .and_then(|e| e.get("errors"))
        .and_then(|e| e.get(0))
        .and_then(|e| e.get("reason"))
        .and_then(|e| e.as_str())
        .and_then(|e|
            if e == "quotaExceeded" {
                Some(())
            } else if e == "uploadLimitExceeded" {
                Some(())
            } else {
                None
            }
        ).is_some();
    is_quota_error
}
