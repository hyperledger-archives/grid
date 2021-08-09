# Release Notes

## Changes in Grid 0.2.2

### Highlights

* Fixes a bug that was causing duplicate events to propagate to the Grid
  database. If gridd was shut down and started back up, another websocket
  connection would be created between Splinter and Grid, but the original
  websocket would also reconnect. State delta functionality would then put a new
  record in the database for each event it received simultaneously, leading to
  multiple identical records in the database. This release fixes that bug by
  serializing the processing of events and ensuring that we only have one
  event processor per service.

### gridd

* Add an early return in the event handler after we detect a duplicate commit,
  so that we stop processing the event after that point.

* Serialize splinter commit event processing.

* Add an EventProcessors collection, which ensures that only a single event
  processor on a given circuit/service connection is only created once.

## Changes in Grid 0.2

### Highlights

* Add Purchase Order smart contract

* Add sub-commands to list and show agents

* Add sub-commands to list and show organizations

* Stabilize the experimental “splinter-support” feature

* Add bash-completion functionality

* Add initial implementation of `griddle`, Grid’s integration component, which
  provides a high-level interface for interacting with the Grid Daemon

* Organize feature sections in Cargo.toml files across Grid’s SDK, CLI, and
  smart contracts to be consistent

### Grid CLI

* Make features guarding Grid’s business functionality default in the
  Cargo.toml file

* Make arguments used for multiple commands global

* Wrap lines of output at 80 characters

* Stabilize the experimental “product-gdsn” feature by removing it, this
  feature guarded the use of XML product definitions

* Add a `--skip` option to the key generation command with updated logic to
  determine how key files are generated if ones already exist

* Update the path searched for key files to match documentation

* Add updated man pages for all commands

* Add environment variable defaults to the After Help across all commands

* Add the Schema namespace to location transaction inputs, as this is required
  for the Location smart contract to read the location schema

* Change action handlers’ `key` argument to `signer`, to take Cylinder’s
  `Signer` object, rather than the key directly to match Splinter CLI’s signing
  pattern

* Add a `signing` module implemented with Cylinder and remove the `key` module
  implemented using the Sawtooth SDK

* Update the construction of `PikePayload`s as the native representation was
  updated to hold an enum variant with the corresponding payload and a
  `timestamp` field was added

* Remove the `xml` argument in the `product create` sub-command, as the `file`
  argument is used instead for all file types

* Allow Product GDSN XML file inputs for the `product update` sub-command,
  behind the experimental “product-gdsn” feature

* Update the `product create` sub-command to allow for multiple product
  definition files to be passed with the `--file` argument

* Add sub-command to create a Product from a file containing the product
  definition

* Add man pages for Pike-related sub-commands, representing commands used by
  the Pike smart contract version 2

* Fix serialization of the `GS1` namespace, natively represented as `Gs1` to
  avoid linting errors, to be serialized as `GS1` rather than `Gs1`

* Update batch submission output in order to simplify output and to give more
  descriptive output if the command encounters an error

* Add an argument, `--alternate-ids`, to the `organization update` sub-command
  to allow alternate IDs to be added to an organization

* Add arguments to define locations and alternate IDs for the `organization`
  sub-commands and remove the `address` argument

* Add `role` sub-commands to create, update, list, and show Pike roles

* Update `schema` sub-commands to require an organization ID, which defines the
  schema’s `owner`

* Fix compilation with the `--no-default-features` flag

* Update the actions for listing resources to handle pagination

### Grid Daemon

* Stabilize the experimental “scabbard-event-restart” feature by removing it

* Reconnect scabbard event processors on restart, guarded by the experimental
  “scabbard-event-restart” feature

* Remove the “rest-api-actix-web-3” feature dependency from the Grid SDK to
  behind the daemon’s default “rest-api” feature

* Update the configuration object’s native representation of the `endpoint`
  value in order to not import data types from the Grid SDK’s Rest API module
  as these modules are not dependent on one another by features and Grid may be
  compiled without the SDK’s Rest API

* Update the `setup_grid` function to load the Schema smart contract first as
  the other smart contracts depend on Schema state

* Add unit tests for the Rest API using a SQLite in-memory database

