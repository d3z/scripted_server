extern crate structopt;
extern crate yaml_rust;

use std::fs::File;
use std::io::prelude::*;
use std::path::{Path, PathBuf};
use structopt::StructOpt;
use yaml_rust::{YamlLoader, Yaml};

#[derive(StructOpt, Debug)]
#[structopt(name = "reqs")]
struct Opt {
    #[structopt(name="file", parse(from_os_str))]
    script_file: PathBuf
}

#[derive(Debug, Clone)]
struct Step {
    name: String,
    code: i64,
    content: String,
    content_type: String
}

#[derive(Debug, Clone)]
struct Script {
    name: String,
    repeat: bool,
    path: String,
    current_step: i32,
    steps: Vec<Step>
}

static TEXT_PLAIN: &str = "text/plain";

fn get_content_from_file(file_path: &str) -> String {
    let mut contents = String::new();
    let mut script_file = File::open(file_path).expect("Could not open response file");
    script_file.read_to_string(&mut contents).expect("Could not read response file");
    return contents;
}

fn get_content(content_str: &str) -> String {
    match Path::new(content_str).exists() {
        true => get_content_from_file(content_str),
        _ => String::from(content_str)
    }
}

fn parse_step(step_definition: &Yaml) -> Vec<Step> {
    println!("{:?}", step_definition);
    let times = match step_definition["times"].as_i64() { Some(times) => times as usize, None => 1 };
    let mut steps = Vec::with_capacity(times);
    let step = Step {
        name: String::from(step_definition["name"].as_str().unwrap()),
        code: step_definition["code"].as_i64().unwrap(),
        content: match step_definition["content"].as_str() { Some(content) => get_content(content), None => String::from("") },
        content_type: match step_definition["content_type"].as_str() { Some(content_type) => String::from(content_type), None => String::from(TEXT_PLAIN) },
    };
    for _ in 0..times {
        steps.push(step.clone());
    }
    println!("{:?}", steps);
    return steps;
}

fn parse_steps(step_definitions: &Vec<Yaml>) -> Vec<Step> {
    let mut steps = Vec::new();
    step_definitions.into_iter().for_each(|step_definition| parse_step(step_definition).into_iter().for_each(|step| steps.push(step)));
    return steps;
}

fn get_path_from_script(path: Option<&str>) -> String {
    match path {
        Some(path_str) => String::from(path_str),
        None => String::from("")
    }
}

fn parse_script(script_str: &str) -> Script {
    let script = &YamlLoader::load_from_str(script_str).unwrap()[0];
    Script {
        name: String::from(script["name"].as_str().unwrap()),
        repeat: script["repeat"].as_bool().unwrap(),
        path: get_path_from_script(script["path"].as_str()),
        current_step: 0,
        steps: parse_steps(script["steps"].as_vec().unwrap())
    }
}

fn main() {
    let opt = Opt::from_args();
    let mut script_contents = String::new();
    let mut script_file = File::open(opt.script_file).expect("Could not open script file");
    script_file.read_to_string(&mut script_contents).expect("Could not read script file");
    let script = parse_script(&script_contents);
    println!("{:?}", script);
}
