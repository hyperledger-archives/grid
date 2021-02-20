# Hyperledger Grid

Hyperledger Grid is a platform for building supply chain solutions that include
distributed ledger components. It provides a growing set of tools that
accelerate development for supply chain smart contracts and client interfaces.

This project is not an implementation of a distributed ledger or a client
application. Instead, Hyperledger Grid provides supply-chain-focused libraries,
data models, and software development kits (SDKs) as modular, reusable
components.

The Hyperledger Grid project includes several repositories:

- [This repository](https://github.com/hyperledger/grid) contains core
  components such as supply-chain-centric data types and smart permissioning
  code.

- The [grid-contrib](https://github.com/hyperledger/grid-contrib) repository
  contains example domain models and reference implementations for smart
  contracts (also called "transaction families").

- The [grid-rfcs](https://github.com/hyperledger/grid-rfcs) repository
  contains RFCs (requests for comments) for proposed and approved changes to
  Hyperledger Grid.

- The [grid-docs](https://github.com/hyperledger/grid-docs) repository
  contains the source files for the Grid documentation and website
  hosted at [grid.hyperledger.com](https://grid.hyperledger.org).

## Project Status

Hyperledger Grid is currently in the
[incubation](https://tsc.hyperledger.org/project-lifecycle.html)
stage of the Hyperledger product lifecycle.
The [Hyperledger Grid
proposal](https://docs.google.com/document/d/1b6ES0bKUK30E2iZizy3vjVEhPn7IvsW5buDo7nFXBE0/)
was accepted in December, 2018.


## How to Participate

We welcome contributors, both organizations and individuals, to help shape
project direction, contribute ideas, provide use cases, and work on specific
tools and examples. Please [join the
discussion](https://grid.hyperledger.org/community/join_the_discussion.html).

## Building Grid

Before building, make sure you `libzmq3-dev package` installed in Debian(Ubuntu). 

This can be installed with:

```
	sudo apt install libzmq3-dev -y
```

To build Grid, run `cargo build` from the root directory. This command
builds all of the Grid components, including `gridd` (the grid daemon),
the CLI, and all of the smart contracts in the `contracts` directory.

To build individual components, run `cargo build` in the component directories.
For example, to build only the grid-cli, navigate to `cli`, then run
`cargo build`.

To build Grid using Docker, run `docker-compose build` from the root directory.
This command builds Docker images for all of the Grid components, including
`gridd` (the grid daemon), the CLI, and all of the smart contracts in the
`contracts` directory.

To build individual components using Docker, run
`docker-compose build <component>` from the root directory. For example, to
build only the grid-cli, run `docker-compose build grid-cli`.

To use Docker to build Grid with experimental features enabled, set an
environment variable in your shell before running the build commands. For
example: `export 'CARGO_ARGS=-- --features experimental'`. To go back to
building with default features, unset the environment variable:
`unset CARGO_ARGS`

## More Information

- [Hyperledger Grid website](https://grid.hyperledger.org)
- [Documentation](https://grid.hyperledger.org/docs/)
- [Hyperledger Grid mailing list](https://lists.hyperledger.org/g/grid)
- [#grid discussion channel](https://chat.hyperledger.org/channel/grid)
- [Hyperledger Grid project overview](https://www.hyperledger.org/projects/grid)
  at [hyperledger.org](https://www.hyperledger.org)


## License

Hyperledger Grid software is licensed under the [Apache License Version
2.0](LICENSE) software license.

The Hyperledger Grid documentation in the
[grid-docs](https://github.com/hyperledger/grid-docs) repository is licensed
under a Creative Commons Attribution 4.0 International License (CC BY 4.0).
You may obtain a copy of the license at
<http://creativecommons.org/licenses/by/4.0/>.
