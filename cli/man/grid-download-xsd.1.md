% GRID-DOWNLOAD-XSD(1) Cargill, Incorporated | Grid

<!--
  Copyright 2022 Cargill Incorporated
  Licensed under Creative Commons Attribution 4.0 International License
  https://creativecommons.org/licenses/by/4.0/
-->

NAME
====

**grid-download-xsd** - Downloads and extracts the XSDs necessary for Grid
validation.

SYNOPSIS
========

**grid download-xsd** \[**FLAGS**\] \[**OPTIONS**\] 

DESCRIPTION
===========

This command downloads GS1 XSD files used by various Grid features. The
downloaded artifacts are first copied into a cache directory. They are then
expanded into Grid's state directory. If the desired artifacts are in the
cache directory, Grid will not attempt to re-download them, and instead
prefer the cache contents.

To avoid downloading from the internet (for example, if a firewall rule
would prevent access to the remote website), use the --copy-from and
--no-download arguments.

If --copy-from is used without --no-download, artifacts will be copied from
the directory provided via --copy-from and any missing artifacts will be
downloaded as usual.

IN DEPTH
========

This utility downloads GS1 schemas from the following URL:

https://www.gs1.org/docs/EDI/xml/3.4.1/GS1\_XML\_3-4-1\_Publication.zip

It places the file in a cache directory GRID\_CACHE\_DIR/xsd\_artifact\_cache
after validating the hash against a known good hash. The utility proceeds to
read the zip in the following manner: It finds a zip file within the root zip
beginning with "BMS Packages EDI XML", and then finds a zip file within that
zip beginning with "BMS\_Package\_Order\_". This file's contents are then
extracted to GRID\_STATE\_DIR/xsd/po.

FLAGS
=====

`--no-download`
: Do not download the XSD even if there is no artifact cached

`--force`
: Continue even if a checksum on the cached file is incorrect

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

`--copy-from`
: Replenish the cache from a directory resource and use that. The directory
  should contain the following files:
  /GS1\_XML\_3-4-1\_Publication.zip

EXAMPLES
========

The command

```
$ grid download-xsd \
    --no-download \
    --copy-from ./local-dir
```

will copy from a local directory ./local-dir without attempting to download any
assets. It will still validate the hashes of the assets as they are copied, and
error unless the `--force` option is enabled.

```
validating hash of ./local-dir/GS1_XML_3-4-1_Publication.zip
extracting to schema directory
```

ENVIRONMENT VARIABLES
=====================

**`GRID_CACHE_DIR`**
: Specifies the local path to the directory containing GRID cache.
  The default value is "/var/cache/grid".

**`GRID_STATE_DIR`**
: Specifies the local path to the directory containing GRID state.
  The default value is "/var/lib/grid".

SEE ALSO
========
| Grid documentation: https://grid.hyperledger.org/docs/0.3/
