use crate::config::Config;
use std::net::SocketAddr;
use tokio::runtime::Builder;
use warp::{http::Response, Filter};

pub fn run(config: &Config) {
    let webserver = config.svd.webserver.clone();
    Builder::new()
        .threaded_scheduler()
        .enable_all()
        .build()
        .unwrap()
        .spawn(async move {
            println!("Spawning the SVD viewer webserver on http://{}", webserver);
            let index = warp::get().and(warp::path::end()).map(|| {
                Response::builder()
                    .header("Content-Type", "text/html")
                    .body(include_str!("../../svd_viewer/static/index.html"))
            });
            let css = warp::path!("css" / "main.css")
                .and(warp::path::end())
                .map(|| {
                    Response::builder()
                        .header("Content-Type", "text/css")
                        .body(include_str!("../../svd_viewer/static/css/main.css"))
                });
            let bootstrap_css = warp::path!("bootstrap.min.css")
                .and(warp::path::end())
                .map(|| {
                    Response::builder()
                        .header("Content-Type", "text/css")
                        .body(include_str!("../../svd_viewer/static/bootstrap.min.css"))
                });
            let js = warp::path!("wasm.js").and(warp::path::end()).map(|| {
                Response::builder()
                    .header("Content-Type", "text/javascript")
                    .body(include_str!("../../svd_viewer/static/wasm.js"))
            });
            let wasm = warp::path!("wasm_bg.wasm").and(warp::path::end()).map(|| {
                Response::builder()
                    .header("Content-Type", "application/wasm")
                    .body(&include_bytes!("../../svd_viewer/static/wasm_bg.wasm")[..])
            });
            let address: SocketAddr = webserver.parse().expect(
                "The given webserver URL is invalid. Please specify one in the format ip:port",
            );
            warp::serve(index.or(css).or(bootstrap_css).or(js).or(wasm))
                .run(address)
                .await;
        });
}
