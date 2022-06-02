# Release Notes

## Changes in Grid 0.3.4

### Build

 * Check for protoc during Grid build

### Grid SDK

* Update Grid `/batches` endpoint to forward errors it receives from Splinter

### Packaging

 * Publish multi-architecture Docker images

## Changes in Grid 0.3.3

### Build

 * Update justfile to load .env files by default

 * Update `dirs` library from `0.3` to `0.4`
 
 * Replace tempdir with tempfile to address RUSTSEC-2018-0017

 * Add development prerequisites to build instructions

 * Add integration tests for `grid download-xsd`, and move data validation
   tests to the integration test step

### Grid SDK

 * Update SplinterBackendClient so it will pass along error messages

### Grid Daemon

 * Updated `/batch_statuses` endpoint to pass along error messages from
   Splinter by updating the SDK

### Grid UI

 * Replace node-sass with sass to address CVE-2020-24025

## Changes in Grid 0.3.2

### Grid CLI

* Handle errors when accessing all Clap arguments

* Remove `hex-literal` dependency

* Add experimental `xsd-downloader-cache-dir` feature. This feature protects
  the ability to specify a cache directory to use with the `grid-download-xsd`
  command.

* Add experimental `xsd-downloader-force-download` feature. This feature
  protects the ability to force the GS1 schema files to be downloaded,
  regardless if the cache is full.

* Improve logging and error messages when downloading XSDs to specify URLs,
  file paths and directories used by the command

* When downloading XSDs, validate cache and state directories exist and are
  writable, and error on failure rather than attempt to create them

### Build

* Update Dockerhub compose file to pull in most recently published smart
  contract

### Packaging

* Add ca-certificates to Docker images with grid-cli

* Update Grid Daemon postinstall script to create cache directory,
  `/var/cache/grid`

* Update example Docker compose files to create a Grid cache volume to persist
  cached data between restarts

## Changes in Grid 0.3.1

### Highlights

* Adds Grid Purchase Order smart contract.

* Adds streaming results to CLI `list` commands. This returns the initial results much faster for large sets of data.

* Adds Lee Bradley as a Grid maintainer.

* Adds support for Splinter v0.6 by supporting Splinter’s new authorization capabilities. Splinter v0.4 and Splinter v0.6 are now both supported by Grid v0.3.

### Grid CLI

* Remove the `grid-admin-keygen` command. This functionality is replaced by the `--system` option of the `grid-keygen` command

* Add unit tests for the `grid keygen` command

* Add Purchase Order CLI commands

* Add remaining and update existing Purchase Order man pages to reflect current functionality

* Remove unused `splinter` feature as this feature did not provide any functionality

* Fix bug with displaying a Grid Location to properly fetch the location

* Update `grid keygen` private key file permissions from `640` to `600`

* Fix typo in `grid keygen` message

* Remove unused fields from `BatchStatus` and `BatchStatusResponse` structs

* Create `ReqwestClientFactory` directly in place of the removed `create_client_factory` function

* Add helper functions to get common CLI arguments

* Make global CLI argument values available to all subcommands

* Update formatting for newest version of Clippy

* Fix typos in some commands’ help messages

* Rename action modules to be singular

* Update feature dependencies in `Cargo.toml` file to be explicit

* Add streaming results to `list` command results

* Stabilize `purchase-order` feature. Grid Purchase Order commands are now available by default

* Add `download-xsd` command to download Purchase Order XSD files from GS1

* Stabilize `xsd-downloader` feature. Functionality to download Purchase Order XSD files is now available by default

### Grid Daemon


* Add collection of `EventProcessors` to the Splinter application authorization handler to ensure only a single event processor is used for a given Splinter service

* Serialize the handling of Splinter commit events to ensure all events are handled in the order they are received and return early if a duplicate commit event is detected

* Add handling for Purchase Order events to state delta export

* Add Purchase Order REST API endpoints

* Rename any plural REST API resource routes to singular (i.e. `/versions` becomes `/version`)

* Update REST API documentation to fully conform to the OpenAPI specification

* Update `run_splinter` method to generate a Cylinder JWT at runtime

* Update Splinter application authorization handler to use Cylinder JWTs to make authorized requests to the Splinter backend

* Update Splinter event processors to accept Cylinder JWT authorization

* Add `SPLINTER_PROTOCOL_VERSION` to Splinter application authorization handler

* Update `sawtooth-sdk` feature to pull in the `sawtooth-sdk` and `sabre-sdk` features individually

* Remove the `pike-rest-api` feature. This was a duplicate of the `rest-api-resources-agent` feature

* Stabilize `cylinder-jwt-support` feature by removing it. This feature adds authorization support for Splinter backends

* Stabilize `purchase-order` feature by moving the feature to `stable`

* Update the version of Splinter and Scabbard to `0.4.3`

