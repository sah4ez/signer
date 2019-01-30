#![feature(proc_macro_hygiene, decl_macro, try_from)]

#[macro_use] extern crate rocket;
extern crate rocket_contrib;
#[macro_use] extern crate serde_derive;
extern crate serde;
extern crate mongodb;
extern crate hex;


use rocket::State;
use rocket::response::{status};
use base64::{decode, encode};
use hex::{decode as hdecode};
use rocket_contrib::json::Json;
use mongodb::{Bson, bson, doc};
use mongodb::{Client, ThreadedClient};
use mongodb::db::ThreadedDatabase;
use mongodb::coll::{Collection, options::IndexOptions as Opt};
use mongodb::spec::BinarySubtype;
use std::prelude::v1::Vec;

#[derive(Serialize, Deserialize)]
struct Key {
    key: String,
}

#[post("/<key>", format = "application/json", data = "<value>")]
fn add(key: String, value: Json<Key>, current_store: State<Collection>)
    -> Result<status::Accepted<String>, status::BadRequest<String>> {
    let b = match decode(&value.0.key) {
        Ok(bb) => bb.to_vec(),
        _ => [0; 32].to_vec(),
    };

    if b != [0;32].to_vec() && b.len() == 32 {
        let k = Bson::Binary(BinarySubtype::Generic, Vec::from(b.as_slice()));
        match current_store.insert_one(doc!{"key":key, "value": k }, None){
            Ok(v) => {
                match v.write_exception {
                    Some(v) => {
                        Err(status::BadRequest(Some(v.write_error.unwrap().message)))
                    },
                    None => {
                        Ok(status::Accepted(Some("".to_string())))
                    },
                }
            },
            Err(e) => {
                Err(status::BadRequest(Some(e.to_string())))
            },
        }
    } else {
        Err(status::BadRequest(Some(String::from("invalid length"))))
    }
}

#[get("/<key>")]
fn get(key: String, current_store: State<Collection>) -> String {
    let value = match current_store.find_one(Some(doc!{"key": key}), None).
        expect("not found") {
        Some(value) => value,
            _ => doc!{},
    };


    match value.get("value") {
        Some(f) => {
            encode( hdecode(
                    String::from(f.to_string())
                        .as_str()[13..77]
                        .as_bytes()
            ).unwrap().as_slice())
        },
        None => String::from("not found"),
    }.parse().unwrap()
}

fn main() {
    let client = Client::with_uri("mongodb://localhost:27017")
        .expect("Failed to initialize standalone client.");

    let coll = client.db("pspk").collection("keys");

    coll.create_index(doc!{
        "key" => "text",
    }, Some(Opt{
        background: Some(true),
        expire_after_seconds: None,
        name: Some("key_unique_1".parse().unwrap()),
        sparse: None,
        storage_engine: None,
        unique: Some(true),
        version: None,
        default_language: None,
        language_override: None,
        text_version: None,
        weights: None,
        sphere_version: None,
        bits: None,
        max: None,
        min: None,
        bucket_size: None
    })).unwrap();

    rocket::ignite()
        .manage(coll)
        .mount("/", routes![get, add]).launch();
}

