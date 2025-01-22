pub use assert_cmd::prelude::*;
pub use assert_fs::prelude::*;
pub use predicates::prelude::*;
pub use std::process::Command;

pub const HELLO_WORLD_PROGRAM: &str = concat!(
    "#include <iostream>\n",
    "\n",
    "int main() {\n",
    "    std::cout << \"Hello World!\\n\";\n",
    "\n",
    "    return 0;\n",
    "}\n"
);
