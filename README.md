# AUCPL CLI

A helpful command-line interface for AUCPL problem setters.

> [!WARNING]
> This tool is currently in active development, and breaking changes can occur at any point.

## Getting started

Make sure you have Rust installed. Build the binary:

```sh
cargo build --release
```

## How it works

Problems are stored in a `problems` folder. This can be changed in the `settings.toml` file. Within this folder, there is a `new` and `archive` folder. The `new` folder is for problems that are not yet put into a competition. The `archive` folder is for problems that have already been put into a competition. Within these folders, there are folders denoting the difficulty of the problems starts from `0800` and goes in increments of 200. Unrated problems are stored in the `unrated` folder.

Within each difficulty folder, there are the individual problems. Each of these folders will contain a `problem.md` which is the problem statement. There will be a `tests` folder for test cases and a `solutions` folder for reference solutions.

Lastly, there is a `problem-mappings.json` file that maps the problem names to its stored location. This is so that in the CLI, you do not have to specify things like the rating or whether it's a new or archived problem. You can also use `aucpl sync` to generate or update the mappings.

The general structure of `problems` looks like this:

```
problems/
    archive/
    new/
        0800/
            problem-foo/
                problem.md
                solutions/
                    solution.cpp
                    solution.py
                tests/
                    a.in
                    a.out
                    b.in
                    b.out
        1000/
        1200/
        unrated/
    problem-mappings.json
```

### Terminology

- **Problem name**: The name that's used to reference the problem in the CLI
- **Problem title**: The title of the problem to put in the problem statement
