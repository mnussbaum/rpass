extern crate clap;
#[macro_use]
extern crate failure;
extern crate gpgme;
extern crate rpass;
extern crate rpassword;

use std::env;
use std::io;
use std::io::prelude::*;
use std::process::exit;
use clap::{App, Arg, ArgMatches, SubCommand};
use failure::Error;
use gpgme::PassphraseRequest;
use rpass::PasswordStore;
use rpass::server::run_server;

fn main() {
    let matches = App::new("rpass")
        .version("1.0")
        .author("mnussbaum <michaelnussbaum08@gmail.com")
        .about("A Unix password manager")
        .subcommand(
            SubCommand::with_name("show").about("Show a password").arg(
                Arg::with_name("pass-name")
                    .help("Name of password to show")
                    .required(true),
            ),
        )
        .subcommand(
            SubCommand::with_name("insert")
                .about("Create a password")
                .arg(
                    Arg::with_name("pass-name")
                        .help("Name of password to create")
                        .required(true),
                ),
        )
        .subcommand(
            SubCommand::with_name("server").about("Run a server to back browser extensions"),
        )
        .get_matches();

    match execute_subcommand(matches) {
        Err(err) => {
            eprintln!("{}", err.cause());
            exit(1);
        }
        Ok(_) => {}
    }
}

fn execute_subcommand(matches: ArgMatches) -> Result<(), Error> {
    match matches.subcommand() {
        ("show", Some(m)) => {
            let pass_name = m.value_of("pass-name").unwrap();
            let password_store_dir = try!(password_store_directory());
            match PasswordStore::new(password_store_dir).decrypt(pass_name, cli_prompt) {
                Err(e) => Err(e),
                Ok(pass) => {
                    io::stdout().write_all(&pass).unwrap();
                    Ok(())
                }
            }
        }

        ("insert", Some(m)) => {
            let pass_name = m.value_of("pass-name").unwrap();
            let pass =
                rpassword::prompt_password_stdout(&format!("Enter a password for {}: ", pass_name))
                    .unwrap();
            let password_store_dir = try!(password_store_directory());
            match PasswordStore::new(password_store_dir).encrypt(pass_name, &pass, cli_prompt) {
                Err(e) => Err(e),
                Ok(_) => Ok(()),
            }
        }

        ("server", Some(_)) => {
            let password_store_dir = try!(password_store_directory());
            run_server(PasswordStore::new(password_store_dir));
            Ok(())
        }

        _ => Err(format_err!("Unknown subcommand")),
    }
}

fn cli_prompt(_: PassphraseRequest, out: &mut Write) -> Result<(), gpgme::Error> {
    try!(
        out.write_all(
            rpassword::prompt_password_stdout("Password: ")
                .unwrap()
                .as_bytes(),
        )
    );
    Ok(())
}

fn password_store_directory() -> Result<std::path::PathBuf, Error> {
    match env::home_dir() {
        Some(home_dir) => Ok(home_dir.join(".password-store")),
        None => return Err(format_err!("Cannot find password store directory")),
    }
}
