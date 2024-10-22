use crate::db_connector::FromRow;


#[derive(Debug, PartialEq, FromRow)]
pub struct File {
    pub id: i32,
    pub name: String,
    pub inode_id: i32,
}


#[derive(Debug, PartialEq, FromRow)]
pub struct Inode {
    pub id: i32,
    pub mode: Vec<u8>,
    pub owner: i32,
    pub group: i32,
    pub size: i32,
}


#[derive(Debug, PartialEq, FromRow)]
pub struct Block {
    pub inode_id: i32,
    pub block_id: i32,
    pub data: Vec<u8>,
}


#[derive(Debug, PartialEq, FromRow)]
pub struct Listing {
    pub parent_file_id: i32,
    pub child_file_id: i32,
}


#[derive(Debug, PartialEq, FromRow)]
pub struct User {
    pub id: i32,
    pub name: String,
}


#[derive(Debug, PartialEq, FromRow)]
pub struct Group {
    pub id: i32,
    pub name: String,
}
