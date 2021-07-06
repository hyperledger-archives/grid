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

**grid organization** \[**FLAGS**\] \[**OPTIONS**\] SUBCOMMAND

DESCRIPTION
===========

This command allows for the creation and management of Grid Pike organizations.

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

ENVIRONMENT VARIABLES
=====================

Many subcommands accept the following environment variables:

**`CYLINDER_PATH`**
: Colon-separated path used to search for the key which will be used
  to sign transactions.

**`GRID_DAEMON_ENDPOINT`**
: Specifies a default value for `--url`.

**`GRID_DAEMON_KEY`**
: Specifies a default value for  `-k`, `--key`.

**`GRID_SERVICE_ID`**
: Specifies a default value for `--service-id`.

SUBCOMMANDS
===========

`create`
: Create an organization.

`help`
: Prints this message or the help of the given subcommand(s).

`update`
: Update an organization.

SEE ALSO
========
| `grid organization create(1)`
| `grid organization update(1)`
| `grid organization list(1)`
| `grid organization show(1)`
| `grid agent(1)`
| `grid role(1)`
|
| Grid documentation: https://grid.hyperledger.org/docs/0.2/
