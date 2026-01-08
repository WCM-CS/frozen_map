use frozen_map::map::FrozenMap;

fn main() {
    // std::thread::sleep(std::time::Duration::from_secs(12));

    let keys = vec!["gamma", "delta", "void", "bump"];

    // from_vec ~ Initialized

    let mut frozen_map: FrozenMap<&str, usize> = FrozenMap::from_vec(keys.clone());

    // upsert(k, v) ~ replace value if the key exists adn is not reaped
    let _ = frozen_map.upsert("gamma", 0);
    let _ = frozen_map.upsert("delta", 1);
    let _ = frozen_map.upsert("void", 2);
    let _ = frozen_map.upsert("bump", 3);

    // get(k) ~ Retreive V if it exists
    let k = frozen_map.get(&"gamma").unwrap();
    println!("{}", k);
    assert_eq!(*k, 0);

    // contains(k) ~ check if the key exists and is not reaped
    assert!(frozen_map.contains(&"gamma"));

    // drop_value(k) ~ drop the value per a given key
    let _ = frozen_map.drop_value(&"gamma");

    // contains_value(k) ~ check if the value is initialized
    assert!(!frozen_map.contains_value(&"gamma"));

    let k = frozen_map.get(&"gamma");
    println!("{:?}", k);
    assert_eq!(k, None);

    let _ = frozen_map.upsert("gamma", 4);

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




    let k = frozen_map.get(&"gamma").unwrap();
    println!("{}", k);




    let  i = frozen_map.get_mut(&"gamma");
    let y = i.unwrap();    
    *y += 1;

        let k = frozen_map.get(&"gamma").unwrap();
    println!("{}", k);


    let keys = vec!["gamma", "delta", "void", "bump"];
    let vals = vec![1, 2, 3, 4];

    let t: FrozenMap<&str, i32> = FrozenMap::unsafe_init(keys, vals);    

    t.iter_keys().for_each(|g| println!("{g}"));

    t.iter().for_each(|f| println!("{:?}", f));



}

// verified hashmap - stores keys, values
// unverified hashmap - stores values
