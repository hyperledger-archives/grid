% GRID-LOCATION-SHOW(1) Cargill, Incorporated | Grid Commands
<!--
  Copyright 2018-2021 Cargill Incorporated
  Licensed under Creative Commons Attribution 4.0 International License
  https://creativecommons.org/licenses/by/4.0/
-->

NAME
====

**grid-location-show** — Show the details of a specific location

SYNOPSIS
========

**grid location show** \[**FLAGS**\] \[**OPTIONS**\]

DESCRIPTION
===========

Show the complete details of a specific location. This command requires the
`<location_id>` argument to specify the unique identifier for the location that
is to be retrieved.

FLAGS
=====

`-h`, `--help`
: Prints help information

`-k`, `--key`
: Base name for private key file

`-q`, `--quiet`
: Do not display output

`--service-id`
: The ID of the service the payload should be sent to; required if running on
  Splinter. Format <circuit-id>::<service-id>

`-V`, `--version`
: Prints version information

`-v`
: Increases verbosity (the opposite of `-q`). Specify multiple times for more
  output

`--url`
: URL for the REST API

`--wait`
: How long to wait for transaction to be committed

ARGS
====

`--location_id`
: A unique identifier for location

EXAMPLES
========

The command

```
$ grid location show --location_id 762111177704
```

Will show the details of the specified location

```
Location ID: 762111177704
Namespace: GS1
Owner: cgl
Properties:
    locationName: Grandma's basement
    locationDescription: My grandma's basement
    locationType: 3
    addressLine1: "612 Worf ave"
    city: St. Paul
    stateOrRegion: MN
    postalCode: "55117"
    country: United States
    latLong: "46729553,-94685898"
    contactName: Lorraine
    contactEmail: lorraine@fake-email.bike
    contactPhone: 612-555-1234
    contactDate: 01/15/2020
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
| `grid-location-create(1)`
| `grid-location-update(1)`
| `grid-location-delete(1)`
| `grid-location-show(1)`
| `grid-location-list(1)`
|
| Grid documentation: https://grid.hyperledger.org/docs/0.1/
