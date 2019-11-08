# Gameroom

Gameroom is an example Splinter application that allows you to set up private,
multi-party circuits (called "gamerooms") and play tic tac toe with shared
state, as managed by two-phase-commit consensus between the participants. This
example application, as configured, sets up Splinter nodes for two imaginary
organizations: Acme Corporation and Bubba Bakery.

To learn about the Splinter functionality that powers this deceptively simple
application, see the [Gameroom Technical
Walkthrough](https://files.splinter.dev/docs/Gameroom_Walkthrough-Splinter_v0.3.4.pdf)
(PDF).

## How to Run the Gameroom Demo

**Note:** This demo uses the Sabre smart contract engine provided in
[Sawtooth Sabre](https://github.com/hyperledger/sawtooth-sabre) and the XO smart
contract provided in the [Hyperledger Sawtooth Rust
SDK](https://github.com/hyperledger/sawtooth-sdk-rust/tree/master/examples/xo_rust).

**Prerequisites**:
This demo requires [Docker Engine](https://docs.docker.com/engine)
and [Docker Compose](https://docs.docker.com/compose).

1. Clone the [splinter repository](https://github.com/Cargill/splinter).

1. To start Gameroom, run the following command from the Splinter root
   directory:

     ```
     docker-compose -f examples/gameroom/docker-compose.yaml up
     ```
1. To extract private keys to use in the web application, run bash using the
   `generate-key-registry` image and read the private key.  For example, to get
   Alice's private key:

    ```
    $ docker-compose -f examples/gameroom/docker-compose.yaml run generate-key-registry bash
    root@<container-id>:/# cat /key_registry/alice.priv; echo ""
    <the private key value>
    root@<container-id>:/#
    ```

    The keys available are `alice` and `bob`.

1. In a browser, navigate to the web application UI for each organization:

    - Acme UI: <http://localhost:8080>

    - Bubba Bakery UI: <http://localhost:8081>

1. When you are finished, shut down the demo with the following command:

     ```
     docker-compose -f examples/gameroom/docker-compose.yaml down
     ```