* Add functionality to send authorized requests, using Cylinder JWTs, to `ScabbardClient`

* Remove `actix-web` default features to fix Linux builds. This removes the transient dependency on brotli and gzip, both used for compression. Compression is not used in any of the clients so it is acceptable to remove these transient dependencies

### Grid SDK


* Add Purchase Order database tables and store operations

* Add Purchase Order REST API handlers

* Add method to the `PermissionChecker` to validate an agent’s workflow permissions

* Add documentation for the workflow module

* Add missing and update existing Purchase Order protobufs to align with the RFC

* Update `data_validation` module to provide functionality to validate Purchase Order XML

* Update the `data_validation` module’s use of the `libc` library to minimize chance of memory leaks by implementing `Drop` for all structs and adding validation for null pointers

* Rename `ROLES_ENDPOINT` `const` in the `ReqwestPikeClient` to `ROLE_ENDPOINT`

* Fix bug preventing Pike roles from properly fetching inherited roles

* Update Splinter backend client to be created with an `authorization` field, used when submitting batches, using the Cylinder JWT generated by the `run_splinter` function

* Add ability to check constraints in a workflow when attempting to move between workflow states

* Validate `self` workflow state in Workflow’s `can_transition` method to allow updating a record

* Add missing `message()` function to `ErrorResponse`

* Add documentation and unit tests for `ErrorResponse`

* Remove unused `sawtooth-compat` feature. This feature did not provide any functionality

* Add paging query string support

* Move path knowledge from the resource layer to the Actix layer

* Derive `Debug` for `ClientError`

* Add `InternalError` variant to `ClientError`

* Remove `create_client_factory` function

* Reorganize client module to group client files together and `Reqwest` client files together

* Rename client “DTO” objects and move them into `data::<module>` modules

* Stabilize `cylinder-jwt-support` feature by removing it. This feature adds authorization support for Splinter backends

* Stabilize `rest-api-resources` feature by moving the feature to `stable`

* Stabilize `client` feature by moving the feature to `stable`

* Stabilize `data-validation` feature by moving the feature to `stable`. This feature includes functionality for validating Grid Product and Purchase Order XML files against GS1 standards

* Stabilize `client-reqwest` feature by moving the feature to `stable`. This feature includes an implementation of the `Client` trait backed by `Reqwest`

* Stabilize `purchase-order` feature by moving the feature to `stable`. This feature enables storing Purchase Order data in the Grid database.

* Stabilize `rest-api-endpoint-purchase-order` feature by moving the feature to `stable`

* Stabilize `workflow` feature by moving the feature to `stable`

* Stabilize `rest-api-resources-purchase-order` feature by moving the feature to `stable`

* Update module-level documentation for the `reqwest_client` module

* Updated feature dependencies in `Cargo.toml` file to be explicit

* Add streaming iterator to list API results. This returns batches of results so they can be returned faster

* Fix paging `total` count to properly reflect the number of results

* Add `StartWorkflowState` struct to define the entrypoint of an item into a workflow

* Remove `actix-web` default features to fix Linux builds. This removes the transient dependency on brotli and gzip, both used for compression. Compression is not used in any of the clients so it is acceptable to remove these transient dependencies

* Add `Reqwest`-backed `Client` trait implementations for all rest endpoints

### Grid Smart Contracts

* Add Purchase Order smart contract

* Add built-in collaborative and system-of-record workflows. These workflows are used by the Purchase Order smart contract

* Remove `allow(dead_code)` from workflow functions as these are now fully implemented and in use

* Update Purchase Order smart contract to check workflow permission aliases rather than Pike permissions

* Add constraints to Purchase Order workflows

### Grid UI

* Fix formatting in Grid UI SCSS files

* Add loading indicator for Product table

### Griddle


* Add `--skip` option to `grid keygen` commands in the Griddle example docker-compose files to speed up builds if those keys already exist

* Update version to match the rest of the Grid modules

* Remove `actix-web` default features to fix Linux builds. This removes transient dependencies on brotli and gzip, both used for compression. Compression is not used in any of the clients so it is acceptable to remove these transient dependencies

### Build

* Switch from Jenkins builds to GitHub actions

* Base Grid CLI dockerfile on Ubuntu Focal instead of Sabre CLI

* Add just recipe for Docker builds

* Update image used by Node in UI Dockerfiles from `lts-alpine` to `14.18.1-alpine3.11`

* Update smart contract Dockerfiles to take advantage of dependency caching

* Add `curl` to Grid Dockerfile to allow XSD files to be downloaded

* Various code formatting fixes

### CI

* Add a GitHub action to validate swagger API documentation to the OpenAPI specific

* Create Grid SDK as `lib` instead of `bin`

* Cache WASM dependencies in `grid-dev` Dockerfile
