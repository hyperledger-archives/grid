% SPLINTER-CERT-GENERATE(1) Cargill, Incorporated | Splinter Commands

NAME
====

**splinter-cert-generate** — Generates test certificates and keys for running
  splinterd with TLS (in insecure mode)

SYNOPSIS
========
| **splinter cert generate** \[**FLAGS**\] \[**OPTIONS**\]

DESCRIPTION
===========
Running Splinter in TLS mode requires valid X.509 certificates from a
certificate authority. When developing against Splinter, you can use this
command to generate development certificates and the associated keys for your
development environment.

The files are generated in the location specified by `--cert-dir`, the
SPLINTER_CERT_DIR environment variable, or in the default location
/etc/splinter/certs/.

The following files are created:

  client.crt, client.key, server.crt, server.key, generated_ca.pem,
  generated_ca.key

FLAGS
=====
`--force`
: Overwrites files if they exist. If this flag is not provided and the file
  exists, an error is returned.

`-h`, `--help`
: Prints help information

`-q`, `--quiet`
: Decrease verbosity (the opposite of -v). When specified, only errors or
  warnings will be output.

`--skip`
: Checks if the files exists and generates the files that are missing. If this
flag is not provided and the file exists, an error is returned.

`-V`, `--version`
: Prints version information

`-v`
: Increases verbosity (the opposite of -q). Specify multiple times for more
  output.

OPTIONS
=======
`-d`, `--cert-dir <cert_dir>`
: Path to the directory certificates are created in. Defaults to
  /etc/splinter/certs/. This location can also be changed with the
  SPLINTER_CERT_DIR environment variable. This directory must exist.

`--common-name <common_name>`
: String that specifies a common name for the generated certificate (defaults to
  localhost). Use this option if the splinterd URL uses a DNS address instead
  of a numerical IP address.

EXAMPLES
========
Generates test certificates and keys:

  `$ splinter cert generate`

To create missing certificates and keys when some files already exist, add the
`--skip` flag. The command will ignore the existing files and create any files
that are missing.

  `$ splinter cert generate --skip`

To recreate the certificates and keys from scratch, use the  `--force` flag to
overwrite all existing files.

  `$ splinter cert generate --force`

ENVIRONMENT
===========
The following environment variables affect the execution of splinter cert
generate:

**SPLINTER_CERT_DIR**

: The certificates and keys will be generated at the location specified by the
  environment variable.  (See `--cert-dir`)

SEE ALSO
========
For more information, see the Splinter documentation at
https://github.com/Cargill/splinter-docs/blob/master/docs/index.md
