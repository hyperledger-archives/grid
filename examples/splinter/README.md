# Running Hyperledger Grid on Splinter

This document shows how to set up a Grid-on-Splinter environment that runs in a
set of Docker containers.

The example Splinter docker-compose file creates a network with three nodes
(alpha, beta, and gamma) that can be used for demos or application development.
This environment includes the Pike, Product, and Schema smart contracts.

- **Pike** handles organization and identity permissions with Sabre, a smart
  contract engine that is included in the Splinter scabbard service.
- **Product** provides a way to share GS1-compatible product data (items that
  are transacted, traded, or referenced in a supply chain).
- **Schema** provides a reusable, standard approach to defining, storing, and
  consuming the product properties. Property definitions are collected into a
  Schema data type that defines all the possible properties for an item.


## Prerequisites

- Docker Engine
- Docker Compose


## Important Notes

The example `docker-compose.yaml` file uses experimental Splinter features that
have not been thoroughly tested or documented.

Due to the rapid and ongoing development of Splinter and its experimental
features, the images in this example can become stale very quickly. If you have
used this procedure before, run the following commands to ensure that your
images are up to date:

```
$ docker pull hyperledger/grid-dev
$ docker-compose -f examples/splinter/docker-compose.yaml pull generate-key-registry db-alpha scabbard-cli-alpha splinterd-alpha
```

## Set Up and Run Grid

