# Hammer

[![Build Status](https://travis-ci.org/kerinin/hammer.svg?branch=master)](https://travis-ci.org/kerinin/hammer)

Hamming distance search platform

NOTE: Web UI currently not working - this is really just a Rust lib at the
moment.  



## Use

```sh
# Start an HTTP server indexing 64-bit keys and returning keys with less than
# 5 bits differing from query keys
./hammer serve --bind localhost:8080 --bits 64 --tolerance 5

# Add some keys
curl -X POST -d '{"Scalars": [1,2,3,10012093810923]}' localhost:8080/add
# {
#   "Scalars":[
#     {"Scalar":1,"Added":true},
#     {"Scalar":2,"Added":true},
#     {"Scalar":3,"Added":true},
#     {"Scalar":10012093810923,"Added":true}
#   ]
# }

# Query for some keys
curl -X POST -d '{"Scalars": [1,10012093810923]}' localhost:3000/query
# {
#   "Scalars": [
#     {
#       "Scalar": 1,
#       "Found": [2,3,1]
#     },
#     {
#       "Scalar": 10012093810923,
#       "Found": [10012093810923]
#     }
#   ]
# }

# Delete keys
curl -X POST -d '{"Scalars": [3]}' localhost:3000/delete
# {
#   "Scalars": [
#     {
#       "Scalar": 3
#       "Deleted": true,
#     }
#   ]
# }
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