* Make the operation to add commits to the database atomic by wrapping the
  database operation in a generic Diesel transaction, ensuring commits are not
  added if another transaction in the event fails

* Rename routes beginning with `fetch` to `get` to match the database
  operations used by these endpoints

* Add `last_updated` field to the Rest API resources representing products,
  locations, agents, organizations, roles, and schemas to match how these
  objects are stored and update the Open API spec to reflect this update

* Add functionality to submit Purchase Order smart contract transactions

* Add experimental feature, `integration`, to guard Rest API resources matching
  the endpoints provided by griddle, Grid’s integration component

* Update the Actix dependency to version 3.0

* Add endpoints to list and show Pike agents, organizations, and roles and
  update the Open API spec to reflect the addition of these endpoints

* Remove the Rest API’s paging module and move to Grid SDK

* Use unsigned integers for each paging argument, `limit` and `offset`, to
  ensure the `offset` is not negative and to place an upper bound on `limit`

* Update the database handler’s Pike-related operations and the Pike-related
  rest endpoints to account for updates made to the Pike store implementation
  and Pike smart contract

* Implement pagination for all endpoints that list resources and update Open
  API spec to reflect this update

* Add a `paging` module to the Rest API `routes` module to hold a generic
  function for creating the paging response from endpoints that list resources

* Separate the `run_splinter` and `run_sawtooth` functions into specific
  modules, the `splinter` module and the `sawtooth` module, respectively

* Fix compilation with the `--no-default-features` flag

### Grid SDK

* Stabilize the experimental “commit-store-service-commits” feature by removing
  it

* Reduce the Commit store’s `get_current_service_commits` operation
  implementations to one which uses generic arguments and may be used by
  multiple Diesel backends

* Add standard error, `InvalidStateError`, to propagate when an operation
  cannot be completed because the state of the underlying struct is inconsistent

* Add operation to Commit store to provide the current commits for all
  services, behind the experimental “commit-store-service-commits” feature

* Implement builder objects and public accessor methods for the Pike store’s
  native structs, including `Agent`, `Organization`, `Role`, and the associated
  structs

* Update data type for bytes stored in the database, from `Bytea` to `Binary`,
  as the `Bytea` data type is no longer supported by Diesel’s SQLite connection

* Stabilize the experimental “product-gdsn” feature, which guards the use of
  XML product definitions

* Stabilize the experimental “postgres” feature, which guards the set-up and
  use of a PostgreSQL database backend

* Stabilize the experimental `sqlite` feature, which guards the set-up and use
  of a SQLite database backend

* Remove the Location store’s `update` operation, as locations may be updated
  using the `add` operation

* Update Location store’s operations to check for the existence of a service ID
  to ensure all fields are updated as expected

* Update Pike store’s `update` operations to check for the existence of a
  service ID to ensure all fields are updated as expected

* Move all conversion methods implemented for the Batch store’s database models
  to the same file as the database models’ definitions

* Move all conversion methods implemented for the Product store’s database
  models to the same file as the database models’ definitions

* Update the schema for Grid trade items to access resources using
  `gdsregistry.org`, as the previously used source would sometimes fail to load
  resources

* Remove duplicate imports in the schema for Grid trade items, defined by an
  XSD file

* Implement a builder struct and public accessor methods for `Product`, in
  place of public fields in the native struct representation

* Remove the `testing` module in favor of the Rest API’s unit tests in the Grid
  Daemon

* Rename `BatchSubmitter` trait to `BackendClient`, to better represent the
  functionality of this trait

* Update the Commit store’s database operations to use Diesel’s `optional`
  convenience method when retrieving results

* Add the Commit store’s `From` conversions implemented for errors to the
  `error` module and implement usage of these conversions in store operations

* Remove `CommitEventError` and replace usage with `CommitStoreError`, as these
  error types were duplicates

* Separate permission error definitions into an `error` module within the
  `permissions` module

* Reorganize the `permissions` module from alongside to inside the `pike`
  module, as this module is directly related to the Pike smart contract

* Use top-level `DEFAULT_GRID_PROTOCOL_VERSION` const across modules, rather
  than individual definitions for this const in each module

* Add a `timestamp` field to the `PikePayload` protocol object

