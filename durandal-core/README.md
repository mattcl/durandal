# durandal-core #

This crate is a collection of shared functionality for writing command line
applications intended to be external subcommands of the `durandal` cli. The top
level `durandal` cli also utilizes some of the functionality provided by this
crate.


While it is not necessary to import this crate in something implementing an
external subcommand, this does serve to cut down on some of the repetition.
