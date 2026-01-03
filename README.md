Frozen-Key HashMap

Build Features 
- SoA (struct of Array) memory layout for cache locality optimizations. 
- Map uses PHast+ hashing for the mphf index, created by: https://arxiv.org/pdf/2504.17918

Usage Features
- Keys are static but you can label keys as dead via a tombstone and also revive them.
- Values are dynamic and can be mutated or dropped during runtime.
- Key verification & concurrency support is optional.

```markdown
```rust

let keys = vec![
"gamma",
"delta",
"void",
"bump"
];

// from_vec ~ Initialized
let mut frozen_map: FrozenMap<&str, usize> = FrozenMap::from_vec(keys.clone());


// upsert(k, v) ~ replace value if the key exists adn is not reaped
let _ = frozen_map.upsert(&"gamma", 0);
let _ = frozen_map.upsert(&"delta", 1);
let _ = frozen_map.upsert(&"void", 2);
let _ = frozen_map.upsert(&"bump", 3);


// get(k) ~ Retreive V if it exists
let k = frozen_map.get(&"gamma").unwrap();
println!("{}", k);
assert_eq!(*k, 0);


// contains(k) ~ check if the key exists and is not reaped
assert_eq!(frozen_map.contains(&"gamma"), true);


// drop_value(k) ~ drop the value per a given key
let _ = frozen_map.drop_value(&"gamma");

// contains_value(k) ~ check if the value is initialized
assert_eq!(frozen_map.contains_value(&"gamma"), false);

let k = frozen_map.get(&"gamma");
println!("{:?}", k);
assert_eq!(k, None);



let _ = frozen_map.upsert(&"gamma", 4);

let k = frozen_map.get(&"gamma");
println!("{:?}", k);
assert_eq!(*k.unwrap(), 4);



// reap_key(k) ~ reap the key (logical deletion)
let _ = frozen_map.reap_key(&"gamma");

let k = frozen_map.get(&"gamma");
println!("{:?}", k);
assert_eq!(k, None);


// rehydrate_key(k) ~ rehydrate key (logical append)
let _ = frozen_map.rehydrate_key(&"gamma");

let k = frozen_map.get(&"gamma");
println!("{:?}", k);
assert_eq!(*k.unwrap(), 4);


// iter() ~ iterate over the keys and value pairs, this will exclude unititialized values
frozen_map.iter().for_each(|(k, v)| {
    println!("Key: {k}, Value: {v}");
});

// iter_keys() ~ iterate over all the keys even if they are reaped
frozen_map.iter_keys().for_each(|k| {
    println!("Key: {k}");
});
