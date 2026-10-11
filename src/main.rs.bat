//use std::env::args;

/* struct Koot {
	id: u8,
	text: String,
	tty: bool,
} */


use std::env;
use std::fs::{self, OpenOptions};
use std::io::{self, Write};

fn main() {
	let args: Vec<String> = env::args().collect();
	let a = args.get(1).map(String::as_str);

	match a {
		Some("add") => add(&args[2..]),
		Some("list") => list(),
		_ => println!("нето"),
	}
}
fn add(args: &[String]) {
/*	eprintln!("IS: add <текст>");
	let kaw = args.join(" ");
	fs::write("sas.txt", kaw).unwrap();
	if args.is_empty() {
	} */
	append_line("sas.txt", &args.join(" ")).unwrap();
}
fn list() {
	let mut h = 1;
	if let Ok(o) = fs::read_to_string("sas.txt") {
		for i in o.lines() {
			println!("|{h}| {i}");
			h += 1;
		}
	}
}


fn append_line(path: &str, text: &str) -> io::Result<()> {
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)?;

    writeln!(file, "{text}")
}
