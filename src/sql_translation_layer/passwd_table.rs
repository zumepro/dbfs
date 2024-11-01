use std::{collections::HashMap, sync::Mutex};
use users::get_user_by_uid;

use super::{commands, database_objects};


pub struct PasswdTable {
    users: HashMap<u32, String>,
    groups: HashMap<u32, String>,
}


impl PasswdTable {
    pub fn new(adapter: &mut Mutex<super::DbConnector>) -> Result<Self, super::Error> {
	let mut conn = adapter.lock().map_err(|_| super::Error::RuntimeError(super::CONN_LOCK_FAILED))?;
	let users: Vec<database_objects::User> = conn.query(commands::SQL_GET_USERS, None)?;
	let groups: Vec<database_objects::Group> = conn.query(commands::SQL_GET_GROUPS, None)?;

	let mut this: Self = Self { users: HashMap::new(), groups: HashMap::new() };

	for user in users.iter() {
	    this.users.insert(user.id, user.name.clone());
	}
	for group in groups.iter() {
	    this.groups.insert(group.id, group.name.clone());
	}

	Ok(this)
    }


    pub fn check(&mut self, adapter: &mut Mutex<super::DbConnector>, user: u32, group: u32) -> Result<(), super::Error> {
	let exists: (bool, bool) = (self.users.contains_key(&user), self.groups.contains_key(&group));
	if exists.0 && exists.1 { return Ok(()); }

	// If user or group does not exist in the table already - let's insert
	let mut conn = adapter.lock().map_err(|_| super::Error::RuntimeError(super::CONN_LOCK_FAILED))?;

	if ! exists.0 {
	    let user_read = get_user_by_uid(user).ok_or(super::Error::RuntimeError("Unable to read user from passwd"))?;
	    let user_name_converted = user_read.name().to_str().ok_or(super::Error::RuntimeError("Unable to convert username from OsString"))?;
	    conn.command(commands::SQL_INSERT_USER, Some(&vec![user.into(), user_name_converted.into()]))?;
	}

	if ! exists.0 {
	    let group_read = get_user_by_uid(user).ok_or(super::Error::RuntimeError("Unable to read group from passwd"))?;
	    let group_name_converted = group_read.name().to_str().ok_or(super::Error::RuntimeError("Unable to convert username from OsString"))?;
	    conn.command(commands::SQL_INSERT_GROUP, Some(&vec![user.into(), group_name_converted.into()]))?;
	}

	Ok(())
    }
}


#[cfg(feature = "integration_testing")]
#[cfg(test)]
mod test {
}
