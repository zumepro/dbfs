use crate::settings;
use futures::TryStreamExt;
use sqlx::{FromRow, MySql, MySqlPool, Pool};


#[derive(Debug)]
/// Database input type interface.
/// Multiple datatypes can be automatically converted.
///
/// # Example usage
/// ```rust
/// db_conn.run_command("INSERT INTO `test` (`id`) VALUES (?)", Some(&vec![42.into()]));
/// db_conn.run_command("INSERT INTO `test` (`char_value`) VALUES (?)", Some(&vec!["Hello, world!".into()]));
/// db_conn.run_command("INSERT INTO `test` (`blob`) VALUES (?)", Some(&vec![vec![1_u8, 2_u8, 3_u8].into()]));
/// ```
///
/// # Supported datatypes
/// Automatic conversion can be done from the following datatypes: `i32`, `&str`, `String`, `Vec<u8>`
pub enum DbInputType {
    Integer(i32),
    Char(String),
    Blob(Vec<u8>),
}
impl Into<DbInputType> for i32 { fn into(self) -> DbInputType { DbInputType::Integer(self) } }
impl Into<DbInputType> for String { fn into(self) -> DbInputType { DbInputType::Char(self) } }
impl<'a> Into<DbInputType> for &'a str { fn into(self) -> DbInputType { DbInputType::Char(String::from(self)) } }
impl Into<DbInputType> for Vec<u8> { fn into(self) -> DbInputType { DbInputType::Blob(self) } }


#[derive(Debug)]
/// SQL connection adapter.
pub struct Adapter(Pool<MySql>);


macro_rules! prepared_stmt_bind_args {
    ($args: ident, $query: ident) => {
        if let Some($args) = $args {
            for arg in $args.iter() {
                $query = match arg {
                    DbInputType::Integer(val) => $query.bind(val),
                    DbInputType::Char(val) => $query.bind(val),
                    DbInputType::Blob(val) => $query.bind(val),
                };
            }
        };
    }
}


impl Adapter {


    pub async fn default() -> Result<Self, String> {
        Ok(Self (
            MySqlPool::connect_lazy(format!(
                "mysql://{}:{}@{}/{}",
                settings::SQL_USER,
                settings::SQL_PASSWD,
                settings::SQL_HOST,
                settings::SQL_DB,
            ).as_str()).map_err(|err| format!("{}", err))?
        ))
    }


    /// Query is a database query which **expects data** as a return. Use `run_command` if
    /// no data is expected.
    ///
    /// Will return `Ok(Vec<YourStruct>)` if the query was executed successfully and deserialized into
    /// `YourStruct`s. Each row is deserialized into one `YourStruct`. The returned set contains all
    /// collected rows.
    ///
    /// Will return `Err(String)` if there was an error while processing the query. The inner
    /// value is either returned directly from the server or contains connection fail info.
    ///
    /// # Example usage
    /// ```rust
    /// let rows: Vec<MyStruct> = adpt.run_query("SELECT * FROM `test_prepared` WHERE `id` = ?", Some(&vec![1.into()])).await.unwrap();
    /// ```
    pub async fn run_query<T>(&mut self, query: &'static str, args: Option<&Vec<DbInputType>>) -> Result<Vec<T>, String>
    where 
        T: for<'r> FromRow<'r, sqlx::mysql::MySqlRow>
    {
        let mut query = sqlx::query(query);
        prepared_stmt_bind_args!(args, query);
        let mut query_result = query.fetch(&self.0);
        let mut result = Vec::new();
        while let Some(row) = query_result.try_next().await.map_err(|err| format!("{}", err))? {
            result.push(T::from_row(&row).map_err(|err| format!("{}", err))?);
        }
        Ok(result)
    }


    /// Command is a query **without expected response data**. Use `run_query` if data is expected.
    ///
    /// Will return `Ok(())` if the command was executed successfully.
    ///
    /// Will return `Err(String)` if there was an error while processing the command. The inner
    /// value is either returned directly from the server or contains connection fail info.
    ///
    /// # Example usage
    /// ```rust
    /// adpt.run_command("INSERT INTO `test` (`id`) VALUES (?)", Some(&vec![42.into()])).await.unwrap();
    /// ```
    pub async fn run_command(&mut self, command: &'static str, args: Option<&Vec<DbInputType>>) -> Result<(), String> {
        let mut query = sqlx::query(command);
        prepared_stmt_bind_args!(args, query);
        query.execute(&self.0).await.map_err(|err| format!("{}", err))?;
        Ok(())
    }
}


#[cfg(feature = "integration_testing")]
#[cfg(test)]
mod test {
    use super::*;


    #[tokio::test]
    async fn test_run_command() {
        let mut adpt = Adapter::default().await.unwrap();
        let result = adpt.run_command("INSERT INTO `test` (`id`) VALUES (?)", Some(&vec![42.into()])).await;
        assert_eq!(result, Ok(()));
    }


    #[derive(FromRow, Debug, PartialEq)]
    struct TestPrepared {
        id: i32,
        test_name: String
    }

    #[tokio::test]
    async fn test_run_select_01() {
        let mut adpt = Adapter::default().await.unwrap();
        let rows: Vec<TestPrepared> = adpt.run_query("SELECT * FROM `test_prepared` WHERE `id` = ?", Some(&vec![1.into()])).await.unwrap();
        assert_eq!(rows, vec![TestPrepared { id: 1, test_name: "aaa".to_string() }]);
    }

    #[tokio::test]
    async fn test_run_select_02() {
        let mut adpt = Adapter::default().await.unwrap();
        let rows: Vec<TestPrepared> = adpt.run_query("SELECT * FROM `test_prepared` WHERE `id` = ?", Some(&vec![2.into()])).await.unwrap();
        assert_eq!(rows, vec![TestPrepared { id: 2, test_name: "bbb".to_string() }]);
    }

    #[tokio::test]
    async fn test_run_select_03() {
        let mut adpt = Adapter::default().await.unwrap();
        let rows: Vec<TestPrepared> = adpt.run_query("SELECT * FROM `test_prepared` WHERE `id` = ?", Some(&vec![3.into()])).await.unwrap();
        assert_eq!(rows, vec![TestPrepared { id: 3, test_name: "ccc".to_string() }]);
    }

    #[derive(FromRow, Debug, PartialEq)]
    struct TestPrepared02 {
        id: i32,
        test_name: Vec<u8>,
    }

    #[tokio::test]
    async fn test_run_select_04() {
        let mut adpt = Adapter::default().await.unwrap();
        let rows: Vec<TestPrepared02> = adpt.run_query("SELECT * FROM `test_prepared` WHERE `id` = ?", Some(&vec![3.into()])).await.unwrap();
        assert_eq!(rows, vec![TestPrepared02 { id: 3, test_name: vec!['c' as u8, 'c' as u8, 'c' as u8] }]);
    }
}