1. Clone the [Hyperledger Grid repository](https://github.com/hyperledger/grid)
   ([https://github.com/hyperledger/grid](https://github.com/hyperledger/grid)).
2. Navigate to the grid root directory and start the Grid Docker containers.

   `$ docker-compose -f examples/splinter/docker-compose.yaml up --build`

   This docker-compose file creates a network with two nodes (alpha and beta)
   that includes the Pike, Schema, and Product smart contracts.


## Create a Circuit

To create a circuit, a user on one node proposes a new circuit that includes one
or more other nodes. When the other nodes accept the circuit proposal, the
circuit is created.

1. Get the gridd public key from the `gridd-alpha` container. You will need this
   key when creating a circuit definition file in step 3.

   `$ docker exec gridd-alpha cat /etc/grid/keys/gridd.pub`

2. Connect to the `splinterd-alpha` container. You will use this container to
   run Splinter commands on alpha-node-000.

   ```
   $ docker-compose -f examples/splinter/docker-compose.yaml exec splinterd-alpha bash
   root@splinterd-alpha:/#
   ```

3. Copy the key and save it in a local file.

   ```
   root@splinterd-alpha:/# echo "<public key>" > gridd.pub
   ```

4. Propose a new circuit with the definition `circuit create` CLI command.

   ```
   root@splinterd-alpha:/# splinter circuit create \
      --key /key_registry_shared/alpha.priv \
      --url http://splinterd-alpha:8085  \
      --node alpha-node-000::tls://splinterd-alpha:8044 \
      --node beta-node-000::tls://splinterd-beta:8044 \
      --service grid-scabbard-a::alpha-node-000 \
      --service grid-scabbard-b::beta-node-000 \
      --service-type *::scabbard \
      --management grid \
      --service-arg *::admin_keys=$(cat gridd.pub) \
      --service-peer-group grid-scabbard-a,grid-scabbard-b
   ```

5. Check the results by displaying the list of proposals. The following example
   sets the CIRCUIT_ID environment variable; this environment variable is for
   the purposes of this procedure and is not used directly by the `splinter`
   CLI commands.

   Set CIRCUIT_ID based on the output of the `proposals` subcommand; for
   example:

   ```
   root@splinterd-alpha:/# splinter circuit proposals --url http://splinterd-alpha:8085
   ID                                      MANAGEMENT MEMBERS
   01234567-0123-0123-0123-012345678901    grid       alpha-node-000;beta-node-000
   ```

   ```
   root@splinterd-alpha:/# export CIRCUIT_ID=01234567-0123-0123-0123-012345678901
   ```

   ```
   root@splinterd-alpha:/# splinter circuit show $CIRCUIT_ID --url http://splinterd-alpha:8085
   Proposal to create: 01234567-0123-0123-0123-012345678901
      Management Type: grid

      alpha-node-000 (tls://splinterd-alpha:8044)
          Vote: ACCEPT (implied as requester):
              <alpha-public-key>
          Service (scabbard): grid-scabbard-a
              admin_keys:
                  <gridd-alpha public key>
              peer_services:
                  grid-scabbard-b

      beta-node-000 (tls://splinterd-beta:8044)
          Vote: PENDING
          Service (scabbard): grid-scabbard-b
              admin_keys:
                  <gridd-alpha public key>
              peer_services:
                  grid-scabbard-a

   ```

6. Connect to the `splinterd-beta` container. You will use this container to run
   Splinter commands on `beta-node-000`.

   ```
   $ docker-compose -f examples/splinter/docker-compose.yaml exec splinterd-beta bash
   root@splinterd-beta:/#
   ```

7. Find the ID of the proposed circuit and save it to an environment variable.
   The ID will be required for voting on the proposals and for interacting with
   the circuit once it is approved. For example:

   ```
   root@splinterd-beta:/# splinter circuit proposals --url http://splinterd-beta:8085
   ID                                      MANAGEMENT MEMBERS
   01234567-0123-0123-0123-012345678901    grid       alpha-node-000;beta-node-000
   ```

   ```
   root@splinterd-beta:/# export CIRCUIT_ID=01234567-0123-0123-0123-012345678901
   ```

8. Use the ID to display the details of the proposed circuit.

   ```
   root@splinterd-beta:/# splinter circuit show $CIRCUIT_ID --url http://splinterd-beta:8085
   Proposal to create: 01234567-0123-0123-0123-012345678901
      Management Type: grid

      alpha-node-000 (tls://splinterd-alpha:8044)
          Vote: ACCEPT (implied as requester):
              <alpha-public-key>
          Service (scabbard): grid-scabbard-a
              admin_keys:
                  <gridd-alpha public key>
              peer_services:
                  grid-scabbard-b

      beta-node-000 (tls://splinterd-beta:8044)
          Vote: PENDING
          Service (scabbard): grid-scabbard-b
              admin_keys:
                  <gridd-alpha public key>
              peer_services:
                  grid-scabbard-a
   ```

9. Then vote to accept the proposal.

   `root@splinterd-beta:/# splinter circuit vote --key /key_registry_shared/beta.priv --url http://splinterd-beta:8085 $CIRCUIT_ID --accept`

10. Run the following command on each node to verify that the new circuit has
    been created. The circuit information should be the same on both nodes.

    ```
    root@splinterd-beta:/# splinter circuit list --url http://splinterd-beta:8085
    ID                                     MANAGEMENT MEMBERS
    01234567-0123-0123-0123-012345678901   grid       alpha-node-000;beta-node-000
    ```

    ```
    root@splinterd-alpha:/# splinter circuit list --url http://splinterd-alpha:8085
    ID                                      MANAGEMENT MEMBERS
    01234567-0123-0123-0123-012345678901    grid       alpha-node-000;beta-node-000
    ```


## Demonstrate Grid Smart Contract Functionality

1. Start a bash session in the `gridd-alpha` Docker container.  You will use
   this container to run Grid commands on `alpha-node-000`.

   ```
   $ docker exec -it gridd-alpha bash
   root@gridd-alpha:/#
   ```

2. Generate a secp256k1 key pair for the alpha node. This key will be used to
   sign Grid transactions.

   `root@gridd-alpha:/# grid keygen alpha-agent`

   This command generates two files, `alpha-agent.priv` and `alpha-agent.pub`,
   in the `~/.grid/keys/` directory.

3. Set an environment variable with the service ID. Use the circuit ID of the
   circuit that was created above. The commands below will check this variable
   to determine which circuit and service the command should be run against. An
   alternative to using the environment variable is to pass the service ID via
   the `--service-id` argument in each of these commands.

   ```
   root@gridd-alpha:/# export `GRID_SERVICE_ID=01234567-0123-0123-0123-012345678901::grid-scabbard-a'
   ```

4. Create a new organization, `myorg`.

   ```
   root@gridd-alpha:/# grid \
   organization create 314156 myorg '123 main street' \
    --metadata gs1_company_prefixes=314156
   ```

   This command creates and submits a transaction to create a new Pike
   organization that is signed by the admin key. It also creates a new Pike
   agent with the “admin” role for the new organization (this agent’s public key
   is derived from the private key used to sign the transaction.) The service ID
   includes the circuit name and the scabbard service name for the alpha node.

5. Update the agent's permissions (Pike roles) to allow creating, updating, and
   deleting Grid products.

   ```
   root@gridd-alpha:/# grid \
   agent update 314156 $(cat ~/.grid/keys/alpha-agent.pub) --active \
   --role can_create_product \
   --role can_update_product \
   --role can_delete_product \
   --role admin
   ```

6. Use `cat` to create a product definition file, `product.yaml`, using the
   following contents.

   ```
   root@gridd-alpha:/# cat > product.yaml
   - product_type: "GS1"
     product_id: "723382885088"
     owner: "314156"
     properties:
       - name: "species"
         data_type: "STRING"
         string_value: "tuna"
       - name: "length"
         data_type: "NUMBER"
         number_value: 22
       - name: "maximum_temperature"
         data_type: "NUMBER"
         number_value: 5
       - name: "minimum_temperature"
         data_type: "NUMBER"
         number_value: 0
   ```

7. Add a new product based on the definition in the example YAML file,
   `product.yaml`.

   ```
   root@gridd-alpha:/# grid \
     product create  product.yaml
   ```

8. Open a new terminal and connect to the `gridd-beta` container.

   `$ docker exec -it gridd-beta bash`

9. Set an environment variable with the service ID.

    ```
    root@gridd-beta:/# export `GRID_SERVICE_ID=01234567-0123-0123-0123-012345678901::grid-scabbard-b'
    ```

10. Display all products.

   ```
   root@gridd-beta:/# grid product list
   ```


## Demonstrate Smart Contract Deployment

The scabbard CLI enables deployment of custom smart contracts to existing
circuits.

1. Start a bash session in the `scabbard-cli-alpha` Docker container. You will
   use this container to send scabbard commands to `splinterd-alpha`.

   ```
   $ docker-compose -f examples/splinter/docker-compose.yaml run scabbard-cli-alpha bash
   root@scabbard-cli-alpha:/#
   ```

2. Set an environment variable to the circuit ID of the circuit that was created
   above.

   ```
   root@scabbard-cli-alpha:/# export CIRCUIT_ID=01234567-0123-0123-0123-012345678901
   ```

3. Download the smart contract.

   `root@scabbard-cli-alpha:/# curl -OLsS https://files.splinter.dev/scar/xo_0.4.2.scar`

4. Create the contract registry for the new smart contract.

   ```
   root@scabbard-cli-alpha:/# scabbard cr create sawtooth_xo \
   --owner $(cat /root/.splinter/keys/gridd.pub) \
   -k gridd \
   -U 'http://splinterd-alpha:8085' \
   --service-id $CIRCUIT_ID::grid-scabbard-a
   ```

5. Upload the smart contract.

   ```
   root@scabbard-cli-alpha:/# scabbard contract upload ./xo_0.4.2.scar \
   -k gridd \
   -U 'http://splinterd-alpha:8085' \
   --service-id $CIRCUIT_ID::grid-scabbard-a
   ```

6. Create the namespace registry for the smart contract.

   ```
   root@scabbard-cli-alpha:/# scabbard ns create 5b7349 \
   --owner $(cat /root/.splinter/keys/gridd.pub) \
   -k gridd \
   -U 'http://splinterd-alpha:8085' \
   --service-id $CIRCUIT_ID::grid-scabbard-a
   ```

7. Grant the appropriate contract namespace permissions.

   ```
   root@scabbard-cli-alpha:/# scabbard perm 5b7349 sawtooth_xo --read --write \
   -k gridd \
   -U 'http://splinterd-alpha:8085' \
   --service-id $CIRCUIT_ID::grid-scabbard-a
   ```


8. Open a new terminal and connect to the `scabbard-cli-beta` container and add
   the circuit ID environment variable
   ```
   $ docker-compose -f examples/splinter/docker-compose.yaml run scabbard-cli-beta bash
   root@scabbard-cli-beta:/#
   ```
   ```
   root@scabbard-cli-beta:/# export CIRCUIT_ID=01234567-0123-0123-0123-012345678901
   ```

9. List all uploaded smart contracts.

   ```
   root@scabbard-cli-beta:/# scabbard contract list -U 'http://splinterd-beta:8085' --service-id $CIRCUIT_ID::grid-scabbard-b
   NAME        VERSIONS OWNERS
   grid_product 1.0      <gridd-alpha public key>
   pike         0.1      <gridd-alpha public key>
   sawtooth_xo  1.0      <gridd-alpha public key>
   ```

10. Display the xo smart contract.

   ```
   root@scabbard-cli-beta:/# scabbard contract show sawtooth_xo:1.0 -U 'http://splinterd-beta:8085' --service-id $CIRCUIT_ID::grid-scabbard-b
   sawtooth_xo 1.0
     inputs:
     - 5b7349
     outputs:
     - 5b7349
     creator: <gridd-alpha public key>
   ```

## Demonstrate Circuit Scope

If a node is not a part of a circuit, that node cannot access information about
that circuit or any transactions that occur on that circuit.

Use the following steps to demonstrate that the third node in the network
(gamma-node-000) cannot see the circuit between alpha and beta, even when it
participates in a new multi-party circuit with those nodes.

1. Connect to the splinterd-gamma Docker container. You will use this container
   to run Splinter commands on gamma-node-000.

   ```
   $ docker-compose -f examples/splinter/docker-compose.yaml exec splinterd-gamma bash
   root@splinterd-gamma:/#
   ```

2. Verify that splinterd-gamma does not see any circuits.
   ```
   root@splinterd-gamma:/# splinter circuit list --url http://splinterd-gamma:8085
   ID MANAGEMENT MEMBERS
   ```

Final note: Splinter strictly enforces privacy for all information on a
circuit, including participants, available smart contracts, and transactions
performed by the participants using those smart contracts.

For example, if gamma creates a circuit with alpha and a separate circuit with
beta, then uploads the XO smart contract and plays a tic-tac-toe game with
alpha, the xo list command on gamma will show only the gamma-alpha game. Even
though alpha and beta are using the same XO smart contract, their game moves
(smart contract transactions) remain private to their two-party circuit.

## For More Information
- Hyperledger Grid documentation: https://grid.hyperledger.org/docs/grid/nightly/master/introduction.html
- Splinter: https://github.com/Cargill/splinter
- Sawtooth Sabre: https://github.com/hyperledger/sawtooth-sabre
- Pike transaction family (defines a Grid Pike smart contract): https://grid.hyperledger.org/docs/grid/nightly/master/transaction_family_specifications/grid_schema_family_specification.html
- Schema transaction family (defines a Grid Schema smart contract): https://grid.hyperledger.org/docs/grid/nightly/master/transaction_family_specifications/grid_schema_family_specification.html
- Product RFC: https://github.com/target/grid-rfcs/blob/d6305b86e2a43e510bb57b297b3ec09b0a66c5b0/0000-product.md
- CLI for the XO smart contract (also called a "transaction processor"): https://sawtooth.hyperledger.org/docs/core/releases/latest/cli/xo.html
