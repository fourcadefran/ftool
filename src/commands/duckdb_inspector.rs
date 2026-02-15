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
            DuckDbError::FileNotFound(path) => write!(f, "Parquet file not found: {}", path),
            DuckDbError::InvalidFileFormat(path) => write!(f, "Invalid parquet file format: {}", path),
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
    connection: Connection
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
            return Err(DuckDbError::InvalidFileFormat(format!("{} is not a file", file_path)));
        }

        // Validate file extension
        if let Some(ext) = path.extension() {
            if ext != "parquet" {
                return Err(DuckDbError::InvalidFileFormat(
                    format!("Expected .parquet file, got .{}", ext.to_string_lossy())
                ));
            }
        } else {
            return Err(DuckDbError::InvalidFileFormat("File has no extension".to_string()));
        }

        // Create connection
        let connection = Connection::open_in_memory()
            .map_err(|e| DuckDbError::ConnectionError(format!("Failed to open in-memory database: {}", e)))?;

        Ok(Self {
            file_path,
            connection,
        })
    }

    /// Sanitize identifier to prevent SQL injection
    fn sanitize_identifier(name: &str) -> Result<String, DuckDbError> {
        // Allow only alphanumeric, underscore, and some safe characters
        if name.chars().all(|c| c.is_alphanumeric() || c == '_') {
            Ok(name.to_string())
        } else {
            Err(DuckDbError::InvalidColumn(
                format!("Column name contains invalid characters: {}", name)
            ))
        }
    }

    /// Returns the parquet schema (column name + type)
    pub fn schema(&self) -> Result<Vec<(String, String)>, DuckDbError> {
        // Use parameterized query to prevent SQL injection
        let query = format!(
            "DESCRIBE SELECT * FROM read_parquet('{}')",
            self.file_path.replace('\'', "''")  // Escape single quotes
        );

        let mut stmt = self.connection.prepare(&query)
            .map_err(|e| DuckDbError::QueryError(format!("Failed to prepare schema query: {}", e)))?;

        let rows = stmt.query_map([], |row| {
            let column_name: String = row.get(0)?;
            let column_type: String = row.get(1)?;
            Ok((column_name, column_type))
        })
        .map_err(|e| DuckDbError::QueryError(format!("Failed to execute schema query: {}", e)))?;

        let mut schema = Vec::new();
        for row_result in rows {
            let row = row_result
                .map_err(|e| DuckDbError::QueryError(format!("Failed to read schema row: {}", e)))?;
            schema.push(row);
        }

        if schema.is_empty() {
            return Err(DuckDbError::InvalidFileFormat("Parquet file has no columns".to_string()));
        }

        Ok(schema)
    }

    /// Returns the number of rows in the parquet file
    pub fn row_count(&self) -> Result<usize, DuckDbError> {
        let query = format!(
            "SELECT COUNT(*) FROM read_parquet('{}')",
            self.file_path.replace('\'', "''")
        );

        self.connection
            .query_row(&query, [], |row| row.get(0))
            .map_err(|e| DuckDbError::QueryError(format!("Failed to count rows: {}", e)))
    }

    /// Returns the number of null values in a column
    pub fn null_count(&self, column_name: &str) -> Result<usize, DuckDbError> {
        // Sanitize column name to prevent SQL injection
        let safe_column = Self::sanitize_identifier(column_name)?;

        let query = format!(
            "SELECT COUNT(*) FROM read_parquet('{}') WHERE {} IS NULL",
            self.file_path.replace('\'', "''"),
            safe_column
        );

        self.connection
            .query_row(&query, [], |row| row.get(0))
            .map_err(|e| DuckDbError::QueryError(
                format!("Failed to count nulls in column '{}': {}", column_name, e)
            ))
    }

    /// Converts the parquet file to CSV or Parquet, depending on the target format
    pub fn convert(&self, target_format: &str) -> Result<String, DuckDbError> {
        let path = Path::new(&self.file_path);
        let ext = path.extension().unwrap_or_default();

        if !["csv", "parquet"].contains(&target_format) {
            return Err(DuckDbError::InvalidFileFormat("Target format not supported".to_string()));
        }

        if ext == target_format {
            return Ok(self.file_path.clone());
        }

        let target_path = path.with_extension(target_format)
            .to_string_lossy()
            .to_string();

        let format_str = if target_format == "csv" { "CSV" } else { "PARQUET" };

        let query = format!(
            "COPY (SELECT * FROM '{}') TO '{}' (FORMAT {})",
            self.file_path.replace('\'', "''"),
            target_path.replace('\'', "''"),
            format_str
        );

        self.connection.execute(&query, [])
            .map_err(|e| DuckDbError::QueryError(format!("Failed to convert file: {}", e)))?;

        Ok(target_path)
    }
}