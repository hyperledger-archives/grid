% GRID-PRODUCT-LIST(1) Cargill, Incorporated | Grid Commands
<!--
  Copyright 2018-2020 Cargill Incorporated
  Licensed under Creative Commons Attribution 4.0 International License
  https://creativecommons.org/licenses/by/4.0/
-->

NAME
====

**grid-product-list** — List all products

SYNOPSIS
========

**grid product list** \[**FLAGS**\]

DESCRIPTION
===========

List all products in grid. If the `service_id` flag is specified, only
products corresponding to that `service_id` will be shown.

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

EXAMPLES
========

The command

```
$ grid product list
```

Will list all products and their properties

```
Product ID: 762111177704
Product Namespace: GS1
Owner: 314156
Properties:
    length: 8
    width: 11
    height: 1
Product ID: 881334009880
Product Namespace: GS1
Owner: 314156
Properties:
    length: 8
    width: 11
    height: 11
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
| `grid-product-create(1)`
| `grid-product-update(1)`
| `grid-product-delete(1)`
| `grid-product-show(1)`
| `grid-product-list(1)`
|
| Grid documentation: https://grid.hyperledger.org/docs/0.1/
