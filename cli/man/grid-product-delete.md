% GRID-PRODUCT-DELETE(1) Cargill, Incorporated | Grid Commands
<!--
  Copyright 2018-2021 Cargill Incorporated
  Licensed under Creative Commons Attribution 4.0 International License
  https://creativecommons.org/licenses/by/4.0/
-->

NAME
====

**grid-product-delete** — Delete an existing product

SYNOPSIS
========

**grid product delete** \[**FLAGS**\] \[**OPTIONS**\] <**product_id**>

DESCRIPTION
===========

Delete an existing product. This command requires the `--product_id` option
to specify the unique identifier of the product that is to be deleted. The
`--product_namespace` option must also be specified (e.g. GS1).

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

OPTIONS
=======

`--namespace`
: Product namespace (e.g. `GS1`)

ARGS
====

`<product_id>`
: Unique identifier of the product

EXAMPLES
========

Delete an existing product.

```
$ grid product delete --product_id 762111177704 --product_namespace GS1
```

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

SEE ALSO
========
| `grid-product-create(1)`
| `grid-product-update(1)`
| `grid-product-delete(1)`
| `grid-product-show(1)`
| `grid-product-list(1)`
|
| Grid documentation: https://grid.hyperledger.org/docs/0.1/
