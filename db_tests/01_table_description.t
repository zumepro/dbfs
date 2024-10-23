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


my @rows_user = get_rows($dbh->prepare("DESCRIBE `user`"));
is(scalar @rows_user, 2);

my %expected_user_01 = ( Field => "id", Type => "int(10) unsigned", Null => "NO", Key => "PRI", Default => "NULL", Extra => "auto_increment" );
my %expected_user_02 = ( Field => "name", Type => "varchar(255)", Null => "YES", Key => "", Default => "NULL", Extra => "" );
is (%{$rows_user[0]}, %expected_user_01);
is (%{$rows_user[1]}, %expected_user_02);


my @rows_group = get_rows($dbh->prepare("DESCRIBE `group`"));
is(scalar @rows_group, 2);

my %expected_group_01 = ( Field => "id", Type => "int(10) unsigned", Null => "NO", Key => "PRI", Default => "NULL", Extra => "auto_increment" );
my %expected_group_02 = ( Field => "name", Type => "varchar(255)", Null => "YES", Key => "", Default => "NULL", Extra => "" );
is (%{$rows_group[0]}, %expected_group_01);
is (%{$rows_group[1]}, %expected_group_02);


done_testing();
