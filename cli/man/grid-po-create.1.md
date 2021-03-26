% GRID-PO-CREATE(1) Cargill, Incorporated | Grid
<!--
  Copyright 2021 Cargill Incorporated
  Licensed under Creative Commons Attribution 4.0 International License
  https://creativecommons.org/licenses/by/4.0/
-->

NAME
====

**grid-po-create** - Create a new Grid Purchase Order.

SYNOPSIS
========

**grid po create** \[**FLAGS**\] \[**OPTIONS**\]

DESCRIPTION
===========

This command allows for the creation of Grid Purchase Orders. It submits a
Sabre transaction to create the purchase order.

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

`--org`
: Specify the organization that owns the purchase order.

`--uuid`
: Optionally specify the UUID of the purchase order. Must conform to the UUID
  spec. If not specified, will be randomly generated.

`--id`
: Optionally include an alternate ID. This may be specified multiple times (0
  to infinity). An ID is of the format `alternate_id_type:alternate_id`.
  Examples: `po_number:12348959` and/or `internal_po_id:a8f9fke`.

`--wait`
: Specify how long to wait, in seconds, for the transaction to be committed.

`--workflow-status`
: Specifies the initial workflow state of the purchase order.

EXAMPLES
========

The command

```
$ grid po create \
    --org=crgl \
    --workflow-status=Issued \
    --id=po_number:8329173 \
    --wait=0
```

will generate a purchase order owned by the `crgl` organization, with the status
of `Issued`, and an alternate ID of `po_number:8329173`. It will generate
output similar to the following (the UUID is randomly generated in this case):

```
Generated Purchase Order UUID: 82urioz098aui3871uc
Submitted Purchase Order create transaction:
    Batch: 8903huoisufoin38908fyhsdfhs098yv98y98v
    Transaction: 24898uofo238098fyu80h028yf082ehf8h
Waiting for transaction to be committed...
Transaction was committed successfully.
```

ENVIRONMENT VARIABLES
=====================

**`GRID_DAEMON_ENDPOINT`**
: Specifies the endpoint for the grid daemon (`gridd`)
  if `-U` or `--url` is not used.

**`GRID_SERVICE_ID`**
: Specifies service ID if `--service-id` is not used

**`GRID_ORG_ID`**
: Specifies the organization id that owns the purchase order.

SEE ALSO
========
| `grid-po-create(1)`
| `grid-po-list(1)`
| `grid-po-show(1)`
| `grid-po-update(1)`
| `grid-po-version(1)`
|
| Grid documentation: https://grid.hyperledger.org/docs/0.2/
