% GRID-PO-REVISION-LIST(1) Cargill, Incorporated | Grid
<!--
  Copyright 2021 Cargill Incorporated
  Licensed under Creative Commons Attribution 4.0 International License
  https://creativecommons.org/licenses/by/4.0/
-->

NAME
====

**grid-po-revision-list** - List all purchase orders revisions for a specified version.

SYNOPSIS
========

**grid po revision list** \[**FLAGS**\] \[**OPTIONS**\] <PURCHASE_ORDER_ID> <VERSION_ID>

DESCRIPTION
===========

List all purchase order revisions for a specified version in grid.

ARGS
====

`PURCHASE_ORDER_ID`
: Either a UID or an alternate ID of a purchase order.

`VERSION_ID`
: The purchase order's version identifier.

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
: Optionally, filter the purchase orders for the organization specified by
  `ORG_ID`.

`--service-id`
: The ID of the service the payload should be sent to; required if running on
  Splinter. Format `<circuit-id>::<service-id>`.

  `--url`
: URL for the REST API.

EXAMPLES
========

The command
```
$ grid po revision list PO-1234-56789 v3
```

will display all purchase order revisions for version `3` of
purchase order with the unique ID of `PO-1234-56789` in human
readable format. It will display in the following format:
```
Revision 4:
    submitter        <public key of submitter>
    created at       <datetime string>
    <summary fields from order_xml_3_4>
Revision 3:
    submitter        <public key of submitter>
    created at       <datetime string>
    <summary fields from order_xml_3_4>
Revision 2:
    submitter        <public key of submitter>
    created at       <datetime string>
    <summary fields from order_xml_3_4>
Revision 1:
    submitter        <public key of submitter>
    created at       <datetime string>
    <summary fields from order_xml_3_4>
```

ENVIRONMENT VARIABLES
=====================

**`GRID_DAEMON_ENDPOINT`**
: Specifies a default value for `--url`.

**`GRID_SERVICE_ID`**
: Specifies a default value for `--service-id`.

SEE ALSO
========
| `grid-po-revision(1)`
| `grid-po-version(1)`
| `grid-po-version-list(1)`
|
| Grid documentation: https://grid.hyperledger.org/docs/0.2/
