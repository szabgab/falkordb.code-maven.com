use strict;
use warnings;
use ExtUtils::MakeMaker;

WriteMakefile(
    NAME               => 'FalkorDB',
    AUTHOR             => 'Antigravity',
    VERSION_FROM       => 'lib/FalkorDB.pm',
    ABSTRACT_FROM      => 'lib/FalkorDB.pm',
    LICENSE            => 'perl_5',
    MIN_PERL_VERSION   => '5.014',
    CONFIGURE_REQUIRES => {
        'ExtUtils::MakeMaker' => '0',
    },
    BUILD_REQUIRES => {
        'Test::More' => '0',
    },
    PREREQ_PM => {
        'Redis::Fast'  => '0',
        'Scalar::Util' => '0',
        'JSON::PP'     => '0',
    },
    dist  => { COMPRESS => 'gzip -9f', SUFFIX => 'gz', },
    clean => { FILES    => 'FalkorDB-*' },
);
