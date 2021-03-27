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

**grid po create** \[**FLAGS**\] \[**OPTIONS**\] VERSION_ID

DESCRIPTION
===========

This command allows for the creation of Grid Purchase Orders versions. It
submits a Sabre transaction to create the purchase order version.

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
: Sets the resulting version as a draft.

OPTIONS
=======

`-k`, `--key`
: Base name for private key file.

`--order-xml`
: Specify the path to an order xml FILE to load.  The file must conform to the
  GS1 Order spec v3.4

`--org`
: Specify the organization that owns the purchase order.

`--po`
: Either a UUID or an alternate ID of a purchase order.

`--service-id`
: The ID of the service the payload should be sent to; required if running on
  Splinter. Format <circuit-id>::<service-id>.

`--url`
: URL for the REST API.

`--wait`
: Specify how long to wait, in seconds, for the transaction to be committed.

`--workflow-status`
: Specifies the initial workflow state of the purchase order.

EXAMPLES
========

The command

```
$ grid po version create \
    --org=crgl \
    --po=82urioz098aui3871uc \
    --order-xml ./my_test_order.xml \
    --draft \
    --workflow-status=Editable \
    --wait \
    v3
```

will generate version `v3` of purchase order `82urioz098aui3871uc` owned by the
organization `crgl`. It will be created as a draft and have the workflow status
of `Editable`.  It will generate output like the following:

```
Creating version "v3" for Purchase Order 82urioz098aui3871uc.
Submitted Purchase Order Version create transaction:
    Batch: 142312uoisufoin38908fyhsdfhs098yv98y98v
    Transaction: af32348uofo238098fyu80h028yf082ehf8h
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
| `grid-po(1)`
| `grid-po-version-create(1)`
| `grid-po-version-list(1)`
| `grid-po-version-update(1)`
|
| Grid documentation: https://grid.hyperledger.org/docs/0.2/
| GS1 Order 3.4:
| https://www.gs1.org/sites/default/files/docs/EDI/ecom-xml/functional-user-guide/3_4/HTML/O/a1.htm
