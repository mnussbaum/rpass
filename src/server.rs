use std;
use std::io::prelude::Write;

use actix;
use actix_web::{http, server, App, HttpRequest, HttpResponse, Json, Result};
use gpgme::PassphraseRequest;
use gpgme;

#[derive(Debug, Serialize, Deserialize)]
struct MyObj {
    name: String,
    number: i32,
}

fn extract_item(item: Json<MyObj>) -> HttpResponse {
    HttpResponse::Ok().json(item.0) // <- send response
}

use PasswordStore;

pub struct Server {
    password_store: PasswordStore,
}

impl Server {
    pub fn new(password_store: PasswordStore) -> Server {
        return Server {
            password_store: password_store,
        };
    }

    pub fn run(&self) {
        let sys = actix::System::new("rpass");

        server::new(|| {
            App::new()
                .resource("/extractor", |r| {
                    r.method(http::Method::POST).with(extract_item).limit(4096); // <- limit size of the payload
                })
                .resource("/show/{pass_name}", |r| {
                    r.method(http::Method::GET)
                        .f(|req: HttpRequest| -> Result<String> {
                            let pass_name: std::path::PathBuf =
                                req.match_info().query("pass_name")?;
                            Ok(format!(
                                "{:?}",
                                self.password_store
                                    .decrypt(pass_name.to_str().unwrap(), fake_prompt)
                            ))
                        });
                })
        }).bind("127.0.0.1:8080")
            .unwrap()
            .shutdown_timeout(1)
            .start();

        println!("Started http server: 127.0.0.1:8080");
        let _ = sys.run();
    }
}

fn fake_prompt(_: PassphraseRequest, out: &mut Write) -> Result<(), gpgme::Error> {
    try!(out.write_all("woo".as_bytes()));
    Ok(())
}
