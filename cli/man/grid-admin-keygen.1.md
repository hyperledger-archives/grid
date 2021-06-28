% GRID-ADMIN-KEYGEN(1) Cargill, Incorporated | Grid

<!--
  Copyright 2021 Cargill Incorporated
  Licensed under Creative Commons Attribution 4.0 International License
  https://creativecommons.org/licenses/by/4.0/
-->

NAME
====

**grid-admin-keygen** - Generates keys for gridd to use to sign transactions and batches.

SYNOPSIS
========

**grid admin keygen** \[**FLAGS**\] \[**OPTIONS**\]

FLAGS
=====

`--force`
: Overwrite files if they exist. Conflicts with `--skip`.

`-h`, `--help`
: Prints help information.

`-q`, `--quiet`
: Do not display output.

`--skip`
: Check if files exist; generate if missing. Conflicts with `--force`.

`-V`, `--version`
: Prints version information.

`-v`
: Log verbosely.

OPTIONS
=======

`-d`, `--directory`
: Specify the directory for the key files; 
  defaults to /etc/grid/keys.

SEE ALSO
========
| `grid admin(1)`
|
| Grid documentation: https://grid.hyperledger.org/docs/0.2/
