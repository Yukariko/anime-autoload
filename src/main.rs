//#![deny(warnings)]
extern crate futures;
extern crate hyper;
extern crate tokio_core;
extern crate serde_json;

use std::io::{self, Write};
use futures::{Future, Stream};
use tokio_core::reactor::Core;
use hyper::{Body, Client, Request};
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
        Ok(_) => {
            print!("{} contains:\n{}", display, s);
        },
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

fn get_json_from_uri(uri : hyper::Uri)
{
    let mut core = Core::new().unwrap();
    let client = Client::new(&core.handle());
    let _work = client.get(uri).and_then(|res| {
        //println!("Response: {}", res.status());

        /*res.body().for_each(|chunk| {
            io::stdout()
                .write_all(&chunk)
                .map_err(From::from)
        })*/

        res.body().concat2().and_then(move |body| {
        let v: Value = serde_json::from_slice(&body).map_err(|e| {
            io::Error::new(
                io::ErrorKind::Other,
                e
            )
        })?;

        let mut i = 0;
        while v[i] != serde_json::Value::Null {
            println!("{}:{}", v[i]["i"], v[i]["s"]);
            i += 1;
        }
        Ok(())
    })

    });
    core.run(_work);
}

fn get_anime_url_list()
{
    let url = String::from("http://www.anissia.net/anitime/list?w=");
    for i in 0..7 {
        let uri = url.clone() + format!("{}", i).as_str();
        get_json_from_uri(uri.parse::<hyper::Uri>().unwrap());
    }
}

fn main() {
    let list = get_anime_list("anime_list.conf");

    for item in list {
        print!("{}\n", item);
    }

    get_anime_url_list();
}
