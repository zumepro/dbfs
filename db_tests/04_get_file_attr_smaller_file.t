#!/usr/bin/perl


use strict;
use warnings;
use Test::More tests => 23;
use DBI;


sub get_rows {
    my ($sth) = @_;
    $sth->execute();
    my @res = ();
    while (my $ref = $sth->fetchrow_hashref()) {
        push(@res, $ref);
    }
    return @res;
}


my $dbh = DBI->connect("DBI:mysql:database=dbfs;host=127.0.0.1:3306", "dbfs", "dbfs");
isnt($dbh, 0);


my $inode_id = "2";


my @size = get_rows($dbh->prepare("WITH `file_tmp` (`blocks`) AS (
    SELECT COUNT(*) FROM `block` WHERE `inode_id` = '$inode_id'
) SELECT
    `blocks` * 4096 - (SELECT 4096 - OCTET_LENGTH(`data`) FROM `block` WHERE `inode_id` = '$inode_id' ORDER BY `block_id` DESC LIMIT 1) AS bytes,
    `blocks` AS blocks
FROM `file_tmp`"));
is(scalar @size, 1);

is($size[0]->{"bytes"}, 14);
is($size[0]->{"blocks"}, 1);


my @hardlinks = get_rows($dbh->prepare("SELECT COUNT(*) AS `hardlinks` FROM `file` WHERE `inode_id` = '$inode_id'"));
is(scalar @hardlinks, 1);

is($hardlinks[0]->{"hardlinks"}, 1);


my @inode = get_rows($dbh->prepare("SELECT
    `id` AS `ino`,
    `owner` AS `uid`,
    `group` AS `gid`,
    (SELECT `description` FROM `file_types` WHERE `id` = (SELECT `file_type` FROM `inode` WHERE `id` = '$inode_id')) AS `file_type`,
    (SELECT `can_read` FROM `permissions` WHERE `id` = (SELECT `user_perm` FROM `inode` WHERE `id` = '$inode_id')) AS `owner_can_read`,
    (SELECT `can_write` FROM `permissions` WHERE `id` = (SELECT `user_perm` FROM `inode` WHERE `id` = '$inode_id')) AS `owner_can_write`,
    (SELECT `can_execute` FROM `permissions` WHERE `id` = (SELECT `user_perm` FROM `inode` WHERE `id` = '$inode_id')) AS `owner_can_execute`,
    (SELECT `description` FROM `file_types` WHERE `id` = (SELECT `file_type` FROM `inode` WHERE `id` = '$inode_id')) AS `file_type`,
    (SELECT `can_read` FROM `permissions` WHERE `id` = (SELECT `group_perm` FROM `inode` WHERE `id` = '$inode_id')) AS `group_can_read`,
    (SELECT `can_write` FROM `permissions` WHERE `id` = (SELECT `group_perm` FROM `inode` WHERE `id` = '$inode_id')) AS `group_can_write`,
    (SELECT `can_execute` FROM `permissions` WHERE `id` = (SELECT `group_perm` FROM `inode` WHERE `id` = '$inode_id')) AS `group_can_execute`,
    (SELECT `can_read` FROM `permissions` WHERE `id` = (SELECT `other_perm` FROM `inode` WHERE `id` = '$inode_id')) AS `other_can_read`,
    (SELECT `can_write` FROM `permissions` WHERE `id` = (SELECT `other_perm` FROM `inode` WHERE `id` = '$inode_id')) AS `other_can_write`,
    (SELECT `can_execute` FROM `permissions` WHERE `id` = (SELECT `other_perm` FROM `inode` WHERE `id` = '$inode_id')) AS `other_can_execute`,
    `created_at` AS `ctime`,
    `modified_at` AS `mtime`,
    `accessed_at` AS `atime`
FROM `inode` WHERE `id` = '$inode_id'"));
is(scalar @inode, 1);

is($inode[0]->{"ino"}, 2);
is($inode[0]->{"uid"}, 2);
is($inode[0]->{"gid"}, 2);
is($inode[0]->{"file_type"}, "Regular file");
is($inode[0]->{"owner_can_read"}, 1);
is($inode[0]->{"owner_can_write"}, 1);
is($inode[0]->{"owner_can_execute"}, 0);
is($inode[0]->{"group_can_read"}, 1);
is($inode[0]->{"group_can_write"}, 0);
is($inode[0]->{"group_can_execute"}, 0);
is($inode[0]->{"other_can_read"}, 1);
is($inode[0]->{"other_can_write"}, 0);
is($inode[0]->{"other_can_execute"}, 0);
is($inode[0]->{"ctime"}, "2024-10-24 17:54:00");
is($inode[0]->{"mtime"}, "2024-10-24 17:54:00");
is($inode[0]->{"atime"}, "2024-10-24 17:54:00");


done_testing();
