use std::{
    fs::{read_to_string, OpenOptions},
    io::Write,
    path::Path,
    panic,
    str::FromStr,
};

use dirs::home_dir;
use sarge::prelude::*;
use todo_lib::{
    colors::{StyleScheme, DEFAULT_STYLE},
    prelude::*,
};

mod helper;
use helper::{log, BLUE, BOLD, ITALIC, RESET, YELLOW, CleanFail};

#[derive(Default, Debug)]
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
            Todo::from_str(line).fail("invalid config line"),
            "Config",
        );
    }

    let mut cfg = Config::default();

    if let Some(source) = config.get_todo("source", "Config") {
        if let Some(source) = source.get_meta("path") {
            cfg.source = Some(source.clone());
        } else {
            log::warn("invalid `source` item in config, skipping");
        }
    }

    if let Some(archive) = config.get_todo("archive", "Config") {
        if let Some(archive) = archive.get_meta("path") {
            cfg.archive = Some(archive.clone());
        } else {
            log::warn("invalid `archive` item in config, skipping");
        }
    }

    cfg
}

struct Args {
    help: bool,
    list: bool,

    new: Option<String>,
    complete: Option<String>,
    context: Option<String>,
    project: Option<String>,

    file: Option<String>,
    archive: bool,
    config: Option<String>,

    minpri: Option<TodoPriority>,
    maxpri: Option<TodoPriority>,
}

fn get_args() -> (String, Args) {
    panic::set_hook(Box::new(|e| {
        if let Some(e) = e.payload().downcast_ref::<&str>() {
            log::err(e);
        } else if let Some(e) = e.payload().downcast_ref::<String>() {
            log::err(e);
        } else {
            log::err(format!("internal error at {}", e.location().unwrap()));
        }

        std::process::exit(1);
    }));
    
    let parser = ArgumentParser::new();

    let help = parser.add(tag::both('h', "help"));
    let list = parser.add(tag::both('l', "list"));

    let new = parser.add(tag::both('n', "new"));
    let complete = parser.add(tag::both('c', "complete"));
    let context = parser.add(tag::long("context"));
    let project = parser.add(tag::long("project"));

    let file = parser.add(tag::both('f', "file"));
    let archive = parser.add(tag::both('a', "archive"));
    let config = parser.add(tag::long("config"));

    let minpri = parser.add::<String>(tag::both('p', "min-priority"));
    let maxpri = parser.add::<String>(tag::both('P', "max-priority"));

    parser.parse().fail("failed to parse arguments");

    let minpri = match minpri.get() {
	Ok(pri) => Some(TodoPriority::try_from(format!("({})", pri.to_uppercase()).as_str()).expect("invalid priority")),
	Err(_) => None,
    };

    let maxpri = match maxpri.get() {
	Ok(pri) => Some(TodoPriority::try_from(format!("({})", pri.to_uppercase()).as_str()).expect("invalid priority")),
	Err(_) => None,
    };

    dbg!(&minpri, &maxpri);

    let args = Args {
        help: help.get().unwrap(),
        list: list.get().unwrap(),

        new: new.get().ok(),
        complete: complete.get().ok(),
        context: context.get().ok(),
        project: project.get().ok(),

        file: file.get().ok(),
        archive: archive.get().unwrap(),
        config: config.get().ok(),

	minpri,
	maxpri: maxpri,
    };

    (parser.binary().unwrap_or("todo".to_string()), args)
}

