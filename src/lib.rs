extern crate actix;
extern crate actix_web;
#[macro_use]
extern crate serde_derive;

extern crate clap;
#[macro_use]
extern crate failure;
extern crate gpgme;

use std::fs::File;
use std::io::prelude::*;
use failure::Error;
use gpgme::{Context, PassphraseProvider, PinentryMode, Protocol};

pub mod server;

#[derive(Debug)]
pub struct PasswordStore {
    directory: std::path::PathBuf,
}

impl PasswordStore {
    pub fn new(password_store_path: std::path::PathBuf) -> PasswordStore {
        return PasswordStore {
            directory: password_store_path,
        };
    }

    pub fn decrypt<P>(&self, pass_name: &str, passphrase_provider: P) -> Result<Vec<u8>, Error>
    where
        P: PassphraseProvider,
    {
        let mut gpg_ctx = Context::from_protocol(Protocol::OpenPgp).unwrap();
        gpg_ctx.set_pinentry_mode(PinentryMode::Loopback).unwrap();
        let mut output = Vec::new();

        let pass_path = self.directory.join(format!("{}.gpg", pass_name));
        if !pass_path.exists() {
            return Err(format_err!("{} is not in the password store", pass_name));
        }

        let mut input = match File::open(pass_path) {
            Ok(encrypted_pass) => encrypted_pass,
            Err(e) => {
                return Err(format_err!("Cannot open password file to decrypt: {}", e));
            }
        };

        let decrypt_result = gpg_ctx.with_passphrase_provider(passphrase_provider, |gpg_ctx| {
            gpg_ctx.decrypt(&mut input, &mut output)
        });

        match decrypt_result {
            Err(err) => Err(format_err!("Error decrypting password: {}", err)),
            Ok(_) => Ok(output),
        }
    }

    fn get_gpg_id(&self) -> Result<String, Error> {
        let mut gpg_id = String::new();
        match File::open(self.directory.join(".gpg-id")) {
            Ok(mut gpg_id_file) => gpg_id_file.read_to_string(&mut gpg_id).unwrap(),
            Err(e) => {
                return Err(format_err!("Cannot open .gpg-id file to decrypt: {}", e));
            }
        };

        Ok(gpg_id)
    }

    pub fn encrypt<P>(
        &self,
        pass_name: &str,
        pass: &str,
        passphrase_provider: P,
    ) -> Result<(), Error>
    where
        P: PassphraseProvider,
    {
        let pass_path = self.directory.join(format!("{}.gpg", pass_name));

        // Return error unless an override arg is set
        // if !pass_path.exists() {
        //     return Err(format_err!("{} is not in the password store", pass_name));
        // }

        let mut gpg_ctx = Context::from_protocol(Protocol::OpenPgp).unwrap();
        gpg_ctx.set_pinentry_mode(PinentryMode::Loopback).unwrap();
        gpg_ctx.set_armor(true);

        let recipients = match self.get_gpg_id() {
            Err(e) => return Err(format_err!("No GPG ID configured for encryption: {}", e)),
            Ok(gpg_id) => [gpg_id],
        };

        let keys: Vec<_> = gpg_ctx
            .find_keys(recipients.iter())
            .unwrap()
            .filter_map(Result::ok)
            .filter(|k| k.can_encrypt())
            .collect();

        let mut encrypted_pass = Vec::new();
        let encrypt_result = gpg_ctx.with_passphrase_provider(passphrase_provider, |gpg_ctx| {
            gpg_ctx.encrypt(&keys, pass.as_bytes(), &mut encrypted_pass)
        });

        match encrypt_result {
            Err(e) => return Err(format_err!("Error encrypting password: {}", e)),
            Ok(_) => {}
        };

        let write_result = match File::create(pass_path) {
            Ok(mut pass_file) => pass_file.write_all(encrypted_pass.as_slice()),
            Err(e) => {
                return Err(format_err!(
                    "Cannot open password file to for writing: {}",
                    e
                ));
            }
        };

        match write_result {
            Ok(_) => Ok(()),
            Err(e) => Err(format_err!("Couldn't write password to file: {}", e)),
        }
    }
}
