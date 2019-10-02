
# How to Run the Private Counter Demo

Private Counter is an example Splinter application that "counts" (adds numbers
to a shared counter) on a multi-party Splinter circuit. This extremely simple
example demonstrates how to create a Splinter service that shares state, as
managed by two-phase commit consensus.

The Private Counter demo uses Docker containers to create three Splinter nodes,
each with a Private Counter service, a node registry, and a local copy of state.

**Prerequisites**:
This demo requires [Docker Engine](https://docs.docker.com/engine)
and [Docker Compose](https://docs.docker.com/compose).

1. Clone the [splinter repository](https://github.com/Cargill/splinter).

1. To start the Private Counter demo, run the the following command from the
   Splinter root directory.

     ```
     docker-compose -f examples/private_counter/docker-compose.yaml up
     ```

   This command starts three Splinter nodes (a, b, and c), each with
   a `private-counter-service`, and a `pcounter` shell container for interacting
   with these services.

1. Connect to the `pcounter-local` container.

     ```
     docker exec -it pcounter-local bash
     ```

1. Use the `pcounter` command to view or change the current counter.

   In the following commands, replace *{URL}* with the Private Counter service
   name and port on one of the nodes (a, b, or c). For example:
   `private-counter-service-a:8000`


   - Run `pcounter show` to display the current value of the counter.

        ```
        pcounter -v --url {URL} show
        ```

   - Run `pcounter add` to increase the value by the specified integer
     (must be u32). For example, this command increments the counter by 1:

        ```
        pcounter -v --url {URL} add 1
        ```

5. When you are finished, shut down the demo with the following command:

     ```
     docker-compose -f examples/private_counter/docker-compose.yaml down
     ```