fn main() {
    let (binary, args) = get_args();

    if args.help {
        println!(
            "{} {YELLOW}{ITALIC}[options]{RESET}\n\
        \x20     {BOLD}--help / -h{RESET}        : prints this help message\n\
        \x20      {BOLD}--new / -n{RESET} {YELLOW}{ITALIC}<todo>{RESET} : creates a new todo, with the given text\n\
        \x20                          parses all metadata tags\n\
        \x20 {BOLD}--complete / -c{RESET} {YELLOW}{ITALIC}<todo>{RESET} : completes the todo, specified by the given text\n\
        \x20                          if no todo matches the text, looks for a todo with\n\
        \x20                          that id (using the `id:` tag)\n\
        \x20     {BOLD}--list / -l{RESET}        : prints this help message\n\
        \x20   {BOLD}--config{RESET}      {YELLOW}{ITALIC}<file>{RESET} : specifies the config file\n\
        \x20                          defaults to ~/.todo-cfg.txt\n\
        \x20  {BOLD}--project{RESET}      {YELLOW}{ITALIC}<tag>{RESET}  : filters by project tag\n\
        \x20  {BOLD}--context{RESET}      {YELLOW}{ITALIC}<tag>{RESET}  : filters by context tag\n\
        \x20  {BOLD}--archive / -a{RESET}        : archives completed tasks\n\
        \x20                          default archive file is source + .archive\n\
        \x20     {BOLD}--file / -f{RESET} {YELLOW}{ITALIC}<file>{RESET} : specifies the source file\n\
        \x20                          if todo.txt exists in the current directory,\n\
        \x20                          defaults to that; otherwise, defaults to config\n\
        \n\
        Config is in the todo.txt format, using metadata:\n\
        \n{ITALIC}{BLUE}\
        source path:<SOURCE-PATH> example:~/todo.txt\n\
        archive path:<ARCHIVE-PATH> example:~/todo.archive.txt{RESET}\n\
        ",
            binary
        );

        return;
    }

    let config = get_config(match args.config {
        Some(path) => path,
        _ => {
            let mut home = home_dir().fail("failed to get home directory");
            home.push(".todo-cfg.txt");

            home.display().to_string()
        }
    });

    let mut filename: String;

    if let Some(f) = args.file {
        filename = f;
    } else if Path::new("todo.txt").exists() {
        filename = String::from("todo.txt");
    } else if let Some(path) = &config.source {
        filename = path.to_string();
    } else {
        let mut path = home_dir().fail("failed to get home directory");
        path.push("todo.txt");

        filename = path.display().to_string();
    }

    if filename.starts_with("~/") {
        filename = filename.replacen(
            "~/",
            &format!(
                "{}{}",
                home_dir().fail("failed to get home directory")
                    .display(),
                std::path::MAIN_SEPARATOR
            ),
            1,
        );
    }

    let mut todos = TodoTable::new(None::<char>);
    todos.add_col("Todos");

    let changed = args.new.is_some() || args.complete.is_some() || args.archive;

    let todo_txt = match read_to_string(filename.clone()) {
        Ok(s) => s,
        Err(e) => {
            if e.raw_os_error().unwrap_or(0) == 2 {
                "".to_string()
            } else {
                panic!("failed to read file: {e}");
            }
        }
    };

    for line in todo_txt.lines() {
        if line.is_empty() || line.chars().all(char::is_whitespace) {
            continue;
        }

        todos.add_todo(
            Todo::from_str(line).fail("invalid todo"),
            "Todos",
        );
    }

    let mut action = false;
    if let Some(todo) = args.new {
        todos.add_todo(
            Todo::from_str(&todo).fail("invalid todo"),
            "Todos",
        );

        action = true;
    }

    if let Some(todo_title) = args.complete {
        let mut todo = todos.get_todo(todo_title.clone(), "Todos".to_string());

        if todo.is_none() {
            todo = todos.get_meta("Todos", "id", todo_title.as_str());
        }

        todo.fail("failed to find todo").complete();

        action = true;
    }

    if (args.list || !action) && !args.archive {
        use std::cmp::Ordering;
        let mut col = todos.col("Todos").unwrap().todos.clone();

        // TODO: is this a total sorting?
        col.sort_by(|x, y| {
            let xdue = x.due();
            let ydue = y.due();
            if !x.completed & y.completed || xdue & !ydue {
                return Ordering::Less;
            } else if x.completed & !y.completed || !xdue & ydue {
                return Ordering::Greater;
            }

            match y.priority.cmp(&x.priority) {
                Ordering::Equal => {}
                ord => return ord,
            }

            match x.deadline {
                TodoDate::Day(dx) => {
                    if let TodoDate::Day(dy) = y.deadline {
                        return dx.cmp(&dy);
                    }
                }
                TodoDate::Always => {
                    if y.deadline != TodoDate::Always {
                        return Ordering::Less;
                    }
                }
                _ => {}
            }

            if let Some(cx) = x.creation {
                if let Some(cy) = y.creation {
                    return cx.cmp(&cy);
                }
            }

            x.description
                .to_string(StyleScheme::default(), "")
                .cmp(&y.description.to_string(StyleScheme::default(), ""))
        });

        col.iter()
            .filter(|t| {
                if let Some(project) = &args.project {
                    return t.has_project_tag(project);
                }

                true
            })
            .filter(|t| {
                if let Some(context) = &args.context {
                    return t.has_context_tag(context);
                }

                true
            })
            .filter(|t| {
		if let Some(minpri) = args.minpri {
		    return t.priority >= minpri;
                }

		true
            })
            .filter(|t| {
		if let Some(maxpri) = args.maxpri {
		    return t.priority <= maxpri;
                }

		true
            })
            .for_each(|t| {
                println!("{}", t.colored(DEFAULT_STYLE));
            });
    } else if args.archive {
        let mut archive: String;
        if Path::new("todo.txt.archive").exists() || Path::new("todo.txt").exists() {
            archive = String::from("todo.txt.archive");
        } else if let Some(path) = config.archive {
            archive = path;
        } else if config.source.is_some() {
            archive = config.source.unwrap();
            archive.push_str(".archive");
        } else {
            let mut path = home_dir().fail("failed to get home directory");
            path.push("todo.txt.archive");

            archive = path.display().to_string();
        }

        let todos = todos.col("Todos").unwrap().todos.clone();

        let keep = todos.iter().filter(|t| !t.completed);

        let archived = todos.iter().filter(|t| t.completed);

        let mut file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(filename.clone())
            .fail("failed to open todo file");

        keep.for_each(|t| {
            writeln!(file, "{t}").fail("failed to write to file");
        });

        file.flush().fail("failed to write to file");

        let mut file = OpenOptions::new()
            .create(true)
            .write(true)
            .append(true)
            .open(archive)
            .fail("failed to open archive file");

        archived.for_each(|t| {
            writeln!(file, "{t}").fail("failed to write to archive file");
        });

        file.flush().fail("failed to write to archive file");
    }

    if changed {
        let mut file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(filename)
            .fail("failed to open todo file");

        todos.col("Todos").unwrap().todos.iter().for_each(|t| {
            writeln!(file, "{t}").fail("failed to write to file");
        });

        file.flush().fail("failed to write to file");
    };
}

