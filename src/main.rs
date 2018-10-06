extern crate structopt;
extern crate either;

use either::*;

use std::net::{TcpListener, TcpStream};
use std::io::prelude::*;
use std::fs::File;
use std::path::PathBuf;
use structopt::StructOpt;

mod script;

#[derive(StructOpt, Debug)]
#[structopt(name = "reqs")]
struct Opt {
    #[structopt(name="file", parse(from_os_str))]
    script_file: PathBuf,
    #[structopt(short="p", long="port", default_value="8000")]
    port: i32
}

fn main() {
    let opt = Opt::from_args();
    let mut script_contents = String::new();
    let mut script_file = File::open(opt.script_file).expect("Could not open script file");
    script_file.read_to_string(&mut script_contents).expect("Could not read script file");
    let mut script = script::parse_script(&script_contents);
    println!("Listening on port {}, running script '{}'", opt.port, script.name);
    serve(opt.port, &mut script);
}

fn serve(port: i32, script: &mut script::Script) {
    let url = format!("127.0.0.1:{}", port);
    let listener = TcpListener::bind(url).unwrap();
    for stream in listener.incoming() {
        let stream = stream.unwrap();
        match handle_connection(stream, script) {
            Left(_) => continue,
            Right(msg) => { 
                println!("{}", msg);
                break;
            }
        }
    }
}

fn handle_connection(mut stream: TcpStream, script: &mut script::Script) -> Either<usize, &str> {
    let mut buf = [0; 512];
    let result: Either<usize, &str>;
    stream.read(&mut buf).unwrap();
    if buf.starts_with(format!("{} {} HTTP/1.1\r\n", script.step_method(), script.step_path()).as_bytes()) {
        println!("Handling request with step '{}'", script.step_name());
        let response = script.step_response();
        stream.write(response.as_bytes()).unwrap();
        stream.flush().unwrap();
        result = script.next_step();
    } else {
        stream.write(format!("HTTP/1.1 400 Bad Request\r\n\r\nExpected {} {}", script.step_method(), script.step_path()).as_bytes()).unwrap();
        stream.flush().unwrap();
        result = Left(script.current_step);
    }
    return result;
}
