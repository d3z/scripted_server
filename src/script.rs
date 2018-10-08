extern crate yaml_rust;
extern crate either;

use either::*;
use yaml_rust::{YamlLoader, Yaml};

use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

#[derive(Debug, Clone)]
struct Step {
    name: String,
    path: String,
    method: String,
    code: i64,
    content: String,
    content_type: String
}

impl Step {
    fn new(step_definition: &Yaml) -> Self {
        let content = get_content(parse_str_value(step_definition, "content", "").as_str()); 
        Step {
            name: parse_str_value(step_definition, "name", ""),
            path: parse_str_value(step_definition, "path", ""),
            method: parse_str_value(step_definition, "method", "GET").to_uppercase(),
            code: parse_int_value(step_definition, "code", 200),
            content_type: parse_str_value(step_definition, "content_type", "text/plain"),
            content
        }
    }
}

#[derive(Debug, Clone)]
pub struct Script {
    pub name: String,
    pub current_step: usize,
    repeat: bool,
    path: String,
    steps: Vec<Step>
}

fn msg_for_code<'a>(code: i64) -> &'a str {
    match code {
        200 => "Ok",
        201 => "Created",
        400 => "Bad Request",
        404 => "Not Found",
        _ => ""
    }
}

impl Script {
    pub fn new(script: &Yaml) -> Self {
        Script {
            name: parse_str_value(script, "name", ""),
            repeat: parse_bool_value(script, "repeat", false),
            path: parse_str_value(script, "path", ""),
            steps: parse_steps(script["steps"].as_vec()),
            current_step: 0
        }
    }

    pub fn step_name(&self) -> String {
        format!("{}: {}", self.current_step, self.steps[self.current_step].name)
    }

    pub fn step_method(&self) -> String {
        self.steps[self.current_step].method.to_string()
    }

    pub fn step_path(&self) -> &String {
        if self.steps[self.current_step].path == "" {
            return &self.path
        }
        &self.steps[self.current_step].path
    }

    pub fn step_headers(&self) -> String {
        let step = &self.steps[self.current_step];
        return format!("Server: scripted_server\r\nContent-Type: {}\r\nContent-Length: {}",
            step.content_type, 
            step.content.len());
    }

    pub fn step_response(&self) -> String {
        let step = &self.steps[self.current_step];
        return format!("HTTP/1.1 {} {}\r\nContent-Type: {}\r\n\r\n{}", step.code, msg_for_code(step.code), self.step_headers(), step.content);
    }

    pub fn next_step(&mut self) -> Either<usize, &str> {
        if self.current_step + 1 == self.steps.len() {
            if !self.repeat {
                return Right("End of non-repeating script");
            }
            self.current_step = 0;
        } else {
            self.current_step += 1;
        }
        Left(self.current_step)
    }
}

fn get_content_from_file(file_path: &str) -> String {
    let mut contents = String::new();
    let mut script_file = File::open(file_path).expect("Could not open response file");
    script_file.read_to_string(&mut contents).expect("Could not read response file");
    contents
}

fn get_content(content_str: &str) -> String {
    if Path::new(content_str).exists() { get_content_from_file(content_str) } else { String::from(content_str) }
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
    let times = match step_definition["times"].as_i64() { Some(times) => times as usize, None => 1 };
    let mut steps = Vec::with_capacity(times);
    let step = Step::new(step_definition);
    for _ in 0..times {
        steps.push(step.clone());
    }
    steps
}

fn parse_steps(step_definitions: Option<&Vec<Yaml>>) -> Vec<Step> {
    let mut steps = Vec::new();
    match step_definitions {
        Some(step_definitions) => step_definitions.into_iter().for_each(|step_definition| parse_step(step_definition).into_iter().for_each(|step| steps.push(step))),
        None => return steps
    }
    steps
}

pub fn parse_script(script_str: &str) -> Script {
    let script = &YamlLoader::load_from_str(script_str).unwrap()[0];
    Script::new(script)
}

#[cfg(test)]
mod test {

    use super::*;

    static SCRIPT_WITHOUT_STEPS: &'static str = "
    name: test
    path: /test
    ";

    static SCRIPT_WITH_ONE_STEP: &'static str = "
    name: test
    steps:
        - name: step 1
    ";

    static SCRIPT_WITH_TWO_STEP_REPEATING: &'static str = "
    name: test
    repeat: true
    path: /test
    steps:
        - name: step 1
          code: 200
        - name: step 2
          code: 404
          path: /test/again
    ";

    #[test]
    fn should_parse_script() {
        let script = parse_script(SCRIPT_WITHOUT_STEPS);
        assert_eq!(script.name, "test");
        assert_eq!(script.path, "/test");
    }

    #[test]
    fn should_default_optional_script_fields() {
        let script = parse_script(SCRIPT_WITHOUT_STEPS);
        assert_eq!(script.repeat, false);
    }

    #[test]
    fn should_parse_script_with_one_step() {
        let script = parse_script(SCRIPT_WITH_ONE_STEP);
        assert_eq!(script.steps.len(), 1);
        assert_eq!(script.step_name(), "0: step 1");
    }

    #[test]
    fn should_default_optional_step_fields() {
        let script = parse_script(SCRIPT_WITH_ONE_STEP);
        assert_eq!(script.step_method(), "GET");
    }

    #[test]
    fn should_progress_to_next_step() {
        let mut script = parse_script(SCRIPT_WITH_TWO_STEP_REPEATING);
        assert_eq!(script.step_name(), "0: step 1");
        script.next_step();
        assert_eq!(script.step_name(), "1: step 2");
    }

    #[test]
    fn should_repeat_steps_for_repeating_script() {
        let mut script = parse_script(SCRIPT_WITH_TWO_STEP_REPEATING);
        script.next_step();
        script.next_step();
        assert_eq!(script.step_name(), "0: step 1");
    }

    #[test]
    fn should_use_script_path_when_not_defined_in_step() {
        let script = parse_script(SCRIPT_WITH_TWO_STEP_REPEATING);
        assert_eq!(script.step_path(), "/test"); // script path
    }


}