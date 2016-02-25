# Hammer

[![Build Status](https://travis-ci.org/kerinin/hammer.svg?branch=master)](https://travis-ci.org/kerinin/hammer)

Hamming distance search platform


## Use

```sh
# Start an HTTP server indexing 64-bit keys and returning keys with less than
# 5 bits differing from query keys
cargo build && target/build/hammer --bind localhost:3000 --bits 64 --tolerance 5

# Add some keys
curl -X POST -d '{"scalars": [1,2,3,10012093810923]}' localhost:3000/add
# {
#   "scalars":[
#     {"scalar":1,"added":true},
#     {"scalar":2,"added":true},
#     {"scalar":3,"added":true},
#     {"scalar":10012093810923,"added":true}
#   ]
# }

# Query for some keys
curl -X POST -d '{"scalars": [1,10012093810923]}' localhost:3000/query
# {
#   "scalars": [
#     {
#       "scalar": 1,
#       "found": [2,3,1]
#     },
#     {
#       "scalar": 10012093810923,
#       "found": [10012093810923]
#     }
#   ]
# }

# Delete keys
curl -X POST -d '{"scalars": [3]}' localhost:3000/delete
# {
#   "scalars": [
#     {
#       "scalar": 3
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

