extern crate md5;
extern crate path_absolutize;
extern crate sha256;

use path_absolutize::*;
use regex::Regex;
use std::fs;
use std::fs::File;
use std::io::prelude::*;
use std::panic;
use std::path::Path;
use std::process;
pub use string::Str;
pub use sum::Sum;
pub use sum::SumTypes;

pub mod path;
pub mod regexp;
pub mod string;
pub mod sum;

impl path::Path for super::Interpreter {
    fn exist(&self, input: &String) -> bool {
        // absolutize path, just to be sure nothing
        // weird happens
        let path = self.absolutize(input);

        // create a new path struct with the
        // absolute path and get if exists
        Path::new(&path).exists()
    }

    fn absolutize(&self, input: &String) -> String {
        // ge the absolute path
        let p = Path::new(&input).absolutize().unwrap();

        // get the &str
        // (i think i cannot get the String directly)
        let path = p.to_str().unwrap();

        // return it converted to String
        path.to_string()
    }

    fn is_file(&self, input: &String) -> Result<bool, ()> {
        // maybe file was delete in previous cycles.
        // if the path just was wrong, its not my fault,
        // user's fault. just re-run voila but reading what
        // you scripted before launching a tool that can be
        // (and in fact is) potentially destructive, im not
        // doing a hashmap of stuff deleted and then a checker,
        // enough overhead & bottlenecks with the async hell
        // of the cycles & the interpreter
        match fs::metadata(input) {
            Ok(md) => Ok(md.is_file()),
            Err(_) => Err(()),
        }
    }
}

impl regexp::RegExp for super::Interpreter {
    fn matches(&self, input: String, regexp: String) -> bool {
        // create a new regex struct from the regex string
        let regex = Regex::new(&regexp).unwrap();

        // return an eval of the regex with the string
        regex.is_match(&input)
    }
}

impl Str for super::Interpreter {
    fn trim_spaces(&self, str: &String) -> String {
        str.trim().to_string()
    }
}

impl Sum for super::Interpreter {
    fn get_sum_of(&self, file: &String, sum: SumTypes) -> Result<String, String> {
        let bytes = self.read_bytes_of_file(file);
        println!("{:?}", sum);
        match sum {
            SumTypes::Sha256 => Ok(sha256::digest_bytes(bytes)),
            SumTypes::Md5 => Ok(format!("{:x}", md5::compute(bytes))),
        }
    }
    fn read_bytes_of_file<'a>(&self, path: &String) -> &'a [u8] {
        let buffer = "";
        let file = panic::catch_unwind(|| File::open(path).unwrap());
        match file {
            Ok(mut f) => f
                .read_to_string(&mut String::from(buffer))
                .unwrap_or_else(|e| {
                    eprintln!("could not read one or more files:\n{:#?}", e);
                    process::exit(1)
                }),
            Err(e) => {
                eprintln!("could not open one or more files:\n{:#?}", e);
                process::exit(1)
            }
        };
        buffer.as_bytes()
    }
}
