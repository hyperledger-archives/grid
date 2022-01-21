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

**grid role show** \[**FLAGS**\] \[**OPTIONS**\] <ORG_ID> <NAME>

ARGS
====

`ORG_ID`
: The organization identifier to show the role for.

`NAME`
: The user-specified name of the role to show.

FLAGS
=====

`-h`, `--help`
: Prints help information.

`-q`, `--quiet`
: Do not display output.

`-V`, `--version`
: Prints version information.

`-v`
: Log verbosely

ENVIRONMENT VARIABLES
=====================

**`GRID_DAEMON_ENDPOINT`**
: Specifies a default value for `--url`.

**`GRID_SERVICE_ID`**
: Specifies a default value for `--service-id`.

SEE ALSO
========
| `grid agent(1)`
| `grid organization(1)`
| `grid role(1)`
| `grid role list(1)`
|
| Grid documentation: https://grid.hyperledger.org/docs/0.3/
