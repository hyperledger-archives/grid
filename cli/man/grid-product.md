% GRID-PRODUCT(1) Cargill, Incorporated | Grid Commands
<!--
  Copyright 2018-2021 Cargill Incorporated
  Licensed under Creative Commons Attribution 4.0 International License
  https://creativecommons.org/licenses/by/4.0/
-->

NAME
====

**grid-product** — Create, update, or delete Grid Products

SYNOPSIS
========

**grid product** \[**FLAGS**\] \[**SUBCOMMAND**\]

DESCRIPTION
===========

This command allows for the creation and management of Grid Products.
Commands to list and display Product data are also available.

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

SUBCOMMANDS
===========

`create`
: Create new products

`update`
: Update existing products

`delete`
: Delete a product

`show`
: Show details of a specified product

`list`
: List details of all existing products

SEE ALSO
========
| `grid-product-create(1)`
| `grid-product-update(1)`
| `grid-product-delete(1)`
| `grid-product-show(1)`
| `grid-product-list(1)`
|
| Grid documentation: https://grid.hyperledger.org/docs/0.1/
