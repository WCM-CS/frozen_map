Frozen-Key HashMap

Build Features 
- SoA (struct of Array) memory layout for cache locality optimizations. 
- Map uses PHast+ hashing for the mphf index, created by: https://arxiv.org/pdf/2504.17918

Usage Features
- Keys are static but you can label keys as dead via a tombstone and also revive them.
- Values are dynamic and can be mutated or dropped during runtime.
- Key verification & concurrency support is optional.

Map Options: 
- SyncUnverifiedFrozenMap  // lowest overhead // not thread safe // no key verification
- AtomicUnverifiedFrozenMap  // medium overhead // thread safe // no key verification
- SyncVerifiedFrozenMap    // higher overhead // not thread safe // key verification
- AtomicVerifiedFrozenMap    // highest overhead // thread safe // key verification

Behavior
- Unverified map
    Do not store the keys. You can only use keys that were used to build the map. Passingin invalid key is undefined behavior.
- Verified maps
    Store the keys intervally. You can safely pass any key into the methods. Invalid keys will return None or an error and block invalid mutations. 
- Sync maps
    Use BaybeUninit wrappers over values and a bloom filter. Low overhead and mmery usage, but not thread safe. Best for single threaded use or wrapped with a atomic rwlock.
- Atomic maps
    Use atomic pointers for values. Thread safe and enables hot swapping of values. Extra overhead occurs from atomic reference counting. Best for multithreaded or async environments. 

Important Notes for Memory Management: 
- Keys are stored in a bump arena allocator.
- The arena does not call drop automatically.
- If your keys require drop, then serialize it to a slice before inseting it itno the frozen map.
- If your keys require drop allwasy build the map from the serialized keys. 

Original goal
- Build a lightweight static map using a mphf that initializes very fast. 
