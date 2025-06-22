# AUCPL CLI

A helpful command-line interface for AUCPL problem setters.

> [!WARNING]
> This tool is currently in active development, and breaking changes can occur at any point.

## Features

- Automatically generate test case outputs when provided input files and reference solutions
- Run batch test cases on problems
- Easily switch between languages when testing solutions
- Check/validate problems to ensure things are correct (e.g. no missing test cases)
- Easy management and organisation of problems and competitions with options to create, rename, archive, and more
- Generate input test cases from generator scripts
- Compare the outputs of two or more solutions
- Fuzz solutions to see if there are any bugs or unhandled edge cases

Planned:

- Automatic formatting of problems and solution files
- Uploading problems and test cases to an online judge
- Testing code within judge environments
- Improve checking/validation of problems, covering more criteria
- Shell auto completions

## Install

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

### Commands

Here is a list of some of the commands.

Problems

- `aucpl problem create`: Create a new problem and generate necessary files
- `aucpl problem solve`: Automatically generate output test cases for a given problem
- `aucpl problem test`: Automatically run all tests for a given problem
- `aucpl problem check`: Ensure test cases and files are not missing
- `aucpl problem generate`: Generate test case inputs with generator files
- `aucpl problem compare`: Compare two or more solutions and their outputs
- `aucpl problem fuzz`: Find potential edge cases and bugs in two or more solutions
- `aucpl problem archive`: Archive a problem

Competitions

- `aucpl comp create`: Create a new competition
- `aucpl comp add`: Add a problem to the competition
- `aucpl comp finish`: Mark a competition as completed and archive all problems under the competition
- `aucpl comp list`: List all competitions or problems in a competition
- `aucpl comp solve`: Generate output test cases for all problems in a given competition
- `aucpl comp test`: Run tests for all problems in a given competition
- `aucpl comp remove`: Remove a problem from the competition
- `aucpl comp rename`: Rename a competition

Other

- `aucpl init`: Create a new project
- `aucpl help`: Show help
- `aucpl sync`: Generate or update the problem mappings file
