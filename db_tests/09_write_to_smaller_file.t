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
    my $inode = "2";
    my $offset_in_blocks = "0";
    my $rows = "1";


    $dbh->do("UPDATE `block` SET `data` = ('Hello, world!') WHERE `inode_id` = '$inode' ORDER BY `block_id` ASC LIMIT $rows");

    my @test_01 = get_rows($dbh->prepare("SELECT `data` FROM `block` WHERE `inode_id` = '$inode'"));
    is(scalar @test_01, 1);
    is($test_01[0]->{"data"}, "Hello, world!");

    $dbh->do("UPDATE `block` SET `data` = 'Hello, world!\n' WHERE `inode_id` = '$inode' ORDER BY `block_id` ASC");

    my @test_02 = get_rows($dbh->prepare("SELECT `data` FROM `block` WHERE `inode_id` = '$inode'"));
    is(scalar @test_01, 1);
    is($test_02[0]->{"data"}, "Hello, world!\n");
}


done_testing();
