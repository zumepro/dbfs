use crate::db_connector::{FromRow, chrono};


#[derive(Debug, PartialEq, FromRow, Clone)]
pub struct FileSize {
    pub bytes: i64,
    pub blocks: i64,
}
impl Copy for FileSize {}


#[derive(Debug, PartialEq, FromRow)]
pub struct FileHardlinks {
    pub hardlinks: i64,
}


#[derive(Debug, PartialEq, FromRow)]
pub struct DirectoryChildrenDirectory {
    pub children_dirs: i64,
}


#[derive(Debug, PartialEq, FromRow)]
pub struct Inode {
    pub id: u32,
    pub owner: u32,
    pub group: u32,
    pub file_type: String,
    pub special_bits: u8,
    pub user_perm: u8,
    pub group_perm: u8,
    pub other_perm: u8,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub modified_at: chrono::DateTime<chrono::Utc>,
    pub accessed_at: chrono::DateTime<chrono::Utc>,
}


#[derive(Debug, PartialEq, FromRow)]
pub struct DirectoryEntry {
    pub name: String,
    pub inode_id: u32,
    pub file_type: String,
}


#[derive(Debug, PartialEq, FromRow)]
pub struct DirectoryParent {
    pub parent_inode_id: u32
}


#[derive(Debug, PartialEq, FromRow)]
pub struct InodeLookup {
    pub inode_id: u32
}


#[derive(Debug, PartialEq, FromRow)]
pub struct FilesystemStat {
    pub used_blocks: i64,
    pub used_inodes: i64
}


#[derive(Debug, PartialEq, FromRow)]
pub struct ChildrenCount {
    pub children: i64
}


#[derive(Debug, PartialEq, FromRow)]
pub struct BlockData {
    pub data: Vec<u8>,
}


#[derive(Debug, PartialEq, FromRow)]
pub struct Block {
    pub inode_id: u32,
    pub block_id: u32,
    pub data: Vec<u8>,
}
