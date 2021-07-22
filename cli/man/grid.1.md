% GRID(1) Cargill, Incorporated | Grid Commands
<!--
  Copyright 2018-2020 Cargill Incorporated
  Licensed under Creative Commons Attribution 4.0 International License
  https://creativecommons.org/licenses/by/4.0/
-->

NAME
====

**grid** — Command-line interface for Grid

SYNOPSIS
========

**grid** \[**FLAGS**\] \[**OPTIONS**\] \[**SUBCOMMAND**\]

DESCRIPTION
===========

The `grid` utility is the command-line interface for Grid.

* Run `grid --help` to see the list of subcommands.

* Run `grid *SUBCOMMAND* --help` to see information about a specific
  subcommand (for example, `grid location create --help`).

* To view the man page for a Grid subcommand, use the "dashed form" of the
  name, where each space is replaced with a hyphen. For example, run
  `man grid-location-create` to see the man page for `grid location create`.

SUBCOMMANDS
===========

`admin`
: Administrative commands for grid.

`agent`
: Create, update, list, or show agents.

`database`
: Manage Grid Daemon database.

`keygen`
: Generate keys with which the user can sign transactions and batches.

`location`
: Provides commands for creating, updating, and deleting locations.

`organization`
: Create, update, list, or show organizations.

`po`
: Create, update, list, or show purchase orders.

`product`
: Create, update, list, show, or delete products.

`schema`
: Update or create schemas.

FLAGS
=====

Most `grid` subcommands accept the following common flags:

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

`-k`, `--key`
: Base name or path to a private signing key file.

`--service-id`
: The ID of the service the payload should be sent to; required if running on
  Splinter. Format: `<circuit-id>::<service-id>`.

`--url`
: URL for the REST API.

`--wait`
: Maximum number of seconds to wait for the batch to be committed.

ENVIRONMENT VARIABLES
=====================

Many `grid` subcommands accept the following environment variables:

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

| `grid admin(1)`
| `grid agent(1)`
| `grid database(1)`
| `grid keygen(1)`
| `grid location(1)`
| `grid organization(1)`
| `grid po(1)`
| `grid product(1)`
| `grid role(1)`
| `grid schema(1)`
|
| Grid documentation: https://grid.hyperledger.org/docs/0.2/
