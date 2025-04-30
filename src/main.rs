// #![allow(warnings, unused)]
use regex::Regex;
use std::collections::HashMap;
use std::env::{self, args};
use std::fs::File;
use std::fs::{self};
use std::io::Read;
use std::io::Write;
use std::process::Command;

mod ast;
mod errors;
mod lexer;

use errors::*;

fn get_type(token: String) -> ast::Types {
    let number_re = Regex::new(r"^[0-9]+$").unwrap();
    let identifier_re = Regex::new(r"^[a-zA-Z_][a-zA-Z0-9_]*$").unwrap();

    if token.starts_with('\"') && token.ends_with('\"') {
        ast::Types::String
    } else if number_re.is_match(&token) {
        ast::Types::Number
    } else if identifier_re.is_match(&token) {
        ast::Types::Identifier
    } else {
        ast::Types::Unknown
    }
}

fn string_to_type(string: String) -> Result<ast::Types, Error> {
    match string.as_str() {
        "Number" => Ok(ast::Types::Number),

        "String" => Ok(ast::Types::String),

        _ => Err(Error::RuntimeError("Invalid type".to_string())),
    }
}

fn rem_first_and_last(value: &str) -> &str {
    let mut chars = value.chars();
    chars.next();
    chars.next_back();
    chars.as_str()
}

fn get_string_content(string: String) -> String {
    rem_first_and_last(&string).to_string().replace("\\n", "\n")
}

fn get_variable(
    variable_name: String,
    variables: HashMap<String, String>,
) -> Result<(String, ast::Types), Error> {
    match variables.get(&variable_name) {
        Some(value) => {
            let value = (*value).to_string();
            let value_type = get_type(value.clone());
            if value_type == ast::Types::Identifier {
                let value = get_variable(value, variables)?;
                Ok((value.0, value.1))
            } else {
                Ok((value, value_type))
            }
        }

        None => Err(Error::RuntimeError(format!(
            "Variable `{}` does not exist.",
            variable_name
        ))),
    }
}

