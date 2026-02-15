use clap::{Args, Parser, Subcommand};

#[derive(Parser)]
#[command(name = "ftool")]
#[command(about = "A small toolbox CLI written in Rust")]
#[command(author = "Francisco Fourcade <franfourcade99@gmail.com>")]
#[command(version = "0.1.0")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Analyze and inspect file properties
    File(FileArgs),
    /// Manage your todo list
    Todo(TodoArgs),
    /// Inspect file metadata (Parquet, etc.)
    Inspect(InspectArgs),
}

#[derive(Args)]
pub struct InspectArgs {
    /// Display the parquet schema
    #[arg(short = 'd', long = "desc")]
    pub desc: bool,

    /// Count total rows in the file
    #[arg(short = 'r', long = "row-count")]
    pub row_count: bool,

    /// Count null values in a specific column
    #[arg(short = 'n', long = "null-count")]
    pub null_count: Option<String>,

    /// Path to the file to inspect
    pub file: String,

    // Convert file
    pub convert: Option<String>,
}

impl InspectArgs {
    /// Valida que solo una acción haya sido especificada
    pub fn validate(&self) -> Result<(), String> {
        let actions = [self.desc, self.row_count, self.null_count.is_some()];
        let count = actions.iter().filter(|&&b| b).count();

        if count == 0 {
            return Err(
                "Must specify at least one action (--desc, --row-count, or --null-count)"
                    .to_string(),
            );
        }

        if count > 1 {
            return Err(
                "Can only specify one action at a time (--desc, --row-count, or --null-count)"
                    .to_string(),
            );
        }

        Ok(())
    }
}

#[derive(Args)]
pub struct FileArgs {
    /// Display general file information (size, permissions, timestamps)
    #[arg(short = 'i', long = "info")]
    pub info: bool,

    /// Show the total number of lines in the file
    #[arg(short = 'l', long = "lines")]
    pub lines: bool,

    /// Display the file size
    #[arg(short = 's', long = "size")]
    pub size: bool,

    /// Display the first N lines of the file
    #[arg(short = 'h', long = "head")]
    pub head: Option<usize>,

    /// Path to the file to analyze
    pub file: String,
}

impl FileArgs {
    /// Valida que solo una acción haya sido especificada
    pub fn validate(&self) -> Result<(), String> {
        let actions = [self.info, self.lines, self.size, self.head.is_some()];
        let count = actions.iter().filter(|&&b| b).count();

        if count == 0 {
            return Err(
                "Must specify at least one action (--info, --lines, --size, or --head)".to_string(),
            );
        }

        if count > 1 {
            return Err(
                "Can only specify one action at a time (--info, --lines, --size, or --head)"
                    .to_string(),
            );
        }

        Ok(())
    }
}

#[derive(Args)]
pub struct TodoArgs {
    /// Add a new todo item
    #[arg(short = 'a', long = "add")]
    pub add: Option<String>,

    /// List all todo items
    #[arg(short = 'l', long = "list")]
    pub list: bool,

    /// Mark a todo as completed by its ID
    #[arg(short = 'd', long = "done")]
    pub done: Option<usize>,

    /// Remove a todo item by its ID
    #[arg(short = 'r', long = "remove")]
    pub remove: Option<usize>,
}

impl TodoArgs {
    /// Valida que solo una acción haya sido especificada
    pub fn validate(&self) -> Result<(), String> {
        let actions = [
            self.add.is_some(),
            self.list,
            self.done.is_some(),
            self.remove.is_some(),
        ];
        let count = actions.iter().filter(|&&b| b).count();

        if count == 0 {
            return Err(
                "Must specify at least one action (--add, --list, --done, or --remove)".to_string(),
            );
        }

        if count > 1 {
            return Err(
                "Can only specify one action at a time (--add, --list, --done, or --remove)"
                    .to_string(),
            );
        }

        Ok(())
    }
}
