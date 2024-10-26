#!/usr/bin/perl


use strict;
use warnings;
use Test::More tests => 3;
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


my $inode = "2";
my $offset_in_blocks = "0";
my $max_size_in_blocks = "1";


my @data = get_rows($dbh->prepare("SELECT `data` FROM `block` WHERE `inode_id` = '$inode' ORDER BY `block_id` ASC LIMIT $max_size_in_blocks OFFSET $offset_in_blocks"));
is(scalar @data, 1);

is($data[0]->{"data"}, "Hello, world!\n");


done_testing();

