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

**grid po create** \[**FLAGS**\] \[**OPTIONS**\] <**--buyer-org** BUYER> <**--seller-org** SELLER>

DESCRIPTION
===========

This command allows for the creation of Grid Purchase Orders. It submits a
Sabre transaction to create the purchase order. `--buyer-org` and
`--seller-org` is required.

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

`--id`
: Optionally include an alternate ID. This may be specified multiple times.
An ID is of the format `alternate_id_type:alternate_id`.
Examples: `po_number:12348959` and/or `internal_po_id:a8f9fke`.

`-k`, `--key`
: base name or path to a private signing key file

`--buyer-org`
: Specify the organization that is buying the purchase order. This option is required.

`--seller-org`
: Specify the organization that is selling the purchase order. This option is required.

`--service-id`
: The ID of the service the payload should be sent to; required if running on
Splinter. Format: `<circuit-id>::<service-id>`.

`--url`
: URL for the REST API

`--uuid`
: Optionally specify the UUID of the purchase order. Must conform to the UUID
spec. If not specified, will be randomly generated.

`--wait`
: Maximum number of seconds to wait for the batch to be committed.

`--workflow-status`
: Specifies the initial workflow state of the purchase order.

EXAMPLES
========

The command

```
$ grid po create \
    --buyer-org=crgl \
    --seller-org=crgl2 \
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

**`CYLINDER_PATH`**
: Colon-separated path used to search for the key which will be used
to sign transactions

**`GRID_DAEMON_ENDPOINT`**
: Specifies a default value for `--url`

**`GRID_DAEMON_KEY`**
: Specifies a default value for `-k`, `--key`

**`GRID_SERVICE_ID`**
: Specifies a default value for `--service-id`

SEE ALSO
========

| `grid-po-list(1)`
| `grid-po-revision(1)`
| `grid-po-show(1)`
| `grid-po-update(1)`
| `grid-po-version(1)`
|
| Grid documentation: https://grid.hyperledger.org/docs/0.2/
