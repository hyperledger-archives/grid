% GRID-ROLE-SHOW(1) Cargill, Incorporated | Grid
<!--
  Copyright 2021 Cargill Incorporated
  Licensed under Creative Commons Attribution 4.0 International License
  https://creativecommons.org/licenses/by/4.0/
-->

NAME
====

**grid-role-show** - Show information about a Grid Pike role.

SYNOPSIS
========

**grid role show** \[**FLAGS**\] \[**OPTIONS**\] ORG_ID NAME

ARGS
====

`ORG_ID`
: The organization identifier to show the role for

`NAME`
: The user-specified name of the role to show

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

SEE ALSO
========
| `grid agent(1)`
| `grid organization(1)`
| `grid role(1)`
| `grid role list(1)`
|
| Grid documentation: https://grid.hyperledger.org/docs/0.2/
