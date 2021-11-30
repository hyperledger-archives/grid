% GRID-PO-SHOW(1) Cargill, Incorporated | Grid

<!--
  Copyright 2021 Cargill Incorporated
  Licensed under Creative Commons Attribution 4.0 International License
  https://creativecommons.org/licenses/by/4.0/
-->

NAME
====

**grid-po-show** - Show the details of an existing Grid Purchase Order.

SYNOPSIS
========

**grid po show** \[**FLAGS**\] \[**OPTIONS**\] <IDENTIFIER>

DESCRIPTION
===========

Show the details of a specific purchase order. This command displays the
specified revision of a specified version of the purchase order.

ARGS
====

`IDENTIFIER`
: Either a UID or an alternate ID of a purchase order.

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

`-F`, `--format=FORMAT`
: Specifies the output format of the list. Possible values for formatting are
`human`, `csv`, `yaml`, and `json`. Defaults to `human`.

`--service-id`
: The ID of the service the payload should be sent to; required if running on
Splinter. Format: `<circuit-id>::<service-id>`.

`--url`
: URL for the REST API

EXAMPLES
========

The command

```
$ grid po show --org=crgl PO-1234-56789
```

will display the current revision of the accepted version of the purchase order
with UID `PO-1234-56789` in human-readable format. It will display
output like the following:

```
Purchase Order PO-1234-56789:
    Buyer Org        crgl (Cargill Incorporated)
    Seller Org       crgl2 (Cargill 2)
    Workflow state  Confirmed
    Created At       <datetime string>
    Closed           false

Accepted Version (v3):
    workflow_state  Editable
    draft            false
    Revisions        4
    Current Revision 4

Revision 4:
    Created At       <datetime string>
    Submitter        0200ef9ab9243baee...
<Revision XML file>
```

The command

```
$ grid po show purchase_order:34595
```

will display the current revision of the accepted version of the purchase order
with the alternate ID of `purchase_order:34595` in human-readable format.
It will display output like the following:

```
Purchase Order PO-1234-56789:
    Buyer Org        crgl (Cargill Incorporated)
    Seller Org       crgl2 (Cargill 2)
    Workflow state  Confirmed
    Created At       <datetime string>
    Closed           false

Accepted Version (v3):
    workflow_state  Editable
    draft            false
    Revisions        4
    Current Revision 4

Revision 4:
    Created At       <datetime string>
    Submitter        0200ef9ab9243baee...
<Revision XML file>
```

The command

```
$ grid po show --org=crgl purchase_order:34595 --version v3 --revision 2
```

will display revision `2` of version `v3` of the purchase order
with the alternate ID of `purchase_order:34595` in human-readable format.
It will display output like the following:

```
Purchase Order PO-1234-56789:
    Buyer Org        crgl (Cargill Incorporated)
    Seller Org       crgl2 (Cargill 2)
    Workflow state  Confirmed
    Created At       <datetime string>
    Closed           false

Accepted Version (v3):
    workflow_state  Editable
    draft            false
    Revisions        4
    Current Revision 4

Revision 2:
    Created At       <datetime string>
    Submitter        0200ef9ab9243baee...
<Revision XML file>
```

ENVIRONMENT VARIABLES
=====================

**`GRID_DAEMON_ENDPOINT`**
: Specifies a default value for `--url`

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
