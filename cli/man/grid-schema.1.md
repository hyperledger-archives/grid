% GRID-SCHEMA(1) Cargill, Incorporated | Grid
<!--
  Copyright 2021 Cargill Incorporated
  Licensed under Creative Commons Attribution 4.0 International License
  https://creativecommons.org/licenses/by/4.0/
-->

NAME
====

**grid-schema** - Create, Update, List or Show Grid schemas.

SYNOPSIS
========

**grid schema** \[**FLAGS**\] \[**OPTIONS**\] SUBCOMMAND

DESCRIPTION
===========

This command allows for the creation and management of Grid schemas.

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

Many subcommands utilize the following options:

`--service-id`
: The ID of the service the payload should be sent to; required if running on
  Splinter. Format: `<circuit-id>::<service-id>`.

`--url`
: URL for the REST API.

ENVIRONMENT VARIABLES
=====================

Many subcommands accept the following environment variables:

**`CYLINDER_PATH`**
: Colon-separated path used to search for the key which will be used
  to sign transactions.

**`GRID_DAEMON_ENDPOINT`**
: Specifies a default value for `--url`.

**`GRID_DAEMON_KEY`**
: Specifies a default value for  `-k`, `--key`.

**`GRID_SERVICE_ID`**
: Specifies a default value for `--service-id`.

SUBCOMMANDS
===========

`create`
: Create schemas from a yaml file.

`help`
: Prints this message or the help of the given subcommand(s).

`list`
: list currently defined schemas.

`show`
: Show schema specified by name argument.

`update`
: Updates schemas from a yaml file.

SEE ALSO
========
| `grid schema create(1)`
| `grid schema update(1)`
| `grid schema list(1)`
| `grid schema show(1)`
|
| Grid documentation: https://grid.hyperledger.org/docs/0.3/
