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


my $dbh = DBI->connect("DBI:mysql:database=dbfs;host=127.0.0.1:3306", "dbfs", "dbfs");
isnt($dbh, 0);


{
    my $inode = 1;

    my @filesize = get_rows($dbh->prepare("WITH `ino` AS (SELECT $inode AS `ino`) SELECT COUNT(*) AS `bc`, IFNULL((SELECT `block_id` FROM `block` WHERE `inode_id` = (SELECT `ino` FROM `ino`) ORDER BY `block_id` DESC LIMIT 1), 0) AS `last_block_id` FROM `block` WHERE `inode_id` = (SELECT `ino` FROM `ino`)"));
    is(scalar @filesize, 1);
    is($filesize[0]->{"bc"}, 0);
    is($filesize[0]->{"last_block_id"}, 0);
    my $bc = $filesize[0]->{"bc"};
}


done_testing();
