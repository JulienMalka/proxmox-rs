#![feature(async_await)]

use std::io;
use std::path::Path;

use failure::{bail, format_err, Error};
use http::Request;
use http::Response;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Server};
use serde_json::Value;
use tokio::io::AsyncReadExt;

use proxmox::api::{api, router};

//
// Configuration:
//

static mut WWW_DIR: Option<String> = None;

pub fn www_dir() -> &'static str {
    unsafe {
        WWW_DIR
            .as_ref()
            .expect("expected WWW_DIR to be initialized")
            .as_str()
    }
}

pub fn set_www_dir(dir: String) {
    unsafe {
        assert!(WWW_DIR.is_none(), "WWW_DIR must only be initialized once!");

        WWW_DIR = Some(dir);
    }
}

//
// API methods
//

router! {
    pub static ROUTER: Router<Body> = {
        GET: hello,
        /www/{path}*: { GET: get_www },
        /api/1: {
            // fill with more stuff
        }
    };
}

#[api({
    description: "Hello API call",
})]
async fn hello() -> Result<Response<Body>, Error> {
    Ok(http::Response::builder()
        .status(200)
        .header("content-type", "text/html")
        .body(Body::from("Hello"))?)
}

#[api({
    description: "Get a file from the www/ subdirectory.",
    parameters: {
        path: "Path to the file to fetch",
    },
})]
async fn get_www(path: String) -> Result<Response<Body>, Error> {
    if path.contains("..") {
        bail!("illegal path");
    }

    // FIXME: Add support for an ApiError type for 404s etc. to reduce error handling code size:
    let mut file = match tokio::fs::File::open(format!("{}/{}", www_dir(), path)).await {
        Ok(file) => file,
        Err(ref err) if err.kind() == io::ErrorKind::NotFound => {
            return Ok(http::Response::builder()
                .status(404)
                .body(Body::from(format!("No such file or directory: {}", path)))?);
        }
        Err(e) => return Err(e.into()),
    };

    let mut data = Vec::new();
    file.read_to_end(&mut data).await?;

    let mut response = http::Response::builder();
    response.status(200);

    let content_type = match Path::new(&path).extension().and_then(|e| e.to_str()) {
        Some("html") => Some("text/html"),
        Some("css") => Some("text/css"),
        Some("js") => Some("application/javascript"),
        Some("txt") => Some("text/plain"),
        // ...
        _ => None,
    };
    if let Some(content_type) = content_type {
        response.header("content-type", content_type);
    }

    Ok(response.body(Body::from(data))?)
}

//
// Hyper glue
//

async fn route_request(request: Request<Body>) -> Result<http::Response<Body>, Error> {
    let path = request.uri().path();

    let (target, params) = ROUTER
        .lookup(path)
        .ok_or_else(|| format_err!("missing path: {}", path))?;

    use hyper::Method;
    let method = match *request.method() {
        Method::GET => target.get.as_ref(),
        Method::PUT => target.put.as_ref(),
        Method::POST => target.post.as_ref(),
        Method::DELETE => target.delete.as_ref(),
        _ => bail!("unexpected method type"),
    };

    method
        .ok_or_else(|| format_err!("no GET method for: {}", path))?
        .call(params.unwrap_or(Value::Null))
        .await
}

async fn service_func(request: Request<Body>) -> Result<http::Response<Body>, hyper::Error> {
    match route_request(request).await {
        Ok(r) => Ok(r),
        Err(err) => Ok(Response::builder()
            .status(400)
            .body(Body::from(format!("ERROR: {}", err)))
            .expect("building an error response...")),
    }
}

//
// Main entry point
//
async fn main_do(www_dir: String) {
    // Construct our SocketAddr to listen on...
    let addr = ([0, 0, 0, 0], 3000).into();

    // And a MakeService to handle each connection...
    let service = make_service_fn(|_| async { Ok::<_, hyper::Error>(service_fn(service_func)) });

    // Then bind and serve...
    let server = Server::bind(&addr).serve(service);

    println!("Serving {} under http://localhost:3000/www/", www_dir);

    if let Err(e) = server.await {
        eprintln!("server error: {}", e);
    }
}

fn main() {
    // We expect a path, where to find our files we expose via the www/ dir:
    let mut args = std::env::args();

    // real code should have better error handling
    let _program_name = args.next();
    let www_dir = args.next().expect("expected a www/ subdirectory");
    set_www_dir(www_dir.to_string());

    // show our api info:
    println!(
        "{}",
        serde_json::to_string_pretty(&ROUTER.api_dump()).unwrap()
    );

    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(main_do(www_dir));
}