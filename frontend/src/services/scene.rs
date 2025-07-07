use gloo_console::log;
use gloo_net::http::Request;
use web_cmn::scene::SceneResponse;
use crate::error::{FrontendError, Result};

pub async fn fetch_scenes() -> Result<Vec<SceneResponse>> {
    log!("Fetching scenes");
    let response = Request::get("/api/scenes")
        .send()
        .await?;
    Ok(response.json::<Vec<SceneResponse>>().await?)
}