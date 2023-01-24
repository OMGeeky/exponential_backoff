use std::error::Error;
use std::future::Future;
use std::path::{Path, PathBuf};

use google_youtube3::api::Video;
use google_youtube3::Error::BadRequest;
use google_youtube3::hyper::{Body, Response};
use google_youtube3::hyper::client::HttpConnector;
use google_youtube3::hyper_rustls::HttpsConnector;
use google_youtube3::YouTube;

const YOUTUBE_DEFAULT_BACKOFF_TIME_S: u64 = 2;
const YOUTUBE_MAX_BACKOFF_TIME_S: u64 = 3600;

struct UploadParameters {
    video: Video,
    path: PathBuf,
    mime_type: mime::Mime
}

//TODO: implement backoff for other youtube calls

async fn generic_check_backoff_youtube<
    T,
    // Fut: Future<Output=Result<google_youtube3::Result<(Response<Body>, T)>, Box<dyn Error>>> ,
    Fut: Future<Output=Result<(Response<Body>, T), google_youtube3::Error>>,
    Para>
(
    client: &YouTube<HttpsConnector<HttpConnector>>,
    para: &Para,
    function: impl Fn(&YouTube<HttpsConnector<HttpConnector>>, &Para) -> Fut
)
    -> Result<google_youtube3::Result<(Response<Body>, T)>, Box<dyn Error>>
// where Fut: Future<Output=google_youtube3::Result<(Response<Body>, T)>>
{
    let mut backoff = 0;
    let mut res: google_youtube3::Result<(Response<Body>, T)>;
    'try_upload: loop {
        // let stream = tokio::fs::File::open(&path).await?;
        // let stream = stream.into_std().await;
        // println!("Uploading video ({}): {:?}", backoff, path.as_ref().to_str());
        //
        // res = client.videos().insert(video.clone()).upload(stream, mime_type.clone()).await;
        res = function(&client, para).await;
        match res {
            Ok(_) => break 'try_upload,
            Err(e) => {
                println!("Error: {}", e);
                if let BadRequest(e1) = &e {
                    let is_quota_error = get_is_quota_error(&e1);
                    backoff += 1;

                    println!("is_quota_error: {}", is_quota_error);
                    if is_quota_error {
                        let backoff_time = YOUTUBE_DEFAULT_BACKOFF_TIME_S.pow(backoff);
                        println!("backoff_time: {}", backoff_time);
                        if backoff_time > YOUTUBE_MAX_BACKOFF_TIME_S {
                            return Err(e.into());
                        }
                        //TODO: test this backoff
                        tokio::time::sleep(std::time::Duration::from_millis(backoff_time * 1000)).await;
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

pub async fn check_backoff_youtube_upload(client: &YouTube<HttpsConnector<HttpConnector>>,
                                          video: Video,
                                          path: impl AsRef<Path>,
                                          mime_type: mime::Mime)
                                          -> Result<google_youtube3::Result<(Response<Body>, Video)>, Box<dyn Error>>
{
    let params = UploadParameters {
        video: video.clone(),
        path: path.as_ref().into(),
        mime_type: mime_type.clone()
    };

    async fn function(client: &YouTube<HttpsConnector<HttpConnector>>, para: &UploadParameters)
                      -> Result<(Response<Body>, Video), google_youtube3::Error> {
        // let para = para.get_parameters();
        // let stream = tokio::fs::File::open(&para.path).await?;
        // let stream = stream.into_std().await;
        let stream = std::fs::File::open(&para.path)?;
        // println!("Uploading video ({}): {:?}", backoff, path.as_ref().to_str());
        client.videos().insert(para.video.clone()).upload(stream, para.mime_type.clone()).await
    }
    let res = generic_check_backoff_youtube(client, &params, function).await??;


    // let mut backoff = 0;
    // let mut res: google_youtube3::Result<(Response<Body>, Video)>;
    // 'try_upload: loop {
    //     let stream = tokio::fs::File::open(&path).await?;
    //     let stream = stream.into_std().await;
    //     println!("Uploading video ({}): {:?}", backoff, path.as_ref().to_str());
    //     res = client.videos().insert(video.clone()).upload(stream, mime_type.clone()).await;
    //     match res {
    //         Ok(_) => break 'try_upload,
    //         Err(e) => {
    //             println!("Error: {}", e);
    //             if let BadRequest(e1) = &e {
    //                 let is_quota_error = get_is_quota_error(&e1);
    //                 backoff += 1;
    //
    //                 println!("is_quota_error: {}", is_quota_error);
    //                 if is_quota_error {
    //                     let backoff_time = YOUTUBE_DEFAULT_BACKOFF_TIME_S.pow(backoff);
    //                     println!("backoff_time: {}", backoff_time);
    //                     if backoff_time > YOUTUBE_MAX_BACKOFF_TIME_S {
    //                         return Err(e.into());
    //                     }
    //                     //TODO: test this backoff
    //                     tokio::time::sleep(std::time::Duration::from_millis(backoff_time * 1000)).await;
    //                 } else {
    //                     return Err(e.into());
    //                 }
    //             } else {
    //                 return Err(e.into());
    //             }
    //         }
    //     }
    // }
    //

    let res: google_youtube3::Result<(Response<Body>, Video)> = Ok(res);
    Ok(res)
}

fn get_is_quota_error(e: &serde_json::value::Value) -> bool {
    let is_quota_error = e.get("error")
        .and_then(|e| e.get("errors"))
        .and_then(|e| e.get(0))
        .and_then(|e| e.get("reason"))
        .and_then(|e| e.as_str())
        .and_then(|e|
            if e == "quotaExceeded" {
                Some(())
            } else {
                None
            }
        ).is_some();
    is_quota_error
}
