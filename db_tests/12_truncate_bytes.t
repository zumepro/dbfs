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
    my $inode = 3;
    my $truncate_to_blocks = 3;
    my $test_pretruncate_last_block_to_bytes = 0;
    my $truncate_last_block_to_bytes = 4096;

    my @filesize = get_rows($dbh->prepare("WITH `ino` AS (SELECT $inode AS `ino`) SELECT COUNT(*) AS `bc`, (SELECT `block_id` FROM `block` WHERE `inode_id` = (SELECT `ino` FROM `ino`) ORDER BY `block_id` DESC LIMIT 1) AS `last_block_id` FROM `block` WHERE `inode_id` = (SELECT `ino` FROM `ino`)"));
    is(scalar @filesize, 1);
    is($filesize[0]->{"bc"}, 4);
    is($filesize[0]->{"last_block_id"}, 4);
    my $bc = $filesize[0]->{"bc"};

    my $strip_count = $bc - $truncate_to_blocks;

    $dbh->do("DELETE FROM `block` WHERE `inode_id` = $inode ORDER BY `block_id` DESC LIMIT $strip_count");
    $dbh->do("UPDATE `block` SET `data` = RPAD(SUBSTR(`data`, 1, $test_pretruncate_last_block_to_bytes), $test_pretruncate_last_block_to_bytes, CHAR(0)) WHERE `inode_id` = $inode ORDER BY `block_id` DESC LIMIT 1");
    my @test_01 = get_rows($dbh->prepare("SELECT `data` FROM `block` WHERE `inode_id` = $inode"));
    is(scalar @test_01, 3);
    is($test_01[0]->{"data"}, "\0" x 4096);
    is($test_01[1]->{"data"}, "\0" x 4096);
    is($test_01[2]->{"data"}, "\0" x $test_pretruncate_last_block_to_bytes);
    is(length($test_01[2]->{"data"}), $test_pretruncate_last_block_to_bytes);

    $dbh->do("UPDATE `block` SET `data` = RPAD(SUBSTR(`data`, 1, $truncate_last_block_to_bytes), $truncate_last_block_to_bytes, '\0') WHERE `inode_id` = $inode ORDER BY `block_id` DESC LIMIT 1");

    my @test_02 = get_rows($dbh->prepare("SELECT `data` FROM `block` WHERE `inode_id` = $inode"));
    is(scalar @test_01, 3);
    is($test_02[0]->{"data"}, "\0" x 4096);
    is($test_02[1]->{"data"}, "\0" x 4096);
    is($test_02[2]->{"data"}, "\0" x $truncate_last_block_to_bytes);
    is(length($test_02[2]->{"data"}), $truncate_last_block_to_bytes);


    $dbh->do("DELETE FROM `block` WHERE `inode_id` = $inode");
    $dbh->do("INSERT INTO `block` (`inode_id`, `block_id`, `data`) VALUES ($inode, 1, REPEAT(CHAR(0), 4096)), ($inode, 2, REPEAT(CHAR(0), 4096)), ($inode, 3, REPEAT(CHAR(0), 4096)), ($inode, 4, 'aaaa\n')");
}


{
    my $inode = 3;
    my $truncate_to_blocks = 4;
    my $truncate_last_block_to_bytes = 4096;

    my @filesize = get_rows($dbh->prepare("SELECT COUNT(*) AS `bc` FROM `block` WHERE `inode_id` = $inode"));
    is(scalar @filesize, 1);
    is($filesize[0]->{"bc"}, 4);
    my $bc = $filesize[0]->{"bc"};

    my $strip_count = $bc - $truncate_to_blocks;

    $dbh->do("DELETE FROM `block` WHERE `inode_id` = $inode ORDER BY `block_id` DESC LIMIT $strip_count");
    $dbh->do("UPDATE `block` SET `data` = RPAD(SUBSTR(`data`, 1, $truncate_last_block_to_bytes), $truncate_last_block_to_bytes, '\0') WHERE `inode_id` = $inode ORDER BY `block_id` DESC LIMIT 1");

    my @test_01 = get_rows($dbh->prepare("SELECT `data` FROM `block` WHERE `inode_id` = $inode"));
    is(scalar @test_01, 4);
    is($test_01[0]->{"data"}, "\0" x 4096);
    is($test_01[1]->{"data"}, "\0" x 4096);
    is($test_01[2]->{"data"}, "\0" x 4096);
    is($test_01[3]->{"data"}, "aaaa\n" . ("\0" x (4096 - 5)));


    $dbh->do("DELETE FROM `block` WHERE `inode_id` = $inode");
    $dbh->do("INSERT INTO `block` (`inode_id`, `block_id`, `data`) VALUES ($inode, 1, REPEAT(CHAR(0), 4096)), ($inode, 2, REPEAT(CHAR(0), 4096)), ($inode, 3, REPEAT(CHAR(0), 4096)), ($inode, 4, 'aaaa\n')");
}


done_testing();
