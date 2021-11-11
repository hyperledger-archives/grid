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

**grid po update** \[**FLAGS**\] \[**OPTIONS**\] <IDENTIFIER>

DESCRIPTION
===========

This command allows for the update of Grid Purchase Orders. It submits a
Sabre transaction to update an existing purchase order.

ARGS
====

`IDENTIFIER`
: Either a unique ID or an alternate ID of a purchase order.

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

`--accepted-version`
: Specifies the accepted version ID of the purchase order.

`--add-id`
: Add an alternate ID. This may be specified multiple times.
  An ID is of the format `alternate_id_type:alternate_id`.  Examples:
  `po_number:12348959` and/or `internal_po_id:a8f9fke`.

`--is-closed`
: Specifies if the purchase order is closed or open. Possible values are `true`
  or `false`.

`-k`, `--key`
: base name or path to a private signing key file

`--org`
: Specify the organization that owns the purchase order.

`--rm-id`
: Remove an alternate ID. This may be specified multiple times.
  An ID is of the format `alternate_id_type:alternate_id`.  Examples:
  `po_number:12348959` and/or `internal_po_id:a8f9fke`.

`--service-id`
: The ID of the service the payload should be sent to; required if running on
  Splinter. Format: `<circuit-id>::<service-id>`.

`--url`
: URL for the REST API

`--wait`
: Maximum number of seconds to wait for the batch to be committed.

`--workflow-state`
: Specifies the workflow state of the purchase order.

EXAMPLES
========

The command

```

$ grid po update \
    --org crgl \
    --add-id po_number:809832081 \
    --wait 10 \
    PO-1234-56789
```

will update the purchase order with UID `PO-1234-56789` and add an
additional alternate ID. It will generate output like the following:

```
Submitting request to update purchase order...
Submitted batch: 0123456789
Batch and transaction structure was valid. Batch queued.
```

The command

```
$ grid po update \
    --org crgl \
    --workflow-state confirmed \
    --accepted-version v3 \
    --wait 10 \
    po_number:809832081
```

will update the purchase order with alternate ID `po_number:809832081`, set the
workflow state to `confirmed`, and the accepted version to `v3`. It will
generate output like the following:

```
Submitting request to update purchase order...
Submitted batch: 987654321
Batch and transaction structure was valid. Batch queued.
```

ENVIRONMENT VARIABLES
=====================

**`CYLINDER_PATH`**
: Colon-separated path used to search for the key which will be used
  to sign transactions

**`GRID_DAEMON_ENDPOINT`**
: Specifies a default value for `--url`

**`GRID_DAEMON_KEY`**
: Specifies a default value for  `-k`, `--key`

**`GRID_SERVICE_ID`**
: Specifies a default value for `--service-id`

SEE ALSO
========
| `grid-po-create(1)`
| `grid-po-list(1)`
| `grid-po-show(1)`
| `grid-po-update(1)`
| `grid-po-version(1)`
|
| Grid documentation: https://grid.hyperledger.org/docs/0.2/
