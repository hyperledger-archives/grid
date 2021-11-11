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
Sabre transaction to create the purchase order. The options `--buyer-org` and
`--seller-org` are required.

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

`--alternate-id`
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

`--uid`
: Optionally specify the unique ID of the purchase order. If not specified,
will be randomly generated.

`--wait`
: Maximum number of seconds to wait for the batch to be committed.

`--workflow-state`
: Specifies the initial workflow state of the purchase order.

EXAMPLES
========

The command

```
$ grid po create \
    --buyer-org crgl \
    --seller-org crgl2 \
    --workflow-state issued \
    --alternate-id po_number:8329173 \
    --wait 10
```

will generate a purchase order owned by the `crgl` organization, with the state
of `issued`, and an alternate ID of `po_number:8329173`. It will generate
output similar to the following (the unique ID is randomly generated in this
case):

```
Submitting request to create purchase order...
Submitted batch: 55f83da3ff883dec1b4a075f55ff57aecf8428b3c07b3e8041bee5012ce2bebe4c43dab59865c30718f6960f48f380758a079597101af57e42295ec0b6203cef
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
