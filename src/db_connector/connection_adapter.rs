use crate::settings;
use futures::TryStreamExt;
use sqlx::{FromRow, MySql, MySqlPool, Pool};


#[derive(Debug)]
pub struct Adapter(Pool<MySql>);


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
    pub async fn run_query<I, T>(&mut self, query: &'static str, args: Option<&Vec<I>>) -> Result<Vec<T>, String>
    where 
        I: for<'r> sqlx::Encode<'r, sqlx::MySql> + sqlx::Type<sqlx::MySql>,
        T: for<'r> FromRow<'r, sqlx::mysql::MySqlRow>
    {
        let mut query = sqlx::query(query);
        if let Some(args) = args {
            for arg in args.iter() {
                query = query.bind(arg);
            }
        };
        let mut query_result = query.fetch(&self.0);
        let mut result = Vec::new();
        while let Some(row) = query_result.try_next().await.map_err(|err| format!("{}", err))? {
            result.push(T::from_row(&row).map_err(|err| format!("{}", err))?);
        }
        Ok(result)
    }


    /// Command is a query **without expected response data**. Use `run_query` if data is expected.
    pub async fn run_command<T>(&mut self, command: &'static str, args: Option<&Vec<T>>) -> Result<(), String>
    where 
        T: for<'r> sqlx::Encode<'r, sqlx::MySql> + sqlx::Type<sqlx::MySql>
    {
        let mut query = sqlx::query(command);
        if let Some(args) = args {
            for arg in args.iter() {
                query = query.bind(arg);
            }
        };
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
        let result = adpt.run_command("INSERT INTO `test` (`id`) VALUES (?)", Some(&vec![42_u32])).await;
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
        let rows: Vec<TestPrepared> = adpt.run_query("SELECT * FROM `test_prepared` WHERE `id` = ?", Some(&vec![1])).await.unwrap();
        assert_eq!(rows, vec![TestPrepared { id: 1, test_name: "aaa".to_string() }]);
    }

    #[tokio::test]
    async fn test_run_select_02() {
        let mut adpt = Adapter::default().await.unwrap();
        let rows: Vec<TestPrepared> = adpt.run_query("SELECT * FROM `test_prepared` WHERE `id` = ?", Some(&vec![2])).await.unwrap();
        assert_eq!(rows, vec![TestPrepared { id: 2, test_name: "bbb".to_string() }]);
    }

    #[tokio::test]
    async fn test_run_select_03() {
        let mut adpt = Adapter::default().await.unwrap();
        let rows: Vec<TestPrepared> = adpt.run_query("SELECT * FROM `test_prepared` WHERE `id` = ?", Some(&vec![3])).await.unwrap();
        assert_eq!(rows, vec![TestPrepared { id: 3, test_name: "ccc".to_string() }]);
    }
}