fn interpret(
    lexed_code: Vec<(usize, lexer::Line)>,
    variables: &mut HashMap<String, String>,
    labels: &mut HashMap<String, Vec<(usize, lexer::Line)>>,
) {
    for (line_number, line) in lexed_code.iter() {
        let line: Vec<String> = line.clone().0;
        let string_line = line.clone().join(" ");

        let command: String = line[0].clone();
        let args: Vec<String> = line[1..].to_vec().clone();
        let args_len = args.len();

        match command.clone().as_str() {
            "exists" => {
                if args_len != 2 {
                    print_error(format!(
                        "\nCode:\n{} | {}\nProblem: Expected 2 arguments, got {}.",
                        line_number,
                        string_line.clone(),
                        args_len
                    ));
                } else {
                    let variable_name = args[0].clone();

                    if !variables.contains_key(&variable_name) {
                        print_error(format!(
                            "\nCode:\n{} | {}\nProblem: Variable `{}` does not exist.",
                            line_number,
                            string_line.clone(),
                            variable_name
                        ));
                    }

                    let variable_type = get_type(variables.get(&variable_name).unwrap().clone());
                    let variable_required_type = string_to_type(args[1].clone());

                    match variable_required_type.clone() {
                        Ok(required_type) => {
                            if variable_type != required_type {
                                print_error(format!("\nCode:\n{} | {}\nProblem: Variable `{}` is of type `{}`, but `{}` is required.", line_number, string_line.clone(), variable_name, variable_type, required_type));
                            }
                        }
                        Err(e) => {
                            print_error(format!(
                                "\nCode:\n{} | {}\nProblem: {}",
                                line_number,
                                string_line.clone(),
                                e
                            ));
                        }
                    }
                }
            }

            "var" => {
                if args_len != 2 {
                    print_error(format!(
                        "\nCode:\n{} | {}\nProblem: Expected 2 arguments, got {}.",
                        line_number,
                        string_line.clone(),
                        args_len
                    ));
                } else {
                    let variable_name = args[0].clone();
                    let mut variable_value = args[1].clone();

                    if get_type(variable_value.clone()) == ast::Types::Identifier {
                        match get_variable(variable_value.clone(), variables.clone()) {
                            Ok((value, _)) => {
                                variable_value = value;
                            }
                            Err(e) => {
                                print_error(format!(
                                    "\nCode:\n{} | {}\nProblem: {}",
                                    line_number,
                                    string_line.clone(),
                                    e
                                ));
                            }
                        }
                    }

                    variables.insert(variable_name.clone(), variable_value.clone());
                }
            }

            "print" => {
                if args_len != 1 {
                    print_error(format!(
                        "\nCode:\n{} | {}\nProblem: Expected 1 argument, got {}.",
                        line_number,
                        string_line.clone(),
                        args_len
                    ));
                } else {
                    let mut to_print = args[0].clone();

                    match get_type(to_print.clone()) {
                        ast::Types::Number => {}

                        ast::Types::String => {
                            to_print = get_string_content(to_print.clone());
                        }

                        // Variables
                        ast::Types::Identifier => {
                            match get_variable(to_print.clone(), variables.clone()) {
                                Ok((value, _)) => {
                                    to_print = value;
                                }
                                Err(e) => {
                                    print_error(format!(
                                        "\nCode:\n{} | {}\nProblem: {}",
                                        line_number,
                                        string_line.clone(),
                                        e
                                    ));
                                }
                            }

                            let to_print_type = get_type(to_print.clone());

                            if to_print_type == ast::Types::String {
                                to_print = get_string_content(to_print.clone());
                            }
                        }

                        _ => {
                            print_error(format!(
                                "\nCode:\n{} | {}\nProblem: Invalid type.",
                                line_number,
                                string_line.clone()
                            ));
                        }
                    }

                    print!("{}", to_print);
                }
            }

            "print_newline" => {
                if args_len != 0 {
                    print_error(format!(
                        "\nCode:\n{} | {}\nProblem: Expected 0 arguments, got {}.",
                        line_number,
                        string_line.clone(),
                        args_len
                    ));
                } else {
                    println!();
                }
            }

            "add" => {
                if args_len != 2 {
                    print_error(format!(
                        "\nCode:\n{} | {}\nProblem: Expected 2 arguments, got {}.",
                        line_number,
                        string_line.clone(),
                        args_len
                    ));
                } else {
                    let mut item1 = args[0].clone();
                    let mut item2 = args[1].clone();

                    if get_type(item1.clone()) == ast::Types::Identifier {
                        match get_variable(item1.clone(), variables.clone()) {
                            Ok((value, _)) => {
                                item1 = value;
                            }
                            Err(e) => {
                                print_error(format!(
                                    "\nCode:\n{} | {}\nProblem: {}",
                                    line_number,
                                    string_line.clone(),
                                    e
                                ));
                            }
                        }
                    }

                    if get_type(item2.clone()) == ast::Types::Identifier {
                        match get_variable(item2.clone(), variables.clone()) {
                            Ok((value, _)) => {
                                item2 = value;
                            }
                            Err(e) => {
                                print_error(format!(
                                    "\nCode:\n{} | {}\nProblem: {}",
                                    line_number,
                                    string_line.clone(),
                                    e
                                ));
                            }
                        }
                    }

                    match (get_type(item1.clone()), get_type(item2.clone())) {
                        (ast::Types::String, ast::Types::String) => {
                            variables.insert(
                                "TEMP".to_string(),
                                "\"".to_owned()
                                    + &get_string_content(item1.clone())
                                    + &get_string_content(item2.clone())
                                    + "\"",
                            );
                        }

                        (ast::Types::Number, ast::Types::String) => {
                            variables.insert(
                                "TEMP".to_string(),
                                "\"".to_owned()
                                    + &item1.clone()
                                    + &get_string_content(item2.clone())
                                    + "\"",
                            );
                        }

                        (ast::Types::String, ast::Types::Number) => {
                            variables.insert(
                                "TEMP".to_string(),
                                "\"".to_owned()
                                    + &get_string_content(item1.clone())
                                    + &item2.clone()
                                    + "\"",
                            );
                        }

                        (ast::Types::Number, ast::Types::Number) => {
                            let new_number = item1.clone().parse::<f64>().unwrap()
                                + item2.clone().parse::<f64>().unwrap();

                            variables.insert("TEMP".to_string(), format!("{}", new_number));
                        }

                        _ => {
                            let mut not_supported_arg = args[0].clone();
                            if get_type(args[0].clone()) != ast::Types::Number
                                && get_type(args[0].clone()) != ast::Types::String
                            {
                                not_supported_arg = args[0].clone();
                            } else if get_type(args[1].clone()) != ast::Types::Number
                                && get_type(args[1].clone()) != ast::Types::String
                            {
                                not_supported_arg = args[1].clone();
                            }
                            print_error(format!("\nCode:\n{} | {}\nProblem: Cannot add as `{}` is neither a string nor a number.", line_number, string_line.clone(), not_supported_arg))
                        }
                    }
                }
            }

            "sub" => {
                if args_len != 2 {
                    print_error(format!(
                        "\nCode:\n{} | {}\nProblem: Expected 2 arguments, got {}.",
                        line_number,
                        string_line.clone(),
                        args_len
                    ));
                } else {
                    let mut item1 = args[0].clone();
                    let mut item2 = args[1].clone();

                    if get_type(item1.clone()) == ast::Types::Identifier {
                        match get_variable(item1.clone(), variables.clone()) {
                            Ok((value, value_type)) => {
                                if value_type != ast::Types::Number {
                                    print_error(format!("\nCode:\n{} | {}\nProblem: Variable `{}` is of type `{}`, but `Number` is required.", line_number, string_line.clone(), item1, value_type));
                                }
                                item1 = value;
                            }
                            Err(e) => {
                                print_error(format!(
                                    "\nCode:\n{} | {}\nProblem: {}",
                                    line_number,
                                    string_line.clone(),
                                    e
                                ));
                            }
                        }
                    }

                    if get_type(item2.clone()) == ast::Types::Identifier {
                        match get_variable(item2.clone(), variables.clone()) {
                            Ok((value, value_type)) => {
                                if value_type != ast::Types::Number {
                                    print_error(format!("\nCode:\n{} | {}\nProblem: Variable `{}` is of type `{}`, but `Number` is required.", line_number, string_line.clone(), item1, value_type));
                                }
                                item2 = value;
                            }
                            Err(e) => {
                                print_error(format!(
                                    "\nCode:\n{} | {}\nProblem: {}",
                                    line_number,
                                    string_line.clone(),
                                    e
                                ));
                            }
                        }
                    }

                    if get_type(item1.clone()) != ast::Types::Number {
                        print_error(format!("\nCode:\n{} | {}\nProblem: Cannot subtract `{}` from `{}` as `{}` is not a numeric type.", line_number, string_line.clone(), args[1].clone(), args[0].clone(), args[0].clone()));
                    }

                    if get_type(item2.clone()) != ast::Types::Number {
                        print_error(format!("\nCode:\n{} | {}\nProblem: Cannot subtract `{}` from `{}` as `{}` is not a numeric type.", line_number, string_line.clone(), args[1].clone(), args[0].clone(), args[1].clone()));
                    }

                    let new_number = item1.clone().parse::<f64>().unwrap()
                        - item2.clone().parse::<f64>().unwrap();
                    variables.insert("TEMP".to_string(), format!("{}", new_number));
                }
            }

            "mul" => {
                if args_len != 2 {
                    print_error(format!(
                        "\nCode:\n{} | {}\nProblem: Expected 2 arguments, got {}.",
                        line_number,
                        string_line.clone(),
                        args_len
                    ));
                } else {
                    let mut item1 = args[0].clone();
                    let mut item2 = args[1].clone();

                    if get_type(item1.clone()) == ast::Types::Identifier {
                        match get_variable(item1.clone(), variables.clone()) {
                            Ok((value, value_type)) => {
                                if value_type != ast::Types::Number {
                                    print_error(format!("\nCode:\n{} | {}\nProblem: Variable `{}` is of type `{}`, but `Number` is required.", line_number, string_line.clone(), item1, value_type));
                                }
                                item1 = value;
                            }
                            Err(e) => {
                                print_error(format!(
                                    "\nCode:\n{} | {}\nProblem: {}",
                                    line_number,
                                    string_line.clone(),
                                    e
                                ));
                            }
                        }
                    }

                    if get_type(item2.clone()) == ast::Types::Identifier {
                        match get_variable(item2.clone(), variables.clone()) {
                            Ok((value, value_type)) => {
                                if value_type != ast::Types::Number {
                                    print_error(format!("\nCode:\n{} | {}\nProblem: Variable `{}` is of type `{}`, but `Number` is required.", line_number, string_line.clone(), item1, value_type));
                                }
                                item2 = value;
                            }
                            Err(e) => {
                                print_error(format!(
                                    "\nCode:\n{} | {}\nProblem: {}",
                                    line_number,
                                    string_line.clone(),
                                    e
                                ));
                            }
                        }
                    }

                    if get_type(item1.clone()) != ast::Types::Number {
                        print_error(format!("\nCode:\n{} | {}\nProblem: Cannot multiply `{}` by `{}` as `{}` is not a numeric type.", line_number, string_line.clone(), args[0].clone(), args[1].clone(), args[0].clone()));
                    }

                    if get_type(item2.clone()) != ast::Types::Number {
                        print_error(format!("\nCode:\n{} | {}\nProblem: Cannot multiply `{}` by `{}` as `{}` is not a numeric type.", line_number, string_line.clone(), args[0].clone(), args[1].clone(), args[1].clone()));
                    }

                    let new_number = item1.clone().parse::<f64>().unwrap()
                        * item2.clone().parse::<f64>().unwrap();
                    variables.insert("TEMP".to_string(), format!("{}", new_number));
                }
            }

            "div" => {
                if args_len != 2 {
                    print_error(format!(
                        "\nCode:\n{} | {}\nProblem: Expected 2 arguments, got {}.",
                        line_number,
                        string_line.clone(),
                        args_len
                    ));
                } else {
                    let mut item1 = args[0].clone();
                    let mut item2 = args[1].clone();

                    if get_type(item1.clone()) == ast::Types::Identifier {
                        match get_variable(item1.clone(), variables.clone()) {
                            Ok((value, value_type)) => {
                                if value_type != ast::Types::Number {
                                    print_error(format!("\nCode:\n{} | {}\nProblem: Variable `{}` is of type `{}`, but `Number` is required.", line_number, string_line.clone(), item1, value_type));
                                }
                                item1 = value;
                            }
                            Err(e) => {
                                print_error(format!(
                                    "\nCode:\n{} | {}\nProblem: {}",
                                    line_number,
                                    string_line.clone(),
                                    e
                                ));
                            }
                        }
                    }

                    if get_type(item2.clone()) == ast::Types::Identifier {
                        match get_variable(item2.clone(), variables.clone()) {
                            Ok((value, value_type)) => {
                                if value_type != ast::Types::Number {
                                    print_error(format!("\nCode:\n{} | {}\nProblem: Variable `{}` is of type `{}`, but `Number` is required.", line_number, string_line.clone(), item1, value_type));
                                }
                                item2 = value;
                            }
                            Err(e) => {
                                print_error(format!(
                                    "\nCode:\n{} | {}\nProblem: {}",
                                    line_number,
                                    string_line.clone(),
                                    e
                                ));
                            }
                        }
                    }

                    if get_type(item1.clone()) != ast::Types::Number {
                        print_error(format!("\nCode:\n{} | {}\nProblem: Cannot divide `{}` by `{}` as `{}` is not a numeric type.", line_number, string_line.clone(), args[0].clone(), args[1].clone(), args[0].clone()));
                    }

                    if get_type(item2.clone()) != ast::Types::Number {
                        print_error(format!("\nCode:\n{} | {}\nProblem: Cannot divide `{}` by `{}` as `{}` is not a numeric type.", line_number, string_line.clone(), args[0].clone(), args[1].clone(), args[1].clone()));
                    }

                    let new_number = item1.clone().parse::<f64>().unwrap()
                        / item2.clone().parse::<f64>().unwrap();
                    variables.insert("TEMP".to_string(), format!("{}", new_number));
                }
            }

            "mod" => {
                if args_len != 2 {
                    print_error(format!(
                        "\nCode:\n{} | {}\nProblem: Expected 2 arguments, got {}.",
                        line_number,
                        string_line.clone(),
                        args_len
                    ));
                } else {
                    let mut item1 = args[0].clone();
                    let mut item2 = args[1].clone();

                    if get_type(item1.clone()) == ast::Types::Identifier {
                        match get_variable(item1.clone(), variables.clone()) {
                            Ok((value, value_type)) => {
                                if value_type != ast::Types::Number {
                                    print_error(format!("\nCode:\n{} | {}\nProblem: Variable `{}` is of type `{}`, but `Number` is required.", line_number, string_line.clone(), item1, value_type));
                                }
                                item1 = value;
                            }
                            Err(e) => {
                                print_error(format!(
                                    "\nCode:\n{} | {}\nProblem: {}",
                                    line_number,
                                    string_line.clone(),
                                    e
                                ));
                            }
                        }
                    }

                    if get_type(item2.clone()) == ast::Types::Identifier {
                        match get_variable(item2.clone(), variables.clone()) {
                            Ok((value, value_type)) => {
                                if value_type != ast::Types::Number {
                                    print_error(format!("\nCode:\n{} | {}\nProblem: Variable `{}` is of type `{}`, but `Number` is required.", line_number, string_line.clone(), item1, value_type));
                                }
                                item2 = value;
                            }
                            Err(e) => {
                                print_error(format!(
                                    "\nCode:\n{} | {}\nProblem: {}",
                                    line_number,
                                    string_line.clone(),
                                    e
                                ));
                            }
                        }
                    }

                    if get_type(item1.clone()) != ast::Types::Number {
                        print_error(format!("\nCode:\n{} | {}\nProblem: Cannot compare `{}` and `{}` as `{}` is not a numeric type.", line_number, string_line.clone(), args[0].clone(), args[1].clone(), args[0].clone()));
                    }

                    if get_type(item2.clone()) != ast::Types::Number {
                        print_error(format!("\nCode:\n{} | {}\nProblem: Cannot compare `{}` and `{}` as `{}` is not a numeric type.", line_number, string_line.clone(), args[0].clone(), args[1].clone(), args[1].clone()));
                    }

                    let new_number = item1.clone().parse::<f64>().unwrap()
                        % item2.clone().parse::<f64>().unwrap();
                    variables.insert("TEMP".to_string(), format!("{}", new_number));
                }
            }

            "jmp" => {
                if args_len != 1 {
                    print_error(format!(
                        "\nCode:\n{} | {}\nProblem: Expected 1 argument, got {}.",
                        line_number,
                        string_line.clone(),
                        args_len
                    ));
                } else {
                    let label_name = args[0].clone();

                    if !labels.contains_key(&label_name) {
                        print_error(format!(
                            "\nCode:\n{} | {}\nProblem: Label `{}` does not exist.",
                            line_number,
                            string_line.clone(),
                            label_name
                        ));
                    }

                    let label_code = labels.get(&label_name).unwrap().clone();

                    interpret(label_code, variables, labels);
                }
            }

            "jmp_gt" => {
                if args_len != 3 {
                    print_error(format!(
                        "\nCode:\n{} | {}\nProblem: Expected 3 arguments, got {}.",
                        line_number,
                        string_line.clone(),
                        args_len
                    ));
                } else {
                    let mut item1 = args[0].clone();
                    let mut item2 = args[1].clone();
                    let label_name = args[2].clone();

                    if get_type(item1.clone()) == ast::Types::Identifier {
                        match get_variable(item1.clone(), variables.clone()) {
                            Ok((value, value_type)) => {
                                if value_type != ast::Types::Number {
                                    print_error(format!("\nCode:\n{} | {}\nProblem: Variable `{}` is of type `{}`, but `Number` is required.", line_number, string_line.clone(), item1, value_type));
                                }
                                item1 = value;
                            }
                            Err(e) => {
                                print_error(format!(
                                    "\nCode:\n{} | {}\nProblem: {}",
                                    line_number,
                                    string_line.clone(),
                                    e
                                ));
                            }
                        }
                    }

                    if get_type(item2.clone()) == ast::Types::Identifier {
                        match get_variable(item2.clone(), variables.clone()) {
                            Ok((value, value_type)) => {
                                if value_type != ast::Types::Number {
                                    print_error(format!("\nCode:\n{} | {}\nProblem: Variable `{}` is of type `{}`, but `Number` is required.", line_number, string_line.clone(), item2, value_type));
                                }
                                item2 = value;
                            }
                            Err(e) => {
                                print_error(format!(
                                    "\nCode:\n{} | {}\nProblem: {}",
                                    line_number,
                                    string_line.clone(),
                                    e
                                ));
                            }
                        }
                    }

                    if item1.parse::<f64>().unwrap() > item2.parse::<f64>().unwrap() {
                        if !labels.contains_key(&label_name) {
                            print_error(format!(
                                "\nCode:\n{} | {}\nProblem: Label `{}` does not exist.",
                                line_number,
                                string_line.clone(),
                                label_name
                            ));
                        }

                        let label_code = labels.get(&label_name).unwrap().clone();

                        interpret(label_code, variables, labels);
                    }
                }
            }

            "jmp_lt" => {
                if args_len != 3 {
                    print_error(format!(
                        "\nCode:\n{} | {}\nProblem: Expected 3 arguments, got {}.",
                        line_number,
                        string_line.clone(),
                        args_len
                    ));
                } else {
                    let mut item1 = args[0].clone();
                    let mut item2 = args[1].clone();
                    let label_name = args[2].clone();

                    if get_type(item1.clone()) == ast::Types::Identifier {
                        match get_variable(item1.clone(), variables.clone()) {
                            Ok((value, value_type)) => {
                                if value_type != ast::Types::Number {
                                    print_error(format!("\nCode:\n{} | {}\nProblem: Variable `{}` is of type `{}`, but `Number` is required.", line_number, string_line.clone(), item1, value_type));
                                }
                                item1 = value;
                            }
                            Err(e) => {
                                print_error(format!(
                                    "\nCode:\n{} | {}\nProblem: {}",
                                    line_number,
                                    string_line.clone(),
                                    e
                                ));
                            }
                        }
                    }

                    if get_type(item2.clone()) == ast::Types::Identifier {
                        match get_variable(item2.clone(), variables.clone()) {
                            Ok((value, value_type)) => {
                                if value_type != ast::Types::Number {
                                    print_error(format!("\nCode:\n{} | {}\nProblem: Variable `{}` is of type `{}`, but `Number` is required.", line_number, string_line.clone(), item2, value_type));
                                }
                                item2 = value;
                            }
                            Err(e) => {
                                print_error(format!(
                                    "\nCode:\n{} | {}\nProblem: {}",
                                    line_number,
                                    string_line.clone(),
                                    e
                                ));
                            }
                        }
                    }

                    if get_type(item1.clone()) != ast::Types::Number {
                        print_error(format!("\nCode:\n{} | {}\nProblem: Cannot compare `{}` and `{}` as `{}` is not a numeric type.", line_number, string_line.clone(), args[0].clone(), args[1].clone(), args[0].clone()));
                    }

                    if get_type(item2.clone()) != ast::Types::Number {
                        print_error(format!("\nCode:\n{} | {}\nProblem: Cannot compare `{}` and `{}` as `{}` is not a numeric type.", line_number, string_line.clone(), args[0].clone(), args[1].clone(), args[1].clone()));
                    }

                    if item1.parse::<f64>().unwrap() < item2.parse::<f64>().unwrap() {
                        if !labels.contains_key(&label_name) {
                            print_error(format!(
                                "\nCode:\n{} | {}\nProblem: Label `{}` does not exist.",
                                line_number,
                                string_line.clone(),
                                label_name
                            ));
                        }

                        let label_code = labels.get(&label_name).unwrap().clone();

                        interpret(label_code, variables, labels);
                    }
                }
            }

            "jmp_eq" => {
                if args_len != 3 {
                    print_error(format!(
                        "\nCode:\n{} | {}\nProblem: Expected 3 arguments, got {}.",
                        line_number,
                        string_line.clone(),
                        args_len
                    ));
                } else {
                    let mut item1 = args[0].clone();
                    let mut item1_type: ast::Types = get_type(item1.clone());

                    let mut item2 = args[1].clone();
                    let mut item2_type: ast::Types = get_type(item2.clone());

                    let label_name = args[2].clone();

                    if item1_type.clone() == ast::Types::Identifier {
                        match get_variable(item1.clone(), variables.clone()) {
                            Ok((value, value_type)) => {
                                item1 = value;
                                item1_type = value_type;
                            }
                            Err(e) => {
                                print_error(format!(
                                    "\nCode:\n{} | {}\nProblem: {}",
                                    line_number,
                                    string_line.clone(),
                                    e
                                ));
                            }
                        }
                    }

                    if item2_type.clone() == ast::Types::Identifier {
                        match get_variable(item2.clone(), variables.clone()) {
                            Ok((value, value_type)) => {
                                item2 = value;
                                item2_type = value_type;
                            }
                            Err(e) => {
                                print_error(format!(
                                    "\nCode:\n{} | {}\nProblem: {}",
                                    line_number,
                                    string_line.clone(),
                                    e
                                ));
                            }
                        }
                    }

                    if item1_type != item2_type {
                        print_error(format!("\nCode:\n{} | {}\nProblem: Cannot compare `{}` and `{}` as they are not the same type.", line_number, string_line.clone(), args[0].clone(), args[1].clone()));
                    }

                    if item1 == item2 {
                        if !labels.contains_key(&label_name) {
                            print_error(format!(
                                "\nCode:\n{} | {}\nProblem: Label `{}` does not exist.",
                                line_number,
                                string_line.clone(),
                                label_name
                            ));
                        }

                        let label_code = labels.get(&label_name).unwrap().clone();

                        interpret(label_code, variables, labels);
                    }
                }
            }

            "jmp_not_eq" => {
                if args_len != 3 {
                    print_error(format!(
                        "\nCode:\n{} | {}\nProblem: Expected 3 arguments, got {}.",
                        line_number,
                        string_line.clone(),
                        args_len
                    ));
                } else {
                    let mut item1 = args[0].clone();
                    let mut item1_type: ast::Types = ast::Types::Number;

                    let mut item2 = args[1].clone();
                    let mut item2_type: ast::Types = ast::Types::Number;

                    let label_name = args[2].clone();

                    if get_type(item1.clone()) == ast::Types::Identifier {
                        match get_variable(item1.clone(), variables.clone()) {
                            Ok((value, value_type)) => {
                                item1 = value;
                                item1_type = value_type;
                            }
                            Err(e) => {
                                print_error(format!(
                                    "\nCode:\n{} | {}\nProblem: {}",
                                    line_number,
                                    string_line.clone(),
                                    e
                                ));
                            }
                        }
                    }

                    if get_type(item2.clone()) == ast::Types::Identifier {
                        match get_variable(item2.clone(), variables.clone()) {
                            Ok((value, value_type)) => {
                                item2 = value;
                                item2_type = value_type;
                            }
                            Err(e) => {
                                print_error(format!(
                                    "\nCode:\n{} | {}\nProblem: {}",
                                    line_number,
                                    string_line.clone(),
                                    e
                                ));
                            }
                        }
                    }

                    if item1_type != item2_type {
                        print_error(format!("\nCode:\n{} | {}\nProblem: Cannot compare `{}` and `{}` as they are not the same type.", line_number, string_line.clone(), args[0].clone(), args[1].clone()));
                    }

                    if item1 != item2 {
                        if !labels.contains_key(&label_name) {
                            print_error(format!(
                                "\nCode:\n{} | {}\nProblem: Label `{}` does not exist.",
                                line_number,
                                string_line.clone(),
                                label_name
                            ));
                        }

                        let label_code = labels.get(&label_name).unwrap().clone();

                        interpret(label_code, variables, labels);
                    }
                }
            }

            "return" => {
                if args_len != 1 {
                    print_error(format!(
                        "\nCode:\n{} | {}\nProblem: Expected 1 argument, got {}.",
                        line_number,
                        string_line.clone(),
                        args_len
                    ));
                } else {
                    let mut item1 = args[0].clone();

                    if get_type(item1.clone()) == ast::Types::Identifier {
                        match get_variable(item1.clone(), variables.clone()) {
                            Ok((value, _)) => {
                                item1 = value;
                            }
                            Err(e) => {
                                print_error(format!(
                                    "\nCode:\n{} | {}\nProblem: {}",
                                    line_number,
                                    string_line.clone(),
                                    e
                                ));
                            }
                        }
                    }

                    variables.insert("TEMP".to_string(), item1);
                }
            }

            "get_os" => {
                if args_len != 0 {
                    print_error(format!(
                        "\nCode:\n{} | {}\nProblem: Expected 0 arguments, got {}.",
                        line_number,
                        string_line.clone(),
                        args_len
                    ));
                } else {
                    variables.insert("TEMP".to_string(), env::consts::OS.to_string());
                }
            }

            "cmd" => {
                if args_len != 1 {
                    print_error(format!(
                        "\nCode:\n{} | {}\nProblem: Expected 1 argument, got {}.",
                        line_number,
                        string_line.clone(),
                        args_len
                    ));
                } else {
                    let mut cmd = args[0].clone();

                    if get_type(cmd.clone()) == ast::Types::Identifier {
                        match get_variable(cmd.clone(), variables.clone()) {
                            Ok((value, _)) => {
                                cmd = value;
                            }
                            Err(e) => {
                                print_error(format!(
                                    "\nCode:\n{} | {}\nProblem: {}",
                                    line_number,
                                    string_line.clone(),
                                    e
                                ));
                            }
                        }
                    }

                    if get_type(cmd.clone()) != ast::Types::String {
                        print_error(format!(
                            "\nCode:\n{} | {}\nProblem: Cannot execute `{}` as it is not a string.",
                            line_number,
                            string_line.clone(),
                            cmd
                        ));
                    }

                    let cmd = cmd.split(' ').collect::<Vec<&str>>();

                    let mut command_to_execute = Command::new(cmd[0]);

                    for i in cmd.iter().skip(1) {
                        command_to_execute.arg(i);
                    }

                    match command_to_execute.output() {
                        Ok(_) => {}

                        Err(e) => {
                            print_error(format!(
                                "\nCode:\n{} | {}\nProblem: Failed to execute command `{}`: {}",
                                line_number,
                                string_line.clone(),
                                get_string_content(cmd.join(" ").clone()),
                                e
                            ));
                        }
                    }
                }
            }

            "input" => {
                if args_len != 0 {
                    print_error(format!(
                        "\nCode:\n{} | {}\nProblem: Expected 0 arguments, got {}.",
                        line_number,
                        string_line.clone(),
                        args_len
                    ));
                } else {
                    let mut input = String::new();

                    match std::io::stdout().flush() {
                        Ok(_) => {}

                        Err(e) => {
                            print_error(format!(
                                "\nCode:\n{} | {}\nProblem: Failed to flush stdout: {}",
                                line_number,
                                string_line.clone(),
                                e
                            ));
                        }
                    }
                    std::io::stdin().read_line(&mut input).unwrap();

                    variables.insert(
                        "TEMP".to_string(),
                        "\"".to_owned() + &input.trim().to_owned() + "\"",
                    );
                }
            }

            "to_number" => {
                if args_len != 1 {
                    print_error(format!(
                        "\nCode:\n{} | {}\nProblem: Expected 1 argument, got {}.",
                        line_number,
                        string_line.clone(),
                        args_len
                    ));
                } else {
                    let mut item = args[0].clone();

                    if get_type(item.clone()) == ast::Types::Identifier {
                        match get_variable(item.clone(), variables.clone()) {
                            Ok((value, _)) => {
                                item = value;
                            }
                            Err(e) => {
                                print_error(format!(
                                    "\nCode:\n{} | {}\nProblem: {}",
                                    line_number,
                                    string_line.clone(),
                                    e
                                ));
                            }
                        }
                    }

                    if get_type(item.clone()) == ast::Types::String {
                        item = get_string_content(item.clone());
                    }

                    match item.parse::<f64>() {
                        Ok(_) => {
                            variables.insert("TEMP".to_string(), item.to_string());
                        }
                        Err(_) => {
                            print_error(format!(
                                "\nCode:\n{} | {}\nProblem: Cannot convert `{}` to a number.",
                                line_number,
                                string_line.clone(),
                                item
                            ));
                        }
                    }
                }
            }

            "to_string" => {
                if args_len != 1 {
                    print_error(format!(
                        "\nCode:\n{} | {}\nProblem: Expected 1 argument, got {}.",
                        line_number,
                        string_line.clone(),
                        args_len
                    ));
                } else {
                    let mut item = args[0].clone();

                    if get_type(item.clone()) == ast::Types::Identifier {
                        match get_variable(item.clone(), variables.clone()) {
                            Ok((value, _)) => {
                                item = value;
                            }
                            Err(e) => {
                                print_error(format!(
                                    "\nCode:\n{} | {}\nProblem: {}",
                                    line_number,
                                    string_line.clone(),
                                    e
                                ));
                            }
                        }
                    }

                    let item_type = get_type(item.clone());

                    if item_type.clone() == ast::Types::String {
                        // Do nothing
                    } else if item_type.clone() == ast::Types::Number {
                        item = "\"".to_owned() + &item.to_string() + "\"";
                    } else {
                        print_error(format!(
                            "\nCode:\n{} | {}\nProblem: Cannot convert `{}` to a string.",
                            line_number,
                            string_line.clone(),
                            item
                        ));
                    }

                    variables.insert("TEMP".to_string(), item);
                }
            }

            "read_file" => {
                if args_len != 1 {
                    print_error(format!(
                        "\nCode:\n{} | {}\nProblem: Expected 1 argument, got {}.",
                        line_number,
                        string_line.clone(),
                        args_len
                    ));
                } else {
                    let mut item = args[0].clone();

                    if get_type(item.clone()) == ast::Types::Identifier {
                        match get_variable(item.clone(), variables.clone()) {
                            Ok((value, _)) => {
                                item = value;
                            }
                            Err(e) => {
                                print_error(format!(
                                    "\nCode:\n{} | {}\nProblem: {}",
                                    line_number,
                                    string_line.clone(),
                                    e
                                ));
                            }
                        }
                    }

                    if get_type(item.clone()) != ast::Types::String {
                        print_error(format!("\nCode:\n{} | {}\nProblem: Cannot read file `{}` as it is not a string.", line_number, string_line.clone(), item));
                    }

                    let mut contents = String::new();

                    match File::open(get_string_content(item.clone())) {
                        Ok(mut file) => match file.read_to_string(&mut contents) {
                            Ok(_) => {
                                variables
                                    .insert("TEMP".to_string(), "\"".to_owned() + &contents + "\"");
                            }
                            Err(e) => {
                                print_error(format!(
                                    "\nCode:\n{} | {}\nProblem: Failed to read file `{}`: {}",
                                    line_number,
                                    string_line.clone(),
                                    item,
                                    e
                                ));
                            }
                        },

                        Err(e) => {
                            print_error(format!(
                                "\nCode:\n{} | {}\nProblem: Failed to open file `{}`: {}",
                                line_number,
                                string_line.clone(),
                                item,
                                e
                            ));
                        }
                    }
                }
            }

            "is_match" => {
                if args_len != 2 {
                    print_error(format!(
                        "\nCode:\n{} | {}\nProblem: Expected 2 arguments, got {}.",
                        line_number,
                        string_line.clone(),
                        args_len
                    ));
                } else {
                    let mut item = args[0].clone();
                    let item_type = get_type(item.clone());

                    if item_type.clone() == ast::Types::Identifier {
                        match get_variable(item.clone(), variables.clone()) {
                            Ok((value, _)) => {
                                item = value;
                            }
                            Err(e) => {
                                print_error(format!(
                                    "\nCode:\n{} | {}\nProblem: {}",
                                    line_number,
                                    string_line.clone(),
                                    e
                                ));
                            }
                        }
                    }

                    if item_type.clone() != ast::Types::String {
                        print_error(format!(
                            "\nCode:\n{} | {}\nProblem: Cannot check if `{}` is a match for `{}` as it is not a string.",
                            line_number,
                            string_line.clone(),
                            item,
                            args[1].clone()
                        ));
                    }

                    let mut pattern = args[1].clone();
                    let mut pattern_type = get_type(pattern.clone());

                    if pattern_type == ast::Types::Identifier {
                        match get_variable(pattern.clone(), variables.clone()) {
                            Ok((value, value_type)) => {
                                pattern = value;
                                pattern_type = value_type;
                            }
                            Err(e) => {
                                print_error(format!(
                                    "\nCode:\n{} | {}\nProblem: {}",
                                    line_number,
                                    string_line.clone(),
                                    e
                                ));
                            }
                        }
                    }

                    if pattern_type != ast::Types::String {
                        print_error(format!(
                            "\nCode:\n{} | {}\nProblem: Pattern `{}` is not of type String.",
                            line_number,
                            string_line.clone(),
                            args[1].clone()
                        ));
                    }

                    pattern = get_string_content(pattern);

                    if let Ok(re) = Regex::new(&pattern) {
                        if re.is_match(&item) {
                            variables.insert("TEMP".to_string(), "1".to_string());
                        } else {
                            variables.insert("TEMP".to_string(), "0".to_string());
                        }
                    } else {
                        print_error(format!(
                            "\nCode:\n{} | {}\nProblem: Pattern `{}` is not a valid regular expression.",
                            line_number,
                            string_line.clone(),
                            args[1].clone()
                        ));
                    }
                }
            }

            "count_matches" => {
                if args_len != 2 {
                    print_error(format!(
                        "\nCode:\n{} | {}\nProblem: Expected 2 arguments, got {}.",
                        line_number,
                        string_line.clone(),
                        args_len
                    ));
                } else {
                    let mut item = args[0].clone();
                    let item_type = get_type(item.clone());

                    if item_type.clone() == ast::Types::Identifier {
                        match get_variable(item.clone(), variables.clone()) {
                            Ok((value, _)) => {
                                item = value;
                            }
                            Err(e) => {
                                print_error(format!(
                                    "\nCode:\n{} | {}\nProblem: {}",
                                    line_number,
                                    string_line.clone(),
                                    e
                                ));
                            }
                        }
                    }

                    if item_type.clone() != ast::Types::String {
                        print_error(format!(
                            "\nCode:\n{} | {}\nProblem: Cannot check if `{}` is a match for `{}` as it is not a string.",
                            line_number,
                            string_line.clone(),
                            item,
                            args[1].clone()
                        ));
                    }

                    let mut pattern = args[1].clone();
                    let mut pattern_type = get_type(pattern.clone());

                    if pattern_type == ast::Types::Identifier {
                        match get_variable(pattern.clone(), variables.clone()) {
                            Ok((value, value_type)) => {
                                pattern = value;
                                pattern_type = value_type;
                            }
                            Err(e) => {
                                print_error(format!(
                                    "\nCode:\n{} | {}\nProblem: {}",
                                    line_number,
                                    string_line.clone(),
                                    e
                                ));
                            }
                        }
                    }

                    if pattern_type != ast::Types::String {
                        print_error(format!(
                            "\nCode:\n{} | {}\nProblem: Pattern `{}` is not of type String.",
                            line_number,
                            string_line.clone(),
                            args[1].clone()
                        ));
                    }

                    pattern = get_string_content(pattern);

                    if let Ok(re) = Regex::new(&pattern) {
                        variables.insert("TEMP".to_string(), re.find_iter(&item).count().to_string());
                    } else {
                        print_error(format!(
                            "\nCode:\n{} | {}\nProblem: Pattern `{}` is not a valid regular expression.",
                            line_number,
                            string_line.clone(),
                            args[1].clone()
                        ));
                    }
                }
            }

            "replace_n" => {
                if args_len != 4 {
                    print_error(format!(
                        "\nCode:\n{} | {}\nProblem: Expected 3 arguments, got {}.",
                        line_number,
                        string_line.clone(),
                        args_len
                    ));
                } else {
                    let mut item = args[0].clone();
                    let item_type = get_type(item.clone());

                    if item_type.clone() == ast::Types::Identifier {
                        match get_variable(item.clone(), variables.clone()) {
                            Ok((value, _)) => {
                                item = value;
                            }
                            Err(e) => {
                                print_error(format!(
                                    "\nCode:\n{} | {}\nProblem: {}",
                                    line_number,
                                    string_line.clone(),
                                    e
                                ));
                            }
                        }
                    }

                    if item_type.clone() != ast::Types::String {
                        print_error(format!(
                            "\nCode:\n{} | {}\nProblem: Cannot check if `{}` is a match for `{}` as it is not a string.",
                            line_number,
                            string_line.clone(),
                            item,
                            args[1].clone()
                        ));
                    }

                    let mut pattern = args[1].clone();
                    let mut pattern_type = get_type(pattern.clone());

                    if pattern_type == ast::Types::Identifier {
                        match get_variable(pattern.clone(), variables.clone()) {
                            Ok((value, value_type)) => {
                                pattern = value;
                                pattern_type = value_type;
                            }
                            Err(e) => {
                                print_error(format!(
                                    "\nCode:\n{} | {}\nProblem: {}",
                                    line_number,
                                    string_line.clone(),
                                    e
                                ));
                            }
                        }
                    }

                    if pattern_type != ast::Types::String {
                        print_error(format!(
                            "\nCode:\n{} | {}\nProblem: Pattern `{}` is not of type String.",
                            line_number,
                            string_line.clone(),
                            args[1].clone()
                        ));
                    }

                    pattern = get_string_content(pattern.clone());

                    let mut replacement = args[2].clone();
                    let mut replacement_type = get_type(replacement.clone());

                    if replacement_type == ast::Types::Identifier {
                        match get_variable(replacement.clone(), variables.clone()) {
                            Ok((value, value_type)) => {
                                replacement = value;
                                replacement_type = value_type;
                            }
                            Err(e) => {
                                print_error(format!(
                                    "\nCode:\n{} | {}\nProblem: {}",
                                    line_number,
                                    string_line.clone(),
                                    e
                                ));
                            }
                        }
                    }

                    if replacement_type != ast::Types::String {
                        print_error(format!(
                            "\nCode:\n{} | {}\nProblem: Replacement `{}` is not of type String.",
                            line_number,
                            string_line.clone(),
                            args[2].clone()
                        ));
                    }

                    replacement = get_string_content(replacement);

                    let mut count = args[3].clone();
                    let mut count_type = get_type(count.clone());

                    if count_type == ast::Types::Identifier {
                        match get_variable(replacement.clone(), variables.clone()) {
                            Ok((value, value_type)) => {
                                count = value;
                                count_type = value_type;
                            }
                            Err(e) => {
                                print_error(format!(
                                    "\nCode:\n{} | {}\nProblem: {}",
                                    line_number,
                                    string_line.clone(),
                                    e
                                ));
                            }
                        }
                    }

                    if count_type != ast::Types::Number {
                        print_error(format!(
                            "\nCode:\n{} | {}\nProblem: Replacement `{}` is not of type Number.",
                            line_number,
                            string_line.clone(),
                            args[2].clone()
                        ));
                    }

                    if let Ok(count) = count.parse::<usize>() {
                        let re = Regex::new(&pattern).unwrap();
                        let result = re.replacen(&item, count, &replacement);

                        variables.insert("TEMP".to_string(), result.to_string());
                    } else {
                        print_error(format!(
                            "\nCode:\n{} | {}\nProblem: `{}` is supposed to be an integer less than 18446744073709551616.",
                            line_number,
                            string_line.clone(),
                            args[3].clone()
                        ));
                    }
                }
            }

            "replace_all" => {
                if args_len != 3 {
                    print_error(format!(
                        "\nCode:\n{} | {}\nProblem: Expected 3 arguments, got {}.",
                        line_number,
                        string_line.clone(),
                        args_len
                    ));
                } else {
                    let mut item = args[0].clone();
                    let item_type = get_type(item.clone());

                    if item_type.clone() == ast::Types::Identifier {
                        match get_variable(item.clone(), variables.clone()) {
                            Ok((value, _)) => {
                                item = value;
                            }
                            Err(e) => {
                                print_error(format!(
                                    "\nCode:\n{} | {}\nProblem: {}",
                                    line_number,
                                    string_line.clone(),
                                    e
                                ));
                            }
                        }
                    }

                    if item_type.clone() != ast::Types::String {
                        print_error(format!(
                            "\nCode:\n{} | {}\nProblem: Cannot check if `{}` is a match for `{}` as it is not a string.",
                            line_number,
                            string_line.clone(),
                            item,
                            args[1].clone()
                        ));
                    }

                    let mut pattern = args[1].clone();
                    let mut pattern_type = get_type(pattern.clone());

                    if pattern_type == ast::Types::Identifier {
                        match get_variable(pattern.clone(), variables.clone()) {
                            Ok((value, value_type)) => {
                                pattern = value;
                                pattern_type = value_type;
                            }
                            Err(e) => {
                                print_error(format!(
                                    "\nCode:\n{} | {}\nProblem: {}",
                                    line_number,
                                    string_line.clone(),
                                    e
                                ));
                            }
                        }
                    }

                    if pattern_type != ast::Types::String {
                        print_error(format!(
                            "\nCode:\n{} | {}\nProblem: Pattern `{}` is not of type String.",
                            line_number,
                            string_line.clone(),
                            args[1].clone()
                        ));
                    }

                    pattern = get_string_content(pattern.clone());

                    let mut replacement = args[2].clone();
                    let mut replacement_type = get_type(replacement.clone());

                    if replacement_type == ast::Types::Identifier {
                        match get_variable(replacement.clone(), variables.clone()) {
                            Ok((value, value_type)) => {
                                replacement = value;
                                replacement_type = value_type;
                            }
                            Err(e) => {
                                print_error(format!(
                                    "\nCode:\n{} | {}\nProblem: {}",
                                    line_number,
                                    string_line.clone(),
                                    e
                                ));
                            }
                        }
                    }

                    if replacement_type != ast::Types::String {
                        print_error(format!(
                            "\nCode:\n{} | {}\nProblem: Replacement `{}` is not of type String.",
                            line_number,
                            string_line.clone(),
                            args[2].clone()
                        ));
                    }

                    replacement = get_string_content(replacement);

                    if let Ok(re) = Regex::new(&pattern) {
                        variables.insert(
                            "TEMP".to_string(),
                            re.replace_all(&item, &replacement).to_string(),
                        );
                    } else {
                        print_error(format!(
                            "\nCode:\n{} | {}\nProblem: Pattern `{}` is not a valid regular expression.",
                            line_number,
                            string_line.clone(),
                            args[1].clone()
                        ));
                    }
                }
            }

            "comment" => {
                // Do nothing
            }

            _ => {
                print_error(format!(
                    "\nCode:\n{} | {}\nProblem: Unknown command `{}`.",
                    line_number,
                    string_line.clone(),
                    command.clone()
                ));
            }
        }
    }
}

