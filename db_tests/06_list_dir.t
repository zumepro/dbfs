#!/usr/bin/perl


use strict;
use warnings;
use Test::More;
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


my $parent_inode_id = "1";


my @listing = get_rows($dbh->prepare("WITH `ino` AS (SELECT '$parent_inode_id' AS `ino`), `file_info` AS (SELECT `name`, `inode_id` FROM `file` WHERE `parent_inode_id` = (SELECT `ino` FROM `ino`) AND `inode_id` != (SELECT `ino` FROM `ino`)) SELECT
    `name` AS `name`,
    `inode_id`,
    (SELECT `file_type` FROM `inode` WHERE `id` = `file_info`.`inode_id`) AS `file_type`
FROM `file_info` ORDER BY `inode_id`"));

is(scalar @listing, 3);

is($listing[0]->{"name"}, "test.txt");
is($listing[0]->{"inode_id"}, 2);
is($listing[0]->{"file_type"}, "-");

is($listing[1]->{"name"}, "test.bin");
is($listing[1]->{"inode_id"}, 3);
is($listing[1]->{"file_type"}, "-");

is($listing[2]->{"name"}, "more_testing");
is($listing[2]->{"inode_id"}, 4);
is($listing[2]->{"file_type"}, "d");


done_testing();
