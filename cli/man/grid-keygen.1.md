% GRID-KEYGEN(1) Cargill, Incorporated | Grid

<!--
  Copyright 2021 Cargill Incorporated
  Licensed under Creative Commons Attribution 4.0 International License
  https://creativecommons.org/licenses/by/4.0/
-->

NAME
====

**grid-keygen** - Generates keys with which the user can sign transactions and batches.

SYNOPSIS
========

**grid keygen** \[**FLAGS**\] \[**OPTIONS**\]

FLAGS
=====

`--force`
: Overwrite files if they exist.

`-h`, `--help`
: Prints help information.

`-q`, `--quiet`
: Do not display output.

`-V`, `--version`
: Prints version information.

`-v`
: Log verbosely.

OPTIONS
=======

`-d`, `--key-dir`
: Specify the directory for the key files; 
  defaults to $HOME/.grid/keys. Conflicts with `--system`.

`--system`
: Generate system keys in /etc/grid/keys.

SEE ALSO
========
| `grid admin(1)`
|
| Grid documentation: https://grid.hyperledger.org/docs/0.3/
