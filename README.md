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

```markdown
```rust

// Step One:Prepare values (only keys are needed to build the map)
let keys: Vec<&str> = vec!["gamma", "alpha", "omega", "delta"];

// Step 2: Build the FrozenMap you selected, here I'm using the Atomic Verified version. They all habe the same build api, some maps have more methods than others though.
let frozen_map: AtomicVerifiedFrozenMap<&str, u32> = AtomicVerifiedFrozenMap::from_vec(keys);

// step 3: Now your map is built, you can load in the keys and query it as needed

// Upsert a value:
// note for the atomic version this can return error so you should unwrap with caution since if you passed in a invalid key it will panic, 
// while the unverified version would return an invalid response for a invalid key and not panic
frozen_map.upsert( "gamma", 99).ok(); 

// Access the value based on the key, will return none if key does not exist or if the key is dead
if let Some(val) = frozen_map.get(&"gamma") {
    println!("value: {:?}", val); // assert it is 99
    assert_eq!(99, *val);
}

// update / ovwerite the value
frozen_map.upsert( "gamma", 1).ok();

if let Some(val) = frozen_map.get(&"gamma") {
    println!("new value: {:?}", val);
    assert_eq!(1, *val);
}

// drop the value
let r = frozen_map.drop_value(&"gamma");
println!("Dropping the value: {:?}", r);
//assert_error

let null_res = frozen_map.get(&"gamma");
println!("value get request: {:?}", null_res);




frozen_map.upsert( "gamma", 1).ok();
let null_res = frozen_map.get(&"gamma");
println!("before key killing: {:?}", null_res);


// kill the key 
// Note reaping keys adds a bit to thier tombstone but this does not delete their correlating value
// the value it will be unaccessible if the key is dead, you can drop the value for that key if you want 
let reap_res = frozen_map.reap_key(&"gamma");

let null_res = frozen_map.get(&"gamma");
println!("after key killing: {:?}", null_res);


// contains:: method exists for verified maps only
let contains_true = frozen_map.contains(&"delta");
let contains_false = frozen_map.contains(&"gamma");
assert_eq!(contains_true, true);
assert_eq!(contains_false, false);

let hydrate_res = frozen_map.rehydrate(&"gamma");
let contains_hydrate = frozen_map.contains(&"gamma");
assert_eq!(contains_hydrate, true);

assert_eq!(frozen_map.len(), 4);

// -----------------------------------------------------
// Example for workign with heap allocated keys, where K is a vector of &str
// collect your starting heap allocated vec of K 

let keys_heap = vec![vec!["green", "blue", "red"], vec!["yellow", "purple"], vec!["orange"]];

//  function for encoding a vec, or use bincode or other
fn encode_vec(v: &[&str]) -> Vec<u8> {
    let mut out = Vec::new();
    for s in v {
        let bytes = s.as_bytes();
        let len = bytes.len() as u32;
        out.extend_from_slice(&len.to_le_bytes());
        out.extend_from_slice(bytes);
    }
    out
}


// Serialize each Vec<&str> to Vec<u8> using bincode
let serialized_keys: Vec<Vec<u8>> = keys_heap
    .iter()
    .map(|v| encode_vec(&v[..]))
    .collect();

// Convert to slices to pass into SyncUnverifiedFrozenMap
let key_refs: Vec<&[u8]> = serialized_keys.iter().map(|v| v.as_slice()).collect();

let mut frozen_map2: SyncUnverifiedFrozenMap<&[u8], u32> = SyncUnverifiedFrozenMap::from_vec(key_refs.clone());

let new_vals = [32, 33, 34];

for (i, k) in key_refs.iter().enumerate() {
    frozen_map2.upsert(k, new_vals[i]);
    let res = frozen_map2.get(k);
    println!("{:?}", res);
    assert_eq!(*res.unwrap(), new_vals[i]);
}