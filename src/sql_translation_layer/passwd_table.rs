use std::{collections::HashMap, sync::Mutex};
use users::{get_user_by_uid, get_group_by_gid};
use super::{commands, database_objects};
use crate::debug;


pub struct PasswdTable {
    users: HashMap<u32, String>,
    groups: HashMap<u32, String>,
}


impl Default for PasswdTable {
    /// Will create a new empty locally-stored [`PasswdTable`].
    ///
    /// # Warning
    /// This function does _not_ populate the created [`PasswdTable`] with data fetched from
    /// database. (it is better suited for unit testing)
    fn default() -> Self {
        Self {
            users: HashMap::new(),
            groups: HashMap::new(),
        }
    }
}


impl PasswdTable {
    /// Try to create a new locally stored [`PasswdTable`] and insert data fetched from a database.
    /// 
    /// This will return [`super::Error`] if some command (or the adapter connection itself) fails.
    pub fn new(adapter: &Mutex<super::DbConnector>) -> Result<Self, super::Error> {
    let mut conn = adapter.lock().map_err(|_| super::Error::RuntimeError(super::CONN_LOCK_FAILED))?;
    let users: Vec<database_objects::User> = conn.query(commands::SQL_GET_USERS, None)?;
    let groups: Vec<database_objects::Group> = conn.query(commands::SQL_GET_GROUPS, None)?;

    let mut this: Self = Self { users: HashMap::new(), groups: HashMap::new() };

    debug!("ownermgr: initializing table");

    for user in users.iter() {
        this.users.insert(user.id, user.name.clone());
        debug!("    fetch user \"{}\" ({})", user.name.clone(), user.id);
    }
    for group in groups.iter() {
        this.groups.insert(group.id, group.name.clone());
        debug!("    fetch group \"{}\" ({})", group.name.clone(), group.id);
    }
    debug!("ownermgr: done table initialization");

    Ok(this)
    }


    /// Check if a user + group exist in the locally stored [`PasswdTable`].
    /// If the user _does not_ exist in the locally stored [`PasswdTable`], then this function will
    /// try to fetch the name of the user and group and insert it into database and the locally
    /// stored [`PasswdTable`].
    pub fn check(&mut self, adapter: &Mutex<super::DbConnector>, user: u32, group: u32) -> Result<(), super::Error> {
    let exists: (bool, bool) = (self.users.contains_key(&user), self.groups.contains_key(&group));
    if exists.0 && exists.1 { return Ok(()); }

    // If user or group does not exist in the table already - let's insert
    let mut conn = adapter.lock().map_err(|_| super::Error::RuntimeError(super::CONN_LOCK_FAILED))?;

    if !exists.0 {
		let name = self.get_user_by_uid(user)?;
        debug!("ownermgr: useradd: Adding user \"{}\" with uid {}", &name, user);
        conn.command(commands::SQL_INSERT_USER, Some(&vec![user.into(), name.as_str().into()]))?;
        self.users.insert(user, name);
    }

    if !exists.1 {
		let name = self.get_group_by_gid(group)?;
        debug!("ownermgr: groupadd: Adding group \"{}\" with gid {}", &name, group);
        conn.command(commands::SQL_INSERT_GROUP, Some(&vec![group.into(), name.as_str().into()]))?;
        self.groups.insert(group, name);
    }

    Ok(())
    }


    fn _check_offline(&mut self, user: u32, group: u32) -> Result<(), super::Error> {
    let exists: (bool, bool) = (self.users.contains_key(&user), self.groups.contains_key(&group));
    if exists.0 && exists.1 { return Ok(()); }

    if !exists.0 {
        self.users.insert(user, self.get_user_by_uid(user)?);
    }

    if !exists.0 {
        self.groups.insert(group, self.get_group_by_gid(group)?);
    }

    Ok(())
    }
	

	fn get_user_by_uid(&self, user: u32) -> Result<String, super::Error> {
        let user_read = match get_user_by_uid(user) {
			Some(val) => val,
			None => return Ok(format!("user{}", user))
		};
        let user_name_converted = user_read.name().to_str().ok_or(super::Error::RuntimeError("Unable to convert user name from OsStr"))?;

		Ok(user_name_converted.to_string())
	}


	fn get_group_by_gid(&self, group: u32) -> Result<String, super::Error> {
        let group_read = match get_group_by_gid(group) {
			Some(val) => val,
			None => return Ok(format!("group{}", group))
		};
        let group_name_converted = group_read.name().to_str().ok_or(super::Error::RuntimeError("Unable to convert group name from OsStr"))?;

		Ok(group_name_converted.to_string())
	}
}


#[cfg(feature = "integration_testing")]
#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn get_root() {
        let mut table = PasswdTable::default();
        table._check_offline(0, 0).unwrap();
        assert_eq!(table.users.get(&0), Some(&"root".to_string()));
        assert_eq!(table.groups.get(&0), Some(&"root".to_string()));
    }
}