* Update the `PikePayload` protocol object to replace multiple payload objects
  with one enum, with variants pertaining to the payload actions

* Add functionality to Product’s `gdsn` module to create a payload to update a
  product

* Update the order of parsing and validating XML files to allow parsing errors
  to be reported before validation

* Update the library used to validate product definition XML files, to better
  handle validation errors

* Remove the memory-based implementation of the Commit store, as memory-based
  stores are no longer supported

* Use the `InternalError` type, a top-level standard error, when creating a
  store factory, instead of unique errors

* Make all database operations atomic by wrapping the operation implementation
  in a generic Diesel transaction

* Add `ConstraintViolationType` error variant to the `CommitEventError` to
  enable database operations to be atomic

* Rename all occurrences of the `fetch` prefix in store operations to `get`

* Add `last_updated` field to database models representing products, locations,
  agents, organizations, roles, and schemas to record the time a record is
  inserted into the database

* Implement function to validate product definitions, represented as XML strings

* Add schema file used for validating products (GDSN trade item definitions),
  represented as XML strings

* Implement a batch processor to handle the submission and results of batches,
  for both Sawtooth and Splinter backends

* Add database operations to manage and send batches

* Update database tables used for batches, to further separate naturally
  independent structs and to add columns to manage batch errors, status
  information, and Splinter service IDs

* Remove database operations to update a batch’s status and listing batches
  with the corresponding status, based on an updated design of the batch
  submitter

* Add standard error, `InvalidArgumentError`, to be propagated in case of an
  invalid user argument

* Add functionality to convert XML files into native Product structs, in the
  `product`’s `gdsn` module

* Add Rest API endpoints to list and show purchase orders, implemented with the
  first version of resources for these endpoints

* Update Rest API client’s `post_batches` method to take additional arguments,
  `service_id` and `wait`, to specify a Splinter service ID and the amount of
  time to wait for a transaction to be committed

* Fix serialization of the `GS1` namespace, natively represented as `Gs1` to
  avoid linting errors, to be serialized as `GS1` rather than `Gs1`

* Reorganize Rest API’s `actix_web_3` module to separate naturally independent
  components, without altering functionality

* Add initial Reqwest-backed `ReqwestClient` trait, extending the `Client`
  trait, and implement the `post_batches` method apart of this trait

* Add `client` module for the Rest API client implementation and initial
  unimplemented `Client` trait

* Add plumbing to implement Purchase Order smart contract functionality,
  including Protobuf messages, protocol functions, addressing, and the store
  implementation

* Add module for the Purchase Order smart contract implementation, behind the
  experimental `purchase_order` feature

* Remove the `update role` database operation, as roles are updated by the
  `add agent` operation

* Fix the database operation to add roles to ensure removed permissions are
  removed from the database as expected

* Add plumbing to support alternate IDs, represented as a list within an
  organization, to the Pike store and addressing

* Add store implementation to hold batches, which enables adding, listing, and
  updating batches

* Add Rest API resources and payloads for the new `submit` endpoint to a `v1`
  module, organized to allow for future iterations

* Add an `actix_web_3` module to the Rest API, with an initial `submit` endpoint

* Add functionality to Pike, enabling role-based permissions and deleting
  agents and organizations

* Add a Rest API paging module (previously defined in the Grid Daemon) to be
  used across resources

* Move the `to_hex` function behind the “batch-store” feature instead of the
  “pike” feature

* Update the Pike store module to match the Identity RFC, which represents the
  next iteration of Pike

* Add support for pagination, as `limit` and `offset` arguments, to database
  operations that list resources

* Consolidate the Pike-related stores into one `pike` store module

* Remove memory-backed store implementations

* Remove high-level features, `database` and `grid_db`, that wrap several other
  features within the Cargo.toml file but do not actually guard code

* Separate database migration files based on the concern of the database tables

* Add more robust permission validation in the `PermissionChecker`’s
  `has_permission` method to reduce code duplication within smart contracts

* Update store modules to fix various comments

* Consolidate addressing functionality for Grid’s smart contracts

* Add `workflow` experimental feature and initial `Workflow` implementation

### Grid Smart Contracts

* Update smart contracts to version 2

