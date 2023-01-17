# todo
A rusty command-line todo app, using the [todo.txt](https://todotxt.org) format.

Supports the following command-line options:
```
todo [options]
      --help / -h        : prints this help message
       --new / -n <todo> : creates a new todo, with the given text
                           parses all metadata tags
  --complete / -c <todo> : completes the todo, specified by the given text
                           if no todo matches the text, looks for a todo with
                           that id (using the `id:` tag)
      --list / -l        : prints this help message
    --config      <file> : specifies the config file
                           defaults to ~/.todo-cfg.txt
   --project      <tag>  : filters by project tag
   --context      <tag>  : filters by context tag
   --archive / -a        : archives completed tasks
                           default archive file is source + .archive
      --file / -f <file> : specifies the source file
                           if todo.txt exists in the current directory,
                           defaults to that; otherwise, defaults to config
```

The config:
```
source path:<SOURCE-PATH> example:~/todo.txt
archive path:<ARCHIVE-PATH> example:~/todo.archive.txt
```
