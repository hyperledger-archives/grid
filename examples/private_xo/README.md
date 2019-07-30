# How to Run the Private XO Demo 

Private XO is a demo Splinter network that allows you to play games of tic tac
toe. It is a two-node network with a single circuit and two services. Each node
shares state and verifies the committed transactions using a two-phase commit
protocol.

**Note:** This demo uses the existing XO transaction processor provided in
[Hyperledger Sawtooth](https://github.com/hyperledger/sawtooth-core).

1. To start Private XO, run the following command from the Splinter root
   directory:

     ```
     docker-compose -f examples/private_xo/docker-compose.yaml up
     ```

   This command starts two Splinter nodes, two private XO services
   (`private-xo-service-a` and `private-xo-service-b`), and a shell container
   for interacting with the nodes (`xo-shell`).

1. Run the following command in a separate terminal to connect to the `xo-shell`
   container:

     ```
     docker exec -it xo-shell bash
     ```

1. Once connected to the `xo-shell` container, you can use the `xo` command to
   play tic tac toe. See the 
   [Sawtooth XO CLI documentation](https://sawtooth.hyperledger.org/docs/core/releases/latest/cli/xo.html)
   for details on how to play. 

   **IMPORTANT:** Each `xo` command must identify which service
   to use: either `private-xo-service-a` or `private-xo-service-b`. Specify the
   URL for the service you want. For example: 

     ```
     xo create game-1 --url http://private-xo-service-a:8000
     ```

1. When you are finished, shut down the demo with the following command:

     ```
     docker-compose -f examples/private_xo/docker-compose.yaml down
     ```

