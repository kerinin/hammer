# Hammer

[![Build Status](https://travis-ci.org/kerinin/hammer.svg?branch=master)](https://travis-ci.org/kerinin/hammer)

Hamming distance search platform


## Use

All requests to the API should be POST's.  Three endpoints are exposed; `/add`,
`/query` and `/delete`, which do what they say.  Databases are identified by the
size of the values they index, their tolerance (the maximum hamming distance of
returned values from a query) and an arbitrary namespace value.  Supported value
sizes:

* `b64/:tolerance/:namespace`: Substitution-variant DB indexing 64-bit binary values
* `b128/:tolerance/:namespace`: Substitution-variant DB indexing 128-bit binary values
* `b256/:tolerance/:namespace`: Substitution-variant DB indexing 256-bit binary values
* `bN/:bits/:tolerance/:namespace`: Substitution-variant DB indexing binary
  values of length `:bits`
* `v64/:length/:tolerance/:namespace`: Deletion-variant DB indexing vectors of
   64-bit values of length `:length`
* `v128/:length/:tolerance/:namespace`: Deletion-variant DB indexing vectors of
   128-bit values of length `:length`
* `v256/:length/:tolerance/:namespace`: Deletion-variant DB indexing vectors of
   256-bit values of length `:length`
* `vN/:length/:tolerance/:namespace`: Deletion-variant DB indexing vectors of
   `:bits`-bit values of length `:length`


```sh
# Start an HTTP server indexing 64-bit keys and returning keys with less than
# 5 bits differing from query keys
cargo build && target/build/hammer --bind localhost:3000 --bits 64 --tolerance 5

# Add some keys
curl -X POST -d '[1,2,3,10012093810923]' localhost:3000/add/b64/8/foo
# {"1":true,"2":true,"3":true,"10012093810923":true}

# Query for some keys
curl -X POST -d '[1,10012093810923]' localhost:3000/query/b64/8/foo
# {"1":[2,3,1],"10012093810923":[10012093810923]}

# Delete keys
curl -X POST -d '[3]' localhost:3000/delete/b64/8/foo
# {"3":true}
```

## Architecture

Keys are partitioned into a set of indices.  Indices consist of a mapping from a
key partition to the key value, as well as mappings from each key's
1-permutation to the key value.  

Keys are added by splitting into partitions, each partition is merged into its
associated index.

Keys are retrieved by splitting the query key into partitions and querying for
the partition in each index.  Results are filtered by the target hamming
distance and returned as a set.

This is mostly an implementation of
[HmSearch](http://www.cse.unsw.edu.au/~weiw/files/SSDBM13-HmSearch-Final.pdf)

