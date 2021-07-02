% GRID-ORGANIZATION-SHOW(1) Cargill, Incorporated | Grid Commands

<!--
  Copyright 2021 Cargill Incorporated
  Licensed under Creative Commons Attribution 4.0 International License
  https://creativecommons.org/licenses/by/4.0/
-->

NAME
====

**grid-organization-show** — Show the details of the specified organization.

SYNOPSIS
========

**grid organization show** \[**FLAGS**\] \[**OPTIONS**\] <org_id>

DESCRIPTION
===========

Show the details of the organization specified. This command requires the
`ORG_ID` argument to specify the unique identifier for the organization being
retrieved.

ARGS
====

`ORG_ID`
: A unique identifier for an organization


FLAGS
=====

`-F`, `--format`
: Specifies the output format of the list. Possible values for formatting are `human` and `csv`. Defaults to `human`.

`-h`, `--help`
: Prints help information

`-q`, `--quiet`
: Do not display output

`-V`, `--version`
: Prints version information

`-v`
: Increases verbosity (the opposite of `-q`). Specify multiple times for more output

OPTIONS
=======

`--service-id`
: The ID of the service the payload should be sent to; required if running on
  Splinter. Format <circuit-id>::<service-id>.

`--url`
: URL for the REST API

EXAMPLES
========

The following command will show an organization with an `org_id` of `crgl`:

```
$ grid organization show crgl
Organization ID: crgl
Name: Cargill
Locations: 0123456789012
Alternate IDs:
    crgl:001
Metadata: -
```

If Grid is running on Splinter, the organization will be shown with its
associated `service_id`. The following command will show an organization with an
`org_id` of `crgl` and a `service_id` of `01234-ABCDE`.

```
$ grid organization show crgl
Organization ID: crgl
Name: Cargill
Service ID: 01234-ABCDE::gsAA
Locations: 0123456789012
Alternate IDs:
    crgl:001
Metadata: -
```

ENVIRONMENT VARIABLES
=====================

**`GRID_DAEMON_ENDPOINT`**
: Specifies a default value for `--url`

**`GRID_SERVICE_ID`**
: Specifies a default value for `--service-id`

SEE ALSO
========

| `grid organization(1)`
| `grid organization create(1)`
| `grid organization update(1)`
| `grid organization list(1)`
| `grid agent(1)`
| `grid role(1)`
|
| Grid documentation: https://grid.hyperledger.org/docs/0.1/
