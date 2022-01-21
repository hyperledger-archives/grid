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

**grid location delete** \[**FLAGS**\] \[**OPTIONS**\] <LOCATION_ID>

DESCRIPTION
===========

Delete an existing location. This command requires the `LOCATION_ID` argument
to specify the unique identifier of the location that is to be deleted. The
`--namespace` option must also be specified otherwise the namespace used will
default to GS1.

ARGS
====

`LOCATION_ID`
: Unique identifier for location

FLAGS
=====

`-h`, `--help`
: Prints help information


`-q`, `--quiet`
: Do not display output

`-V`, `--version`
: Prints version information

`-v`
: Increases verbosity (the opposite of `-q`). Specify multiple times for more
  output

OPTIONS
=======

`-k`, `--key`
: Base name or path to a private signing key file

`--namespace`
: Location name space (defaults to `GS1`)

`--service-id`
: The ID of the service the payload should be sent to; required if running on
  Splinter. Format: `<circuit-id>::<service-id>`.

`--url`
: URL for the REST API

`--wait`
: Maximum number of seconds to wait for the batch to be committed.

EXAMPLES
========

Delete an existing location.

```
$ grid location delete --location_id 762111177704 --namespace GS1
```

ENVIRONMENT VARIABLES
=====================

**`CYLINDER_PATH`**
: Colon-separated path used to search for the key which will be used
  to sign transactions

**`GRID_DAEMON_ENDPOINT`**
: Specifies a default value for `--url`

**`GRID_DAEMON_KEY`**
: Specifies a default value for  `-k`, `--key`

**`GRID_SERVICE_ID`**
: Specifies a default value for `--service-id`

SEE ALSO
========
| `grid-location-create(1)`
| `grid-location-update(1)`
| `grid-location-delete(1)`
| `grid-location-show(1)`
| `grid-location-list(1)`
|
| Grid documentation: https://grid.hyperledger.org/docs/0.3/
