% GRID-PO-VERSION-CREATE(1) Cargill, Incorporated | Grid
<!--
  Copyright 2021 Cargill Incorporated
  Licensed under Creative Commons Attribution 4.0 International License
  https://creativecommons.org/licenses/by/4.0/
-->

NAME
====

**grid-po-version-create** - Create a new Grid Purchase Order Version.

SYNOPSIS
========

**grid po version create** \[**FLAGS**\] \[**OPTIONS**\] <PO_NUMBER> <VERSION_ID>

DESCRIPTION
===========

This command allows for the creation of Grid Purchase Orders versions. It
submits a Sabre transaction to create the purchase order version.

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
: Sets the resulting version as a draft. this is the default for create. Conflicts with `--non-draft`

`--not-draft`
: Sets the resulting version as a non-draft. Conflicts with `--draft`

OPTIONS
=======

`-k`, `--key`
: base name or path to a private signing key file

`--order-xml`
: Specify the path to an order xml FILE to load.  The file must conform to the
  GS1 Order spec v3.4

`--org`
: Specify the organization that owns the purchase order. This option is required.

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
$ grid po version create \
    PO-1234-56789 \
    v3 \
    --order-xml ./my_test_order.xml \
    --draft \
    --workflow-state editable \
    --wait 10
```

will generate version `v3` of purchase order `PO-1234-56789`. It will be
created as a draft and have the workflow state of `editable`.  It will
generate output like the following:

```
Submitting request to create purchase order version...
Submitted batch: 123678945
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
  "/usr/share/grid/schemas".

SEE ALSO
========
| `grid-po(1)`
| `grid-po-version-create(1)`
| `grid-po-version-list(1)`
| `grid-po-version-update(1)`
| `grid-po-revision(1)`
|
| Grid documentation: https://grid.hyperledger.org/docs/0.2/
| GS1 Order 3.4:
| https://www.gs1.org/sites/default/files/docs/EDI/ecom-xml/functional-user-guide/3_4/HTML/O/a1.htm
