% GRID-DATABASE(1) Cargill, Incorporated | Grid
<!--
  Copyright 2021 Cargill Incorporated
  Licensed under Creative Commons Attribution 4.0 International License
  https://creativecommons.org/licenses/by/4.0/
-->

NAME
====

**grid-database** - Commands to manage the Grid daemon Database.

SYNOPSIS
========

**grid database** \[**FLAGS**\] \[**OPTIONS**\] SUBCOMMAND

DESCRIPTION
===========

This command allows for the management of Grid daemon Database.

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

`migrate`
: Migrates the Grid database to the latest version.

`help`
: Prints this message or the help of the given subcommand(s).

SEE ALSO
========
| `grid database migrate(1)`
|
| Grid documentation: https://grid.hyperledger.org/docs/0.2/
