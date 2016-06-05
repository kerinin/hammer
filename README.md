# Hammer

[![Build Status](https://travis-ci.org/kerinin/hammer.svg?branch=master)](https://travis-ci.org/kerinin/hammer)

Hamming distance search index

## Use

All requests to the API should be POST's.  Three endpoints are exposed; `/add`,
`/query` and `/delete`, which do what they say.  Databases are identified by the
size of the values they index, their tolerance (the maximum hamming distance of
returned values from a query) and an arbitrary namespace value.  Supported value
sizes:

* `b/64/:tolerance/:namespace`: Substitution-variant DB indexing 64-bit binary values
* `b/128/:tolerance/:namespace`: Substitution-variant DB indexing 128-bit binary values
* `b/256/:tolerance/:namespace`: Substitution-variant DB indexing 256-bit binary values
* `bN/:bits/:tolerance/:namespace`: Substitution-variant DB indexing binary
  values of length `:bits` (multiples of 8)
* `v/64/:length/:tolerance/:namespace`: Deletion-variant DB indexing vectors of
   64-bit values of length `:length`
* `v/128/:length/:tolerance/:namespace`: Deletion-variant DB indexing vectors of
   128-bit values of length `:length`
* `v/256/:length/:tolerance/:namespace`: Deletion-variant DB indexing vectors of
   256-bit values of length `:length`

End points accept arrays of additions, queries and deletions and return results
arrays - each element in the result array relates to the corresponding element
in the request array.

```sh
# Start an HTTP server on port 3000
cargo build && target/build/hammer --bind localhost:3000

# Add some keys
curl -X POST -d '["AAAAAAAAAAA=","AAAAAAAAAAA=","AAAAAAAA","AADZvdpG3MA="]' localhost:3000/add/b/64/8/foo
# ["ok",exists","err: unable to decode 'AAAAAAAA': (...)","ok"]
curl -X POST -d '[["AAAAAAAAAAA=","AAAAAAAAAAE="],["AAAAAAAAAAI=","AADZvdpG3MA="]]' localhost:3000/add/v/64/2/8/foo
# ["ok","ok"]

# Query for some keys
curl -X POST -d '["AAAAAAAAAAA=","AADZvdpG3MA="]' localhost:3000/query/b/64/8/foo
# [["AAAAAAAAAAI=","AAAAAAAAAAE=","AAAAAAAAAAA="],["AADZvdpG3MA="]]

# Delete keys
curl -X POST -d '["AAAAAAAAAAA="]' localhost:3000/delete/b/64/8/foo
# ["ok"]
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

