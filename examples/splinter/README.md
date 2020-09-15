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

Due to ongoing development of Splinter the images in this example can become
stale. If you have used this procedure before, run the following command to
ensure that your images are up to date:

```
$ docker-compose -f examples/splinter/docker-compose.yaml pull generate-registry db-alpha scabbard-cli-alpha splinterd-alpha
```

## Set Up and Run Grid

1. Clone the [Hyperledger Grid repository](https://github.com/hyperledger/grid)
   ([https://github.com/hyperledger/grid](https://github.com/hyperledger/grid)).
2. Navigate to the grid root directory and build the Grid Docker containers.

   `$ docker-compose -f examples/splinter/docker-compose.yaml build --pull`

3. Start the Grid Docker containers.

   `$ docker-compose -f examples/splinter/docker-compose.yaml up`

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

4. Propose a new circuit with the definition `circuit propose` CLI command.

   ```
   root@splinterd-alpha:/# splinter circuit propose \
      --key /registry/alpha.priv \
      --url http://splinterd-alpha:8085  \
      --node alpha-node-000::tcps://splinterd-alpha:8044 \
      --node beta-node-000::tcps://splinterd-beta:8044 \
      --service gsAA::alpha-node-000 \
      --service gsBB::beta-node-000 \
      --service-type *::scabbard \
      --management grid \
      --service-arg *::admin_keys=$(cat gridd.pub) \
      --service-peer-group gsAA,gsBB
   ```

5. Check the results by displaying the list of proposals. The following example
   sets the CIRCUIT_ID environment variable; this environment variable is for
   the purposes of this procedure and is not used directly by the `splinter`
   CLI commands.

   Set CIRCUIT_ID based on the output of the `proposals` subcommand; for
   example:

   ```
   root@splinterd-alpha:/# splinter circuit proposals --url http://splinterd-alpha:8085
   ID            MANAGEMENT MEMBERS
   01234-ABCDE   grid       alpha-node-000;beta-node-000
   ```

   ```
   root@splinterd-alpha:/# export CIRCUIT_ID=01234-ABCDE
   ```

   ```
   root@splinterd-alpha:/# splinter circuit show $CIRCUIT_ID --url http://splinterd-alpha:8085
   Proposal to create: 01234-ABCDE
      Management Type: grid

      alpha-node-000 (tcps://splinterd-alpha:8044)
          Vote: ACCEPT (implied as requester):
              <alpha-public-key>
          Service (scabbard): gsAA
              admin_keys:
                  <gridd-alpha public key>
              peer_services:
                  gsBB

      beta-node-000 (tcps://splinterd-beta:8044)
          Vote: PENDING
          Service (scabbard): gsBB
              admin_keys:
                  <gridd-alpha public key>
              peer_services:
                  gsAA

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
   ID            MANAGEMENT MEMBERS
   01234-ABCDE   grid       alpha-node-000;beta-node-000
   ```

   ```
   root@splinterd-beta:/# export CIRCUIT_ID=01234-ABCDE
   ```

8. Use the ID to display the details of the proposed circuit.

   ```
   root@splinterd-beta:/# splinter circuit show $CIRCUIT_ID --url http://splinterd-beta:8085
   Proposal to create: 01234-ABCDE
      Management Type: grid

      alpha-node-000 (tcps://splinterd-alpha:8044)
          Vote: ACCEPT (implied as requester):
              <alpha-public-key>
          Service (scabbard): gsAA
              admin_keys:
                  <gridd-alpha public key>
              peer_services:
                  gsBB

      beta-node-000 (tcps://splinterd-beta:8044)
          Vote: PENDING
          Service (scabbard): gsBB
              admin_keys:
                  <gridd-alpha public key>
              peer_services:
                  gsAA
   ```

9. Then vote to accept the proposal.

   `root@splinterd-beta:/# splinter circuit vote --key /registry/beta.priv --url http://splinterd-beta:8085 $CIRCUIT_ID --accept`

10. Run the following command on each node to verify that the new circuit has
    been created. The circuit information should be the same on both nodes.

    ```
    root@splinterd-beta:/# splinter circuit list --url http://splinterd-beta:8085
    ID            MANAGEMENT MEMBERS
    01234-ABCDE   grid       alpha-node-000;beta-node-000
    ```

    ```
    root@splinterd-alpha:/# splinter circuit list --url http://splinterd-alpha:8085
    ID            MANAGEMENT MEMBERS
    01234-ABCDE   grid       alpha-node-000;beta-node-000
    ```


## Demonstrate Grid Smart Contract Functionality

**Note:** To simplify this procedure, the example `docker-compose.yaml` file
defines environment variables for the ``gridd-alpha`` and ``gridd-beta``
containers. These variables define the Grid daemon's key file and endpoint,
so you don't have to use the `-k` and `--url` options with the `grid` command in
this section.

The following environment variables apply only to this example. If you want to
override these values, you can edit `docker-compose.yaml` to redefine the
variables, or you can use the associated option with the `grid` commands in
steps 3 through 10.

- `GRID_DAEMON_KEY` defines the key file name for the Grid daemon, as generated
   by the `docker-compose.yaml` file. Use `-k <keyfile>` to override this
   variable on the command line.

- `GRID_DAEMON_ENDPOINT` defines the endpoint for the ``gridd-alpha`` or
   ``gridd-beta`` container. Use `--url <endpoint>` to override this variable
   on the command line.


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
   root@gridd-alpha:/# export GRID_SERVICE_ID=01234-ABCDE::gsAA
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
   --role can_create_schema \
   --role admin
   ```

6. Use `cat` to create a schema definition file, `product_schema.yaml`, using
   the following contents.

   ```
    - name: gs1_product
      description: GS1 product schema
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
            in order to accurately create a product description as needed for their
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

7. Use `cat` to create a product definition file, `product.yaml`, using the
   following contents.

   ```
   - product_namespace: "GS1"
     product_id: "013600000929"
     owner: "314156"
     properties:
       - name: "product_name"
         data_type: "STRING"
         string_value: "Truvia 80 ct."
       - name: "image_url"
         data_type: "STRING"
         string_value:
          "https://target.scene7.com/is/image/Target/GUEST_b7a6e983-b391-40a5-ad89-2f906bce5743?fmt=png&wid=1400&qlt=80"
       - name: "brand_name"
         data_type: "STRING"
         string_value: "Truvia"
       - name: "product_description"
         data_type: "STRING"
         string_value: "Truvia Sugar 80CT"
       - name: "gpc"
         data_type: "NUMBER"
         number_value: 30016951
       - name: "net_content"
         data_type: "STRING"
         string_value: "80CT"
       - name: "target_market"
         data_type: "NUMBER"
         number_value: 840
   ```

8. Add the product schema based on the definition in the example YAML file,
   `product_schema.yaml`.

    ```
    root@gridd-alpha:/# grid -k alpha-agent \
        schema create product_schema.yaml
    ```

9. Add a new product based on the definition in the example YAML file,
   `product.yaml`.

   ```
   root@gridd-alpha:/# grid -k alpha-agent \
     product create  product.yaml
   ```

10. Open a new terminal and connect to the `gridd-beta` container.

   `$ docker exec -it gridd-beta bash`

11. Set an environment variable with the service ID.

    ```
    root@gridd-beta:/# export GRID_SERVICE_ID=01234-ABCDE::gsBB
    ```

12. Display all products.

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
   root@scabbard-cli-alpha:/# export CIRCUIT_ID=01234-ABCDE
   ```

3. Download the smart contract.

   `root@scabbard-cli-alpha:/# curl -OLsS https://files.splinter.dev/scar/xo_0.4.2.scar`

4. Create the contract registry for the new smart contract.

   ```
   root@scabbard-cli-alpha:/# scabbard cr create sawtooth_xo \
   --owners $(cat /root/.splinter/keys/gridd.pub) \
   -k gridd \
   -U 'http://splinterd-alpha:8085' \
   --service-id $CIRCUIT_ID::gsAA
   ```

5. Upload the smart contract.

   ```
   root@scabbard-cli-alpha:/# scabbard contract upload xo:0.4.2 \
   --path . \
   -k gridd \
   -U 'http://splinterd-alpha:8085' \
   --service-id $CIRCUIT_ID::gsAA
   ```

6. Create the namespace registry for the smart contract.

   ```
   root@scabbard-cli-alpha:/# scabbard ns create 5b7349 \
   --owners $(cat /root/.splinter/keys/gridd.pub) \
   -k gridd \
   -U 'http://splinterd-alpha:8085' \
   --service-id $CIRCUIT_ID::gsAA
   ```

7. Grant the appropriate contract namespace permissions.

   ```
   root@scabbard-cli-alpha:/# scabbard perm 5b7349 sawtooth_xo --read --write \
   -k gridd \
   -U 'http://splinterd-alpha:8085' \
   --service-id $CIRCUIT_ID::gsAA
   ```


8. Open a new terminal and connect to the `scabbard-cli-beta` container and add
   the circuit ID environment variable
   ```
   $ docker-compose -f examples/splinter/docker-compose.yaml run scabbard-cli-beta bash
   root@scabbard-cli-beta:/#
   ```
   ```
   root@scabbard-cli-beta:/# export CIRCUIT_ID=01234-ABCDE
   ```

9. List all uploaded smart contracts.

   ```
   root@scabbard-cli-beta:/# scabbard contract list -U 'http://splinterd-beta:8085' --service-id $CIRCUIT_ID::gsBB
   NAME        VERSIONS OWNERS
   grid_product 1.0      <gridd-alpha public key>
   pike         0.1      <gridd-alpha public key>
   sawtooth_xo  1.0      <gridd-alpha public key>
   ```

10. Display the xo smart contract.

   ```
   root@scabbard-cli-beta:/# scabbard contract show sawtooth_xo:1.0 -U 'http://splinterd-beta:8085' --service-id $CIRCUIT_ID::gsBB
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
