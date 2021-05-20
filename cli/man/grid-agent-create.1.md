% GRID-AGENT-CREATE(1) Cargill, Incorporated | Grid

<!--
  Copyright 2021 Cargill Incorporated
  Licensed under Creative Commons Attribution 4.0 International License
  https://creativecommons.org/licenses/by/4.0/
-->

NAME
====

**grid-agent-create** - Create a new Grid Pike agent.

SYNOPSIS
========

**grid agent create** \[**FLAGS**\] \[**OPTIONS**\] ORG_ID PUBLIC_KEY --active --inactive

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

OPTIONS
=======

`-k`, `--key`
: Base name for private signing key file

`--metadata`
: Key-value pairs (format: <key>=<value>) in a comma-separated list

`--role`
: Roles assigned to the agent

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
| `grid organization(1)`
| `grid agent(1)`
| `grid agent create(1)`
| `grid agent list(1)`
| `grid agent show(1)`
| `grid agent update(1)`
| `grid role(1)`
|
| Grid documentation: https://grid.hyperledger.org/docs/0.2/
