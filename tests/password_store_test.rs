extern crate gpgme;
extern crate rpass;

use rpass::PasswordStore;
use std::io::prelude::Write;
use std::path::PathBuf;
use gpgme::PassphraseRequest;

fn fake_prompt(_: PassphraseRequest, out: &mut Write) -> Result<(), gpgme::Error> {
    try!(out.write_all("woo".as_bytes()));
    Ok(())
}

#[test]
fn decrypt_returns_a_password() {
    let store = PasswordStore::new(PathBuf::from("tests/fixtures/password_store"));
    let pass_bytes = store.decrypt("mywebsite.com", fake_prompt).unwrap();
    let pass = String::from_utf8_lossy(&pass_bytes);
    assert_eq!(pass, "wooo\n");
}

#[test]
fn encrypt_writes_a_password() {
    let store = PasswordStore::new(PathBuf::from("tests/fixtures/password_store"));
    store.encrypt("test", "hiya", fake_prompt).unwrap();
    let pass_bytes = store.decrypt("test", fake_prompt).unwrap();
    let pass = String::from_utf8_lossy(&pass_bytes);
    assert_eq!(pass, "hiya");
}
