use std::{
    fs::{read_to_string, OpenOptions},
    io::Write,
    path::Path,
    process::exit,
    str::FromStr,
};

use dirs::home_dir;
use sarge::*;
use todo_lib::*;

struct Config {
    pub source: Option<String>,
}

fn get_config(filename: String) -> Config {
    let config_txt = read_to_string(filename).unwrap_or("".to_string());

    let mut config = TodoTable::new(Some("Config"));
    config.add_col("Config");
    for line in config_txt.lines() {
        config.add_todo(
            Todo::from_str(line).unwrap_or_else(|e| {
                eprintln!("invalid config line: {e}");
                exit(1);
            }),
            "Config",
        );
    }

    let mut src = None;

    if let Some(source) = config.get_todo("source", "Config") {
        if let Some(source) = source.get_meta("path") {
            src = Some(source);
        }
    }

    Config {
        source: src.cloned(),
    }
}

fn main() {
    let mut parser = ArgumentParser::new();
    parser.add(arg!(flag, both, 'h', "help"));

    parser.add(arg!(str, both, 'n', "new"));

    parser.add(arg!(str, both, 'c', "complete"));

    parser.add(arg!(flag, both, 'l', "list"));

    parser.add(arg!(str, both, 'f', "file"));

    parser.add(arg!(str, double, "config"));

    let _remainder = match parser.parse() {
        Err(e) => {
            eprintln!("error (while parsing arguments): {e}");
            exit(1);
        }
        Ok(r) => r,
    };

    if get_flag!(parser, both, 'h', "help") {
        println!("{} [options]", parser.binary.unwrap_or("todo".to_string()));
        println!("      --help / -h        : prints this help message");
        println!("       --new / -n <todo> : creates a new todo, with the given text");
        println!("  --complete / -c <todo> : completes the todo, specified by the given text");
        println!("      --list / -l        : prints this help message");
        println!("    --config      <file> : specifies the config file");
        println!("                             defaults to ~/.todo-cfg.txt");
        println!("      --file / -f <file> : specifies the source file");
        println!("                           if todo.txt exists in the current directory,");
        println!("                           defaults to that; otherwise, defaults to config");
        println!();
        println!("Config is in the todo.txt format, using metadata.");
        println!("Default config:");
        println!("```");
        println!("source path:~/todo.txt");
        // println!("x sort list");
        println!("```");

        exit(0);
    }

    let config = get_config(match get_val!(parser, double, "config") {
        Some(ArgValue::String(path)) => path,
        _ => {
            let mut home = home_dir().unwrap_or_else(|| {
                eprintln!("error: failed to get home directory");
                exit(1)
            });

            home.push(".todo-cfg.txt");

            home.display().to_string()
        }
    });

    let filename: String;
    if let Some(ArgValue::String(f)) = get_val!(parser, both, 'f', "file") {
        filename = f;
    } else if Path::new("todo.txt").exists() {
        filename = String::from("todo.txt");
    } else if let Some(path) = config.source {
        filename = path;
    } else {
        let mut path = home_dir().unwrap_or_else(|| {
            eprintln!("error: failed to get home directory");
            exit(1);
        });
        path.push("todo.txt");

        filename = path.display().to_string();
    }

    let mut todos = TodoTable::new(None::<char>);
    todos.add_col("Todos");

    let changed = get_val!(parser, both, 'n', "new").is_some()
        || get_val!(parser, both, 'c', "complete").is_some();

    let todo_txt = match read_to_string(filename.clone()) {
        Ok(s) => s,
        Err(e) => {
            if e.raw_os_error().unwrap_or(0) == 2 {
                "".to_string()
            } else {
                eprintln!("error (while reading file): {e}");
                exit(1);
            }
        }
    };

    for line in todo_txt.lines() {
        todos.add_todo(
            Todo::from_str(line).unwrap_or_else(|e| {
                eprintln!("invalid todo: {e}");
                exit(1);
            }),
            "Todos",
        );
    }

    let mut action = false;
    if let Some(ArgValue::String(todo)) = get_val!(parser, both, 'n', "new") {
        todos.add_todo(
            Todo::from_str(&todo).unwrap_or_else(|e| {
                eprintln!("invalid todo: {e}");
                exit(1);
            }),
            "Todos",
        );

        action = true;
    }

    if let Some(ArgValue::String(todo)) = get_val!(parser, both, 'c', "complete") {
        todos
            .get_todo(todo.clone(), "Todos".to_string())
            .unwrap_or_else(|| {
                eprintln!("couldn't find todo {todo}");
                exit(1);
            })
            .complete();

        action = true;
    }

    if get_flag!(parser, both, 'l', "list") || !action {
        todos.col("Todos").unwrap().todos.iter().for_each(|t| {
            println!("{t}");
        });
    }

    if changed {
        let mut file = match OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(filename)
        {
            Ok(file) => file,
            Err(e) => {
                eprintln!("error (while opening file to write): {e}");
                exit(1);
            }
        };

        todos.col("Todos").unwrap().todos.iter().for_each(|t| {
            writeln!(file, "{t}").unwrap_or_else(|e| {
                eprintln!("error (while writing to file): {e}");
                exit(1);
            });
        });

        file.flush().unwrap_or_else(|e| {
            eprintln!("error (while writing to file): {e}");
            exit(1);
        });
    };
}
