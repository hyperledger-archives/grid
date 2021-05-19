% GRID-agent-LIST(1) Cargill, Incorporated | Grid Commands

<!--
  Copyright 2021 Cargill Incorporated
  Licensed under Creative Commons Attribution 4.0 International License
  https://creativecommons.org/licenses/by/4.0/
-->

NAME
====

**grid-agent-list** — List all agents

SYNOPSIS
========

**grid agent list** \[**FLAGS**\]

DESCRIPTION
===========

List all agents in grid. If the `service_id` flag is specified, only
agents corresponding to that `service_id` will be shown.

FLAGS
=====

`-h`, `--help`
: Prints help information

`-k`, `--key`
: Base name for private key file

`-q`, `--quiet`
: Do not display output

`--service-id`
: The ID of the service the payload should be sent to; required if running on
Splinter. Format <circuit-id>::<service-id>

`-V`, `--version`
: Prints version information

`-v`
: Increases verbosity (the opposite of `-q`). Specify multiple times for more
output

`--url`
: URL for the REST API

`--line-per-role`
: Displays agent information for each role on it's own line. Useful when filtering by role.

`-F`, `--format=FORMAT`
: Specifies the output format of the list. Possible values for formatting are `human` and `csv`. Defaults to `human`.

EXAMPLES
========

The command

```
$ grid agent list
```

Will list all agents and their associated roles

```
PUBLIC_KEY     ORG_ID ACTIVE ROLES              
03a3374bc95... crgl   true   productowner, admin
```

```
$ grid agent list --line-per-role
```

Will list all agents with roles on their own lines.

```
PUBLIC_KEY     ORG_ID ACTIVE ROLES              
03a3374bc95... crgl   true   productowner
03a3374bc95... crgl   true   admin
```

ENVIRONMENT VARIABLES
=====================

**`GRID_DAEMON_ENDPOINT`**
: Specifies the endpoint for the grid daemon (`gridd`)
if `-U` or `--url` is not used.

**`GRID_SERVICE_ID`**
: Specifies service ID if `--service-id` is not used

SEE ALSO
========

| `grid organization(1)`
| `grid agent(1)`
| `grid agent create(1)`
| `grid agent show(1)`
| `grid agent update(1)`
| `grid role(1)`
|
| Grid documentation: https://grid.hyperledger.org/docs/0.1/
