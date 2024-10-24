#!/usr/bin/perl


use strict;
use warnings;
use Test::More tests => 13;
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


my $dbh = DBI->connect("DBI:mysql:database=dbfs;host=[::1]:3306", "dbfs", "dbfs");
isnt($dbh, 0);


my $inode_id = "2";


my @size = get_rows($dbh->prepare("WITH `inode_tmp` (`blocks`) AS (
        SELECT COUNT(*) FROM `block` WHERE `inode_id` = '$inode_id'
) SELECT
    `blocks` * 4096 - (SELECT 4096 - OCTET_LENGTH(`data`) FROM `block` WHERE `inode_id` = '$inode_id' ORDER BY `block_id` DESC LIMIT 1) AS bytes,
    `blocks` AS blocks
FROM `inode_tmp`"));
is(scalar @size, 1);

is($size[0]->{"bytes"}, 14);
is($size[0]->{"blocks"}, 1);


my @inode = get_rows($dbh->prepare("SELECT
    `id` AS `ino`,
    `owner` AS `uid`,
    `group` AS `gid`,
    `created_at` AS `ctime`,
    `modified_at` AS `mtime`,
    `accessed_at` AS `atime`
FROM `inodes` WHERE `id` = '$inode_id'"));
is(scalar @inode, 1);


# TODO: Add permissions


is($inode[0]->{"ino"}, 2);
is($inode[0]->{"uid"}, 1);
is($inode[0]->{"gid"}, 1);
is($inode[0]->{"ctime"}, "2024-10-23 12:41:11");
is($inode[0]->{"mtime"}, "2024-10-23 12:41:11");
is($inode[0]->{"atime"}, "2024-10-23 12:41:11");


my @hardlinks = get_rows($dbh->prepare("SELECT COUNT(*) AS `hardlinks` FROM `file` WHERE `inode_id` = '$inode_id'"));
is(scalar @hardlinks, 1);
is($hardlinks[0]->{"hardlinks"}, 1);


done_testing();

