#!/usr/bin/perl


use strict;
use warnings;
use Test::More tests => 8;
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


my @listing = get_rows($dbh->prepare("WITH `file_info` AS (SELECT `name`, `inode_id` FROM `file` WHERE `parent_id` = (SELECT `id` FROM `file` WHERE `inode_id` = '$parent_inode_id' LIMIT 1)) SELECT
    `name` AS `name`,
    `inode_id`,
    (SELECT `file_type` FROM `inode` WHERE `id` = `file_info`.`inode_id`) AS `file_type`
FROM `file_info`"));

# WARNING:  The SELECT for `parent_id` can return multiple results if there are multiple hardlinks to a directory `inode`
#           Here I temporarily artificially limited it (such that the query doesn't fail in such case).

is(scalar @listing, 2);

is($listing[0]->{"name"}, "test.txt");
is($listing[0]->{"inode_id"}, 2);
is($listing[0]->{"file_type"}, "-");

is($listing[1]->{"name"}, "test.bin");
is($listing[1]->{"inode_id"}, 3);
is($listing[1]->{"file_type"}, "-");


done_testing();

