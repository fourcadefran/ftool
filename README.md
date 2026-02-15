# ftool 🦀

**ftool** is a modular command-line toolbox written in Rust. It combines classic Unix-style file utilities with modern data inspection features, allowing you to inspect files, manage local todos, and analyze Parquet datasets using an embedded DuckDB engine.

This project is primarily a **learning-oriented Rust CLI**, focused on writing idiomatic Rust while building real, useful tooling.

---

## ✨ Features

### 📂 File utilities

* File metadata inspection
* File size
* Line count
* Preview file contents (`head`)

### 📋 Todo manager

* Add todos
* List todos
* Mark todos as done
* Remove todos

### 🔍 Data inspection (Parquet & CSV)

Powered by **DuckDB (embedded)**:

* Inspect Parquet/CSV schema
* Count total rows in Parquet/CSV files
* Count NULL values per column in Parquet/CSV files
* Convert between Parquet and CSV formats

---

## 🚀 Installation 고려

### Prerequisites

* Rust (stable)
* Cargo

### Build and install locally

```bash
cargo install --path .
```

This will install the `ftool` binary into:

```bash
~/.cargo/bin/ftool
```

Make sure `~/.cargo/bin` is in your `PATH`.

---

## 🛠 Usage

### File inspection

```bash
ftool file -i Cargo.toml
ftool file -s Cargo.toml
ftool file -l src/main.rs
ftool file -h 10 Cargo.toml
```

### Todo management

```bash
ftool todo -a "Learn Rust ownership"
ftool todo -l
ftool todo -d 1
ftool todo -r 2
```

### Data file inspection

```bash
# Inspect Parquet files
ftool inspect -d data.parquet
ftool inspect -r data.parquet
ftool inspect -n geometry data.parquet

# Inspect CSV files
ftool inspect -d data.csv
ftool inspect -r data.csv
ftool inspect -n name data.csv

# Convert between formats
ftool inspect -c csv data.parquet
ftool inspect -c parquet data.csv
```

Example output:

```
id                   BIGINT
geometry             BLOB
name                 VARCHAR
```

---

## 🧠 Design goals

* Write **idiomatic Rust**
* Practice ownership, borrowing, and error handling
* Keep the CLI Unix-like and predictable
* Prefer explicitness over magic
* Build something extensible rather than toy examples

---

## 📦 Tech stack

* **Rust**
* **clap** – CLI argument parsing
* **serde / serde_json** – serialization
* **DuckDB (bundled)** – embedded analytics engine
* **Parquet / Arrow** – columnar data formats

---

## 🧪 Development

During development:

```bash
cargo run -- inspect -r data.parquet
```

After changes:

```bash
cargo install --path .
```

---

## 📚 What this project helped me learn

* Structuring a real-world Rust CLI
* Error handling with `Result` and `anyhow`
* Working with embedded databases (DuckDB)
* Designing extensible command hierarchies
* Writing maintainable Rust modules

---

## 🔮 Future ideas

* Column statistics (min / max / avg)
* JSON output mode
* JSON file inspector
* Shell autocompletion
* Query mode for complex data filtering

---

## 📄 License

MIT
