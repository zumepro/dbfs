# dbfs

Absolutely sane project. A working fuse adapter for DB-based filesystem.


### Database structure
<img src="er.jpg">


### Integration testing
You have to have a MariaDB local instance running on `[::1]:3306` with user `dbfs` password `dbfs` and with setup from `sql/testing.sql` in database `dbfs`.
