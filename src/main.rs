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

#[derive(Default)]
struct Config {
    pub source: Option<String>,
    pub archive: Option<String>,
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

    let mut cfg = Config::default();

    if let Some(source) = config.get_todo("source", "Config") {
        if let Some(source) = source.get_meta("path") {
            cfg.source = Some(source.clone());
        }
    }

    if let Some(archive) = config.get_todo("archive", "Config") {
        if let Some(archive) = archive.get_meta("path") {
            cfg.archive = Some(archive.clone());
        }
    }

    cfg
}

fn main() {
    let mut parser = ArgumentParser::new();
    parser.add(arg!(flag, both, 'h', "help"));
    parser.add(arg!(str, both, 'n', "new"));
    parser.add(arg!(str, both, 'c', "complete"));
    parser.add(arg!(flag, both, 'l', "list"));
    parser.add(arg!(str, both, 'f', "file"));
    parser.add(arg!(str, long, "context"));
    parser.add(arg!(str, long, "project"));
    parser.add(arg!(str, long, "config"));
    parser.add(arg!(flag, both, 'a', "archive"));

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
        println!("                           parses all metadata tags");
        println!("  --complete / -c <todo> : completes the todo, specified by the given text");
        println!("                           if no todo matches the text, looks for a todo with");
        println!("                           that id (using the `id:` tag)");
        println!("      --list / -l        : prints this help message");
        println!("    --config      <file> : specifies the config file");
        println!("                           defaults to ~/.todo-cfg.txt");
        println!("   --project      <tag>  : filters by project tag");
        println!("   --context      <tag>  : filters by context tag");
        println!("   --archive / -a        : archives completed tasks");
        println!("                           default archive file is source + .archive");
        println!("      --file / -f <file> : specifies the source file");
        println!("                           if todo.txt exists in the current directory,");
        println!("                           defaults to that; otherwise, defaults to config");
        println!();
        println!("Config is in the todo.txt format, using metadata:");
        println!("```");
        println!("source path:<SOURCE-PATH> example:~/todo.txt");
        println!("archive path:<ARCHIVE-PATH> example:~/todo.archive.txt");
        // println!("x sort list");
        println!("```");

        exit(0);
    }

    let config = get_config(match get_val!(parser, long, "config") {
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

    if Path::new("todo.txt").exists() {
        filename = String::from("todo.txt");
    } else if let Some(ArgValue::String(f)) = get_val!(parser, both, 'f', "file") {
        filename = f;
    } else if let Some(path) = &config.source {
        filename = path.to_string();
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

    let changed = (get_val!(parser, both, 'n', "new").is_some()
        || get_val!(parser, both, 'c', "complete").is_some())
        && !get_flag!(parser, both, 'a', "archive");

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

    if let Some(ArgValue::String(todo_title)) = get_val!(parser, both, 'c', "complete") {
        let mut todo = todos.get_todo(todo_title.clone(), "Todos".to_string());

        if todo.is_none() {
            todo = todos.get_meta("Todos", "id", todo_title.as_str());
        }

        if let Some(todo) = todo {
            todo.complete();
        } else {
            eprintln!("couldn't find todo {todo_title}");
            exit(1);
        }

        action = true;
    }

    if (get_flag!(parser, both, 'l', "list") || !action) && !get_flag!(parser, both, 'a', "archive")
    {
        use std::cmp::Ordering;
        let mut col = todos.col("Todos").unwrap().todos.clone();

        // TODO: is this a complete sorting?
        col.sort_by(|x, y| {
            if !x.due() & y.due() {
                return Ordering::Greater;
            } else if x.due() & !y.due() {
                return Ordering::Less;
            } else if !x.completed & y.completed {
                return Ordering::Greater;
            } else if x.completed & !y.completed {
                return Ordering::Less;
            }

            if x.priority != y.priority {
                if y.priority.is_none() {
                    return Ordering::Greater;
                } else if x.priority.is_none() {
                    return Ordering::Less;
                }

                return x.priority.cmp(&y.priority);
            }

            match x.deadline {
                TodoDate::Day(dx) => {
                    if let TodoDate::Day(yx) = y.deadline {
                        return dx.cmp(&yx);
                    }
                }
                TodoDate::Daily(dx) => {
                    if let TodoDate::Daily(yx) = y.deadline {
                        return dx.cmp(&yx);
                    }
                }
                TodoDate::Instant(dx) => {
                    if let TodoDate::Instant(yx) = y.deadline {
                        return dx.cmp(&yx);
                    }
                }
                TodoDate::Always => match y.deadline {
                    TodoDate::Always => {}
                    _ => return Ordering::Less,
                },
                _ => {}
            }

            if let Some(cx) = x.created {
                if let Some(cy) = y.created {
                    return cx.cmp(&cy);
                }
            }

            x.title.cmp(&y.title)
        });

        col.iter()
            .filter(|t| {
                if let Some(ArgValue::String(project)) = get_val!(parser, long, "project") {
                    return t.has_project_tag(project);
                }

                true
            })
            .filter(|t| {
                if let Some(ArgValue::String(context)) = get_val!(parser, long, "context") {
                    return t.has_context_tag(context);
                }

                true
            })
            .for_each(|t| {
                println!("{t}");
            });
    }

    if get_flag!(parser, both, 'a', "archive") {
        let mut archive: String;
        if Path::new("todo.txt.archive").exists() || Path::new("todo.txt").exists() {
            archive = String::from("todo.txt.archive");
        } else if let Some(path) = config.archive {
            archive = path;
        } else if config.source.is_some() {
            archive = config.source.unwrap();
            archive.push_str(".archive");
        } else {
            let mut path = home_dir().unwrap_or_else(|| {
                eprintln!("error: failed to get home directory");
                exit(1);
            });
            path.push("todo.txt.archive");

            archive = path.display().to_string();
        }

        let todos = todos.col("Todos").unwrap().todos.clone();

        let keep = todos.iter().filter(|t| !t.completed);

        let arch = todos.iter().filter(|t| t.completed);

        let mut file = match OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(filename.clone())
        {
            Ok(file) => file,
            Err(e) => {
                eprintln!("error (while opening file to write): {e}");
                exit(1);
            }
        };

        keep.for_each(|t| {
            writeln!(file, "{t}").unwrap_or_else(|e| {
                eprintln!("error (while writing to file): {e}");
                exit(1);
            });
        });

        file.flush().unwrap_or_else(|e| {
            eprintln!("error (while writing to file): {e}");
            exit(1);
        });

        let mut file = match OpenOptions::new()
            .create(true)
            .write(true)
            .append(true)
            .open(archive)
        {
            Ok(file) => file,
            Err(e) => {
                eprintln!("error (while opening archive file to write): {e}");
                exit(1);
            }
        };

        arch.for_each(|t| {
            writeln!(file, "{t}").unwrap_or_else(|e| {
                eprintln!("error (while writing to archive file): {e}");
                exit(1);
            });
        });

        file.flush().unwrap_or_else(|e| {
            eprintln!("error (while writing to archive file): {e}");
            exit(1);
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
