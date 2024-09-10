# Cppargo

Hola estoy cambiando la documentación.

`cppargo` is a tool to mimic the basic functionalities of the `cargo` utility 
for Rust projects but for C++ projects.

## Installation

### Using `cargo`

Installation can currently only be done using `cargo`, the Rust project management tool this project aims to emulate.

1. Clone the repo locally.

Using `gh` (GitHub's CLI Tool).

```
gh repo clone BassedWarrior/cppargo
```

2. Compile and add to `$PATH` using `cargo` from inside the cloned repo.

```
cargo install --path .
```

## Usage

### Create new project

In order to create a new project, use the command

```
cppargo new {project_name}
```

This will create a new directory `project_name` in the current working 
directory. 

Inside of which it will also create a `src` with a mock `project_name.cpp` 
hello world program. This is the directory where all of the source files for 
the project.

It will also create a `target` directory where the excecutable files are to 
be stored and run from.

### Build projects

In order to build a project, use the command

```
cppargo build
```

Make sure to run this command within the project directory. As it will look 
for the `src` directory to compile all of the `.cpp` files insde it using `g++`
to create the `project_name` file within the `target` directory.

### Run projects

In order to run a project, use the command

```
cppargo run
```

Within the project directory. This will first perform a `cppargo build` and 
then run the `target/project_name` file generated after a successful 
compilation.
