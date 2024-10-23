#!/usr/bin/perl


use strict;
use warnings;
use Test::More tests => 1;
use DBI;


my $dbh = DBI->connect("DBI:mysql:database=dbfs;host=[::1]:3306", "dbfs", "dbfs");
isnt($dbh, 0);


done_testing();
