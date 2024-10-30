#[doc(hidden)]
pub const SQL_HOST: &'static str = "127.0.0.1:3306";
#[doc(hidden)]
pub const SQL_USER: &'static str = "dbfs";
#[doc(hidden)]
pub const SQL_PASSWD: &'static str = "dbfs";
#[doc(hidden)]
pub const SQL_DB: &'static str = "dbfs";


macro_rules! block_size{
    () => {
        4096
    }
}


// Please don't touch this
#[cfg(feature = "integration_testing")]
pub const FILE_BLOCK_SIZE: u64 = 4096;
#[cfg(feature = "integration_testing")]
pub const FILE_BLOCK_SIZE_32: u32 = 4096;
#[cfg(feature = "integration_testing")]
pub const FILE_BLOCK_SIZE_USIZE: usize = 4096;
#[cfg(not(feature = "integration_testing"))]
pub const FILE_BLOCK_SIZE: u64 = block_size!();
#[cfg(not(feature = "integration_testing"))]
pub const FILE_BLOCK_SIZE_32: u32 = block_size!();
#[cfg(not(feature = "integration_testing"))]
pub const FILE_BLOCK_SIZE_USIZE: usize = block_size!();
