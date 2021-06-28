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

**grid role create** \[**FLAGS**\] \[**OPTIONS**\] <ORG_ID> <NAME>

ARGS
====

`ORG_ID`
: The organization identifier to create the role for.

`NAME`
: The user-specified name of the role.

FLAGS
=====

`--active`
: Set role as active. Conflicts with `--inactive`.

`-h`, `--help`
: Prints help information.

`--inactive`
: Set role as inactive. Conflicts with `--active`.

`-q`, `--quiet`
: Do not display output.

`-V`, `--version`
: Prints version information.

`-v`
: Log verbosely.

OPTIONS
=======

`--allowed-orgs`
: List of organizations allowed use of the role.

`-d`, `--description`
: Description of the role.

`--inherit-from`
: List of roles to inherit permissions from.

`-k`, `--key`
: Base name or path to a private signing key file.

`--permissions`
: List of permissions belonging to the role. Multiple permissions can be 
  assigned in a comma-separated list.

`--service-id`
: The ID of the service the payload should be sent to; required if running on
  Splinter. Format <circuit-id>::<service-id>.

`--url`
: URL for the REST API.

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
| `grid role delete(1)`
| `grid role update(1)`
| `grid role list(1)`
| `grid role show(1)`
|
| Grid documentation: https://grid.hyperledger.org/docs/0.2/
