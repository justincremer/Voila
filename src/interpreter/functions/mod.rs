extern crate async_trait;
extern crate fs_extra;

mod functions;

use super::exceptions::Exceptions;
use super::utils::path::Path;
use super::variables::Variables;
use super::Literal;
use super::Str;
use fs_extra::dir;
pub use functions::Functions;
use std::fs;
use std::process;

type Args = Vec<String>;

impl Functions for super::Interpreter {
    // Parser returns the args of a function as a vector of vector of Literals,
    // because an argument might have Text & Variables. Each vector inside the
    // super-vector is a function argument (those separated by ","), and the
    // Literals inside those vectors must be merged into a unique Literal,
    // creating a vector of literals, being each one a function argument
    // for making something that a function can deal with more easily.
    // Then, the best is to create directly a vector of strings, because
    // we do not care anymore about the type of the literal.
    fn supervec_literals_to_args(&self, supervec: Vec<Vec<Literal>>) -> Args {
        let mut final_args: Args = vec![];
        for vec_of_literals in supervec {
            let mut literals_str = String::from("");
            for literal in vec_of_literals {
                let str: String = self.get_var_if_any(&literal).unwrap().content.to_owned();

                literals_str = format!("{literals_str}{str}")
            }
            final_args.push(literals_str.clone());
        }
        return final_args;
    }

    // Functions definitions
    fn r#print(&self, args: Args) {
        // mitigate printing bottleneck by using only 1 print
        // context: i used to have a for loop
        println!("{}", args.join("\n"));
    }

    fn r#create(&self, args: Args) {
        if args.len() != 2 {
            self.raise_error(
                "UNEXPECTED QUANTITY ARGUMENTS",
                "create() function is expected to take only 2 args, the file and the content"
                    .to_string(),
            );
        } else {
            match fs::write(
                self.trim_spaces(&args[0]),
                self.trim_spaces(&args[1]),
            ) {
                Err(err) => self.raise_error(
                    "ERROR CREATING FILE",
                    format!("echo {} > {}': {err}", args[1], args[0]),
                ),
                _ => {}
            }
        }
    }

    fn r#mkdir(&self, args: Args) {
        for arg in args {
            match fs::create_dir_all(self.trim_spaces(&arg)) {
                Err(err) => self.raise_error(
                    "ERROR WHILE CREATING DIRECTORY",
                    format!("An error occurred:\n'mkdir --parent {arg}': {err}"),
                ),
                _ => {}
            };
        }
    }

    fn r#delete(&self, args: Args) {
        for arg in args {
            let is_file: Result<bool, ()> = self.is_file(&self.trim_spaces(&arg));

            // maybe file was delete in previous cycles.
            // if the path just was wrong, its not my fault,
            // user's fault. just re-run voila but reading what
            // you scripted before launching a tool that can be
            // (and in fact is) potentially destructive, im not
            // doing a hashmap of stuff deleted and then a checker,
            // enough overhead & bottlenecks with the async hell
            // of the cycles & the interpreter
            match is_file {
                Err(_) => return,
                _ => {}
            }

            // there is not a way of deleting something without
            // without caring if its a directory or a file, so
            // we have to get its type and call whatever needed
            if is_file.unwrap() {
                match fs::remove_file(self.trim_spaces(&arg)) {
                    Err(err) => self.raise_error(
                        "ERROR WHILE DELETING FILE",
                        format!("An error occurred:\n'rm -f {arg}': {err}"),
                    ),
                    _ => {}
                };
            } else {
                match fs::remove_dir_all(self.trim_spaces(&arg)) {
                    Err(err) => self.raise_error(
                        "ERROR WHILE DELETING DIRECTORY",
                        format!("An error occurred:\n'rm -rf {arg}': {err}"),
                    ),
                    _ => {}
                };
            }
        }
    }

    fn r#move(&self, args: Args) {
        // moving is literally copying and then deleting,
        // so i prefer to call their respective functions
        // instead of mashing them up
        self.r#copy(args.clone());
        self.r#delete(vec![args[0].clone()]);
    }

    fn r#copy(&self, args: Args) {
        // arguments must be exactly 2
        if args.len() != 2 {
            self.raise_error(
                "UNEXPECTED QUANTITY ARGUMENTS",
                "copy() & move() functions are expected to take only 2 args, the origin and the destination".to_string(),
            );
        } else {
            let is_file: Result<bool, ()> = self.is_file(&self.trim_spaces(&args[0]));

            // maybe file was delete in previous cycles.
            // if the path just was wrong, its not my fault,
            // user's fault. just re-run voila but reading what
            // you scripted before launching a tool that can be
            // (and in fact is) potentially destructive, im not
            // doing a hashmap of stuff deleted and then a checker,
            // enough overhead & bottlenecks with the async hell
            // of the cycles & the interpreter
            match is_file {
                Err(_) => return,
                _ => {}
            }

            if is_file.unwrap() {
                match fs::copy(
                    self.trim_spaces(&args[0]),
                    self.trim_spaces(&args[1]),
                ) {
                    Err(err) => self.raise_error(
                        "ERROR WHILE COPYING FILE",
                        format!("An error occurred:\n'cp {} {}': {err}", args[0], args[1]),
                    ),
                    _ => {}
                };
            } else {
                match dir::copy(
                    self.trim_spaces(&args[0]),
                    self.trim_spaces(&args[1]),
                    &dir::CopyOptions::new(),
                ) {
                    Err(err) => self.raise_error(
                        "ERROR WHILE COPYING DIR",
                        format!(
                            "An error occurred:\n'cp -r --parents --copy-contents {} {}': {err}",
                            args[0], args[1]
                        ),
                    ),
                    _ => {}
                };
            }
        }
    }
    fn r#shell(&self, args: Args) {
        for arg in &args {
            // get if operating system is *nix or Windows,
            // then launch whatever needed
            if cfg!(windows) {
                // Windows' shell is powershell/pwsh
                match process::Command::new("powershell")
                .arg("-Command")
                .arg(self.trim_spaces(&arg))
                .output() // spawn() does not await the cmd to finish, output() does
            {
                Err(err) => self.raise_error(
                    "ERROR WHILE EXECUTING SHELL",
                    format!("An error occurred:\n'powershell -Command {}': {err}", arg),
                ),
                _ => {}
            }
            } else if cfg!(unix) {
                // unix' shell is the bourne shell, aka sh
                match process::Command::new("sh")
                .arg("-c")
                .arg(self.trim_spaces(&arg))
                .output() // spawn() does not await the cmd to finish, output() does
            {
                Err(err) => self.raise_error(
                    "ERROR WHILE EXECUTING SHELL",
                    format!("An error occurred:\n'sh -c {}': {err}", arg),
                ),
                _ => {}
            }
            } else {
                self.raise_error(
                    "UNSUPPORTED PLATFORM",
                    "Voila is only supported on Windows & Unix-like systems".to_string(),
                )
            }
        }
    }
}
