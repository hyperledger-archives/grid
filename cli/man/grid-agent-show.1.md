% GRID-agent-SHOW(1) Cargill, Incorporated | Grid Commands

<!--
  Copyright 2021 Cargill Incorporated
  Licensed under Creative Commons Attribution 4.0 International License
  https://creativecommons.org/licenses/by/4.0/
-->

NAME
====

**grid-agent-show** — Show the details of a specific agent

SYNOPSIS
========

**grid agent show** \[**FLAGS**\] \[**OPTIONS**\] <PUBLIC_KEY>

DESCRIPTION
===========

Show the complete details of a specific agent. This command requires the
`PUBLIC_KEY` argument to specify the unique identifier for the agent that is to
be retrieved.

ARGS
====

`PUBLIC_KEY`
: A public key that is used as an unique identifier for agents

FLAGS
=====

`-h`, `--help`
: Prints help information

`-q`, `--quiet`
: Do not display output

`-V`, `--version`
: Prints version information

`-v`
: Increases verbosity (the opposite of `-q`). Specify multiple times for more
output

OPTIONS
=======

`--service-id`
: The ID of the service the payload should be sent to; required if running on
  Splinter. Format: `<circuit-id>::<service-id>`.

`--url`
: URL for the REST API

EXAMPLES
========

The command

```
$ grid agent show 03a3374bc95109d0fe4be641fa0100853de34f46452bd688936f73ad986729e9c0
```

Will show the details of the specified agent

```
Public Key: 03a3374bc95109d0fe4be641fa0100853de34f46452bd688936f73ad986729e9c0
Organization Id: crgl
Active: true
Service ID: b8xWJ-0QcMy::gsAA
Roles: productowner, admin
Metadata:
    field1: value1
    field2: value2
```

ENVIRONMENT VARIABLES
=====================

**`GRID_DAEMON_ENDPOINT`**
: Specifies a default value for `--url`

**`GRID_SERVICE_ID`**
: Specifies a default value for `--service-id`

SEE ALSO
========

| `grid organization(1)`
| `grid agent(1)`
| `grid agent create(1)`
| `grid agent list(1)`
| `grid agent update(1)`
| `grid role(1)`
|
| Grid documentation: https://grid.hyperledger.org/docs/0.3/
