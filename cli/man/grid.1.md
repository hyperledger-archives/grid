% GRID(1) Cargill, Incorporated | Grid Commands
<!--
  Copyright 2018-2021 Cargill Incorporated
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
: Administrative commands for grid

`agent`
: Update or create an agent

`database`
: Manage Grid Daemon database

`keygen`
: Generate keys with which the user can sign transactions and batches

`location`
: Provides commands for creating, updating, and deleting locations

`organization`
: Update or create an organization

`product`
: Create, update or delete products

`schema`
: Update or create schemas

FLAGS
=====

Most `grid` subcommands accept the following common flags:

`-h`, `--help`
: Prints help information

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

OPTIONS
=======

`-k`, `--key`
: Base name for private key file

`--url`
: URL for the REST API

`--wait`
: How long to wait for transaction to be committed

ENVIRONMENT VARIABLES
=====================

Many `grid` subcommands accept the following environment variable:

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
| `grid-location-create(1)`
| `grid-location-update(1)`
| `grid-location-delete(1)`
| `grid-location-show(1)`
| `grid-location-list(1)`
|
| `grid-product-create(1)`
| `grid-product-update(1)`
| `grid-product-delete(1)`
| `grid-product-show(1)`
| `grid-product-list(1)`
|
| `grid(1)`
|
| Grid documentation: https://grid.hyperledger.org/docs/0.1/
