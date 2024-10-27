//! SQL Command for use in `sql_translation_layer` module


/// # Binds
/// - `inode_id`
///
/// # Columns
/// - `bytes`
/// - `blocks`
pub const SQL_GET_FILE_SIZE: &'static str = r#"WITH `ino` AS (SELECT ? AS `ino`), `file_tmp` (`blocks`) AS (
    SELECT COUNT(*) FROM `block` WHERE `inode_id` = (SELECT `ino` FROM `ino`)
) SELECT
    `blocks` * 4096 - (SELECT 4096 - OCTET_LENGTH(`data`) FROM `block` WHERE `inode_id` = (SELECT `ino` FROM `ino`) ORDER BY `block_id` DESC LIMIT 1) AS bytes,
    `blocks` AS blocks
FROM `file_tmp`"#;


/// # Binds
/// - `inode_id`
///
/// # Columns
/// - `children_dirs`
pub const SQL_COUNT_CHILDREN_OF_TYPE_DIRECTORY: &'static str = r#"WITH `ino` AS (SELECT ? AS `ino`) SELECT COUNT(*) AS `children_dirs` FROM `inode` WHERE `id` IN (SELECT `inode_id` FROM `file` WHERE `parent_inode_id` = (SELECT `ino` FROM `ino`)) AND `id` != (SELECT `ino` FROM `ino`) AND `file_type` = 'd'"#;


/// # Binds
/// - `inode_id`
///
/// # Columns
/// - `hardlinks`
pub const SQL_COUNT_HARDLINKS: &'static str = r#"SELECT COUNT(*) AS `hardlinks` FROM `file` WHERE `inode_id` = ?"#;


/// # Binds
/// - `parent_inode_id`
///
/// # Columns
/// - `children`
pub const SQL_COUNT_DIRECTORY_CHILDREN: &'static str = r#"SELECT COUNT(*) AS `children` FROM `file` WHERE `parent_inode_id` = ?"#;


/// # Binds
/// - `inode_id`
///
/// # Columns
/// _all `inode` fields_
pub const SQL_GET_INODE: &'static str = r#"SELECT * FROM `inode` WHERE `id` = ?"#;


/// # Binds
/// - `inode_id`
///
/// # Columns
///
/// - `name`
/// - `inode_id`
/// - `file_type`
pub const SQL_LIST_DIRECTORY: &'static str = r#"WITH `ino` AS (SELECT ? AS `ino`), `file_info` AS (SELECT `name`, `inode_id` FROM `file` WHERE `parent_inode_id` = (SELECT `ino` FROM `ino`) AND `inode_id` != (SELECT `ino` FROM `ino`)) SELECT
    `name` AS `name`,
    `inode_id`,
    (SELECT `file_type` FROM `inode` WHERE `id` = `file_info`.`inode_id`) AS `file_type`
FROM `file_info` ORDER BY `inode_id`"#;


/// # Binds
/// - `inode_id`
///
/// # Columns
/// - `parent_inode_id`
pub const SQL_GET_DIRECTORY_PARENT: &'static str = r#"SELECT `parent_inode_id` FROM `file` WHERE `inode_id` = ?"#;


/// # Binds
/// - `name`
/// - `parent_inode_id`
///
/// # Columns
/// - `inode_id`
pub const SQL_LOOKUP_INODE_ID: &'static str = r#"SELECT `inode_id` FROM `file` WHERE `name` = ? AND `parent_inode_id` = ?"#;


/// # Binds
/// 
/// # Columns
/// - `used_inodes`
/// - `used_blocks`
pub const SQL_GET_FS_STAT: &'static str = r#"SELECT
(SELECT COUNT(*) FROM `inode`) AS `used_inodes`,
(SELECT COUNT(*) FROM `block`) AS `used_blocks`"#;


/// # Binds
/// - `dest_parent_inode_id`
/// - `dest_name`
/// - `src_parent_inode_id`
/// - `src_name`
pub const SQL_RENAME_FILE: &'static str = r#"UPDATE `file`
SET `parent_inode_id` = ?, `name` = ?
WHERE `parent_inode_id` = ? AND `name` = ?"#;


/// # Binds
/// - `name`
/// - `parent_inode_id`
pub const SQL_DELETE_FILE: &'static str = r#"DELETE FROM `file` WHERE `name` = ? AND `parent_inode_id` = ?"#;


/// # Binds
/// - `id`
pub const SQL_DELETE_INODE: &'static str = r#"DELETE FROM `inode` WHERE `id` = ?"#;


/// # Binds
/// - `inode_id`
/// - `max_blocks`
/// - `offset_blocks`
///
/// # Columns
/// - `data`
pub const SQL_READ_FILE: &'static str = r#"SELECT `data` FROM `block` WHERE `inode_id` = ? ORDER BY `block_id` ASC LIMIT ? OFFSET ?"#;
