extern crate libjlox;

use clap::{Arg,App};
use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::io::BufReader;
use std::io::BufRead;


fn main() {
    let matches = App::new("rlox interpreter")
    .version(".01").author("Ian Smith").arg(Arg::with_name("SCRIPT").help("The script to run").required(false).index(1)).get_matches();
    let script = matches.value_of("SCRIPT");

    match script {
    	None => run_prompt().unwrap(),
    	Some(file) => run_file(file).unwrap()
    }
}


fn run_prompt() -> io::Result<()> {
	let mut rdr = BufReader::new(io::stdin());
	
	loop {	
		print!("> ");
		io::stdout().flush().unwrap();
		let mut contents = String::new();
		rdr.read_line(&mut contents)?;
	    libjlox::run(contents)
	}
}

fn run_file(fname: &str) -> io::Result<()> {
	let mut file = File::open(fname)?;
	let mut contents = String::new();
	file.read_to_string(&mut contents)?;
	libjlox::run(contents);
	Ok(())
}

