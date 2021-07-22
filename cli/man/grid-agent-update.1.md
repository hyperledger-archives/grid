% GRID-AGENT-UPDATE(1) Cargill, Incorporated | Grid
<!--
  Copyright 2021 Cargill Incorporated
  Licensed under Creative Commons Attribution 4.0 International License
  https://creativecommons.org/licenses/by/4.0/
-->

NAME
====

**grid-agent-update** - Update an existing Grid Pike agent.

SYNOPSIS
========

**grid agent update** \[**FLAGS**\] \[**OPTIONS**\] <ORG_ID**> <PUBLIC_KEY> <{**--active**|**--inactive**}>

DESCRIPTION
===========

Updates an existing agent. ORG_ID and PUBLIC_KEY arguments are required, 
as well as the --active or --inactive flag.

ARGS
====

`ORG_ID`
: The Pike organization identifier to create the agent for

`PUBLIC_KEY`
: The user-specified public key of the agent to create

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

`--active`
: sets agent as active. conflicts with `--inactive`

`--inactive`
: sets agent as inactive. conflicts with `--active`

OPTIONS
=======

`-k`, `--key`
: base name or path to a private signing key file

`--metadata`
: Key-value pairs (format: `<key>=<value>`) in a comma-separated list

`--role`
: Roles assigned to the agent. Multiple roles can be assigned in a 
  comma-separated list.

`--service-id`
: The ID of the service the payload should be sent to; required if running on
  Splinter. Format: `<circuit-id>::<service-id>`.

`--url`
: URL for the REST API

`--wait`
: How long to wait for transaction to be committed

ENVIRONMENT VARIABLES
=====================

**`CYLINDER_PATH`**
: Colon-separated path used to search for the key which will be used
  to sign transactions

**`GRID_DAEMON_ENDPOINT`**
: Specifies a default value for `--url`

**`GRID_DAEMON_KEY`**
: Specifies a default value for  `-k`, `--key`

**`GRID_SERVICE_ID`**
: Specifies a default value for `--service-id`

SEE ALSO
========
| `grid organization(1)`
| `grid agent(1)`
| `grid agent create(1)`
| `grid agent list(1)`
| `grid agent show(1)`
| `grid role(1)`
|
| Grid documentation: https://grid.hyperledger.org/docs/0.2/
