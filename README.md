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

## Quickstart

1. Create a new project at any `PATH` and move to that `PATH`.

```sh
cppargo new <PATH>
cd <PATH>
```

2. Run the sample hello world program automatically created by `cppargo`.

```sh
cppargo run
```

3. Edit the main `src/main.cpp` file as desired.
4. Build or build and run the new project.
    - Build

        ```sh
        cppargo build
        ```

    - Build and run

        ```sh
        cppargo run
        ```

5. Build, but run manually.
    1. Build

        ```sh
        cppargo build
        ```

    2. Run manually

        ```sh
        ./target/<PROJECT-NAME>
        ```

## Usage

### Create new project

In order to create a new project, use the command

```sh
cppargo new <PATH>
```

This will create a new directory at `PATH`. Relative or absolute paths are
accepted, and will be created accordingly. This directory will be considered
the root of the `cppargo` project.

Inside the project root directory, `cppargo` will also create a `Cppargo.toml`
manifest file akin to a `Cargo.toml` file used by `cargo`. Internally,
`cppargo` looks for such a file to determine the project root, or determine
that it is not within a `cppargo` project.

Additionally, `cppargo` will attempt to initialize the created directory as a
`git` repo. However it is not needed for it to be a git repo in order to
create the project successfully. In the event that it cannot find `git` as an
excecutable, it will simply issue a warning and continue.

And finally, `cppargo` will create a `src` directory with a basic
`src/main.cpp` "Hello World!" C++ program. This is the directory where all of
the source files for the project should be included.

### Build projects

From inside a `cppargo` project, in order to build a project, use the command

```sh
cppargo build
```

Firstly, `cppargo` will look for a `Cppargo.toml` manifest file in the current
directory. If it doesn't find one, then it will check the parent directory's
contents recursively for it. If it fails to find a `Cppargo.toml` file, then it
exits with an error due to not being inside a `cppargo` project, as the
`Cppargo.toml` determines the root of any `cppargo` project.

From the project root, it will look for the `PROJECT_ROOT/src` directory to
find all of the `.cpp` files insde it. The first file it looks for, is the
`src/main.cpp` file. This file is meant to be the main project file, and is
therefore required to be in the project. If `cppargo` fails to find a
`src/main.cpp` file, it will exit with an error. This to establish a convention
for file structure within any `cppargo` project.

Apart from the presence of the `src/main.cpp` file, the file structure inside
the `src` directory is irrelevant to `cppargo`, since it will search
exhaustively all subdirectories to find all `.cpp` files. Once gathered, all of
the `.cpp` files are sent as arguments to `g++` to be compiled. This means that
`cppargo` does no linkage or compilation of its own, nor does it check for any
bad `#include` statements, or the lack thereof.

The compiled excecutable file is then stored within a `PROJECT_ROOT/target`
directory. `cppargo` first checks to ensure that the directory exists, and
creates it if it doesn't exist. The excecutable file's name is gathered from
the project manifest by reading the project's name. The compiled excecutable is
then placed at `PROJECT_ROOT/target/PROJECT_NAME`.

### Run projects

From inside a `cppargo` project, in order to run a project, use the command

```sh
cppargo run
```

This will first perform a `cppargo build` and then run the
`PROJECT_ROOT/target/PATH` file generated after a successful compilation. The
proper working directory for the excecutable will be the same as the one
`cppargo` was excecuted in. This should be kept in mind when the program
expects a certain file structure or a certain working directory.
