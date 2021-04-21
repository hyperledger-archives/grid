% GRID-ROLE-CREATE(1) Cargill, Incorporated | Grid
<!--
  Copyright 2021 Cargill Incorporated
  Licensed under Creative Commons Attribution 4.0 International License
  https://creativecommons.org/licenses/by/4.0/
-->

NAME
====

**grid-role-create** - Create a new Grid Pike role.

SYNOPSIS
========

**grid role create** \[**FLAGS**\] \[**OPTIONS**\] ORG_ID NAME

ARGS
====

`ORG_ID`
: The organization identifier to create the role for

`NAME`
: The user-specified name of the role

FLAGS
=====

`--active`
: Set role as active

`-h`, `--help`
: Prints help information

`--inactive`
: Set role as inactive

`-q`, `--quiet`
: Do not display output

`-V`, `--version`
: Prints version information

`-v`
: Log verbosely

OPTIONS
=======

`--allowed-orgs`
: List of organizations allowed use of the role

`-d`, `--description`
: Description of the role

`--inherit-from`
: List of roles to inherit permissions from

`-k`, `--key`
: Base name for private signing key file

`--permissions`
: List of permissions belonging to the role

`--service-id`
: The ID of the service the payload should be sent to; required if running on
Splinter. Format <circuit-id>::<service-id>

`--url`
: URL for the REST API

`--wait`
: How long to wait for transaction to be committed

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
| `grid role delete(1)`
| `grid role update(1)`
|
| Grid documentation: https://grid.hyperledger.org/docs/0.2/
