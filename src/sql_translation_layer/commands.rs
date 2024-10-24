//! SQL Command for use in `sql_translation_layer` module


pub const SQL_GET_FILE_SIZE: &'static str = r#"WITH `ino` AS (SELECT ? AS `ino`), `file_tmp` (`blocks`) AS (
    SELECT COUNT(*) FROM `block` WHERE `inode_id` = (SELECT `ino` FROM `ino`)
) SELECT
    `blocks` * 4096 - (SELECT 4096 - OCTET_LENGTH(`data`) FROM `block` WHERE `inode_id` = (SELECT `ino` FROM `ino`) ORDER BY `block_id` DESC LIMIT 1) AS bytes,
    `blocks` AS blocks
FROM `file_tmp`"#;


pub const SQL_COUNT_HARDLINKS: &'static str = r#"SELECT COUNT(*) AS `hardlinks` FROM `file` WHERE `inode_id` = ?"#;


pub const SQL_GET_INODE: &'static str = r#"SELECT * FROM `inode` WHERE `id` = ?"#;
