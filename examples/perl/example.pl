use strict;
use warnings;
use feature 'say';
use Data::Dumper qw(Dumper);

use FalkorDB;
my $db = FalkorDB->new(
    host => 'falkordb',
    port => 6379,
);

my %DISP = (
    add => \&add_person,
    del => \&del_everything,
    list => \&list,
    people => \&add_people,
    all => \&list_all,
);

my $cmd = shift or usage();

main();
exit(0);


sub main {
    my $graph = $db->select_graph('Example');

    if ($DISP{$cmd}) {
        $DISP{$cmd}->($graph);
    } else {
        usage();
    }

}

sub add_people {
    my ($graph) = @_;

    my @people = ("Joe", "Jane", "Mary", "q'ote", 'Other"quote');
    for my $person_name (@people) {
        $graph->query("CREATE (:Person {name: \$name})", { name => $person_name });
    }
}

sub add_person {
    my ($graph) = @_;

    my $res = $graph->query("CREATE (p:Person {name: 'Alice', age: 30}) RETURN p");
    while (my $row = $res->next_row()) {
        for my $node (@$row) {
            my ($name, $age) = ($node->{properties}{name}, $node->{properties}{age});
            print "Added Person: $name ($age)\n";
        }
    }
}

sub del_everything {
    my ($graph) = @_;

    $graph->query("MATCH (n) DETACH DELETE n");
}

sub list {
    my ($graph) = @_;

    # Execute a parameterized read query
    my $res = $graph->query(
        "MATCH (p:Person) WHERE p.age = \$age RETURN p.name, p.age",
        { age => 30 }
    );
    # Iterate over results
    while (my $row = $res->next_row()) {
        my ($name, $age) = @$row;
        print "Found Person: $name ($age)\n";
    }
}

sub list_all {
    my ($graph) = @_;

    my $res = $graph->query(
        "MATCH (p:Person) RETURN p",
    );
    while (my $row = $res->next_row()) {
        for my $node (@$row) {
            print "Found Person: $node->{properties}{name}\n";
        }
    }
}


sub usage {
    my $params = join "|", sort keys %DISP;
    die "Usage: $0 [$params]\n";
}
