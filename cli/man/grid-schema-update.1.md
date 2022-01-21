% GRID-SCHEMA-UPDATE(1) Cargill, Incorporated | Grid Commands
<!--
  Copyright 2021 Cargill Incorporated
  Licensed under Creative Commons Attribution 4.0 International License
  https://creativecommons.org/licenses/by/4.0/
-->

NAME
====

**grid-schema-update** — Update schemas from a YAML file.

SYNOPSIS
========

**grid schema update** \[**FLAGS**\] \[**OPTIONS**\] <PATH>

DESCRIPTION
===========

Update existing schemas from a YAML file. This command requires `PATH` argument.

ARGS
====

`PATH`
: Path to yaml file containing a list of schema definitions.

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

`-k`, `--key`
: Base name or path to a private signing key file.

`--service-id`
: The ID of the service the payload should be sent to; required if running on
  Splinter. Format: `<circuit-id>::<service-id>`.

`--url`
: URL for the REST API.

`--wait`
: Maximum number of seconds to wait for the batch to be committed.

EXAMPLES
========

Schemas can be updated by using the `update` command.

Using a YAML file
```
$ grid schema update path/to/schema.yaml
```

Sample YAML file describing a schema.

```
- name: gs1_product
  description: GS1 product schema
  owner: crgl
  properties:
    - name: product_name
      data_type: STRING
      description:
        Consumer friendly short description of the product suitable for compact
        presentation.
      required: true
    - name: image_url
      data_type: STRING
      description: URL link to an image of the product.
      required: false
    - name: brand_name
      data_type: STRING
      description: The brand name of the product that appears on the consumer package.
      required: true
    - name: product_description
      data_type: STRING
      description:
        "An understandable and useable description of a product using brand and
        other descriptors. This attribute is filled with as little abbreviation
        as possible, while keeping to a reasonable length. This should be a
        meaningful description of the product with full spelling to facilitate
        essage processing. Retailers can use this description as the base to
        fully understand the brand, flavour, scent etc. of the specific product,
        in order to accurately update a product description as needed for their
        internal systems. Examples: XYZ Brand Base Invisible Solid Deodorant AP
        Stick Spring Breeze."
      required: true
    - name: gpc
      data_type: NUMBER
      number_exponent: 1
      description:
        8-digit code (GPC Brick Value) specifying a product category according
        to the GS1 Global Product Classification (GPC) standard.
      required: true
    - name: net_content
      data_type: STRING
      description:
        The amount of the consumable product of the trade item contained in a
        package, as declared on the label.
      required: true
    - name: target_market
      data_type: NUMBER
      number_exponent: 1
      description:
        ISO numeric country code representing the target market country for the
        product.
      required: true
```

ENVIRONMENT VARIABLES
=====================

**`CYLINDER_PATH`**
: Colon-separated path used to search for the key which will be used
  to sign transactions.

**`GRID_DAEMON_ENDPOINT`**
: Specifies a default value for `--url`.

**`GRID_DAEMON_KEY`**
: Specifies a default value for  `-k`, `--key`.

**`GRID_SERVICE_ID`**
: Specifies a default value for `--service-id`.

SEE ALSO
========
| `grid schema(1)`
| `grid schema create(1)`
| `grid schema list(1)`
| `grid schema show(1)`
|
| Grid documentation: https://grid.hyperledger.org/docs/0.3/
