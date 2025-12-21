use std::{collections::HashMap};

use frozen_map::map::{ SyncUnverifiedFrozenMap, SyncVerifiedFrozenMap, AtomicUnverifiedFrozenMap, AtomicVerifiedFrozenMap };



/*
 SyncUnverifiedFrozenMap  // lowest overhead //not thread safe // no key verification

 AtomicUnverifiedFrozenMap  // medium overhead // thread safe // no key verification

 SyncVerifiedFrozenMap    // higher overhead // not thread safe // key verification

 AtomicVerifiedFrozenMap    // highest overhead // thread safe // key verification
*/

fn main() {
    println!("Hello, world!");

    let keys = vec!["gamma", "alpha", "omega", "delta"];
    let mut values = vec![0, 1, 2, 3];



    
    //SyncUnverifiedFrozenMap  // lowest overhead // not thread safe // no key verification

    // Initialize Map 
    let mut su: SyncUnverifiedFrozenMap<&str, i32> = SyncUnverifiedFrozenMap::from_vec(keys.clone());

    // Load in value

    keys.iter().zip(values.iter()).for_each(|(key, val)| {
        let res = su.get(key);
        println!("res: {:?}", res);

        su.upsert(key, *val);

        let res = su.get(key);
        println!("res: {:?}", res);

    });




    //AtomicUnverifiedFrozenMap  // medium overhead // thread safe // no key verification

    let mut au: AtomicUnverifiedFrozenMap<&str, i32> = AtomicUnverifiedFrozenMap::from_vec(keys.clone());


    keys.iter().zip(values.iter().enumerate()).for_each(|(key, (idx, val))| {
        let res = au.get(key);
        println!("res: {:?}", res);

        au.upsert(key, *val);

        let res = au.get(key);
        println!("res: {:?}", res);

    });

    //SyncVerifiedFrozenMap    // higher overhead // no thread safe // key verification

    let mut sv: SyncVerifiedFrozenMap<&str, i32> = SyncVerifiedFrozenMap::from_vec(keys.clone());

    keys.iter().zip(values.iter().enumerate()).for_each(|(key, (idx, val))| {
        let res = sv.get(key);
        println!("res: {:?}", res);

        let _ = sv.upsert(key, *val);

        let res = sv.get(key);
        println!("res: {:?}", res);

    });

    //AtomicVerifiedFrozenMap    // highest overhead // thread safe // key verification
  
    let mut au: AtomicVerifiedFrozenMap<&str, i32> = AtomicVerifiedFrozenMap::from_vec(keys.clone());

    keys.iter().zip(values.iter().enumerate()).for_each(|(key, (idx, val))| {
        let res = au.get(key);
        println!("res: {:?}", res);

        au.upsert(key, *val).ok();

        let res = au.get(key);
        println!("res: {:?}", res);

    });

   






}






// verified hashmap - stores keys, values
// unverified hashmap - stores values
