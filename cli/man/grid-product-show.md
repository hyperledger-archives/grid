% GRID-PRODUCT-SHOW(1) Cargill, Incorporated | Grid Commands
<!--
  Copyright 2018-2020 Cargill Incorporated
  Licensed under Creative Commons Attribution 4.0 International License
  https://creativecommons.org/licenses/by/4.0/
-->

NAME
====

**grid-product-show** — Show the details of a specific product

SYNOPSIS
========

**grid product show** \[**FLAGS**\] <**product_id**>

DESCRIPTION
===========

Show the complete details of a specific product. This command requires the
`<product_id>` argument to specify the unique identifier for the product that
is to be retrieved.

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

ARGS
====

`<product_id>`
: Unique identifier of the product

EXAMPLES
========

The command

```
$ grid product show --product_id 762111177704
```

Will show the details of the specified product

```
Product ID: 762111177704
Product Namespace: GS1
Owner: 314156
Properties:
    length: 8
    width: 11
    height: 1
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
