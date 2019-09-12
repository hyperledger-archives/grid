# Release Notes

## Changes in Splinter 0.3.1

### Highlights

* Completion of circuit proposal validation, voting, and dynamic circuit creation
* Addition of key generation and management, as well as role-based permissions
* Continued progress towards proposing, accepting, and creating a gameroom in the
  Gameroom example application

### libsplinter

* Add AdminService, with support for:
  * Accepting and verifying votes on circuit proposals
  * Committing approved circuit proposals to SplinterState
* Add notification to be sent to application authorization handlers when a
  circuit is ready
* Update scabbard to properly set up Sabre state by adding admin keys
* Add support for exposing service endpoints using the orchestrator and service
  factories
* Add WebSocketClient for consuming Splinter service events
* Add KeyRegistry trait for managing key information with a StorageKeyRegistry
  implementation, backed by the storage module
* Add KeyPermissionsManager trait for accessing simple, role-based permissions
  using public keys and an insecure AllowAllKeyPermissionManager implementation
* Add SHA512 hash implementation of signing traits, for test cases
* Add Sawtooth-compatible signing trait implementations behind the
  "sawtooth-signing-compat" feature flag.

### splinterd

* Add package metadata and license field to Cargo.toml file
* Add example configuration files, systemd files, and postinst script to Debian
  package
* Reorder internal service startup to ensure that the admin service and
  orchestrator can appropriately connect and start up
* Use SawtoothSecp256k1SignatureVerifier for admin service

### splinter-cli

* Add "splinter-cli admin keygen" command to generate secp256k1 public/private
  key pairs
* Add "splinter-cli admin keyregistry" command to generate a key registry and
  key pairs based on a YAML specification

### Private XO and Private Counter Examples
* Add license field to all Cargo.toml files
* Rename private-xo package to private-xo-service-<version>.deb
* Rename private-counter packages to private-counter-cli-<version>.deb and
  private-counter-service-<version>.deb

### Gameroom Example
* Add package metadata and license field to gameroomd Cargo.toml file
* Add example configs, systemd files, and postinst script to gameroomd Debian
  package; rename package to gameroom-<version>.deb
* Implement notification retrieval using WebSocket subscription and
  notifications endpoints
* Show pending and accepted gamerooms in the Gameroom UI
* Add full support for signing CircuitManagementPayloads with the user's
  private key and submitting it to splinterd
* Update gameroomd to specify itself as the scabbard admin and submit the XO
  smart contract when the circuit is ready
* Make various UI enhancements

## Changes in Splinter 0.3.0

### Highlights

* Completion of the two-phase-commit consensus algorithm with deterministic
  coordination
* Continued progress towards dynamically generating circuits, including
  dynamic peering and circuit proposal validation
* Continued progress on the Gameroom example, including UI updates and
  automatic reconnection

### libsplinter

* Add a service orchestration implementation
* Add Scabbard service factory 
* Implement a deterministic two-phase-commit coordinator
* Reorder the commit/reject process for the two-phase-commit coordinator. The
  coordinator now tells proposal manager to commit/reject before broadcasting
  the corresponding message to other verifiers.
* Refactor two-phase-commit complete_coordination. Move the process of 
  finishing the coordination of a proposal in two-phase commit to a single
  function to reduce duplication.
* Implement a two-phase-commit timeout for consensus proposals
* Update the two-phase-commit algorithm to ignore duplicate proposals
* Allow dynamic verifiers for a single instance of two-phase-commit consensus
* Add an Authorization Inquisitor trait for inspecting peer authorization state
* Add the ability to queue messages from unauthorized peers and unpeered nodes
  to the admin service
* Fix an issue that caused the admin service to deadlock when handling proposals
* Add Event Dealers for services to construct websocket endpoints
* Add a subscribe endpoint to Scabbard
* Validate circuit proposals against existing Splinter state
* Update create-circuit notification messages to include durability field

### splinterd

* Log only warning-level messages from Tokio and Hyper
* Improve Splinter component build times
* Add a NoOp registry to handle when a node registry backend is not specified

### Private XO and Private Counter Examples

* Use service IDs as peer node IDs, in order to make them compatible with
  two-phase consensus

### Gameroom Example

* Add server-side WebSocket notifications to the UI 
* Add borders to the Acme UI
* Improve error handling and add reconnects to the Application Authorization
  Handler
* Add a circuit ID and hash to GET /proposals endpoint
* Standardize buttons and forms in the UI
* Improve error formatting in the UI by adding toasts and progress bar spinners
* Change the Gameroom REST API to retrieve node data automatically on startup
* Split the circuit_proposals table into gameroom and gameroom_proposals tables
* Use the [Material elevation strategy](https://material.io/design/color/dark-theme.html)
  for coloring the UI
* Decrease the font size
* Change the UI to redirect users who are not logged in to login page
* Add a dashboard view
* Add an invitation cards view
* Add a button for creating a new gameroom to the UI

## Changes in Splinter 0.2.0

### libsplinter

* Add new consensus API (libsplinter::consensus)
* Add new consensus implementation for N-party, two-phase commit
  (libsplinter::consensus::two_phase)
* Add new service SDK with in-process service implementations
  (libsplinter::service)
* Add initial implementation for Scabbard, a Splinter service for running Sabre
  transactions with two-phase commit between services
(libsplinter::service::scabbard)
* Add REST API SDK (consider this experimental, as the backing implementation
  may change)
* Add new node registry REST API endpoint for providing information about all
  possible nodes in the network, with initial YAML-file backed implementation.
* Add new signing API for verifying and signing messages, with optional
  Ursa-backed implementation (libsplinter::signing, requires the use of
"ursa-compat" feature flag)
* Add MultiTransport for managing multiple transport types and selecting
  connections based on a URI (libsplinter::transport::multi)
* Add ZMQ transport implementation (libsplinter::transport::zmq, requires the
  use of the "zmq-transport" feature flag)
* Add peer authorization callbacks, in order to notify other system entities
  that a peer is fully ready to receive messages


### splinterd

* Add REST API instance to provide node registry API endpoints
* Add CLI parameter --bind for the REST API port
* Add CLI parameters for configuring node registryy; the default registry type
  is "FILE"

### Gameroom Example

* Add gameroom example infrastructure, such as the gameroomd binary, docker
  images, and compose files
* Add Login and Register UI
* Add New Gameroom UI
* Add UI themes for both parties in demo
* Initialize Gameroom database
* Add circuit proposals table
* Initialize Gameroom REST API
* Implement Gameroom REST API authentication routes
* Implement Gameroom REST API create gameroom endpoint
* Implement Gameroom REST API proposals route
* Implement /nodes endpoint in gameroomd
