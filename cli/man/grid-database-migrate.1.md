% GRID-DATABASE-MIGRATE(1) Cargill, Incorporated | Grid
<!--
  Copyright 2021 Cargill Incorporated
  Licensed under Creative Commons Attribution 4.0 International License
  https://creativecommons.org/licenses/by/4.0/
-->

NAME
====

**grid-database-migrate** - Performs database migrations.

SYNOPSIS
========

**grid database migrate** \[**FLAGS**\] \[**OPTIONS**\]

DESCRIPTION
===========

This command performs any outstanding database migrations to the 
Grid daemon database.

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

OPTIONS
=======

`-C`, `--connect`
: Specifies the URL for the database.

SEE ALSO
========
| `grid database(1)`
|
| Grid documentation: https://grid.hyperledger.org/docs/0.2/
