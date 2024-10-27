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
    my $inode = "3";
    my $offset_in_blocks = "0";
    my $rows = "1";


    $dbh->do("UPDATE `block` SET `data` = (REPEAT(CHAR(1), 4096)) WHERE `inode_id` = '$inode' ORDER BY `block_id` ASC LIMIT $rows");

    my @test_01 = get_rows($dbh->prepare("SELECT `data` FROM `block` WHERE `inode_id` = '$inode'"));
    is(scalar @test_01, 4);
    is($test_01[0]->{"data"}, "\1" x 4096);

    $dbh->do("UPDATE `block` SET `data` = (REPEAT(CHAR(0), 4096)) WHERE `inode_id` = '$inode' ORDER BY `block_id` ASC LIMIT $rows");

    my @test_02 = get_rows($dbh->prepare("SELECT `data` FROM `block` WHERE `inode_id` = '$inode'"));
    is(scalar @test_01, 4);
    is($test_02[0]->{"data"}, "\0" x 4096);
}


{
    my $inode = "3";


    $dbh->do("INSERT INTO `block` (`inode_id`, `block_id`, `data`) VALUES
        ($inode, 1, REPEAT(CHAR(1), 4096)),
        ($inode, 2, REPEAT(CHAR(2), 4096))
    ON DUPLICATE KEY UPDATE `inode_id`=VALUES(`inode_id`), `block_id`=VALUES(`block_id`), `data`=VALUES(`data`)");

    my @test_01 = get_rows($dbh->prepare("SELECT `data` FROM `block` WHERE `inode_id` = '$inode'"));
    is(scalar @test_01, 4);
    is($test_01[0]->{"data"}, "\1" x 4096);
    is($test_01[1]->{"data"}, "\2" x 4096);

    $dbh->do("INSERT INTO `block` (`inode_id`, `block_id`, `data`) VALUES
        ($inode, 1, REPEAT(CHAR(0), 4096)),
        ($inode, 2, REPEAT(CHAR(0), 4096))
    ON DUPLICATE KEY UPDATE `inode_id`=VALUES(`inode_id`), `block_id`=VALUES(`block_id`), `data`=VALUES(`data`)");

    my @test_02 = get_rows($dbh->prepare("SELECT `data` FROM `block` WHERE `inode_id` = '$inode'"));
    is(scalar @test_01, 4);
    is($test_02[0]->{"data"}, "\0" x 4096);
    is($test_02[1]->{"data"}, "\0" x 4096);
}


{
    my $inode = "3";


    $dbh->do("INSERT INTO `block` (`inode_id`, `block_id`, `data`) VALUES
        ($inode, 2, REPEAT(CHAR(2), 4096)),
        ($inode, 3, REPEAT(CHAR(3), 4096))
    ON DUPLICATE KEY UPDATE `inode_id`=VALUES(`inode_id`), `block_id`=VALUES(`block_id`), `data`=VALUES(`data`)");

    my @test_01 = get_rows($dbh->prepare("SELECT `data` FROM `block` WHERE `inode_id` = '$inode'"));
    is(scalar @test_01, 4);
    is($test_01[0]->{"data"}, "\0" x 4096);
    is($test_01[1]->{"data"}, "\2" x 4096);
    is($test_01[2]->{"data"}, "\3" x 4096);
    is($test_01[3]->{"data"}, "aaaa\n");

    $dbh->do("INSERT INTO `block` (`inode_id`, `block_id`, `data`) VALUES
        ($inode, 2, REPEAT(CHAR(0), 4096)),
        ($inode, 3, REPEAT(CHAR(0), 4096))
    ON DUPLICATE KEY UPDATE `inode_id`=VALUES(`inode_id`), `block_id`=VALUES(`block_id`), `data`=VALUES(`data`)");

    my @test_02 = get_rows($dbh->prepare("SELECT `data` FROM `block` WHERE `inode_id` = '$inode'"));
    is(scalar @test_01, 4);
    is($test_02[0]->{"data"}, "\0" x 4096);
    is($test_02[1]->{"data"}, "\0" x 4096);
}


done_testing();
