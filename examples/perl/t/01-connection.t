use strict;
use warnings;
use Test::More;
use FalkorDB;

if (!defined $ENV{FALKORDB} || $ENV{FALKORDB} eq '') {
    plan skip_all => 'FALKORDB environment variable is not set';
}
plan tests => 9;

# Initialize FalkorDB connection
my ($host, $port) = ($ENV{FALKORDB}, 6379);
if ($ENV{FALKORDB} =~ /^(.*):(\d+)$/) {
    $host = $1;
    $port = $2;
}
my $db = FalkorDB->new(
    host => $host,
    port => $port,
);

ok(defined($db), 'FalkorDB instance created');
isa_ok($db, 'FalkorDB');

ok(defined($db->redis), 'Underlying Redis instance initialized');

# Test select_graph / graph methods
my $graph = $db->select_graph('test_conn_graph');
ok(defined($graph), 'Graph selected via select_graph');
isa_ok($graph, 'FalkorDB::Graph');
is($graph->name, 'test_conn_graph', 'Graph name matches');

my $graph2 = $db->graph('test_conn_graph');
ok(defined($graph2), 'Graph selected via graph alias');
isa_ok($graph2, 'FalkorDB::Graph');

# Test list_graphs
my $graphs = $db->list_graphs();
ok(ref $graphs eq 'ARRAY', 'list_graphs returns an array ref');
