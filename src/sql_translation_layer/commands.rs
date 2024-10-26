//! SQL Command for use in `sql_translation_layer` module


pub const SQL_GET_FILE_SIZE: &'static str = r#"WITH `ino` AS (SELECT ? AS `ino`), `file_tmp` (`blocks`) AS (
    SELECT COUNT(*) FROM `block` WHERE `inode_id` = (SELECT `ino` FROM `ino`)
) SELECT
    `blocks` * 4096 - (SELECT 4096 - OCTET_LENGTH(`data`) FROM `block` WHERE `inode_id` = (SELECT `ino` FROM `ino`) ORDER BY `block_id` DESC LIMIT 1) AS bytes,
    `blocks` AS blocks
FROM `file_tmp`"#;


pub const SQL_COUNT_HARDLINKS: &'static str = r#"SELECT COUNT(*) AS `hardlinks` FROM `file` WHERE `inode_id` = ?"#;


pub const SQL_GET_INODE: &'static str = r#"SELECT * FROM `inode` WHERE `id` = ?"#;


pub const SQL_LIST_DIRECTORY: &'static str = r#"WITH `ino` AS (SELECT ? AS `ino`), `file_info` AS (SELECT `name`, `inode_id` FROM `file` WHERE `parent_inode_id` = (SELECT `ino` FROM `ino`) AND `inode_id` != (SELECT `ino` FROM `ino`)) SELECT
    `name` AS `name`,
    `inode_id`,
    (SELECT `file_type` FROM `inode` WHERE `id` = `file_info`.`inode_id`) AS `file_type`
FROM `file_info` ORDER BY `inode_id`"#;


pub const SQL_GET_DIRECTORY_PARENT: &'static str = r#"SELECT `parent_inode_id` FROM `file` WHERE `inode_id` = ?"#;
