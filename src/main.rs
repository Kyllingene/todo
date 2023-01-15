use std::{
    fs::{read_to_string, OpenOptions},
    io::Write,
    process::exit,
    str::FromStr,
};

use sarge::*;
use todo_lib::*;

fn main() {
    let mut parser = ArgumentParser::new();
    parser.add(arg!(flag, both, 'h', "help"));

    parser.add(arg!(str, both, 'n', "new"));

    parser.add(arg!(str, both, 'c', "complete"));

    parser.add(arg!(flag, both, 'l', "list"));

    parser.add(arg!(str, both, 'f', "file"));

    let _remainder = match parser.parse() {
        Err(e) => {
            eprintln!("error (while parsing arguments): {e}");
            exit(1);
        }
        Ok(r) => r,
    };

    let filename = get_val!(parser, both, 'f', "file")
        .unwrap_or(ArgValue::String("todo.txt".into()))
        .get_str();

    if get_flag!(parser, both, 'h', "help") {
        println!("{} [options]", parser.binary.unwrap_or("todo".to_string()));
        println!("  -h /     --help        : prints this help message");
        println!("  -n /      --new <todo> : creates a new todo, with the given text");
        println!("  -c / --complete <todo> : completes the todo, specified by the given text");
        println!("  -l /     --list        : prints this help message");
        println!("  -f /     --file <file> : specifies the file (defaults to todo.txt)");

        exit(0);
    }

    let mut todos = TodoTable::new(None::<char>);
    todos.add_col("Todos");

    let todo_txt = match read_to_string(filename.clone()) {
        Ok(s) => s,
        Err(e) => {
            if e.raw_os_error().unwrap_or(0) == 2 {
                "".to_string()
            } else {
                eprintln!("error (while reading file): {}", e);
                exit(1);
            }
        }
    };

    let mut file = match OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(filename)
    {
        Ok(file) => file,
        Err(e) => {
            eprintln!("error (while opening file to write): {}", e);
            exit(1);
        }
    };

    for line in todo_txt.lines() {
        todos.add_todo(
            Todo::from_str(line).unwrap_or_else(|e| {
                eprintln!("invalid todo: {}", e);
                exit(1);
            }),
            "Todos",
        );
    }

    let mut action = false;
    let mut changed = false;
    if let Some(ArgValue::String(todo)) = get_val!(parser, both, 'n', "new") {
        todos.add_todo(
            Todo::from_str(&todo).unwrap_or_else(|e| {
                eprintln!("invalid todo: {}", e);
                exit(1);
            }),
            "Todos",
        );

        action = true;
        changed = true;
    }

    if let Some(ArgValue::String(todo)) = get_val!(parser, both, 'c', "complete") {
        todos
            .get_todo(todo.clone(), "Todos".to_string())
            .unwrap_or_else(|| {
                eprintln!("couldn't find todo {}", todo);
                exit(1);
            })
            .complete();

        action = true;
        changed = true;
    }

    if get_flag!(parser, both, 'l', "list") || !action {
        todos.col("Todos").unwrap().todos.iter().for_each(|t| {
            println!("{}", t.to_string());
        });
    }

    if changed {
        todos.col("Todos").unwrap().todos.iter().for_each(|t| {
            writeln!(file, "{}", t.to_string()).unwrap_or_else(|e| {
                eprintln!("error (while writing to file): {}", e);
                exit(1);
            });
        });

        file.flush().unwrap_or_else(|e| {
            eprintln!("error (while writing to file): {}", e);
            exit(1);
        });
    };
}
