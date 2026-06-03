use strict;
use warnings;
use Test::More tests => 6;

BEGIN {
    use_ok('FalkorDB')              || print "Bail out!\n";
    use_ok('FalkorDB::Graph')       || print "Bail out!\n";
    use_ok('FalkorDB::QueryResult') || print "Bail out!\n";
    use_ok('FalkorDB::Node')        || print "Bail out!\n";
    use_ok('FalkorDB::Edge')        || print "Bail out!\n";
    use_ok('FalkorDB::Path')        || print "Bail out!\n";
}

diag("Testing FalkorDB $FalkorDB::VERSION, Perl $], $^X");
