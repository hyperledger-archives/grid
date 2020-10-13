% GRIDD(1) Cargill, Incorporated | Grid Commands
<!--
  Copyright 2018-2020 Cargill Incorporated
  Licensed under Creative Commons Attribution 4.0 International License
  https://creativecommons.org/licenses/by/4.0/
-->

NAME
====

**gridd** — Manage the Grid daemon

SYNOPSIS
========

**gridd** \[**FLAGS**\] \[**OPTIONS**\]

DESCRIPTION
===========

This command starts the Grid Daemon, `gridd`. This daemon provides core Grid
functionality such as the REST API.

**Backends**

Grid supports connections to either Sawtooth or Splinter. By default, `gridd`
will connect to a Sawtooth backend. The connection URL can be configured using
the `-C, --connect` command-line option. If a valid Splinter connection
endpoint is provided, `gridd` will connect to a Splinter backend.

**Directory Locations**

This command includes an option to change the default Grid admin key directory.
For more information, see `--admin-key-dir`, and "GRID DIRECTORY PATHS", below.

FLAGS
=====

`-h`, `--help`
: Prints help information

`-V`, `--version`
: Prints version information

`-v`
: Increases verbosity. Specify multiple times for more
  output

OPTIONS
=======

`--admin-key-dir`
: Directory containing the Scabbard admin key files. (Default: `/etc/grid/keys`)

`-b`, `--bind`
: Connection endpoint for the REST API. (Default: `127.0.0.1:8080`)

`-C`, `--connect`
: The connection endpoint for Sawtooth or Splinter. (Default:
`tcp://127.0.0.1:4004`, a Sawtooth connection)

`--database-url`
: Specifies the database URL to connect to. (Default: `postgres://grid:grid_example@localhost/grid`)

GRID DIRECTORY PATHS
====================

The Grid admin key directory has the following default location:

* Scabbard admin key directory: `/etc/grid/keys`

EXAMPLES
========
```
$ gridd
```

In this example, a different database connection is specified along with a
different directory for the Scabbard admin keys.

```
$ gridd --database-url postgres://acme:acme_corp@localhost/acme --admin-key-dir /etc/acme/keys
```

SEE ALSO
========
| Grid documentation: https://grid.hyperledger.org/docs/0.1/
