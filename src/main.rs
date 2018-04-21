//#![deny(warnings)]
extern crate futures;
extern crate hyper;
extern crate tokio_core;
extern crate serde_json;


use futures::{Future, Stream};
use tokio_core::reactor::*;
use hyper::Client ;
use serde_json::Value;

use std::error::Error;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

fn get_anime_list(file_path : &str) -> Vec<String> {
    let path = Path::new(file_path);
    let display = path.display();
    let mut file = match File::open(&path) {
        Err(why) => panic!("couldn't open {}: {}", display, why.description()),
        Ok(file) => file,
    };

    let mut list : Vec<String> = Vec::new();
    let mut s =  String::new();
    match file.read_to_string(&mut s) {
        Err(why) => panic!("couldn't read {}: {}", display, why.description()),
        Ok(_) => {},
    }

    let mut lines = s.lines();
    loop {
        match lines.next() {
            Some(x) => {
                list.push(x.to_string());
            },
            None => { break }
        }
    }

    list
}

fn get_body(uri : String) -> String {
    let mut core = Core::new().unwrap();
    let client = Client::new(&core.handle());
    let f = client.get(uri.parse().unwrap()).map_err(|_err| ()).and_then(|resp| {
        resp.body().concat2().map_err(|_err| ()).map(|chunk| {
            let v = chunk.to_vec();
            String::from_utf8_lossy(&v).to_string()
        })
    });
    core.run(f).unwrap()
}

struct Anime {
    id : i64,
    name : String,
}

fn get_anime_id_list() -> Vec<Anime> {
    let url = String::from("http://www.anissia.net/anitime/list?w=");
    let mut list : Vec<Anime> = Vec::new();
    for i in 0..7 {
        let uri = url.clone() + format!("{}", i).as_str();
        let res : Value = serde_json::from_str(get_body(uri).as_str()).unwrap();
        {
            let mut j = 0;
            while res[j] != Value::Null {
                list.push(Anime {
                    id : res[j]["i"].as_i64().unwrap(),
                    name: res[j]["s"].as_str().unwrap().to_string(),
                });
                j += 1;
            }
        }
    }
    list
}

fn get_anime_subtitles_uri(id : i64) {
    let url = String::from("http://www.anissia.net/anitime/cap?i=");
    let uri = url + id.to_string().as_str();
    let res : Value = serde_json::from_str(get_body(uri).as_str()).unwrap();
    {
        let mut i = 0;
        while res[i] != Value::Null {
            println!("{}: {} - {}", res[i]["s"], res[i]["a"], res[i]["n"]);
            i += 1;
        }
    }
}

fn main() {
    let list = get_anime_list("anime_list.conf");
    let id_list = get_anime_id_list();
    for item in &id_list {
        for item2 in &list {
            if &item.name == item2 {
                println!("[{}]", item.name);
                get_anime_subtitles_uri(item.id);
                break;
            }
        }
    }
}
