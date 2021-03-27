% GRID-PO-VERSION-LIST(1) Cargill, Incorporated | Grid
<!--
  Copyright 2021 Cargill Incorporated
  Licensed under Creative Commons Attribution 4.0 International License
  https://creativecommons.org/licenses/by/4.0/
-->

NAME
====

**grid-po-version-list** - List all purchase orders versions.

SYNOPSIS
========

**grid po version list** \[**FLAGS**\] \[**OPTIONS**\]

DESCRIPTION
===========

List all purchase orders in grid.


FLAGS
=====

`-h`, `--help`
: Prints help information.

`-q`, `--quiet`
: Do not display output.

`-V`, `--version`
: Prints version information.

`--accepted`
: Filter on whether the purchase order version is an accepted version.
  Conflicts with `--not-accepted`.

`--not-accepted`
: Filter on whether the purchase order version is not an accepted version.
  Conflicts with `--accepted`.

`--draft`
: Filter on whether the purchase order version is a draft. Conflicts with
  `--not-draft`.

`--not-draft`
: Filter on whether the purchase order version is not a draft. Conflicts with
  `--draft`.

`--closed`
: Selects closed purchase orders only. Conflicts with `--open`.

`--open`
: Selects open purchase orders only. Conflicts with `--closed`.

`-v`
: Increases verbosity (the opposite of `-q`). Specify multiple times for more
  output.

OPTIONS
=======

`--url`
: URL for the REST API.

`--service-id`
: The ID of the service the payload should be sent to; required if running on
  Splinter. Format <circuit-id>::<service-id>.

`--org`
: Optionally, filter the purchase orders for the organization specified by
  `ORG_ID`.  This does *not* use the `GRID_ORG_ID` environment variable.

`-F`, `--format=FORMAT`
: Specifies the output format of the list. Possible values for formatting are
  `human`, `csv`, `yaml`, and `json`. Defaults to `human`.

EXAMPLES
========

The command

```
$ grid po version list --org=crgl
```

will list all purchase orders version for the org `crgl` in human-readable
format:

```
ORG  PO                  ID STATUS   REVISON
crgl 82urioz098aui3871uc v3 Editable       2
```

ENVIRONMENT VARIABLES
=====================

**`GRID_DAEMON_ENDPOINT`**
: Specifies the endpoint for the grid daemon (`gridd`)
  if `-U` or `--url` is not used.

**`GRID_SERVICE_ID`**
: Specifies service ID if `--service-id` is not used

SEE ALSO
========
| `grid-po(1)`
| `grid-po-version-create(1)`
| `grid-po-version-list(1)`
| `grid-po-version-update(1)`
|
| Grid documentation: https://grid.hyperledger.org/docs/0.2/
