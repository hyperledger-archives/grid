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

**grid po version update** \[**FLAGS**\] \[**OPTIONS**\] <VERSION_ID> <**--org** ORG_ID>

DESCRIPTION
===========

This command allows for the update of Grid Purchase Orders versions. It
submits a Sabre transaction to create the purchase order version. Each update
creates a new revision of that version.

VERSION_ID argument and --org option are required.


ARGS
====

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

`--org`
: Specify the organization that owns the purchase order.

`--service-id`
: The ID of the service the payload should be sent to; required if running on
  Splinter. Format: `<circuit-id>::<service-id>`.

`--url`
: URL for the REST API

`--wait`
: Maximum number of seconds to wait for the batch to be committed.

`--workflow-status`
: Specifies the initial workflow state of the purchase order.

EXAMPLES
========

The command

```
$ grid po version update \
    --org=crgl \
    --po=po_number:809832081 \
    --workflow-status=Review \
    --wait \
    v3
```

will update the version `v3` of purchase order `82urioz098aui3871uc`, owned by
the organization `crgl`, to have the workflow status of `Review`. It will
generate output like the following:

```
Updating version "v3" for Purchase Order 82urioz098aui3871uc.
Submitted Purchase Order Version create transaction:
    Batch: 52342uoisufoin38908fyhsdfhs098yv98y98v
    Transaction: 123456uofo238098fyu80h028yf082ehf8h
Waiting for transaction to be committed...
Transaction was committed successfully.
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
| `grid-po(1)`
| `grid-po-version-create(1)`
| `grid-po-version-list(1)`
| `grid-po-version-update(1)`
|
| Grid documentation: https://grid.hyperledger.org/docs/0.2/
