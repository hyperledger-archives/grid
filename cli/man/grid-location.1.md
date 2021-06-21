% GRID-LOCATION(1) Cargill, Incorporated | Grid
<!--
  Copyright 2021 Cargill Incorporated
  Licensed under Creative Commons Attribution 4.0 International License
  https://creativecommons.org/licenses/by/4.0/
-->

NAME
====

**grid-location** - Create, Delete, Update, List or Show Grid Locations.

SYNOPSIS
========

**grid location** \[**FLAGS**\] \[**OPTIONS**\] SUBCOMMAND

DESCRIPTION
===========

This command allows for the creation and management of Grid Locations.

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

Many subcommands use the following environment variables:

**`CYLINDER_PATH`**
: Colon-separated path used to search for the key which will be used
  to sign transactions

**`GRID_DAEMON_ENDPOINT`**
: Specifies a default value for `--url`

**`GRID_DAEMON_KEY`**
: Specifies a default value for  `-k`, `--key`

**`GRID_SERVICE_ID`**
: Specifies a default value for `--service-id`

SUBCOMMANDS
===========

`create`
: Create a location

`delete`
: Deletes a location

`help`
: Prints this message or the help of the given subcommand(s)

`list`
: Displays list of locations

`show`
: Displays details of a location

`update`
: Update a location

SEE ALSO
========
| `grid location create(1)`
| `grid location delete(1)`
| `grid location list(1)`
| `grid location show(1)`
| `grid location update(1)`
|
| Grid documentation: https://grid.hyperledger.org/docs/0.2/
