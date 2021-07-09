% GRID-ORGANIZATION-LIST(1) Cargill, Incorporated | Grid Commands

<!--
  Copyright 2021 Cargill Incorporated
  Licensed under Creative Commons Attribution 4.0 International License
  https://creativecommons.org/licenses/by/4.0/
-->

NAME
====

**grid-organization-list** — List all organizations

SYNOPSIS
========

**grid organization list** \[**FLAGS**\]

DESCRIPTION
===========

List all organizations in grid. If the `service_id` flag is specified, only
organizations corresponding to that `service_id` will be shown.

FLAGS
=====

`-F`, `--format`
: Specifies the output format of the list. Possible values for formatting are `human` and `csv`. Defaults to `human`.

`-h`, `--help`
: Prints help information

`--alternate-ids`
: Displays the Alternate IDs of the organizations being listed.

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

The following command will list all organizations:

```
$ grid organization list
ORG_ID NAME    LOCATIONS
crgl   Cargill 0123456789012
```

The next command will list all organizations, including the organizations'
Alternate IDs using the `--alternated-ids` flag:

```
$ grid organization list --alternate-ids
ORG_ID NAME    LOCATIONS     ALTERNATE_IDS
crgl   Cargill 0123456789012 crgl:001
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
| `grid agent(1)`
| `grid role(1)`
|
| Grid documentation: https://grid.hyperledger.org/docs/0.1/
