# ftool 🦀

**ftool** is a modular command-line toolbox written in Rust. It combines classic Unix-style file utilities with modern data inspection features, and an interactive TUI for navigating the filesystem and inspecting CSV/Parquet/JSON datasets visually.

This project is primarily a **learning-oriented Rust CLI**, focused on writing idiomatic Rust while building real, useful tooling.

---

## ✨ Features

### 🖥 Interactive TUI

Built with **ratatui** — launch it by just running `ftool`:

* Home menu with quick actions
* File browser with directory navigation and file metadata preview
* Data inspector for CSV and Parquet files with Schema and Preview tabs
* JSON and GeoJSON inspector with Tree, Raw, and Features views
* In-TUI file format conversion (CSV ↔ Parquet)

### 📊 Data Inspector (CSV & Parquet)

Powered by **DuckDB (embedded)**:

* **Schema tab** — column names, types, null counts, min/max/avg statistics (loaded lazily on first view)
* **Preview tab** — paginated data view (25 rows per page, 10 columns per page)
  * `↑` / `↓` (or `j` / `k`) paginate rows
  * `←` / `→` move the column cursor (auto-advances column page at the boundary)
  * `h` / `l` jump an entire column page at once
  * Mouse scroll to scroll rows within the current page
  * Selected column highlighted in header and cells
* **Filters** — multi-condition filter builder with AND logic; 9 operators supported (`=`, `!=`, `>`, `<`, `>=`, `<=`, `LIKE`, `IS NULL`, `IS NOT NULL`); active filter count shown in the info bar
* In-TUI format conversion (CSV ↔ Parquet)

### 🗺 JSON & GeoJSON Inspector

* **Tree view** — collapsible key/value tree for JSON objects and arrays
* **Raw view** — pretty-printed JSON source
* **GeoJSON** — dedicated Summary, Features, and Tree tabs; feature properties table and bounding box

### 📂 File utilities

* File metadata inspection
* File size
* Line count
* Preview file contents (`head`)

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
ftool tui data.json
ftool tui data.geojson
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
| Data Inspector | `Tab` | Switch Schema / Preview tabs |
| | `↑↓` / `j k` | Previous / next row page (Preview tab) |
| | `←` / `→` | Move column cursor left / right (Preview tab) |
| | `h` / `l` | Jump entire column page left / right (Preview tab) |
| | `scroll` | Scroll rows within current page / scroll schema |
| | `f` | Open filter editor (Preview tab) |
| | `c` | Convert format (CSV ↔ Parquet) |
| | `Esc` | Back to File Browser |
| | `q` | Quit |
| Filter Editor | `Tab` | Next field (Column → Operator → Value) |
| | `↑↓` | Change selected column or operator |
| | `Enter` | Add condition |
| | `r` | Apply all conditions |
| | `d` | Remove last condition |
| | `Esc` | Cancel |
| JSON Inspector | `Tab` | Switch tabs |
| | `↑↓` / `j k` | Scroll |
| | `Enter` | Expand / collapse node (Tree tab) |
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
* Parsing and rendering nested JSON/GeoJSON structures
* Building dynamic SQL filters with safe identifier handling

---

## 🔮 Future ideas

See [ROADMAP.md](ROADMAP.md) for the full list of planned improvements.

---

## 📄 License

MIT
