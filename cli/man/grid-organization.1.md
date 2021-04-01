% GRID-ORGANIZATION(1) Cargill, Incorporated | Grid
<!--
  Copyright 2021 Cargill Incorporated
  Licensed under Creative Commons Attribution 4.0 International License
  https://creativecommons.org/licenses/by/4.0/
-->

NAME
====

**grid-organization** - Create or update Grid Pike organizations.

SYNOPSIS
========

**grid organization** \[**FLAGS**\] \[**OPTIONS**\] <**SUBCOMMAND**>

DESCRIPTION
===========

This command allows for the creation and management of Grid Pike organizations.

FLAGS
=====

`-h`, `--help`
: Prints help information

`-q`, `--quiet`
: Do not display output

`-V`, `--version`
: Prints version information

`-v`
: Log verbosely

ENVIRONMENT VARIABLES
=====================

**`GRID_DAEMON_ENDPOINT`**
: Specifies the endpoint for the grid daemon (`gridd`)
  if `-U` or `--url` is not used.

**`GRID_DAEMON_KEY`**
: Specifies key used to sign transactions if `k` or `--key`
  is not used.

**`GRID_SERVICE_ID`**
: Specifies service ID if `--service-id` is not used

SUBCOMMANDS
===========

`create`
: Create an organization

`help`
: Prints this message or the help of the given subcommand(s)

`update`
: Update an organization

SEE ALSO
========
| `grid organization create(1)`
| `grid organization update(1)`
| `grid agent(1)`
| `grid role(1)`
|
| Grid documentation: https://grid.hyperledger.org/docs/0.2/
