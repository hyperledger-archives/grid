% GRID-PRODUCT-LIST(1) Cargill, Incorporated | Grid Commands
<!--
  Copyright 2018-2021 Cargill Incorporated
  Licensed under Creative Commons Attribution 4.0 International License
  https://creativecommons.org/licenses/by/4.0/
-->

NAME
====

**grid-product-list** — List all products.

SYNOPSIS
========

**grid product list** \[**FLAGS**\] \[**OPTIONS**\]

DESCRIPTION
===========

List all products in grid. If the `service_id` option is specified, only
products corresponding to that `service_id` will be shown.

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

EXAMPLES
========

The following command will list all products and their properties:
```
$ grid product list
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
: Specifies a default value for `--url`.

**`GRID_SERVICE_ID`**
: Specifies a default value for `--service-id`.

SEE ALSO
========
| `grid-product-create(1)`
| `grid-product-update(1)`
| `grid-product-delete(1)`
| `grid-product-show(1)`
| `grid-product-list(1)`
|
| Grid documentation: https://grid.hyperledger.org/docs/0.2/
