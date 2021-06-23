% GRID-PO-LIST(1) Cargill, Incorporated | Grid
<!--
  Copyright 2021 Cargill Incorporated
  Licensed under Creative Commons Attribution 4.0 International License
  https://creativecommons.org/licenses/by/4.0/
-->

NAME
====

**grid-po-list** - List all purchase orders.

SYNOPSIS
========

**grid po list** \[**FLAGS**\] \[**OPTIONS**\]

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
: Filter on whether the purchase order has an accepted version. Conflicts with
  `--not-accepted`.

`--not-accepted`
: Filter on whether the purchase order does not have an accepted version.
  Conflicts with `--accepted`.

`--closed`
: Selects closed purchase orders only. Conflicts with `--open`.

`--open`
: Selects open purchase orders only. Conflicts with `--closed`.

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
  Splinter. Format <circuit-id>::<service-id>.

  `--url`
: URL for the REST API.

EXAMPLES
========

The command

```
$ grid po list --org=crgl
```

will list all purchase orders for the org `crgl` in human-readable format:

```
ORG   UUID                STATUS    ACCEPTED CLOSED
crgl  82urioz098aui3871uc Confirmed v3       False
```

The command

```
$ grid po list --accepted
```

will list all the purchase orders that have an accepted version in
human-readable format:

```
ORG    UUID                STATUS    ACCEPTED CLOSED
crgl   82urioz098aui3871uc Confirmed v3       False
tst    2389f7987d9s09df98f Issued    v1       False
tst    f808u23hjiof09ufs0d Closed    v1       True
```
The command

```
% grid po list \
    --accepted \
    --open \
    --format=csv
```

will display all of the purchase orders that have an accepted version and are
open.  The output is formatted in csv:

```
ORG,UUID,STATUS,ACCEPTED,CLOSED
crgl,82urioz098aui3871uc,Confirmed,v3,False
tst,2389f7987d9s09df98f,Issued,v1,False
```

ENVIRONMENT VARIABLES
=====================

**`GRID_DAEMON_ENDPOINT`**
: Specifies a default value for `--url`

**`GRID_SERVICE_ID`**
: Specifies a default value for `--service-id`

SEE ALSO
========
| `grid-po-show(1)`
| `grid-po-list(1)`
|
| Grid documentation: https://grid.hyperledger.org/docs/0.2/
