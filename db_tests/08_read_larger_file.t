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


{
    my $inode = "3";
    my $offset_in_blocks = "0";
    my $max_size_in_blocks = "4";


    my @data = get_rows($dbh->prepare("SELECT `data` FROM `block` WHERE `inode_id` = '$inode' ORDER BY `block_id` ASC LIMIT $max_size_in_blocks OFFSET $offset_in_blocks"));
    is(scalar @data, 4);

    is($data[0]->{"data"}, "\0" x 4096);
    is($data[1]->{"data"}, "\0" x 4096);
    is($data[2]->{"data"}, "\0" x 4096);
    is($data[3]->{"data"}, "aaaa\n");
}


{
    my $inode = "3";
    my $offset_in_blocks = "1";
    my $max_size_in_blocks = "4";


    my @data = get_rows($dbh->prepare("SELECT `data` FROM `block` WHERE `inode_id` = '$inode' ORDER BY `block_id` ASC LIMIT $max_size_in_blocks OFFSET $offset_in_blocks"));
    is(scalar @data, 3);

    is($data[0]->{"data"}, "\0" x 4096);
    is($data[1]->{"data"}, "\0" x 4096);
    is($data[2]->{"data"}, "aaaa\n");
}


{
    my $inode = "3";
    my $offset_in_blocks = "0";
    my $max_size_in_blocks = "2";


    my @data = get_rows($dbh->prepare("SELECT `data` FROM `block` WHERE `inode_id` = '$inode' ORDER BY `block_id` ASC LIMIT $max_size_in_blocks OFFSET $offset_in_blocks"));
    is(scalar @data, 2);

    is($data[0]->{"data"}, "\0" x 4096);
    is($data[1]->{"data"}, "\0" x 4096);
}


{
    my $inode = "3";
    my $offset_in_blocks = "3";
    my $max_size_in_blocks = "4";


    my @data = get_rows($dbh->prepare("SELECT `data` FROM `block` WHERE `inode_id` = '$inode' ORDER BY `block_id` ASC LIMIT $max_size_in_blocks OFFSET $offset_in_blocks"));
    is(scalar @data, 1);

    is($data[0]->{"data"}, "aaaa\n");
}


{
    my $inode = "3";
    my $offset_in_blocks = "4";
    my $max_size_in_blocks = "4";


    my @data = get_rows($dbh->prepare("SELECT `data` FROM `block` WHERE `inode_id` = '$inode' ORDER BY `block_id` ASC LIMIT $max_size_in_blocks OFFSET $offset_in_blocks"));
    is(scalar @data, 0);
}


{
    my $inode = "3";
    my $offset_in_blocks = "0";
    my $max_size_in_blocks = "0";


    my @data = get_rows($dbh->prepare("SELECT `data` FROM `block` WHERE `inode_id` = '$inode' ORDER BY `block_id` ASC LIMIT $max_size_in_blocks OFFSET $offset_in_blocks"));
    is(scalar @data, 0);
}


done_testing();
