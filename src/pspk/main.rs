#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use] extern crate rocket;
extern crate rocket_contrib;
#[macro_use] extern crate serde_derive;
extern crate serde;

use std::collections::HashMap;
use std::sync::RwLock;
use rocket::State;
use rocket_contrib::json::Json;

struct PublicKeys  {
    store:  HashMap<String,String>,
}

#[derive(Serialize, Deserialize)]
struct Key {
    key: String,
}

#[post("/<name>", format = "application/json", data = "<key>")]
fn add(name: String, key: Json<Key>, current_store: State<RwLock<PublicKeys>>) {
    let mut cur = current_store.write().expect("map lock");
    cur.store.insert(name, key.0.key);
}

#[get("/<name>")]
fn get(name: String, current_store: State<RwLock<PublicKeys>>) -> String {
    let key = current_store.read().unwrap();
    let key = key.store.get(name.as_str()).unwrap();

    format!("{}", &key)
}

fn main() {
    rocket::ignite()
        .manage(RwLock::new(PublicKeys{ store: HashMap::<String, String>::new() }))
        .mount("/", routes![get, add]).launch();
}

