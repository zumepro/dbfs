pub use sqlx::FromRow;
use tokio::runtime::Builder;


use connection_adapter::Adapter;
pub use connection_adapter::DbInputType;
mod connection_adapter;


#[derive(Debug, PartialEq)]
/// Generic error that can be returned when interacting with a database.
///
/// # Subtypes
///
/// ```rust
/// RuntimeStartFail
/// ```
/// This error indicates that an async wrapper could not be started to transform async network IO
/// to a blocking context.
///
///
/// ```rust
/// AdapterError(String)
/// ```
/// The inner SQL adapter encountered an error while interacting with the database.
/// This is usually some SQL error (such as SQL command/query syntax error).
///
/// # Conversion
///
/// This enum can be easily converted into a `String` as it implements `std::fmt::Display` and
/// `Into<String>`.
///
/// However explicit mapping is recommended.
/// ```rust
/// value_with_db_connector_error.map_err(|err| err.into::<String>());
/// ```
///
pub enum DbConnectorError {
    RuntimeStartFail,
    AdapterError(String),
}

impl std::fmt::Display for DbConnectorError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "db_connector_error: {}", match self {
            Self::RuntimeStartFail => "db_connector_error: could not start async runner".to_string(),
            Self::AdapterError(val) => format!("sql_adapter: {}", val),
        })
    }
}
impl Into<String> for DbConnectorError { fn into(self) -> String { format!("{}", self) } }


/// Interface for database communication
///
/// # Creating a `DbConnector`
/// The recommended method is `DbConnector::default()` - this method will use values from
/// `crate::settings` to find the host, port, user, password and database.
///
/// # Command
/// A command is an SQL query that expects no results (just success or fail). Command has a separate method to save
/// resources.
/// ```rust
/// let result = conn.command("INSERT INTO `test` (`id`) VALUES (?)", Some(&vec![42.into()]));
/// ```
/// # Query
/// A query is an SQL query that expects results.
/// ```rust
/// let rows: Vec<MyStruct> = conn.query("SELECT * FROM `test_prepared` WHERE `id` = ?", Some(&vec![3.into()])).unwrap();
/// ```
///
#[derive(Debug)]
pub struct DbConnector {
    adapter: connection_adapter::Adapter,
}


macro_rules! blockingify {
    ($expr: expr) => {
        let rt = Builder::new_current_thread()
            .worker_threads(4)
            .enable_time()
            .enable_io()
            .build()
            .map_err(|_| DbConnectorError::RuntimeStartFail)?;
        return rt.block_on(async move {
            $expr
        });
    }
}


impl DbConnector {
    pub fn default() -> Result<Self, DbConnectorError> {
        blockingify!(Ok(Self {
                adapter: Adapter::default().await.map_err(|err| DbConnectorError::AdapterError(err))?,
        }));
    }

    /// A command is an SQL query with **no expected response data**.
    /// 
    /// `Ok(())` is returned on success.
    ///
    /// `Err(DbConnectorError)` is returned on fail.
    ///
    /// # Example usage
    /// ```rust
    /// conn.command("INSERT INTO `test` (`id`) VALUES (?)", Some(&vec![42.into()])).unwrap();
    /// ```
    pub fn command(&mut self, command: &'static str, args: Option<&Vec<DbInputType>>) -> Result<(), DbConnectorError> {
        blockingify!({
            self.adapter.run_command(command, args).await.map_err(|err| DbConnectorError::AdapterError(err))?;
            Ok(())
        });
    }

    /// A query is an SQL query with **expected response data**.
    /// 
    /// `Ok(Vec<YourStruct>)` is returned on success.
    ///
    /// `Err(DbConnectorError)` is returned on fail.
    ///
    /// This function will try to deserialize the response into `YourStruct`.
    /// `FromRow` must be derived (or implemented) for `YourStruct` to deserialize successfully.
    ///
    /// # Example usage
    /// ```rust
    /// let rows: Vec<MyStruct> = conn.query("SELECT * FROM `test_prepared` WHERE `id` = ?", Some(&vec![3.into()])).unwrap();
    /// ```
    pub fn query<T>(&mut self, query: &'static str, args: Option<&Vec<DbInputType>>) -> Result<Vec<T>, DbConnectorError>
    where
        T: for<'r> FromRow<'r, sqlx::mysql::MySqlRow>
    {
        blockingify!({
            Ok(self.adapter.run_query(query, args).await.map_err(|err| DbConnectorError::AdapterError(err))?)
        });
    }
}


#[cfg(feature = "integration_testing")]
#[cfg(test)]
mod test {
    use super::*;


    #[test]
    fn test_run_command() {
        let mut conn = DbConnector::default().unwrap();
        let result = conn.command("INSERT INTO `test` (`id`) VALUES (?)", Some(&vec![42.into()]));
        assert_eq!(result, Ok(()));
    }


    #[derive(FromRow, Debug, PartialEq)]
    struct TestPrepared {
        id: i32,
        test_name: String
    }

    #[test]
    fn test_run_select_01() {
        let mut conn = DbConnector::default().unwrap();
        let rows: Vec<TestPrepared> = conn.query("SELECT * FROM `test_prepared` WHERE `id` = ?", Some(&vec![1.into()])).unwrap();
        assert_eq!(rows, vec![TestPrepared { id: 1, test_name: "aaa".to_string() }]);
    }

    #[test]
    fn test_run_select_02() {
        let mut conn = DbConnector::default().unwrap();
        let rows: Vec<TestPrepared> = conn.query("SELECT * FROM `test_prepared` WHERE `id` = ?", Some(&vec![2.into()])).unwrap();
        assert_eq!(rows, vec![TestPrepared { id: 2, test_name: "bbb".to_string() }]);
    }

    #[test]
    fn test_run_select_03() {
        let mut conn = DbConnector::default().unwrap();
        let rows: Vec<TestPrepared> = conn.query("SELECT * FROM `test_prepared` WHERE `id` = ?", Some(&vec![3.into()])).unwrap();
        assert_eq!(rows, vec![TestPrepared { id: 3, test_name: "ccc".to_string() }]);
    }
}
