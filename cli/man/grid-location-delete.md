% GRID-LOCATION-DELETE(1) Cargill, Incorporated | Grid Commands
<!--
  Copyright 2018-2021 Cargill Incorporated
  Licensed under Creative Commons Attribution 4.0 International License
  https://creativecommons.org/licenses/by/4.0/
-->

NAME
====

**grid-location-delete** — Delete an existing location

SYNOPSIS
========

**grid location delete** \[**FLAGS**\] \[**OPTIONS**\]

DESCRIPTION
===========

Delete an existing location. This command requires the `<location_id>` argument
to specify the unique identifier of the location that is to be deleted. The
`--namespace` option must also be specified otherwise the namespace used will
default to GS1.

FLAGS
=====

`-h`, `--help`
: Prints help information

`-k`, `--key`
: Base name for private key file

`-q`, `--quiet`
: Do not display output

`-V`, `--version`
: Prints version information

`-v`
: Increases verbosity (the opposite of `-q`). Specify multiple times for more
  output

OPTIONS
=======

`--namespace`
: Location name space (defaults to `GS1`)

ARGS
====

`<location_id>`
: Unique identifier for location

EXAMPLES
========

Delete an existing location.

```
$ grid location delete --location_id 762111177704 --namespace GS1
```

ENVIRONMENT VARIABLES
=====================

**`GRID_DAEMON_ENDPOINT`**
: Specifies the endpoint for the grid daemon (`gridd`)
  if `-U` or `--url` is not used.

**`GRID_DAEMON_KEY`**
: Specifies key used to sign transactions if `k` or `--key`
  is not used.

**`GRID_SERVICE_ID`**
: Specifies service ID if `--service-id` is not used

SEE ALSO
========
| `grid-location-create(1)`
| `grid-location-update(1)`
| `grid-location-delete(1)`
| `grid-location-show(1)`
| `grid-location-list(1)`
|
| Grid documentation: https://grid.hyperledger.org/docs/0.1/
