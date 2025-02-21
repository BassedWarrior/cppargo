pub use clap::{Parser, Subcommand};

use std::path::PathBuf;

#[derive(Parser)]
#[command(version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Create a new project.
    ///
    /// This command will create a new cppargo project at `<PATH>` by creating a
    /// directory at `<PATH>`, a manifest `<PATH>/Cppargo.toml` file with the
    /// project name, a `<PATH>/src` directory for source `.cpp` files, and a
    /// `<PATH>/src/main.cpp` sample "Hello World!" file.
    ///
    /// This command fails if `<PATH>` already exists.
    #[command(visible_alias = "n")]
    New {
        // Path where the project will be created.
        #[arg(required = true)]
        path: PathBuf
    },
    /// Compile a project.
    ///
    /// Search for the project root by looking for a project manifest
    /// `Cppargo.toml` file in the current, and any parent directories and read
    /// the project name, to use it for the compiled binary file name.
    ///
    /// Iterates over the project `PROJECT_ROOT/src` directory in order to
    /// find all `.cpp` source files and give them to the `g++` compiler. The
    /// compiled binary file is stored at `PROJECT_ROOT/target/PROJECT_NAME`.
    /// If the `PROJECT_ROOT/target` directory doesn't already exist, it
    /// creates it before compiling.
    #[command(visible_alias = "b")]
    Build,
    /// Run a project.
    ///
    /// Compile the project by using the same functionality as the `build`
    /// subcommand (see `cppargo help build`), and then excecute the compiled
    /// binary `PROJECT_ROOT/target/PROJECT_NAME` from the current directory.
    #[command(visible_alias = "r")]
    Run,
}
