% GRID-PRODUCT-DELETE(1) Cargill, Incorporated | Grid Commands
<!--
  Copyright 2018-2021 Cargill Incorporated
  Licensed under Creative Commons Attribution 4.0 International License
  https://creativecommons.org/licenses/by/4.0/
-->

NAME
====

**grid-product-delete** — Delete an existing product.

SYNOPSIS
========

**grid product delete** \[**FLAGS**\] \[**OPTIONS**\] <PRODUCT_ID> <**--namespace** NAMESPACE>

DESCRIPTION
===========

Delete an existing product. This command requires the `PRODUCT_ID` argument
to specify the unique identifier of the product that is to be deleted. The
`--namespace` option must also be specified (e.g. GS1).

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

`-k`, `--key`
: Base name or path to a private signing key file.

`--namespace`
: Product namespace (e.g. `GS1`).

`--service-id`
: The ID of the service the payload should be sent to; required if running on
  Splinter. Format: `<circuit-id>::<service-id>`.

`--url`
: URL for the REST API.

`--wait`
: Maximum number of seconds to wait for the batch to be committed.

EXAMPLES
========

Delete an existing product:
```
$ grid product delete --product_id 762111177704 --product_namespace GS1
```

ENVIRONMENT VARIABLES
=====================

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
| `grid-product-create(1)`
| `grid-product-update(1)`
| `grid-product-delete(1)`
| `grid-product-show(1)`
| `grid-product-list(1)`
|
| Grid documentation: https://grid.hyperledger.org/docs/0.3/
