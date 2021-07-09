% GRID-ORGANIZATION-UPDATE(1) Cargill, Incorporated | Grid
<!--
  Copyright 2021 Cargill Incorporated
  Licensed under Creative Commons Attribution 4.0 International License
  https://creativecommons.org/licenses/by/4.0/
-->

NAME
====

**grid-organization-update** - Updates an existing Grid Pike organization.

SYNOPSIS
========

**grid organization update** \[**FLAGS**\] \[**OPTIONS**\] <ORG_ID> <NAME>

ARGS
====

`ORG_ID`
: The user-specified organization identifier.

`NAME`
: The user-specified name of the organization.

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

`--alternate-ids`
: Alternate IDs for organization in a comma-separated list. 
  Format: <id_type>:<id>.

`-k`, `--key`
: Base name or path to a private signing key file.

`--locations`
: List of comma-separated locations associated with this organization.

`--metadata`
: Key-value pairs in a comma-separated list.
  Format: <key>=<value>.

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
| `grid organization(1)`
| `grid organization create(1)`
| `grid organization list(1)`
| `grid organization show(1)`
| `grid agent(1)`
| `grid role(1)`
|
| Grid documentation: https://grid.hyperledger.org/docs/0.2/
