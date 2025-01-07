pub use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Creates a new cppargo project at <path>
    ///
    /// This command will create a new cppargo project in the given directory.
    /// This includes a sample source file "Hello World!" program
    /// (`src/main.cpp`).
    #[command(visible_alias = "n")]
    New { path: String },
    /// Compile a local project.
    ///
    /// Iterates over the `src` directory in order to find all `.cpp` source files
    /// and gives it to the `g++` compiler to store it in the `target` directory
    /// with the project name.
    #[command(visible_alias = "b")]
    Build,
    /// Run a binary of the local project.
    ///
    /// Compile the project, and then excecute the compiled binary stored in
    /// the `target` directory.
    #[command(visible_alias = "r")]
    Run,
}
