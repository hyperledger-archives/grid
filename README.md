# Introduction to Splinter

Splinter is a blockchain-inspired networking platform for distributed
communications between organizations. Using Splinter, it is possible to combine
blockchain-related technologies such as smart contracts, consensus, and circuits
to build a wide variety of architectural patterns.

Splinter allows the same network to do two-party private communication,
multi-party private communication, and network-wide multi-party shared state,
all managed via consensus. A Splinter network enables multi-party or two-party
private conversations using circuits and services.

- A **circuit** is a virtual network within the Splinter network which safely
  and securely enforces privacy boundaries.

- A **service** is an endpoint within a circuit that sends and receives private
  messages.

- A **Splinter application** is a set of distributed services that can
  communicate with each other across a Splinter circuit.

## How to Build Splinter

Build Splinter by running `cargo build` from the root directory. This command
builds all of the Splinter components, including `libsplinter` (the main
library), `splinterd` (the splinter daemon), the CLI, the client, and all
examples in the `examples` directory.

To build individual components, run `cargo build` in the component directories.
For example, to build only the Private XO demo, navigate to
`examples/private_xo`, then run `cargo build`.

## How to Run Example Demos

Splinter includes example applications that you can run as demos.

- Private XO demo: Two services talk over a circuit to play a private game of
  tic tac toe. See the [Private XO README](examples/private_xo/README.md).

- Gameroom demo: Web application that allows you to set up a dynamic
  two-party circuit (called a "gameroom") and play tic tac toe on a distributed
  ledger. See the [Gameroom README](examples/gameroom/README.md).

