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

fn main() {
    let list = get_anime_list("anime_list.conf");

    for item in list {
        print!("{}\n", item);
    }
}
