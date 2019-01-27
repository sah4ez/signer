extern crate clap;
extern crate crypto;
extern crate rand;

use rand::{Rng};
use clap::{App, Arg, SubCommand};
use crypto::curve25519::{curve25519, curve25519_base};
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
        ).subcommand(
        SubCommand::with_name("secret").
                about("create shared secret")
            .arg( Arg::with_name("pub").default_value("pub.bin").index(1))
            .arg( Arg::with_name("priv").default_value("priv.bin").index(2))
            .arg( Arg::with_name("out").default_value("secret.bin").index(3))
        ,)
        .get_matches();



    match signer_matcher.subcommand_name() {
        Some("generate") => {
            let mut name = "";
            if let Some(signer_matcher) = signer_matcher.subcommand_matches("generate") {
                name = signer_matcher.value_of("name").unwrap();
            }
            let mut private = rand::thread_rng().gen::<[u8; 32]>();

            private[0] &= 248;
            private[31] &= 127;
            private[31] |= 64;

            let public = curve25519_base(private.as_ref());

            let mut file = File::create([name,"pub.bin"].join("").as_mut_str()).unwrap();
            file.write_all(public.as_ref()).unwrap();

            let mut file = File::create([name,"priv.bin"].join("").as_mut_str()).unwrap();
            file.write_all(private.as_ref()).unwrap();

        },
        Some("secret") => {
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
            let mut private = [0; 32];
            f.read(private.as_mut()).unwrap();

            let secret = curve25519(private.as_ref(), public.as_ref());

            let mut file = File::create([secret_name, "secret.bin"].join("").as_mut_str()).unwrap();
            file.write_all(secret.as_ref()).unwrap();
        },
        None => println!("nope..."),
        _ => println!("others..."),
    }
}

