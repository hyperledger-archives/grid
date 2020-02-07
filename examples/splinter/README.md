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
$ docker-compose -f examples/splinter/docker-compose.yaml pull generate-key-registry db-alpha splinterd-alpha
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

3. Use cat to create a circuit definition file, `circuit.yaml`, using the
   following contents.

   **Note**: In the lines below, replace `<gridd-alpha public key>` with the key
   from step 1.

   ```
   root@splinterd-alpha:/# cat > circuit.yaml
   circuit_id: my-grid-circuit
   roster:
     - service_id: grid-scabbard-a
       service_type: scabbard
       allowed_nodes:
         - alpha-node-000
       arguments:
         - ["admin_keys", "[\"<gridd-alpha public key>\"]"]
         - ["peer_services", "[\"grid-scabbard-b\"]"]    
     - service_id: grid-scabbard-b
       service_type: scabbard
       allowed_nodes:
        - beta-node-000
       arguments:
         - ["admin_keys", "[\"<gridd-alpha public key>\"]"]
         - ["peer_services", "[\"grid-scabbard-a\"]"]
   members:
     - node_id: alpha-node-000
       endpoint: tls://splinterd-alpha:8044
     - node_id: beta-node-000
       endpoint: tls://splinterd-beta:8044
   authorization_type: Trust
   durability: NoDurability
   circuit_management_type: grid
   ```

   This YAML file defines a circuit between two nodes, `alpha-node-000` and
   `beta-node-000`. Each node runs scabbard, the Splinter service that will
   execute Sabre smart contracts.

4. Propose a new circuit with the definition in `circuit.yaml`.

   `root@splinterd-alpha:/# splinter circuit create --key /key_registry_shared/alpha.priv --url http://splinterd-alpha:8085 circuit.yaml`

5. Check the results by displaying the list of proposals. Then use the circuit
   ID to view the details of the new proposal.

   ```
   root@splinterd-alpha:/# splinter circuit proposals --url http://splinterd-alpha:8085
   ID              MANAGEMENT MEMBERS
   my-grid-circuit grid       alpha-node-000;beta-node-000
   ```

   ```
   root@splinterd-alpha:/# splinter circuit show my-grid-circuit --url http://splinterd-alpha:8085
   Proposal to create: my-grid-circuit
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

7. Find the ID of the proposed circuit.

   ```
   root@splinterd-beta:/# splinter circuit proposals --url http://splinterd-beta:8085
   ID              MANAGEMENT MEMBERS
   my-grid-circuit grid       alpha-node-000;beta-node-000
   ```

8. Use the ID to display the details of the proposed circuit.

   ```
   root@splinterd-beta:/# splinter circuit show my-grid-circuit --url http://splinterd-beta:8085
   Proposal to create: my-grid-circuit
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

   `root@splinterd-beta:/# splinter circuit vote --key /key_registry_shared/beta.priv --url http://splinterd-beta:8085 my-grid-circuit --accept`

10. Run the following command on each node to verify that the new circuit has
    been created. The circuit information should be the same on both nodes.

    ```
    root@splinterd-beta:/# splinter circuit list --url http://splinterd-beta:8085
    ID              MANAGEMENT MEMBERS
    my-grid-circuit grid       alpha-node-000;beta-node-000
    ```

    ```
    root@splinterd-alpha:/# splinter circuit list --url http://splinterd-alpha:8085
    ID              MANAGEMENT MEMBERS
    my-grid-circuit grid       alpha-node-000;beta-node-000
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

3. Create a new organization, `myorg`.

   `root@gridd-alpha:/# grid organization create 314156 myorg '123 main street' --metadata gs1_company_prefixes=314156`

   This command creates and submits a transaction to create a new Pike
   organization that is signed by the admin key. It also creates a new Pike
   agent with the “admin” role for the new organization (this agent’s public key
   is derived from the private key used to sign the transaction.) The service ID
   includes the circuit name and the scabbard service name for the alpha node.

4. Update the agent's permissions (Pike roles) to allow creating, updating, and
   deleting Grid products.

   ```
   root@gridd-alpha:/# grid \
   agent update 314156 $(cat ~/.grid/keys/alpha-agent.pub) --active \
   --role can_create_product \
   --role can_update_product \
   --role can_delete_product \
   --role admin
   ```

5. Use `cat` to create a product definition file, `product.yaml`, using the
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

6. Add a new product based on the definition in the example YAML file,
   `product.yaml`.

   ```
   root@gridd-alpha:/# grid product create product.yaml
   ```

7. Open a new terminal and connect to the `gridd-beta` container.

   `$ docker exec -it gridd-beta bash`

8. Display all products.

   ```
   root@gridd-beta:/# grid product list
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
(smart contract transactions) remain private to their two-party circuit,
my-grid-circuit.

## For More Information
- Hyperledger Grid documentation: https://grid.hyperledger.org/docs/grid/nightly/master/introduction.html
- Splinter: https://github.com/Cargill/splinter
- Sawtooth Sabre: https://github.com/hyperledger/sawtooth-sabre
- Pike transaction family (defines a Grid Pike smart contract): https://grid.hyperledger.org/docs/grid/nightly/master/transaction_family_specifications/grid_schema_family_specification.html
- Schema transaction family (defines a Grid Schema smart contract): https://grid.hyperledger.org/docs/grid/nightly/master/transaction_family_specifications/grid_schema_family_specification.html
- Product RFC: https://github.com/target/grid-rfcs/blob/d6305b86e2a43e510bb57b297b3ec09b0a66c5b0/0000-product.md
- CLI for the XO smart contract (also called a "transaction processor"): https://sawtooth.hyperledger.org/docs/core/releases/latest/cli/xo.html
