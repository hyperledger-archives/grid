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

Show the details of a specific purchase order.  This command displays the
specified revision of a specified version of the purchase order.

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

`-F`, `--format=FORMAT`
: Specifies the output format of the list. Possible values for formatting are
  `human`, `csv`, `yaml`, and `json`. Defaults to `human`.

`--org`
: Specify the organization that owns the purchase order.

`--revision`
: Specify the revision number for the specified version. Defaults to the latest
  revision.

`--service-id`
: The ID of the service the payload should be sent to; required if running on
  Splinter. Format: `<circuit-id>::<service-id>`.

`--url`
: URL for the REST API

`--version`
: Specify the version ID of the purchase order to display. Defaults to the
  accepted version.

EXAMPLES
========

The command

```

$ grid po show --org=crgl 82urioz098aui3871uc
```

will display the current revision of the accepted version of the purchase order
with UUID `82urioz098aui3871uc` in human-readable format. It will display
output like the following:

```
Purchase Order:
    organization     crgl (Cargill Incorporated)
    uuid             82urioz098aui3871uc
    purchase_order   809832081
    workflow status  Confirmed
    is closed        False
    created          <datetime string>

Accepted Version (v3):
    workflow_status  Editable
    draft            False
    latest revision  4

Revision 4:
    <summary fields from order_xml_3_4>
```

The command

```
$ grid po show --org=crgl purchase_order:809832081
```

will display the current revision of the accepted version of the purchase order
with the alternate ID of `purchase_order:809832081` in human-readable format.
It will display output like the following:

```
Purchase Order:
    organization     crgl (Cargill Incorporated)
    uuid             82urioz098aui3871uc
    purchase_order   809832081
    workflow status  Confirmed
    is closed        False
    created          <datetime string>

Accepted Version (v3):
    workflow_status  Editable
    draft            False
    latest revision  4

Revision 4:
    <summary fields from order_xml_3_4>
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
