% GRID-ADMIN(1) Cargill, Incorporated | Grid
<!--
  Copyright 2021 Cargill Incorporated
  Licensed under Creative Commons Attribution 4.0 International License
  https://creativecommons.org/licenses/by/4.0/
-->

NAME
====

**grid-admin** - Supports Grid Administrative functions.

SYNOPSIS
========

**grid admin** \[**FLAGS**\] \[**OPTIONS**\] SUBCOMMAND

DESCRIPTION
===========

Administrative commands for grid daemon.

FLAGS
=====

`-h`, `--help`
: Prints help information.

`-q`, `--quiet`
: Do not display output.

`-V`, `--version`
: Prints version information.

`-v`
: Log verbosely.

SUBCOMMANDS
===========

`keygen`
: Generates keys for gridd to use to sign transactions and batches.

`help`
: Prints this message or the help of the given subcommand(s).


SEE ALSO
========
| `grid admin keygen(1)`
|
| Grid documentation: https://grid.hyperledger.org/docs/0.2/
