% GRID-PRODUCT-SHOW(1) Cargill, Incorporated | Grid Commands
<!--
  Copyright 2018-2021 Cargill Incorporated
  Licensed under Creative Commons Attribution 4.0 International License
  https://creativecommons.org/licenses/by/4.0/
-->

NAME
====

**grid-product-show** — Show the details of a specific product.

SYNOPSIS
========

**grid product show** \[**FLAGS**\] \[**OPTIONS**\] <PRODUCT_ID>

DESCRIPTION
===========

Show the complete details of a specific product. This command requires the
`PRODUCT_ID` argument to specify the unique identifier for the product that
is to be retrieved.

ARGS
====

`PRODUCT_ID`
: Unique identifier of the product.

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
  Splinter. Format: `<circuit-id>::<service-id>`.

`--url`
: URL for the REST API.

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
| Grid documentation: https://grid.hyperledger.org/docs/0.3/
