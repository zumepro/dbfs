mod settings;
mod db_connector;
mod sql_translation_layer;


fn main() {
    println!("Hello, world!");
}


#[cfg(test)]
mod test {
    #[test]
    fn test_test() {
        assert_eq!(1, 1);
    }
}