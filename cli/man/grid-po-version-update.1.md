% GRID-PO-VERSION-UPDATE(1) Cargill, Incorporated | Grid
<!--
  Copyright 2021 Cargill Incorporated
  Licensed under Creative Commons Attribution 4.0 International License
  https://creativecommons.org/licenses/by/4.0/
-->

NAME
====

**grid-po-version-update** - Update an existing Grid Purchase Order Version.

SYNOPSIS
========

**grid po version update** \[**FLAGS**\] \[**OPTIONS**\] <PO_NUMBER> <VERSION_ID>

DESCRIPTION
===========

This command allows for the update of Grid Purchase Orders versions. It
submits a Sabre transaction to create the purchase order version. Each update
creates a new revision of that version.

The PO_NUMBER and VERSION_ID arguments are required.


ARGS
====

`PO_NUMBER`
: The UID or an alternate ID of the purchase order this version is for.

`VERSION_ID`
: The user-specified version identifier.

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

`--draft`
: Sets the resulting version as a draft. Conflicts with `--not-draft`.

`--not-draft`
: Sets the resulting version as a non-draft. Conflicts with `--draft`.

OPTIONS
=======

`-k`, `--key`
: base name or path to a private signing key file

`--order-xml`
: Specify the path to an order xml FILE to load.  The file must conform to the
  GS1 Order spec v3.4

`--service-id`
: The ID of the service the payload should be sent to; required if running on
  Splinter. Format: `<circuit-id>::<service-id>`.

`--url`
: URL for the REST API

`--wait`
: Maximum number of seconds to wait for the batch to be committed.

`--workflow-state`
: Specifies the initial workflow state of the purchase order.

EXAMPLES
========

The command

```
$ grid po version update \
    PO-00000-1111 \
    v3 \
    --workflow-state review \
    --wait=10
```

will update the version `v3` of purchase order `PO-00000-1111` to have the
workflow state of `review`. It will generate output like the following:

```
Submitting request to update purchase order version...
Submitted batch: efed2ff3efc50ac32a81b05b8d330fdb154d...
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

**`GRID_ORDER_SCHEMA_DIR`**
: Specifies the local path to the directory containing the `Order.xsd`
  schema used to validate the purchase order. The default value is
  "/usr/share/grid/xsd".

SEE ALSO
========
| `grid-po(1)`
| `grid-po-version-create(1)`
| `grid-po-version-list(1)`
| `grid-po-version-update(1)`
| `grid-po-revision(1)`
|
| Grid documentation: https://grid.hyperledger.org/docs/0.2/
