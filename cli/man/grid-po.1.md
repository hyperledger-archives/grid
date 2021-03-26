% GRID-PO(1) Cargill, Incorporated | Grid Commands
<!--
  Copyright 2021 Cargill Incorporated
  Licensed under Creative Commons Attribution 4.0 International License
  https://creativecommons.org/licenses/by/4.0/
-->

NAME
====

**grid-po** — Create or update Grid Purchase Orders.

SYNOPSIS
========

**grid po** \[**FLAGS**\] \[**SUBCOMMAND**\]

DESCRIPTION
===========

This command allows for the creation and management of Grid Purchase Orders.
Commands to list and display Purchase Order data are also available.

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

`-k`, `--key`
: Base name for private key file.

`--url`
: URL for the REST API.

`--service-id`
: The ID of the service the payload should be sent to; required if running on
  Splinter. Format <circuit-id>::<service-id>.

`--wait`
: How long to wait for transaction to be committed.

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
: Create a new purchase order.

`list`
: List details of all existing purchase orders.

`show`
: Show details of a specific purchase order.

`update`
: Update an existing purchase order.

`version`
: Create or update versions of a purchase order.

SEE ALSO
========
| `grid-po-create(1)`
| `grid-po-list(1)`
| `grid-po-show(1)`
| `grid-po-update(1)`
| `grid-po-version(1)`
|
| Grid documentation: https://grid.hyperledger.org/docs/0.1/
