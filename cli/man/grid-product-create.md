% GRID-PRODUCT-CREATE(1) Cargill, Incorporated | Grid Commands
<!--
  Copyright 2018-2020 Cargill Incorporated
  Licensed under Creative Commons Attribution 4.0 International License
  https://creativecommons.org/licenses/by/4.0/
-->

NAME
====

**grid-product-create** — Create products from a YAML file

SYNOPSIS
========

**grid product create** \[**FLAGS**\] \[**OPTIONS**\] <**product_id**>

DESCRIPTION
===========

Create new products from a YAML file. This command requires the `--path` option to
specify a path to a YAML file containing the list of products.

FLAGS
=====

`-h`, `--help`
: Prints help information

`-k`, `--key`
: Base name for private key file

`-q`, `--quiet`
: Do not display output

`--service-id`
: The ID of the service the payload should be sent to; required if running on
  Splinter. Format <circuit-id>::<service-id>

`-V`, `--version`
: Prints version information

`-v`
: Increases verbosity (the opposite of `-q`). Specify multiple times for more
  output

`--url`
: URL for the REST API

`--wait`
: How long to wait for transaction to be committed

OPTIONS
=======

`--owner`
: Organization ID of the owner

`--namespace`
: Namespace of the product (default: "GS1")

`--property`
: A product property (format: key=value)

`--file, -f`
: Path or URL to a YAML file containing a list of products

ARGS
====

`<product_id>`
: Unique identifier of the product

EXAMPLES
========

Products can be created by using the `create` command

Using a YAML file
```
$ grid product create \
    --file products.yaml
```

Using command-line arguments
```
$ grid product create 762111177704 \
    --owner cgl
    --property width=10
    --property length=10
    --property depth=10
```

Sample YAML file describing a list of products.

```
- product_namespace: "GS1"
  product_id: "762111177704"
  owner: "314156"
  properties:
    - name: "length"
      data_type: "NUMBER"
      number_value: 8
    - name: "width"
      data_type: "NUMBER"
      number_value: 11
    - name: "height"
      data_type: "NUMBER"
      number_value: 1
- product_namespace: "GS1"
  product_id: "881334009880"
  owner: "314156"
  properties:
    - name: "length"
      data_type: "NUMBER"
      number_value: 8
    - name: "width"
      data_type: "NUMBER"
      number_value: 11
    - name: "height"
      data_type: "NUMBER"
      number_value: 11
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
| `grid-product-create(1)`
| `grid-product-update(1)`
| `grid-product-delete(1)`
| `grid-product-show(1)`
| `grid-product-list(1)`
|
| Grid documentation: https://grid.hyperledger.org/docs/0.1/
