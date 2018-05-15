//#![deny(warnings)]
extern crate futures;
extern crate hyper;
extern crate tokio_core;
extern crate serde_json;
extern crate hyper_openssl;

use futures::{Future, Stream};
use tokio_core::reactor::*;
use hyper::Client;
use hyper::Request;
use hyper::Method;
use hyper::client::HttpConnector;
use hyper::header::Pragma;
use hyper_openssl::HttpsConnector;
use serde_json::Value;
use std::error::Error;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

struct AnimeConf {
    name : String,
    magnet : String,
}

fn get_anime_list(file_path : &str) -> Vec<AnimeConf> {
    let path = Path::new(file_path);
    let display = path.display();
    let mut file = match File::open(&path) {
        Err(why) => panic!("couldn't open {}: {}", display, why.description()),
        Ok(file) => file,
    };

    let mut list : Vec<AnimeConf> = Vec::new();
    let mut s =  String::new();
    match file.read_to_string(&mut s) {
        Err(why) => panic!("couldn't read {}: {}", display, why.description()),
        Ok(_) => {},
    }

    let mut lines = s.lines();
    loop {
        match lines.next() {
            Some(x) => {
                let d : Vec<_> = x.split("\\").collect();
                if d[0].to_string().len() < 4 {
            				continue;
            		}
                list.push(AnimeConf {
                    name : d[0].to_string(),
                    magnet : d[1].to_string(),
                });
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

fn get_body_ssl(core : &mut Core, client : &Client<HttpsConnector<HttpConnector>>, uri : String) -> String {
    let mut request = Request::new(Method::Get, uri.parse().unwrap());
    {
    	let ref mut headers = request.headers_mut();
			headers.set(Pragma::NoCache);
		}

    let f = client.request(request).map_err(|_err| ()).and_then(|resp| {
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

fn convert_series_to_float(series : &str) -> f64 {
    series.to_string().parse::<f64>().unwrap() / 10.0
}

fn get_anime_magnet(core : &mut Core, client : &Client<HttpsConnector<HttpConnector>>, anime_list : &mut Vec<AnimeConf>) {

    let url = String::from("https://torrents.ohys.net/download/json.php?dir=new&p=");
    for i in 0..4 {
        let uri = format!("{}{}", url, i);
        let buf = format!("{}", get_body_ssl(&mut *core, &client, uri).as_str());
        let res : Value = serde_json::from_str(
            buf.get(3..).unwrap()
        ).unwrap();
        {
            let mut j = 0;
            while res[j] != Value::Null {
                let title = res[j]["t"].as_str().unwrap().to_string();
                let magnet = res[j]["a"].as_str().unwrap().to_string();
                for mut item in &mut *anime_list {
                    if !title.starts_with(item.magnet.as_str()) {
                        continue;
                    }
                    item.magnet = format!("<a href=\"https://torrents.ohys.net/download/{}\">{}</a>", magnet, title);
                    break;
                }
                j += 1;
            }
        }
    }
}

fn get_anime_subtitles_uri(core : &mut Core, client : &Client<HttpsConnector<HttpConnector>>, id : i64) {
    let url = String::from("http://www.anissia.net/anitime/cap?i=");
    let uri = url + id.to_string().as_str();
    let res : Value = serde_json::from_str(get_body_ssl(&mut *core, &client, uri).as_str()).unwrap();
    {
        let mut i = 0;
        let mut max_series = 0.5;
        while res[i] != Value::Null {
            let series = convert_series_to_float(res[i]["s"].as_str().unwrap());
            let link = &res[i]["a"];
            let name = res[i]["n"].as_str().unwrap();
            let date = &res[i]["d"].as_str().unwrap();
            if max_series <= series {
                println!("<li><a target=\"_blank\" href={}>[{}화] {} - {}</a></li>", link, series, name, date);
                max_series = series;
            }
            i += 1;
        }
    }
}

fn main() {
    let mut list = get_anime_list("anime_list.conf");
    let id_list = get_anime_id_list();

    println!("<html>
    <head>
        <title>Anime Autoload System</title>
    </head>
    <body>");

    let mut core = Core::new().unwrap();
	let client = Client::configure()
			.keep_alive(true)
    	.connector(HttpsConnector::new(4, &core.handle()).unwrap())
	    .build(&core.handle());
    get_anime_magnet(&mut core, &client, &mut list);

    for item in &id_list {
        for item2 in &list {
            if item.name == item2.name {
                println!("<h3>{}</h3>\n<ul>", item.name);
                println!("<li>{}</li>", item2.magnet);
                get_anime_subtitles_uri(&mut core, &client, item.id);
                println!("</ul>");
                break;
            }
        }
    }
    println!("    </body>
</html>");
}
