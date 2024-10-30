//! SQL Command for use in `sql_translation_layer` module


use const_format::formatcp;
use crate::settings;


/// # Binds
/// - `inode_id`
///
/// # Columns
/// - `bytes`
/// - `blocks`
pub const SQL_GET_FILE_SIZE: &'static str = formatcp!(r#"WITH `ino` AS (SELECT ? AS `ino`), `file_tmp` (`blocks`) AS (
    SELECT COUNT(*) FROM `block` WHERE `inode_id` = (SELECT `ino` FROM `ino`)
) SELECT
    `blocks` * {block_size} - (SELECT {block_size} - OCTET_LENGTH(`data`) FROM `block` WHERE `inode_id` = (SELECT `ino` FROM `ino`) ORDER BY `block_id` DESC LIMIT 1) AS bytes,
    `blocks` AS blocks
FROM `file_tmp`"#, block_size=settings::FILE_BLOCK_SIZE);


/// # Binds
/// - `inode_id`
///
/// # Columns
/// - `bytes`
/// - `blocks`
/// - `last_block_id`
pub const SQL_GET_SIZE_AND_HEAD: &'static str = formatcp!(r#"WITH `ino` AS (SELECT ? AS `ino`), `file_tmp` (`blocks`) AS (
    SELECT COUNT(*) FROM `block` WHERE `inode_id` = (SELECT `ino` FROM `ino`)
) SELECT
    `blocks` * {block_size} - (SELECT {block_size} - OCTET_LENGTH(`data`) FROM `block` WHERE `inode_id` = (SELECT `ino` FROM `ino`) ORDER BY `block_id` DESC LIMIT 1) AS bytes,
    `blocks` AS blocks,
    (IFNULL((SELECT `block_id` FROM `block` WHERE `inode_id` = (SELECT `ino` FROM `ino`) ORDER BY `block_id` DESC LIMIT 1), CAST(0 AS UNSIGNED))) AS `last_block_id`
FROM `file_tmp`"#, block_size=settings::FILE_BLOCK_SIZE);


/// # Binds
/// - `inode_id`
/// - `start_block_id`
/// - `end_block_id`
///
/// # Columns
/// - `blocks`
/// - `start_block_data`
/// - `end_block_data`
pub const SQL_GET_SIZE_AND_BLOCK_DATA: &'static str = r#"WITH `ino` AS (SELECT ? AS `ino`) SELECT (SELECT COUNT(*) FROM `block` WHERE `inode_id` = (SELECT `ino` FROM `ino`)) AS `blocks`, IFNULL((SELECT `data` FROM `block` WHERE `inode_id` = (SELECT `ino` FROM `ino`) ORDER BY `block_id` ASC LIMIT 1 OFFSET ?), '') AS `start_block_data`, IFNULL((SELECT `data` FROM `block` WHERE `inode_id` = (SELECT `ino` FROM `ino`) ORDER BY `block_id` ASC LIMIT 1 OFFSET ?), '') AS `end_block_data`"#;


/// # Binds
/// - `inode_id`
/// - `block_id`
///
/// # Columns
/// - `blocks`
/// - `block_data`
pub const SQL_GET_SIZE_AND_SINGLE_BLOCK_DATA: &'static str = r#"WITH `ino` AS (SELECT ? AS `ino`) SELECT (SELECT COUNT(*) FROM `block` WHERE `inode_id` = (SELECT `ino` FROM `ino`)) AS `blocks`, IFNULL((SELECT `data` FROM `block` WHERE `inode_id` = (SELECT `ino` FROM `ino`) ORDER BY `block_id` ASC LIMIT 1 OFFSET ?), '') AS `block_data`"#;


/// # Binds
/// - `inode_id`
///
/// # Columns
/// - `blocks`
pub const SQL_GET_SIZE_ONLY: &'static str = r#"SELECT COUNT(*) as `blocks` FROM `block` WHERE `inode_id` = ?"#;


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
/// - `owner`
/// - `group`
/// - `accessed_at`
/// - `modified_at`
/// - `created_at`
/// - `special_bits`
/// - `user_perm`
/// - `group_perm`
/// - `other_perm`
/// - `id`
pub const SQL_UPDATE_INODE: &'static str = r#"UPDATE `inode` SET
`owner` = ?,
`group` = ?,
`accessed_at` = ?,
`modified_at` = ?,
`created_at` = ?,
`special_bits` = ?,
`user_perm` = ?,
`group_perm` = ?,
`other_perm` = ?
WHERE `id` = ?"#;


/// # Binds
/// - `id`
pub const SQL_UPDATE_INODE_CTIME_MTIME: &'static str = r#"UPDATE `inode` SET
`modified_at` = CURRENT_TIMESTAMP(),
`created_at` = CURRENT_TIMESTAMP()
WHERE `id` = ?"#;


/// # Binds
/// - `owner`
/// - `group`
/// - `file_type`
/// - `special_bits`
/// - `user_perm`
/// - `group_perm`
/// - `other_perm`
///
/// # Columns
/// - `id`
pub const SQL_CREATE_INODE: &'static str = r#"INSERT INTO `inode`
(`owner`, `group`, `file_type`, `special_bits`, `user_perm`, `group_perm`, `other_perm`)
VALUES (?, ?, ?, ?, ?, ?, ?)"#;


