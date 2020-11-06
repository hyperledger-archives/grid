% GRID-LOCATION-CREATE(1) Cargill, Incorporated | Grid Commands
<!--
  Copyright 2018-2020 Cargill Incorporated
  Licensed under Creative Commons Attribution 4.0 International License
  https://creativecommons.org/licenses/by/4.0/
-->

NAME
====

**grid-location-create** — Create a new location

SYNOPSIS
========

**grid location create** \[**FLAGS**\] \[**OPTIONS**\]

DESCRIPTION
===========

Create a new location. This command requires the `<location_id>` argument to
specify a unique identifier for the location, the `--owner` option to
specify the `org_id` of the Pike organization that owns the location, and a
list of `properties` that describe the product. The `--namespace` option must
also be specified otherwise the namespace used will default to GS1.

Alternatively the `--file` option my be used with a YAML file describing
multiple locations to create one or more locations at once.

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

`--namespace`
: Location name space (defaults to `GS1`)

`--owner`
: `org_id` of the Pike organization that owns the location

`--property`
: Key value pair describing a property of the location (example: locationName=Foo)

`-f`, `--file`
: Path to YAML file containing one or more location definitions. If this option is
  used, `location_id`, `namespace`, and `property` cannot be specified.

ARGS
====

`<location_id>`
: Unique identifier for location

EXAMPLES
========

A location can be created by using the `--property` option

```
$ grid location create \
    --location_id 762111177704 \
    --namespace GS1 \
    --owner cgl \
    --property locationName="Grandma's basement" \
    --property locationDescription="My grandma's basement" \
    --property locationType=3 \
    --property addressLine1="612 Worf ave" \
    --property city="St. Paul" \
    --property stateOrRegion="MN" \
    --property postalCode="55117" \
    --property country="United States" \
    --property latLong="46729553,-94685898" \
    --property contactName="Lorraine"
    --property contactEmail="lorraine@fake-email.bike"
    --property contactPhone="612-555-1234"
    --property contactDate="01/15/2020"
```

Alternatively, the `--file` option can be used to create one or more locations.

Sample YAML file describing a location.

```
- namespace: GS1
  location_id: "762111177704"
  owner: cgl
  properties:
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

YAML files can be used to describe locations using the `--file` argument

```
$ grid location create --file locations.yaml
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
