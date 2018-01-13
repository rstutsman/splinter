This directory contains some ideas for some benchmark tests for the Sandstorm key-val store.

Algorithms contains some python code that describe programs that can be used to time Sandstorm.

*sequential_read.py*
Performs something like a linked list traversal from beginning to end, with each next key being
discovered from the value of the current key in the key-val store.

*hot_key_read.py*
Out of a very large pool of keys, performs ~1e9 reads of the same ~1e3 keys. A good indicator of
the quality of caching in the system. Might be interesting to change the sizes involved here
to push the limits of the cache.

More benchmark test programs should be written to test performance of writes, and also performance
in parallel scenarios.

DynamoDB, BerkelyDB, or Reddis will be good to run this benchmark suite against as well as 
Sandstorm.