/// # Binds
/// - `parent_inode_id`
/// - `name`
/// - `inode_id`
pub const SQL_CREATE_FILE: &'static str = r#"INSERT INTO `file`
(`parent_inode_id`, `name`, `inode_id`)
VALUES (?, ?, ?)"#;


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


/// # Binds
/// - `inode_id`
/// - `max_blocks`
/// - `offset_blocks`
///
/// # Columns
/// - `data`
pub const SQL_GET_FULL_BLOCKS: &'static str = r#"SELECT * FROM `block` WHERE `inode_id` = ? ORDER BY `block_id` ASC LIMIT ? OFFSET ?"#;


/// # Binds
/// - `inode_id`
///
/// # Columns
/// - `bc` (block count)
/// - `last_block_id`
pub const SQL_GET_FILE_HEAD: &'static str = r#"WITH `ino` AS (SELECT ? AS `ino`) SELECT COUNT(*) AS `bc`, IFNULL((SELECT `block_id` FROM `block` WHERE `inode_id` = (SELECT `ino` FROM `ino`) ORDER BY `block_id` DESC LIMIT 1), CAST(0 AS UNSIGNED)) AS `last_block_id` FROM `block` WHERE `inode_id` = (SELECT `ino` FROM `ino`)"#;


/// # Binds
/// - `inode_id`
/// - `strip_blocks_count`
pub const SQL_TRIM_BLOCKS: &'static str = r#"DELETE FROM `block` WHERE `inode_id` = ? ORDER BY `block_id` DESC LIMIT ?"#;


/// # Binds
/// - `last_block_length`
/// - `last_block_length`
/// - `inode_id`
pub const SQL_RESIZE_LAST_BLOCK: &'static str = r#"UPDATE `block` SET `data` = RPAD(SUBSTR(`data`, 1, ?), ?, CHAR(0)) WHERE `inode_id` = ? ORDER BY `block_id` DESC LIMIT 1"#;


/// # Binds
/// - `last_block_length`
/// - `inode_id`
pub const SQL_PAD_LAST_BLOCK: &'static str = r#"UPDATE `block` SET `data` = RPAD(`data`, ?, CHAR(0)) WHERE `inode_id` = ? ORDER BY `block_id` DESC LIMIT 1"#;


/// # Binds
/// - `inode_id`
pub const SQL_DROP_BLOCKS: &'static str = r#"DELETE FROM `block` WHERE `inode_id` = ?"#;


pub mod dynamic_queries {
    use const_format::formatcp;
    use crate::settings;
    use crate::sql_translation_layer::database_objects;

    /// # Binds
    /// - `data` _for every block_
    pub fn sql_write(blocks: &Vec<database_objects::Block>) -> String {
        let mut query = String::with_capacity(500);
        query.push_str("INSERT INTO `block` (`inode_id`, `block_id`, `data`) VALUES");
        for block in blocks.iter() {
            query.push_str(" (");
            query.push_str(&block.inode_id.to_string());
            query.push_str(",");
            query.push_str(&block.block_id.to_string());
            query.push_str(",?),");
        }
        query.pop();
        query.push_str(" ON DUPLICATE KEY UPDATE `inode_id`=VALUES(`inode_id`), `block_id`=VALUES(`block_id`), `data`=VALUES(`data`)");
        query
    }

    /// # Binds
    /// - `data` _for every block_
    pub fn sql_unsafe_write(inode_id: u64, start_block_id: u64, end_block_id: u64) -> String {
        let inode_id = inode_id.to_string();
        let mut query = String::with_capacity(500);
        query.push_str("INSERT INTO `block` (`inode_id`, `block_id`, `data`) VALUES");
        for block_id in start_block_id..=end_block_id {
            query.push_str(" (");
            query.push_str(&inode_id);
            query.push_str(",");
            query.push_str(&block_id.to_string());
            query.push_str(",?),");
        }
        query.pop();
        query.push_str(" ON DUPLICATE KEY UPDATE `inode_id`=VALUES(`inode_id`), `block_id`=VALUES(`block_id`), `data`=VALUES(`data`)");
        query
    }

    pub fn sql_pad_until(inode_id: u64, start_block_id: u64, until_block_id: u64) -> String {
        let inode_id = inode_id.to_string();
        let mut query = String::with_capacity(500);
        query.push_str("INSERT INTO `block` (`inode_id`, `block_id`, `data`) VALUES");
        for block_id in start_block_id..until_block_id {
            query.push_str(" (");
            query.push_str(&inode_id);
            query.push_str(", ");
            query.push_str(&block_id.to_string());
            query.push_str(formatcp!(", REPEAT(CHAR(0), {block_size})),", block_size=settings::FILE_BLOCK_SIZE));
        }
        query.pop();
        query.push_str(" ON DUPLICATE KEY UPDATE `inode_id`=VALUES(`inode_id`), `block_id`=VALUES(`block_id`), `data`=VALUES(`data`)");
        query
    }

    pub fn sql_pad_file(inode_id: u32, last_block_id: u32, count: u32) -> String {
        let mut query = String::with_capacity(500);
        query.push_str("INSERT INTO `block` (`inode_id`, `block_id`, `data`) VALUES");
        for block in 1..=count {
            query.push_str(" (");
            query.push_str(&inode_id.to_string());
            query.push_str(", ");
            query.push_str(&(last_block_id + block).to_string());
            query.push_str(formatcp!(", REPEAT(CHAR(0), {block_size})),", block_size=settings::FILE_BLOCK_SIZE));
        }
        query.pop();
        query
    }
}
