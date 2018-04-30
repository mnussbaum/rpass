use std;

use actix;
use actix_web::{http, server, App, HttpRequest, HttpResponse, Json, Result};

#[derive(Debug, Serialize, Deserialize)]
struct MyObj {
    name: String,
    number: i32,
}

fn extract_item(item: Json<MyObj>) -> HttpResponse {
    HttpResponse::Ok().json(item.0) // <- send response
}

fn show(req: HttpRequest) -> Result<String> {
    let path: std::path::PathBuf = req.match_info().query("pass_name")?;
    Ok(format!("{:?}", path))
}

use PasswordStore;

pub fn run_server(_: PasswordStore) {
    let sys = actix::System::new("rpass");

    server::new(|| {
        App::new()
            .resource("/extractor", |r| {
                r.method(http::Method::POST).with(extract_item).limit(4096); // <- limit size of the payload
            })
            .resource("/show/{pass_name}", |r| {
                r.method(http::Method::GET).f(show);
            })
    }).bind("127.0.0.1:8080")
        .unwrap()
        .shutdown_timeout(1)
        .start();

    println!("Started http server: 127.0.0.1:8080");
    let _ = sys.run();
}
