#!/usr/bin/perl


use strict;
use warnings;
use Test::More tests => 7;
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


my @rows_users = get_rows($dbh->prepare("SELECT * FROM `user`"));
is(scalar @rows_users, 1);

is ($rows_users[0]->{'id'}, 1);
is ($rows_users[0]->{'name'}, "root");


my @rows_groups = get_rows($dbh->prepare("SELECT * FROM `group`"));
is (scalar @rows_users, 1);

is ($rows_groups[0]->{'id'}, 1);
is ($rows_groups[0]->{'name'}, "root");


done_testing();
