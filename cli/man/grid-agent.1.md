% GRID-AGENT(1) Cargill, Incorporated | Grid
<!--
  Copyright 2021 Cargill Incorporated
  Licensed under Creative Commons Attribution 4.0 International License
  https://creativecommons.org/licenses/by/4.0/
-->

NAME
====

**grid-agent** - Create or update Grid Pike agents.

SYNOPSIS
========

**grid agent** \[**FLAGS**\] \[**OPTIONS**\] SUBCOMMAND

DESCRIPTION
===========

This command allows for the creation and management of Grid Pike agents.

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

Many subcommands accept the following environment variables:

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
: Create an agent

`help`
: Prints this message or the help of the given subcommand(s)

`update`
: Update an agent

SEE ALSO
========
| `grid agent create(1)`
| `grid agent update(1)`
| `grid organization(1)`
| `grid role(1)`
|
| Grid documentation: https://grid.hyperledger.org/docs/0.2/
