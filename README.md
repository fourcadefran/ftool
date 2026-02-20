# ftool 🦀

**ftool** is a modular command-line toolbox written in Rust. It combines classic Unix-style file utilities with modern data inspection features, and an interactive TUI for navigating the filesystem and inspecting CSV/Parquet datasets visually.

This project is primarily a **learning-oriented Rust CLI**, focused on writing idiomatic Rust while building real, useful tooling.

---

## ✨ Features

### 🖥 Interactive TUI

Built with **ratatui** — launch it by just running `ftool`:

* Home menu with quick actions
* File browser with directory navigation and file preview
* Data inspector with Schema and Preview tabs
* In-TUI file format conversion (CSV ↔ Parquet)

### 📂 File utilities

* File metadata inspection
* File size
* Line count
* Preview file contents (`head`)

### 🔍 Data inspection (Parquet & CSV)

Powered by **DuckDB (embedded)**:

* Inspect schema (column names + types)
* Count total rows
* Count NULL values per column
* Convert between Parquet and CSV formats

### 📋 Todo manager *(coming soon)*

---

## 🚀 Installation

### Prerequisites

* Rust (stable)
* Cargo

### Build and install locally

```bash
cargo install --path .
```

This will install the `ftool` binary into `~/.cargo/bin/ftool`. Make sure `~/.cargo/bin` is in your `PATH`.

---

## 🛠 Usage

### Interactive TUI (default)

```bash
# Launch TUI at the Home screen
ftool

# Open file browser in a specific directory
ftool tui .
ftool tui ~/data

# Open data inspector directly on a file
ftool tui data.csv
ftool tui data.parquet
```

**TUI controls:**

| Screen | Key | Action |
|---|---|---|
| Home | `↑↓` / `j k` | Navigate menu |
| | `Enter` | Select |
| | `q` | Quit |
| File Browser | `↑↓` / `j k` | Navigate files |
| | `Enter` | Open directory / inspect file |
| | `Esc` | Back to Home |
| | `q` | Quit |
| Data Inspector | `Tab` | Switch Schema / Preview |
| | `↑↓` / `j k` | Scroll |
| | `c` | Convert format (CSV ↔ Parquet) |
| | `Esc` | Back to File Browser |
| | `q` | Quit |

---

### File inspection (CLI)

```bash
ftool file -i Cargo.toml      # metadata
ftool file -s Cargo.toml      # size
ftool file -l src/main.rs     # line count
ftool file -h 10 Cargo.toml   # first 10 lines
```

### Data file inspection (CLI)

```bash
# Schema, row count, null count
ftool inspect -d data.parquet
ftool inspect -r data.csv
ftool inspect -n column_name data.csv

# Convert formats
ftool inspect -c parquet data.csv
ftool inspect -c csv data.parquet
```

---

## 🧠 Design goals

* Write **idiomatic Rust**
* Practice ownership, borrowing, and error handling
* Keep the CLI Unix-like and predictable
* Build something extensible rather than toy examples

---

## 📦 Tech stack

| Crate | Purpose |
|---|---|
| **clap** | CLI argument parsing |
| **ratatui** | Terminal UI framework |
| **crossterm** | Terminal input/output |
| **DuckDB (bundled)** | Embedded analytics engine |
| **serde / serde_json** | Serialization |

---

## 🧪 Development

```bash
# Run TUI in development
cargo run

# Run a specific CLI command
cargo run -- inspect -d data.parquet

# Install after changes
cargo install --path .
```

---

## 📚 What this project helped me learn

* Structuring a real-world Rust CLI
* Error handling with `Result` and `anyhow`
* Working with embedded databases (DuckDB)
* Building interactive TUIs with ratatui (TEA pattern)
* Designing extensible command hierarchies

---

## 🔮 Future ideas

* Column statistics (min / max / avg)
* JSON file inspector
* Shell autocompletion
* Query mode for complex data filtering
* Todo list in TUI

---

## 📄 License

MIT
