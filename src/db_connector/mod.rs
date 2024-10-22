pub use sqlx::FromRow;
use tokio::runtime::Runtime;
use tokio::runtime::Builder;


use connection_adapter::Adapter;
mod connection_adapter;


#[derive(Debug, PartialEq)]
pub enum DbConnectorError {
    RuntimeStartFail,
    AdapterError(String),
}

impl std::fmt::Display for DbConnectorError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "db_connector_error: {}", match self {
            Self::RuntimeStartFail => "could not start async runner".to_string(),
            Self::AdapterError(val) => format!("sql_adapter: {}", val),
        })
    }
}


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

    pub fn command<T>(&mut self, command: &'static str, args: Option<&Vec<T>>) -> Result<(), DbConnectorError>
    where 
        T: for<'r> sqlx::Encode<'r, sqlx::MySql> + sqlx::Type<sqlx::MySql>
    {
        blockingify!({
            self.adapter.run_command(command, args).await.map_err(|err| DbConnectorError::AdapterError(err))?;
            Ok(())
        });
    }

    pub fn query<I, T>(&mut self, query: &'static str, args: Option<&Vec<I>>) -> Result<Vec<T>, DbConnectorError>
    where
        I: for<'r> sqlx::Encode<'r, sqlx::MySql> + sqlx::Type<sqlx::MySql>,
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
        let result = conn.command("INSERT INTO `test` (`id`) VALUES (?)", Some(&vec![42_u32]));
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
        let rows: Vec<TestPrepared> = conn.query("SELECT * FROM `test_prepared` WHERE `id` = ?", Some(&vec![1])).unwrap();
        assert_eq!(rows, vec![TestPrepared { id: 1, test_name: "aaa".to_string() }]);
    }

    #[test]
    fn test_run_select_02() {
        let mut conn = DbConnector::default().unwrap();
        let rows: Vec<TestPrepared> = conn.query("SELECT * FROM `test_prepared` WHERE `id` = ?", Some(&vec![2])).unwrap();
        assert_eq!(rows, vec![TestPrepared { id: 2, test_name: "bbb".to_string() }]);
    }

    #[test]
    fn test_run_select_03() {
        let mut conn = DbConnector::default().unwrap();
        let rows: Vec<TestPrepared> = conn.query("SELECT * FROM `test_prepared` WHERE `id` = ?", Some(&vec![3])).unwrap();
        assert_eq!(rows, vec![TestPrepared { id: 3, test_name: "ccc".to_string() }]);
    }
}
