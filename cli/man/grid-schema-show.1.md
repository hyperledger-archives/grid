% GRID-SCHEMA-SHOW(1) Cargill, Incorporated | Grid Commands
<!--
  Copyright 2021 Cargill Incorporated
  Licensed under Creative Commons Attribution 4.0 International License
  https://creativecommons.org/licenses/by/4.0/
-->

NAME
====

**grid-schema-show** — Shows schema specified by NAME argument.

SYNOPSIS
========

**grid schema show** \[**FLAGS**\] \[**OPTIONS**\] <NAME>

DESCRIPTION
===========

Shows Schema specified by NAME argument.

ARGS
====

`NAME`
: Name of Schema.

FLAGS
=====

`-h`, `--help`
: Prints help information.

`-q`, `--quiet`
: Do not display output.

`-V`, `--version`
: Prints version information.

`-v`
: Increases verbosity (the opposite of `-q`). Specify multiple times for more
  output.

OPTIONS
=======

`--service-id`
: The ID of the service the payload should be sent to; required if running on
  Splinter. Format <circuit-id>::<service-id>.

`--url`
: URL for the REST API.

ENVIRONMENT VARIABLES
=====================

**`GRID_DAEMON_ENDPOINT`**
: Specifies a default value for `--url`.

**`GRID_SERVICE_ID`**
: Specifies a default value for `--service-id`.

SEE ALSO
========
| `grid schema(1)`
| `grid schema create(1)`
| `grid schema update(1)`
| `grid schema list(1)`
|
| Grid documentation: https://grid.hyperledger.org/docs/0.2/
