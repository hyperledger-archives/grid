% GRID-ROLE-DELETE(1) Cargill, Incorporated | Grid
<!--
  Copyright 2021 Cargill Incorporated
  Licensed under Creative Commons Attribution 4.0 International License
  https://creativecommons.org/licenses/by/4.0/
-->

NAME
====

**grid-role-delete** - Removes a Grid Pike role.

SYNOPSIS
========

**grid role delete** \[**FLAGS**\] \[**OPTIONS**\] <ORG_ID> <NAME>

ARGS
====

`ORG_ID`
: The organization identifier to delete the role from.

`NAME`
: The user-specified name of the role.

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

`-k`, `--key`
: Base name or path to a private signing key file.

`--wait`
: Maximum number of seconds to wait for the batch to be committed.

ENVIRONMENT VARIABLES
=====================

**`CYLINDER_PATH`**
: Colon-separated path used to search for the key which will be used
  to sign transactions.

**`GRID_DAEMON_ENDPOINT`**
: Specifies a default value for `--url`.

**`GRID_DAEMON_KEY`**
: Specifies a default value for  `-k`, `--key`.

**`GRID_SERVICE_ID`**
: Specifies a default value for `--service-id`.

SEE ALSO
========
| `grid agent(1)`
| `grid organization(1)`
| `grid role(1)`
| `grid role create(1)`
| `grid role update(1)`
| `grid role list(1)`
| `grid role show(1)`
|
| Grid documentation: https://grid.hyperledger.org/docs/0.3/
