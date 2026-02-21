mod cli;
mod commands;
mod tui;

use clap::Parser;
use cli::{Cli, Commands};

fn main() {
    let cli = Cli::parse();

    match cli.command {
        None => {
            // No subcommand -> launch TUI
            if let Err(e) = tui::run(None) {
                eprintln!("TUI error: {}", e);
                std::process::exit(1);
            }
        }
        Some(Commands::File(args)) => {
            if let Err(e) = args.validate() {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }

            let file = commands::File::new(args.file);

            if args.info {
                match file.info() {
                    Ok(result) => println!("{}", result),
                    Err(e) => eprintln!("Error: {}", e),
                }
            }

            if let Some(n) = args.head {
                match file.head(n) {
                    Ok(result) => println!("{}", result),
                    Err(e) => eprintln!("Error: {}", e),
                }
            }

            if args.size {
                match file.size() {
                    Ok(result) => println!("{}", result),
                    Err(e) => eprintln!("Error: {}", e),
                }
            }

            if args.lines {
                match file.lines() {
                    Ok(result) => println!("{}", result),
                    Err(e) => eprintln!("Error: {}", e),
                }
            }
        }

        Some(Commands::Inspect(args)) => {
            if let Err(e) = args.validate() {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }

            let inspector = match commands::DuckDbInspector::new(args.file) {
                Ok(i) => i,
                Err(e) => {
                    eprintln!("Error initializing DuckDB: {}", e);
                    return;
                }
            };

            if args.desc {
                match inspector.schema() {
                    Ok(schema) => {
                        for (name, ty) in schema {
                            println!("{:<20} {}", name, ty);
                        }
                    }
                    Err(e) => eprintln!("Error reading schema: {}", e),
                }
            }

            if args.row_count {
                match inspector.row_count() {
                    Ok(count) => println!("Row count: {}", count),
                    Err(e) => eprintln!("Error counting rows: {}", e),
                }
            }

            if let Some(column) = args.null_count {
                match inspector.null_count(&column) {
                    Ok(count) => println!("Null values in column '{}': {}", column, count),
                    Err(e) => eprintln!("Error counting nulls: {}", e),
                }
            }
            
            if let Some(format) = args.convert {
                match inspector.convert(&format) {
                    Ok(path) => println!("File converted to {}", path),
                    Err(e) => eprintln!("Error converting file: {}", e),
                }
            }
        }
        Some(Commands::Tui(args)) => {
            if let Err(e) = tui::run(args.path) {
                eprintln!("TUI error: {}", e);
                std::process::exit(1);
            }
        }
        Some(Commands::Todo(args)) => {
            if let Err(e) = args.validate() {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }

            if let Some(task) = args.add {
                todo!("Implement add todo: {}", task);
            }

            if args.list {
                todo!("Implement list todos");
            }

            if let Some(id) = args.done {
                todo!("Implement mark todo {} as done", id);
            }

            if let Some(id) = args.remove {
                todo!("Implement remove todo {}", id);
            }
        }
    }
}
