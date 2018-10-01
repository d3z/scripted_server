extern crate structopt;

use std::io::prelude::*;
use std::fs::File;
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "reqs")]
struct Opt {
    #[structopt(name="file", parse(from_os_str))]
    script_file: PathBuf
}

fn main() {
    let opt = Opt::from_args();
    let mut script_contents = String::new();
    let mut script_file = File::open(opt.script_file).expect("Could not open script file");
    script_file.read_to_string(&mut script_contents).expect("Could not read script file");
}
