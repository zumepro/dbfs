use std::{array::TryFromSliceError, string::FromUtf8Error};
use super::ColumnType;




#[derive(Debug)]
enum SQLGetColError {
    ColumnNotFound(String),
    ParseError(SQLParseError),
}


impl std::fmt::Display for SQLGetColError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ColumnNotFound(name) => write!(f, "sql_connector: get_col: could not find column with name \"{}\"", name),
            Self::ParseError(err) => write!(f, "sql_connector: get_col: parse_value: {}", err),
        }
    }
}


impl From<SQLParseError> for SQLGetColError {
    fn from(value: SQLParseError) -> Self { Self::ParseError(value) }
}


impl From<SQLGetColError> for String {
    fn from(value: SQLGetColError) -> Self { format!("{}", value) }
}




#[derive(Debug)]
enum SQLParseError {
    CorruptedData,
    TypeDiscrepancy((&'static str, &'static str)),
}


impl From<TryFromSliceError> for SQLParseError {
    fn from(_: TryFromSliceError) -> Self { SQLParseError::CorruptedData }
}


impl From<FromUtf8Error> for SQLParseError {
    fn from(_: FromUtf8Error) -> Self { SQLParseError::CorruptedData }
}


impl std::fmt::Display for SQLParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::CorruptedData => write!(f, "encountered corrupted data when parsing value"),
            Self::TypeDiscrepancy((expected, got)) => write!(f, "runtime type checking failed wanted \"{}\" (however database returned \"{}\")", expected, got),
        }
    }
}




#[derive(Default, Debug)]
pub struct Row(Vec<RowCell>);




#[derive(Default, Debug)]
struct RowCell {
    col_type: ColumnType,
    col_name: String,
    value: Vec<u8>,
}




trait FromSQLResponseData: Sized {
    fn from_sql_data(response_data: &Vec<u8>) -> Result<Self, SQLParseError>;
}


impl FromSQLResponseData for u32 {
    fn from_sql_data(response_data: &Vec<u8>) -> Result<Self, SQLParseError> {
        Ok(u32::from_le_bytes(response_data[0..4].try_into()?))
    }
}


impl FromSQLResponseData for String {
    fn from_sql_data(response_data: &Vec<u8>) -> Result<Self, SQLParseError> {
        Ok(String::from_utf8(response_data.clone())?)
    }
}


impl FromSQLResponseData for Vec<u8> {
    fn from_sql_data(response_data: &Vec<u8>) -> Result<Self, SQLParseError> {
        Ok(response_data.clone())
    }
}




impl Row {

    pub fn get_col<'a, T: FromSQLResponseData>(&self, column_name: &'a str) -> Result<T, SQLGetColError> {
        for col in self.0.iter() {
            if col.col_name != column_name { continue; }
            return Ok(T::from_sql_data(&col.value)?)
        }
        Err(SQLGetColError::ColumnNotFound(column_name.to_string()))
    }

}


#[cfg(not(feature = "inet_testing"))]
#[cfg(test)]
mod test {


    use super::*;


    #[test]
    fn sql_to_u32() {
        let row = Row(vec![RowCell {
            col_type: ColumnType::MysqlInteger,
            col_name: "test".to_string(),
            value: vec![42, 0, 0, 0],
        }]);
        assert_eq!(row.get_col::<u32>("test").unwrap(), 42);
    }


    #[test]
    fn sql_to_string() {
        let row = Row(vec![RowCell {
            col_type: ColumnType::MysqlString,
            col_name: "test".to_string(),
            value: vec!['t' as u8, 'e' as u8, 's' as u8, 't' as u8],
        }]);
        assert_eq!(row.get_col::<String>("test").unwrap(), "test");
    }


    #[test]
    fn sql_to_vec_u8() {
        let row = Row(vec![RowCell {
            col_type: ColumnType::MysqlInteger,
            col_name: "test".to_string(),
            value: vec![1, 2, 3, 4],
        }]);
        assert_eq!(row.get_col::<Vec<u8>>("test").unwrap(), vec![1, 2, 3, 4]);
    }


    #[test]
    fn sql_error_to_string() {
        let row: Result<Vec<u8>, String> = Row(vec![RowCell {
            col_type: ColumnType::MysqlString,
            col_name: "testing".to_string(),
            value: vec![1, 2, 3, 4],
        }]).get_col("test").map_err(|err| err.into());
        assert_eq!(row, Err("sql_connector: get_col: could not find column with name \"test\"".to_string()));
    }

}
