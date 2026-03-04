use duckdb::Connection;
use std::path::Path;

#[derive(Debug)]
pub enum DuckDbError {
    FileNotFound(String),
    InvalidFileFormat(String),
    ConnectionError(String),
    QueryError(String),
    InvalidColumn(String),
    DatabaseError(String),
}

impl std::fmt::Display for DuckDbError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DuckDbError::FileNotFound(path) => write!(f, "File not found: {}", path),
            DuckDbError::InvalidFileFormat(path) => {
                write!(f, "Invalid file format: {}", path)
            }
            DuckDbError::ConnectionError(msg) => write!(f, "Database connection error: {}", msg),
            DuckDbError::QueryError(msg) => write!(f, "Query execution error: {}", msg),
            DuckDbError::InvalidColumn(col) => write!(f, "Invalid column name: {}", col),
            DuckDbError::DatabaseError(msg) => write!(f, "Database error: {}", msg),
        }
    }
}

impl std::error::Error for DuckDbError {}

impl From<duckdb::Error> for DuckDbError {
    fn from(error: duckdb::Error) -> Self {
        DuckDbError::DatabaseError(error.to_string())
    }
}

pub struct DuckDbInspector {
    file_path: String,
    connection: Connection,
}

impl DuckDbInspector {
    /// Constructor - validates the file path before creating the connection
    pub fn new(file_path: String) -> Result<Self, DuckDbError> {
        // Validate file exists
        let path = Path::new(&file_path);
        if !path.exists() {
            return Err(DuckDbError::FileNotFound(file_path.clone()));
        }

        // Validate it's a file
        if !path.is_file() {
            return Err(DuckDbError::InvalidFileFormat(format!(
                "{} is not a file",
                file_path
            )));
        }

        // Validate file extension
        if let Some(ext) = path.extension() {
            if ext != "parquet" && ext != "csv" {
                return Err(DuckDbError::InvalidFileFormat(format!(
                    "Expected .parquet or .csv file, got .{}",
                    ext.to_string_lossy()
                )));
            }
        } else {
            return Err(DuckDbError::InvalidFileFormat(
                "File has no extension".to_string(),
            ));
        }

        // Create connection
        let connection = Connection::open_in_memory().map_err(|e| {
            DuckDbError::ConnectionError(format!("Failed to open in-memory database: {}", e))
        })?;

        Ok(Self {
            file_path,
            connection,
        })
    }