* Add prefix to Pike smart contract name, the name is now `grid_pike` to match
  the naming convention used in other smart contracts

* Move the struct used to manage the state of the Pike smart contract into its
  own `state` module, within the Pike smart contract

* Add `workflow` module to the Purchase Order smart contract with initial
  `Workflow` implementation

* Update the Pike smart contract to account for the addition of alternate IDs

* Update smart contracts to check an organization’s list of alternate IDs,
  rather than metadata which previously held the information now recorded by
  alternate IDs

* Replace custom permission checking in the Schema, Product, Pike, and Location
  smart contracts with the SDK’s `PermissionChecker`’s `has_permission` check

* Update the Pike address prefix to a sub-namespace of the Grid address
  namespace, rather than a unique prefix

* Update the smart contracts’ permission checking to validate roles and
  inherited roles

### Grid UI

* Remove deprecated form components previously used for adding, editing and
  deleting products

* Update the Grid logo displayed by the Grid UI to have a white background to
  improve visibility

* Update the Product sapling to use newly added `TopBar` and `Table` components
  to display product definitions for a chosen service in the sapling’s table,
  which allows filtering and downloading product XML files directly

* Add a `Table` component with the updated theme styles to the Product sapling

* Add a `TopBar` component with the update theme styles to the Product sapling,
  replacing the previously implemented component used to filter products

* Add theme styles to the Product sapling, allowing the use of SCSS
  functionality and the Grid Canopy theme styles

* Update Grid Canopy theme styles, including updates to colors, sizes, and a
  new CSS grid layout

* Fix Docker compose files based on updates made to the project’s file structure

* Update the sidebar to correctly show icons for different Saplings

* Update Product sapling to handle pagination returned when listing resources

* Update products table in Product sapling to have a loading indicator when
  fetching data

* Remove element namespaces from GDSN XML parsing in Product UI. These
  namespaces were previously hardcoded to work with example data, but not all
  GDSN data has the same namespace for the same attribute. Files downloaded from
  the UI will no longer include the original namespace.

### Griddle

* Add Docker compose file to run `griddle` with a Sawtooth backend

* Add plumbing for load balancing

* Add `--connect` argument to the start command, to allow `griddle` to access
  the backend

* Add functionality to process batches, starting a `BatchProcessor` at
  `griddle`’s runtime

* Update Docker compose files to use the `--connect` argument when starting
  `griddle`

### Build

* Update justfile to include the `purchase_order` smart contract crate

* Update the default `CARGO_ARGS` environment variable to work with `cargo
  build` over `cargo deb`

* Compile crates with `cargo build` instead of `cargo deb` for more consistent
  results

* Add .env files to the same directories as Docker compose files as needed, as
  is expected by Docker Compose as of version 1.28.0

* Ignore target directories when building Docker containers

* Add a justfile, including recipes to build, lint and unit test Grid

### CI

* Add `TEST_ARGS` environment variable to justfile and `grid-tests` Docker
  compose file, allowing for arguments to be passed when running tests in CI

* Update the Jenkinsfile to down testing Docker containers to ensure no
  conflicts on subsequent builds

* Use just recipes in the Jenkinsfile, allowing developers to locally reproduce
  the steps taken in CI

* Add recipes to the justfile to run in CI, for linting and testing

* Update Docker setup for running integration tests to ensure tests are run the
  same in Docker as they are locally, using the `just test` command

* Update Dockerfile used to lint the project to use the grid-dev image and add
  a Docker compose file to run linting using the `just lint` command

### Packaging

* Add plumbing to publish the Grid SDK to crates.io

* Cache dependencies for all features in `grid-dev` image

* Add `just` to the `grid-dev` base image to use justfile recipes in CI

* Add `griddle`, Grid’s integration component, to the `grid-dev` image

* Update the `grid-dev` image to propagate errors encountered in the build
  process

### Grid Examples

* Update the Docker compose file for running Grid on Sawtooth to account for
  adding the `grid` prefix to the Pike smart contract’s name

* Add Docker volumes, used to store smart contracts, to the Scabbard CLI
  containers in the Grid on Splinter Docker compose file

* Add Docker compose file to run Grid on Splinter using published artifacts

* Add Docker compose files for development and simple demos with a Splinter
  backend
