# Hammer


Hamming distance search platform

## Architecture

Keys are partitioned into a set of indices.  Indices consist of a mapping from a
key partition to the key value, as well as mappings from each key's
1-permutation to the key value.  

Keys are added by splitting into partitions, each partition is merged into its
associated index.

Keys are retrieved by splitting the query key into partitions and querying for
the partition in each index.  Results are filtered by the target hamming
distance and returned as a set.
