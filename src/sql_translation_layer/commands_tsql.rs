//! Microsoft SQL variant Commands for use in `sql_translation_layer` module


use const_format::formatcp;
use crate::settings;


/// # Binds
/// - `inode_id`
///
/// # Columns
/// - `bytes`
/// - `blocks`
pub const SQL_GET_FILE_SIZE: &'static str = formatcp!(r#"SELECT
    ([block_id] - 1) * {block_size} + DATALENGTH([data]) as [bytes],
    [block_id] as [blocks]
FROM [block] WHERE [inode_id] = @inode_id 
ORDER BY [block_id] DESC 
OFFSET 0 ROWS FETCH FIRST 1 ROW ONLY"#, block_size=settings::FILE_BLOCK_SIZE);


/// # Binds
/// - `inode_id`
///
/// # Columns
/// - `bytes`
/// - `blocks`
/// - `last_block_id`
pub const SQL_GET_SIZE_AND_HEAD: &'static str = formatcp!(r#"WITH [ino] AS (SELECT @inode_id AS [ino]), [file_tmp] ([blocks]) AS (
    SELECT COUNT(*) FROM [block] WHERE [inode_id] = (SELECT [ino] FROM [ino])
) SELECT
    [blocks] * {block_size} - (
        SELECT {block_size} - DATALENGTH([data]) 
        FROM [block] 
        WHERE [inode_id] = (SELECT [ino] FROM [ino]) 
        ORDER BY [block_id] DESC 
        OFFSET 0 ROWS FETCH FIRST 1 ROW ONLY
    ) AS [bytes],
    [blocks] AS [blocks],
    ISNULL(
        (
            SELECT [block_id] 
            FROM [block] 
            WHERE [inode_id] = (SELECT [ino] FROM [ino]) 
            ORDER BY [block_id] DESC 
            OFFSET 0 ROWS FETCH FIRST 1 ROW ONLY
        ), 
        CAST(0 AS BIGINT)
    ) AS [last_block_id]
FROM [file_tmp]"#, block_size=settings::FILE_BLOCK_SIZE);
