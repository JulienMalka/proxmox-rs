#![feature(async_await)]

use failure::{format_err, Error};
use http::Request;
use http::Response;
use hyper::service::service_fn;
use hyper::{Body, Server};
use serde_json::Value;

mod api;

async fn run_request(request: Request<Body>) -> Result<http::Response<Body>, hyper::Error> {
    match route_request(request).await {
        Ok(r) => Ok(r),
        Err(err) => Ok(Response::builder()
            .status(400)
            .body(Body::from(format!("ERROR: {}", err)))
            .expect("building an error response...")),
    }
}

async fn route_request(request: Request<Body>) -> Result<http::Response<Body>, Error> {
    let path = request.uri().path();

    let (target, params) = api::ROUTER
        .lookup(path)
        .ok_or_else(|| format_err!("missing path: {}", path))?;

    let handler = target
        .get
        .as_ref()
        .ok_or_else(|| format_err!("no GET method for: {}", path))?
        .handler();

    Ok(handler(params.unwrap_or(Value::Null)).await?)
}

type BoxFut = Box<dyn futures_01::Future<Item = Response<Body>, Error = hyper::Error> + Send>;

fn main() {
    // We expect a path, where to find our files we expose via the www/ dir:
    let mut args = std::env::args();

    // real code should have better error handling
    let _program_name = args.next();
    let www_dir = args.next().expect("expected a www/ subdirectory");
    api::set_www_dir(www_dir.to_string());

    // show our api info:
    println!(
        "{}",
        serde_json::to_string_pretty(&api::ROUTER.api_dump()).unwrap()
    );

    // Construct our SocketAddr to listen on...
    let addr = ([0, 0, 0, 0], 3000).into();

    // And a MakeService to handle each connection...
    let make_service = || {
        service_fn(|req| {
            let fut: BoxFut = Box::new(futures::compat::Compat::new(Box::pin(run_request(req))));
            fut
        })
    };

    // Then bind and serve...
    let server = {
        use futures_01::Future;
        Server::bind(&addr)
            .serve(make_service)
            .map_err(|err| eprintln!("server error: {}", err))
    };

    tokio::run(server);
}