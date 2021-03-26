% GRID-PO-UPDATE(1) Cargill, Incorporated | Grid
<!--
  Copyright 2021 Cargill Incorporated
  Licensed under Creative Commons Attribution 4.0 International License
  https://creativecommons.org/licenses/by/4.0/
-->

NAME
====

**grid-po-update** - Update an existing Grid Purchase Order.

SYNOPSIS
========

**grid po update** \[**FLAGS**\] \[**OPTIONS**\] IDENTIFIER

DESCRIPTION
===========

This command allows for the update of Grid Purchase Orders. It submits a
Sabre transaction to update an existing purchase order.

ARGS
====

`IDENTIFIER`
: Either a UUID or an alternate ID of a purchase order.

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

`--add-id`
: Add an alternate ID. This may be specified multiple times (0 to infinity).
  An ID is of the format `alternate_id_type:alternate_id`.  Examples:
  `po_number:12348959` and/or `internal_po_id:a8f9fke`.

`--rm-id`
: Remove an alternate ID. This may be specified multiple times (0 to infinity).
  An ID is of the format `alternate_id_type:alternate_id`.  Examples:
  `po_number:12348959` and/or `internal_po_id:a8f9fke`.

`--is-closed`
: Specifies if the purchuse order is closed or open. Possible values are `true`
  or `false`.

`--accepted-version`
: Specifies the accepted version ID of the purchase order.

`--wait`
: Specify how long to wait, in seconds, for the transaction to be committed.

`--workflow-status`
: Specifies the workflow state of the purchase order.

EXAMPLES
========

The command

```

$ grid po update \
    --org=crgl \
    --add-id=po_number:809832081 \
    --wait=0 \
    82urioz098aui3871uc
```

will update the purchase order with UUID `82urioz098aui3871uc` and add an
additional alternate ID. It will generate output like the following:

```
Submitted Purchase Order update transaction:
    Batch: 8903huoisufoin38908fyhsdfhs098yv98y98v
    Transaction: 24898uofo238098fyu80h028yf082ehf8h
Waiting for transaction to be committed...
Transaction was committed successfully.
```

The command

```
$ grid po update \
    --org=crgl \
    --workflow-status=Confirmed \
    --accepted-version=v3 \
    --wait \
    po_number:809832081
```

will update the purchase order with alternate ID `po_number:809832081`, set the
workflow status to `Confirmed`, and the accepted version to `v3`. It will
generate output like the following:

```
Found Purchase Order UID: 82urioz098aui3871uc
Submitted Purchase Order update transaction:
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
