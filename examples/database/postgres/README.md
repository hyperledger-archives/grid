# Generating Database Schema Documentation

This document outlines a procedure for generating the Grid database
documentation and updating the Grid website.

The docker-compose file builds the documentation website and serves it locally
at [localhost:9001](http://localhost:9001).

Run the following command to run and build the website:

```
$ docker-compose -f examples/database/postgres/docker-compose.yaml up --build
```

## Copy Generated Site to Grid Docs

1. Clone the [Hyperledger Grid Website repository](https://github.com/hyperledger/grid-website)
   ([https://github.com/hyperledger/grid](https://github.com/hyperledger/grid-website)).
1. Navigate to the `grid-website/docs/<release_number>/database/postgres` directory
1. Run the command:
```
$ docker cp griddb-console:/usr/local/apache2/htdocs/* .
```
