use std::{collections::HashMap, hash::Hash};

use frozen_map::VerifiedFrozenMap;


fn main() {
    println!("Hello, world!");

    let keys = vec!["gamma", "alpha", "omega", "Walker"];
    let mut values = vec![0, 1, 2, 3];


    let mut my_map = HashMap::new();

    for (i, key) in keys.iter().enumerate() {
        my_map.insert(key, values[i]);
    }


    let answer = my_map.get(&"Walker");






    let mut jj: VerifiedFrozenMap<&str, u32> = VerifiedFrozenMap::from_vec(keys.clone());



    

    for (idx, k) in keys.iter().enumerate() {
        let value = jj.contains(&keys[idx]);
        println!("Value: {:?}", value);
    }

    for (idx, k) in keys.iter().enumerate() {
        jj.upsert(k, values[idx]);
        let value = jj.get(k);
        println!("Value: {:?}", value);
    }

    for (idx, k) in keys.iter().enumerate() {
        jj.upsert(k, 32);
        let value = jj.get(k);
        println!("Value: {:?}", value);
    }


    //values.pop();
    let new_jj = VerifiedFrozenMap::unsafe_init(keys.clone(), values.clone()).unwrap();

    for (idx, k) in keys.iter().enumerate() {
        let value = new_jj.get(k);
        println!("Values: {:?}", value);
    }




  //  let r = VerifiedFrozenMap
}






// verified hashmap - stores keys, values
// unverified hashmap - stores values