    fn read_function(&self) -> &'static str {
        let ext = Path::new(&self.file_path)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");
        if ext == "csv" { "read_csv_auto" } else { "read_parquet" }
    }

    fn escape_path(&self) -> String {
        self.file_path.replace('\'', "''")
    }

    /// Sanitize identifier to prevent SQL injection
    fn sanitize_identifier(name: &str) -> Result<String, DuckDbError> {
        // Allow only alphanumeric, underscore, and some safe characters
        if name.chars().all(|c| c.is_alphanumeric() || c == '_') {
            Ok(name.to_string())
        } else {
            Err(DuckDbError::InvalidColumn(format!(
                "Column name contains invalid characters: {}",
                name
            )))
        }
    }

    /// Returns the file schema (column name + type) for CSV or Parquet files
    pub fn schema(&self) -> Result<Vec<(String, String)>, DuckDbError> {
        let query = format!(
            "DESCRIBE SELECT * FROM {}('{}')",
            self.read_function(),
            self.escape_path()
        );

        let mut stmt = self.connection.prepare(&query).map_err(|e| {
            DuckDbError::QueryError(format!("Failed to prepare schema query: {}", e))
        })?;

        let rows = stmt
            .query_map([], |row| {
                let column_name: String = row.get(0)?;
                let column_type: String = row.get(1)?;
                Ok((column_name, column_type))
            })
            .map_err(|e| {
                DuckDbError::QueryError(format!("Failed to execute schema query: {}", e))
            })?;

        let mut schema = Vec::new();
        for row_result in rows {
            let row = row_result.map_err(|e| {
                DuckDbError::QueryError(format!("Failed to read schema row: {}", e))
            })?;
            schema.push(row);
        }

        if schema.is_empty() {
            return Err(DuckDbError::InvalidFileFormat(
                "File has no columns".to_string(),
            ));
        }

        Ok(schema)
    }

    /// Returns the number of rows in the file (CSV or Parquet)
    pub fn row_count(&self) -> Result<usize, DuckDbError> {
        self.row_count_filtered("")
    }

    /// Returns the number of rows matching an optional WHERE clause
    pub fn row_count_filtered(&self, where_clause: &str) -> Result<usize, DuckDbError> {
        let query = format!(
            "SELECT COUNT(*) FROM {}('{}') {}",
            self.read_function(),
            self.escape_path(),
            where_clause,
        );

        self.connection
            .query_row(&query, [], |row| row.get(0))
            .map_err(|e| DuckDbError::QueryError(format!("Failed to count rows: {}", e)))
    }

    /// Returns the number of null values in a single column (used by CLI)
    pub fn null_count(&self, column_name: &str) -> Result<usize, DuckDbError> {
        let safe_column = Self::sanitize_identifier(column_name)?;
        let query = format!(
            "SELECT COUNT(*) FROM {}('{}') WHERE {} IS NULL",
            self.read_function(),
            self.escape_path(),
            safe_column
        );
        self.connection
            .query_row(&query, [], |row| row.get(0))
            .map_err(|e| {
                DuckDbError::QueryError(format!(
                    "Failed to count nulls in column '{}': {}",
                    column_name, e
                ))
            })
    }

    /// Returns (null_counts, min_values, max_values, mean_values) for all columns in a single query
    pub fn column_stats(
        &self,
        schema: &[(String, String)],
    ) -> Result<(Vec<usize>, Vec<String>, Vec<String>, Vec<String>), DuckDbError> {
        // Build a single query that computes all stats for all columns
        // For each column: null_count, min, max, mean
        let mut select_parts = vec!["COUNT(*) AS total_rows".to_string()];

        for (name, _) in schema {
            let safe = name.replace('"', "\"\"");
            select_parts.push(format!("COUNT(\"{}\")", safe));
            select_parts.push(format!("CAST(MIN(\"{}\") AS VARCHAR)", safe));
            select_parts.push(format!("CAST(MAX(\"{}\") AS VARCHAR)", safe));
            select_parts.push(format!(
                "CAST(ROUND(AVG(TRY_CAST(\"{}\" AS DOUBLE)), 2) AS VARCHAR)",
                safe
            ));
        }

        let query = format!(
            "SELECT {} FROM {}('{}')",
            select_parts.join(", "),
            self.read_function(),
            self.escape_path()
        );

        let mut stmt = self.connection.prepare(&query).map_err(|e| {
            DuckDbError::QueryError(format!("Failed to prepare column stats query: {}", e))
        })?;

        let mut null_counts = Vec::with_capacity(schema.len());
        let mut min_values = Vec::with_capacity(schema.len());
        let mut max_values = Vec::with_capacity(schema.len());
        let mut mean_values = Vec::with_capacity(schema.len());

        stmt.query_row([], |row| {
            let total_rows: usize = row.get(0)?;

            for i in 0..schema.len() {
                let base = 1 + i * 4;
                let nonnull: usize = row.get(base)?;
                null_counts.push(total_rows - nonnull);

                let min_val: Option<String> = row.get(base + 1)?;
                min_values.push(min_val.unwrap_or_else(|| "-".to_string()));

                let max_val: Option<String> = row.get(base + 2)?;
                max_values.push(max_val.unwrap_or_else(|| "-".to_string()));

                let mean_val: Option<String> = row.get(base + 3)?;
                mean_values.push(mean_val.unwrap_or_else(|| "-".to_string()));
            }

            Ok(())
        })
        .map_err(|e| {
            DuckDbError::QueryError(format!("Failed to execute column stats query: {}", e))
        })?;

        Ok((null_counts, min_values, max_values, mean_values))
    }

    /// Returns a preview of rows as (headers, rows_of_strings), with optional WHERE clause.
    /// If `columns` is provided, only those columns are selected; otherwise all columns are used.
    pub fn preview(
        &self,
        limit: usize,
        offset: usize,
        where_clause: &str,
        columns: Option<&[String]>,
    ) -> Result<(Vec<String>, Vec<Vec<String>>), DuckDbError> {
        let headers: Vec<String> = if let Some(cols) = columns {
            cols.to_vec()
        } else {
            let schema = self.schema()?;
            schema.into_iter().map(|(name, _)| name).collect()
        };

        // Cast all columns to VARCHAR, replacing NULLs with the string "NULL"
        let columns: Vec<String> = headers
            .iter()
            .map(|name| {
                let escaped = name.replace('"', "\"\"");
                format!("COALESCE(CAST(\"{}\" AS VARCHAR), 'NULL')", escaped)
            })
            .collect();

        let query = format!(
            "SELECT {} FROM {}('{}') {} LIMIT {} OFFSET {}",
            columns.join(", "),
            self.read_function(),
            self.escape_path(),
            where_clause,
            limit,
            offset
        );

        let mut stmt = self.connection.prepare(&query).map_err(|e| {
            DuckDbError::QueryError(format!("Failed to prepare preview query: {}", e))
        })?;

        let column_count = headers.len();
        let mut result = Vec::new();

        let rows = stmt
            .query_map([], |row| {
                let mut values = Vec::with_capacity(column_count);
                for i in 0..column_count {
                    let val: String = row.get(i)?;
                    values.push(val);
                }
                Ok(values)
            })
            .map_err(|e| {
                DuckDbError::QueryError(format!("Failed to execute preview query: {}", e))
            })?;

        for row_result in rows {
            result.push(row_result.map_err(|e| {
                DuckDbError::QueryError(format!("Failed to read preview row: {}", e))
            })?);
        }

        Ok((headers, result))
    }

    /// Converts the parquet file to CSV or Parquet, depending on the target format
    pub fn convert(&self, target_format: &str) -> Result<String, DuckDbError> {
        let path = Path::new(&self.file_path);
        let ext = path.extension().unwrap_or_default();

        if !["csv", "parquet"].contains(&target_format) {
            return Err(DuckDbError::InvalidFileFormat(
                "Target format not supported".to_string(),
            ));
        }

        if ext == target_format {
            return Ok(self.file_path.clone());
        }

        let target_path = path
            .with_extension(target_format)
            .to_string_lossy()
            .to_string();

        let format_str = if target_format == "csv" {
            "CSV"
        } else {
            "PARQUET"
        };

        let query = format!(
            "COPY (SELECT * FROM '{}') TO '{}' (FORMAT {})",
            self.escape_path(),
            target_path.replace('\'', "''"),
            format_str
        );

        self.connection
            .execute(&query, [])
            .map_err(|e| DuckDbError::QueryError(format!("Failed to convert file: {}", e)))?;

        Ok(target_path)
    }
}