fn main() {
    let mut args = args();
    args.next().unwrap();

    match args.next() {
        Some(input_file) => match fs::read_to_string(input_file.clone()) {
            Ok(code) => {
                let (lexed_code, lexing_err) = lexer::lex(code);

                if lexing_err != Error::None {
                    if let Error::LexingError(err) = lexing_err {
                        print_error(err);
                    }
                }

                let mut variables: HashMap<String, String> = HashMap::new();
                let mut labels: HashMap<String, Vec<(usize, lexer::Line)>> = HashMap::new();
                let mut current_label: String = "".to_string();
                let mut label_code: Vec<(usize, lexer::Line)> = Vec::new();

                for (line_number, line) in lexed_code.iter().enumerate() {
                    let line_number = line_number + 1;
                    let line: Vec<String> = line.clone().0;
                    if line.is_empty() {
                        continue;
                    }
                    let string_line = line.clone().join(" ");

                    let command: String = line[0].clone();
                    let args: Vec<String> = line[1..].to_vec().clone();
                    let args_len = args.len();

                    match command.clone().as_str() {
                        "label" => {
                            if args_len != 1 {
                                print_error(format!(
                                    "\nCode:\n{} | {}\nProblem: Expected 1 argument, got {}.",
                                    line_number,
                                    string_line.clone(),
                                    args_len
                                ));
                            } else {
                                labels.insert(current_label.clone(), label_code);
                                label_code = Vec::new();

                                let label_name = args[0].clone();
                                current_label = label_name.clone();

                                if labels.contains_key(&label_name) {
                                    print_error(format!(
                                        "\nCode:\n{} | {}\nProblem: Label `{}` already exists.",
                                        line_number,
                                        string_line.clone(),
                                        label_name
                                    ));
                                }
                            }
                        }

                        _ => {
                            label_code.push((line_number, lexer::Line(line.clone())));
                        }
                    }

                    if line_number == lexed_code.len() {
                        labels.insert(current_label.clone(), label_code);
                        label_code = Vec::new();
                    }
                }

                if labels.clone().contains_key(".ENTRY") {
                    let entry_code = labels.get(".ENTRY").unwrap().clone();
                    interpret(entry_code, &mut variables, &mut labels);
                } else {
                    print_error(
                        "\nError: Could not execute\nProblem: No `.ENTRY` label.".to_string(),
                    );
                }
            }

            Err(e) => print_error(format!(
                "Error: Could not open file `{}`\nProblem: {}",
                input_file, e
            )),
        },

        None => {
            println!("Usage: script-ll <source_code>.ll\nExample: script-ll examples/tutorial.ll");
        }
    }
}
