% GRID-PO-REVISION-SHOW(1) Cargill, Incorporated | Grid
<!--
  Copyright 2021 Cargill Incorporated
  Licensed under Creative Commons Attribution 4.0 International License
  https://creativecommons.org/licenses/by/4.0/
-->

NAME
====

**grid-po-revision-show** - Show the details of an existing Grid purchase order revision for a specific version.

SYNOPSIS
========

**grid po revision show** \[**FLAGS**\] \[**OPTIONS**\] <PURCHASE_ORDER_ID> <VERSION_ID> <REVISION_ID>

DESCRIPTION
===========

Show the details of a specific purchase order revision.  This command displays the
specified revision of a specified version of a purchase order.

ARGS
====

`PURCHASE_ORDER_ID`
: Either a UID or an alternate ID of a purchase order.

`VERSION_ID`
: The purchase order version identifier.

`REVISION_ID`
: The purchase order revision identifier.

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
  Splinter. Format `<circuit-id>::<service-id>`.

`--url`
: URL for the REST API.

EXAMPLES
========

The command
```
$ grid po revision show PO-1234-56789 v3 3
```

will display revision number `3` for the `v3` version of the purchase order
with UID `PO-1234-56789` in human-readable format. It will display output like
the following:
```
Revision 3:
    submitter        <public key of submitter>
    created at       <datetime string>
    <order_xml_3_4>
```

In contrast, a summary view is available using the command
```
$ grid po show PO-1234-56789 --version v3 --revision 3
```
with details provided in `grid-po-show(1)`.

ENVIRONMENT VARIABLES
=====================

**`GRID_DAEMON_ENDPOINT`**
: Specifies a default value for `--url`.

**`GRID_SERVICE_ID`**
: Specifies a default value for `--service-id`.

SEE ALSO
========
| `grid-po-revision(1)`
| `grid-po-revision-list(1)`
| `grid-po-version(1)`
|
| Grid documentation: https://grid.hyperledger.org/docs/0.3/
