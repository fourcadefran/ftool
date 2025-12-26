use clap::{Parser, Subcommand, Args};

#[derive(Parser)]
#[command(name = "ftool")]
#[command(about = "A small toolbox CLI written in Rust")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    File(FileArgs),
    Todo(TodoArgs),
    Inspect(InspectArgs)
}

#[derive(Args)]
pub struct InspectArgs {
    /// Describe parquet schema
    #[arg(short = 'd', long = "desc")]
    pub desc: bool,

    /// Count total rows
    #[arg(short = 'r', long = "row-count")]
    pub row_count: bool,

    /// Count nulls in a column
    #[arg(short = 'n', long = "null-count")]
    pub null_count: Option<String>,

    pub file: String,
}

#[derive(Args)]
pub struct  FileArgs {
    //info about a file
    #[arg(short='i', long="info")]
    pub info: bool,

    //lines of a file
    #[arg(short='l', long="lines")]
    pub lines: bool,

    //size of a file
    #[arg(short='s', long="size")]
    pub size: bool,

    #[arg(short='h', long="head")]
    pub head: Option<usize>,

    pub file: String
}

#[derive(Args)]
pub struct TodoArgs {
    //Add a new todo
    #[arg(short='a', long="add")]
    pub add: Option<String>,

    //list all todos
    #[arg(short='l', long="list")]
    pub list: bool,

    //mark as done
    #[arg(short='d', long="done")]
    pub done: Option<usize>,

    //remove a todo
    #[arg(short='r', long="remove")]
    pub remove: Option<usize>
}