extern crate either;
extern crate structopt;
extern crate yaml_rust;

use either::*;
use std::fs::File;
use std::io::prelude::*;
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use structopt::StructOpt;
use yaml_rust::{Yaml, YamlLoader};

#[derive(StructOpt, Debug)]
#[structopt(name = "reqs")]
struct Opt {
    #[structopt(name = "file", parse(from_os_str))]
    script_file: PathBuf,
    #[structopt(short = "p", long = "port", default_value = "8000")]
    port: i32,
}

#[derive(Debug, Clone)]
struct Step {
    name: String,
    path: String,
    method: String,
    code: i64,
    content: String,
    content_type: String,
}

impl Step {
    fn new(step_definition: &Yaml) -> Self {
        let content = get_content(parse_str_value(step_definition, "content", "").as_str());
        Step {
            name: parse_str_value(step_definition, "name", ""),
            path: parse_str_value(step_definition, "path", ""),
            method: parse_str_value(step_definition, "method", "GET").to_uppercase(),
            code: parse_int_value(step_definition, "code", 200),
            content: content,
            content_type: parse_str_value(step_definition, "content_type", "text/plain"),
        }
    }
}

#[derive(Debug, Clone)]
struct Script {
    name: String,
    repeat: bool,
    path: String,
    current_step: usize,
    steps: Vec<Step>,
}

fn msg_for_code<'a>(code: i64) -> &'a str {
    match code {
        200 => "Ok",
        201 => "Created",
        400 => "Bad Request",
        404 => "Not Found",
        _ => "",
    }
}

impl Script {
    fn new(script: &Yaml) -> Self {
        Script {
            name: parse_str_value(script, "name", ""),
            repeat: parse_bool_value(script, "repeat", false),
            path: parse_str_value(script, "path", ""),
            steps: parse_steps(script["steps"].as_vec().unwrap()),
            current_step: 0,
        }
    }

    fn step_name(&self) -> String {
        format!(
            "{}: {}",
            self.current_step, self.steps[self.current_step].name
        )
    }

    fn step_method(&self) -> String {
        format!("{}", self.steps[self.current_step].method)
    }

    fn step_path(&self) -> &String {
        if self.steps[self.current_step].path == "" {
            return &self.path;
        }
        return &self.steps[self.current_step].path;
    }

    fn step_response(&self) -> String {
        let step = &self.steps[self.current_step];
        return format!(
            "HTTP/1.1 {} {}\r\n\r\n{}",
            step.code,
            msg_for_code(step.code),
            step.content
        );
    }

    fn next_step(&mut self) -> Either<usize, &str> {
        if self.current_step + 1 == self.steps.len() {
            if !self.repeat {
                return Right("End of non-repeating script");
            }
            self.current_step = 0;
        } else {
            self.current_step += 1;
        }
        return Left(self.current_step);
    }
}

fn get_content_from_file(file_path: &str) -> String {
    let mut contents = String::new();
    let mut script_file = File::open(file_path).expect("Could not open response file");
    script_file
        .read_to_string(&mut contents)
        .expect("Could not read response file");
    return contents;
}

fn get_content(content_str: &str) -> String {
    match Path::new(content_str).exists() {
        true => get_content_from_file(content_str),
        _ => String::from(content_str),
    }
}

fn parse_str_value<'a>(yaml: &'a Yaml, key: &str, default: &'a str) -> String {
    String::from(yaml[key].as_str().unwrap_or(default))
}

fn parse_int_value<'a>(yaml: &'a Yaml, key: &str, default: i64) -> i64 {
    yaml[key].as_i64().unwrap_or(default)
}

fn parse_bool_value<'a>(yaml: &'a Yaml, key: &str, default: bool) -> bool {
    yaml[key].as_bool().unwrap_or(default)
}

fn parse_step(step_definition: &Yaml) -> Vec<Step> {
    let times = match step_definition["times"].as_i64() {
        Some(times) => times as usize,
        None => 1,
    };
    let mut steps = Vec::with_capacity(times);
    let step = Step::new(step_definition);
    for _ in 0..times {
        steps.push(step.clone());
    }
    return steps;
}

fn parse_steps(step_definitions: &Vec<Yaml>) -> Vec<Step> {
    let mut steps = Vec::new();
    step_definitions.into_iter().for_each(|step_definition| {
        parse_step(step_definition)
            .into_iter()
            .for_each(|step| steps.push(step))
    });
    return steps;
}

fn parse_script(script_str: &str) -> Script {
    let script = &YamlLoader::load_from_str(script_str).unwrap()[0];
    return Script::new(script);
}

fn main() {
    let opt = Opt::from_args();
    let mut script_contents = String::new();
    let mut script_file = File::open(opt.script_file).expect("Could not open script file");
    script_file
        .read_to_string(&mut script_contents)
        .expect("Could not read script file");
    let mut script = parse_script(&script_contents);
    println!(
        "Listening on port {}, running script '{}'",
        opt.port, script.name
    );
    serve(opt.port, &mut script);
}

fn serve(port: i32, script: &mut Script) {
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

fn handle_connection(mut stream: TcpStream, script: &mut Script) -> Either<usize, &str> {
    let mut buf = [0; 512];
    let result: Either<usize, &str>;
    stream.read(&mut buf).unwrap();
    if buf.starts_with(
        format!(
            "{} {} HTTP/1.1\r\n",
            script.step_method(),
            script.step_path()
        )
        .as_bytes(),
    ) {
        println!("Handling request with step '{}'", script.step_name());
        let response = script.step_response();
        stream.write(response.as_bytes()).unwrap();
        stream.flush().unwrap();
        result = script.next_step();
    } else {
        stream
            .write(
                format!(
                    "HTTP/1.1 400 Bad Request\r\n\r\nExpected {} {}",
                    script.step_method(),
                    script.step_path()
                )
                .as_bytes(),
            )
            .unwrap();
        stream.flush().unwrap();
        result = Left(script.current_step);
    }
    return result;
}
