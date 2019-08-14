# How to Run the Gameroom Demo

Gameroom is a demo Splinter application that allows you to set up a dynamic
two-party circuit (called a "gameroom") and play tic tac toe with shared state,
as managed by two-phase-commit consensus. This example application sets up
Splinter nodes for two imaginary organizations: Acme Corporation and Bubba
Bakery.

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

1. In a browser, navigate to the web application UI for each organization:

    - Acme UI: <http://localhost:8080>

    - Bubba Bakery UI: <http://localhost:8081>

1. When you are finished, shut down the demo with the following command:

     ```
     docker-compose -f examples/gameroom/docker-compose.yaml down
     ```

