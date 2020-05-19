Packaging Saplings
------------------

To make distribution easier, saplings are packaged as bzipped tar files with
the extension ".sapling". A Docker Compose file that creates packages for the
saplings in this repo can be found in the `ci/` directory. Because the packaging
process takes place in Docker containers, both Docker and Docker Compose are
required to run the following commands.

To generate sapling packages, from the root of the `splinter-ui` repository,
run:

```
$ docker-compose -f ci/package-sapling.yaml up
```

Compiling the saplings and building the packages may take a few minutes. Once
the process is complete, the packaged saplings will be placed in
`splinter-ui/build`.

```
$ ls build
circuit_0.1.1-dev.sapling
profile_0.1.1-dev.sapling
register-login_0.1.1-dev.sapling
```

It's also possible to create packages for individual saplings by providing the
sapling name as an argument. For example, to package only `circuits`, run the
following:

```
$ docker-compose -f ci/package-sapling.yaml up circuits
```
