extern crate base64;
extern crate clap;
extern crate crypto;
extern crate rand;

use base64::encode;
use clap::{App, Arg, SubCommand};
use crypto::ed25519;
use rand::Rng;
use std::fs::File;
use std::io::prelude::*;

//use untrusted;

fn main() {
    let signer_matcher = App::new("signer")
        .version("1.0")
        .author("Alexandr Kozlenkov <sah4ez32@gmail.com>")
        .about("Simple signer document via ECDH")
        .subcommand(
            SubCommand::with_name("generate")
                .about("generate keys pair")
                .arg(
                    Arg::with_name("name")
                        .help("name of pair keys")
                        .index(1)
                        .required(true),
                ),
        )
        .subcommand(
            SubCommand::with_name("secret")
                .about("create shared secret")
                .arg(Arg::with_name("pub").default_value("pub.bin").index(1))
                .arg(Arg::with_name("priv").default_value("priv.bin").index(2))
                .arg(Arg::with_name("out").default_value("secret.bin").index(3)),
        )
        .subcommand(
            SubCommand::with_name("sign")
                .arg(
                    Arg::with_name("priv")
                        .default_value("priv.bin")
                        .index(1)
                        .required(true),
                )
                .arg(Arg::with_name("message").index(2).required(true)),
        )
        .subcommand(
            SubCommand::with_name("verify")
                .arg(
                    Arg::with_name("pub")
                        .default_value("pub.bin")
                        .index(1)
                        .required(true),
                )
                .arg(Arg::with_name("message").index(2).required(true))
                .arg(
                    Arg::with_name("signature")
                        .index(3)
                        .default_value("signature.sha512")
                        .required(true),
                ),
        )
        .get_matches();

    match signer_matcher.subcommand_name() {
        Some("generate") => generate(signer_matcher),
        Some("secret") => secret(signer_matcher),
        Some("sign") => sign(signer_matcher),
        Some("verify") => verify(signer_matcher),
        None => println!("nope..."),
        _ => println!("others..."),
    }
}

fn generate(signer_matcher: clap::ArgMatches) {
    let mut name = "";
    if let Some(signer_matcher) = signer_matcher.subcommand_matches("generate") {
        name = signer_matcher.value_of("name").unwrap();
    }
    let seed = rand::thread_rng().gen::<[u8; 32]>();

    let pair = ed25519::keypair(seed.as_ref());

    let mut file = File::create([name, "pub.bin"].join("").as_mut_str()).unwrap();
    file.write_all(pair.1.as_ref()).unwrap();

    let mut file = File::create([name, "priv.bin"].join("").as_mut_str()).unwrap();
    file.write_all(pair.0.as_ref()).unwrap();
}

fn secret(signer_matcher: clap::ArgMatches) {
    let mut public_name = "";
    let mut private_name = "";
    let mut secret_name = "";

    if let Some(signer_matcher) = signer_matcher.subcommand_matches("secret") {
        public_name = signer_matcher.value_of("pub").unwrap();
        private_name = signer_matcher.value_of("priv").unwrap();
        secret_name = signer_matcher.value_of("out").unwrap();
    }

    let mut f = File::open(public_name).unwrap();
    let mut public = [0; 32];
    f.read(public.as_mut()).unwrap();

    let mut f = File::open(private_name).unwrap();
    let mut private = [0; 64];
    f.read(private.as_mut()).unwrap();

    let secret = ed25519::exchange(public.as_ref(), private.as_ref());

    let mut file = File::create([secret_name, "secret.bin"].join("").as_mut_str()).unwrap();
    file.write_all(secret.as_ref()).unwrap();
}

fn sign(signer_matcher: clap::ArgMatches) {
    let mut private_name = "";
    let mut msg = "";

    if let Some(signer_matcher) = signer_matcher.subcommand_matches("sign") {
        private_name = signer_matcher.value_of("priv").unwrap();
        msg = signer_matcher.value_of("message").unwrap();
    }
    let mut f = File::open(private_name).unwrap();
    let mut private = [0; 64];
    f.read(private.as_mut()).unwrap();

    //let msg = signer_matcher.value_of("from-message").unwrap();
    //if signer_matcher.is_present("from-file") {
    //    let mut f = File::open(signer_matcher.value_of("from-file").unwrap()).unwrap();
    //    f.read_to_string(&mut String::from(msg)).unwrap();
    //}

    let signature = ed25519::signature(msg.as_ref(), private.as_ref());

    let mut file = File::create("signature.sha512").unwrap();
    file.write_all(signature.as_ref()).unwrap();
    println!("msg:{}\nsignature:{}", msg, encode(signature.as_ref()));
}

fn verify(signer_matcher: clap::ArgMatches) {
    let mut public_name = "";
    let mut msg = "";
    let mut signature_path = "";

    if let Some(signer_matcher) = signer_matcher.subcommand_matches("verify") {
        public_name = signer_matcher.value_of("pub").unwrap();
        msg = signer_matcher.value_of("message").unwrap();
        signature_path = signer_matcher.value_of("signature").unwrap();
    }
    let mut f = File::open(public_name).unwrap();
    let mut public = [0; 32];
    f.read(public.as_mut()).unwrap();

    let mut f = File::open(signature_path).unwrap();
    let mut signature = [0; 64];
    f.read(signature.as_mut()).unwrap();

    //let msg = signer_matcher.value_of("from-message").unwrap();
    //if signer_matcher.is_present("from-file") {
    //    let mut f = File::open(signer_matcher.value_of("from-file").unwrap()).unwrap();
    //    f.read_to_string(&mut String::from(msg)).unwrap();
    //}

    let valid = ed25519::verify(msg.as_ref(), public.as_ref(), signature.as_ref());

    println!("msg:{}\nvalid:{}", msg, valid);
}
