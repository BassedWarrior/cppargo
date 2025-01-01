# Cppargo

`cppargo` is a tool to mimic the basic functionalities of the `cargo` utility 
for Rust projects but for C++ projects.

## Dependencies

Currently, `cppargo` is hard-coded to use `g++` as the compiler for the `C++`
projects it manages. So ensure that you have it installed. Most Unix systems
have it already installed. [Installation instructions](https://gcc.gnu.org/install/) for `g++` can be found
on the website, or with your system's package manager. `g++` is not required
to install `cppargo`, but it is currently indispensible to compile `C++`
projects.

### Fedora Linux

```sh
sudo dnf install gcc-c++
```

### Ubuntu Linux

```sh
sudo apt install gcc
```

### Arch Linux

```sh
sudo pacman -S gcc
```

### Brew

```sh
brew install gcc
```

## Installation

### Using `cargo`

Installation can currently only be done using `cargo`, the Rust project
management tool this project aims to emulate. If you haven't already installled
Rust, follow the [installation instructions](https://www.rust-lang.org/tools/install) from their website on how to
install utilizing `rustup`.

#### From [`crates.io`](https://www.crates.io)

```sh
cargo install cppargo
```

#### Directly from repo

```sh
cargo install --git https://www.github.com/bassedwarrior/cppargo
```

#### Building

1. Clone the repo locally.

    - Using `gh` (GitHub's CLI Tool).

    ```sh
    gh repo clone BassedWarrior/cppargo
    ```
    - Using `git`

    ```sh
    git clone https://www.github.com/bassedwarrior/cppargo
    ```

2. From inside the cloned repo, compile the binary.

```sh
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
