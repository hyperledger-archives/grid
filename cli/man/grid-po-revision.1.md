% GRID-PO-REVISION(1) Cargill, Incorporated | Grid Commands
<!--
  Copyright 2021 Cargill Incorporated
  Licensed under Creative Commons Attribution 4.0 International License
  https://creativecommons.org/licenses/by/4.0/
-->

NAME
====

**grid-po-revision** — List or show Grid Purchase Order revisions based on a version.

SYNOPSIS
========

**grid po revision** \[**FLAGS**\] \[**OPTIONS**\] SUBCOMMAND

DESCRIPTION
===========

Lists or shows Grid Purchase Order revisions for a specified Purchase Order
and version.

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

`--url`
: URL for the REST API.

`--service-id`
: The ID of the service the payload should be sent to; required if running on
  Splinter. Format `<circuit-id>::<service-id>`.

ENVIRONMENT VARIABLES
=====================

**`GRID_DAEMON_ENDPOINT`**
: Specifies a default value for `--url`.

**`GRID_SERVICE_ID`**
: Specifies a default value for `--service-id`.

SUBCOMMANDS
===========

`list`
: List details of all purchase orders revisions for a specified version.

`show`
: Display details of a purchase order revision.

SEE ALSO
========
| `grid-po(1)`
| `grid-po-version(1)`
| `grid-po-revision-list(1)`
| `grid-po-revision-show(1)`
|
| Grid documentation: https://grid.hyperledger.org/docs/0.1/
