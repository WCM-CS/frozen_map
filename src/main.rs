use std::{collections::HashMap, time::Instant};

use frozen_map::map::{ SyncUnverifiedFrozenMap, SyncVerifiedFrozenMap, AtomicUnverifiedFrozenMap, AtomicVerifiedFrozenMap };


/*
 SyncUnverifiedFrozenMap  // lowest overhead //not thread safe // no key verification

 AtomicUnverifiedFrozenMap  // medium overhead // thread safe // no key verification

 SyncVerifiedFrozenMap    // higher overhead // not thread safe // key verification

 AtomicVerifiedFrozenMap    // highest overhead // thread safe // key verification
*/

fn main() {

    // phast hash functio  by itself 100k keys = 80 seconds

    
    // Step One:Prepare values (only keys are needed to build the map)
    //let keys: Vec<&str> = vec!["gamma", "alpha", "omega", "delta"];

    let start = Instant::now();

    // Step 2: Build the FrozenMap you selected, here I'm using the Atomic Verified version. They all habe the same build api, some maps have more methods than others though.
    //let frozen_map: AtomicVerifiedFrozenMap<&str, u32> = AtomicVerifiedFrozenMap::from_vec(keys);

    let n = 100_000_000;
    //let mut key_vec = vec![];
      let mut key_storage: Vec<Vec<u8>> = Vec::with_capacity(n);
   

    // 100 Million String keys in the map no values 

    // Atomic verified 
    // 19.2 seconds to build
    // 12.2gb max then 9.8gb after build is completed
    
    // sync verified
    // 12.4 seconds
    // 8.2 for max build, 5.8gb final

//-------------------------------//

    // atomic unverified 
    // 6.45 seconds to build
    // 6.4, 4.2 gb final


    // sync unverified
    // 6.5 seconds
    // 6.6gb max, 3.4gb final build





    // 100 Million integer keys in the map no values 

    // Atomic verified 
    // 4.31 seconds to build
    // 1.5gb max then 1.3gb after build is completed
    
    // sync verified
    // 3.67 seconds
    // 1.4gb for max build, 470mb final

//-------------------------------//

    // atomic unverified 
    // 2.08 seconds to build
    // 1.3gb max, 860mb final


    // sync unverified
    // 1.72 seconds
    // 1.4gb max, 70mb final build



    println!("Getting keys ready");
    for i in 0..n {
        let bytes = i.to_string().into_bytes();
        key_storage.push(bytes);
    }
    let key_slices: Vec<&[u8]> = key_storage.iter().map(|v| v.as_slice()).collect();

// key slices can be created temporarily when needed
    //let key_slices: Vec<&[u8]> = key_storage.iter().map(|v| v.as_slice()).collect();

    println!("Building map");
    let mut frozen_map: SyncVerifiedFrozenMap<&[u8], u32> = SyncVerifiedFrozenMap::from_vec(key_slices);

    
    let end = start.elapsed();
    println!("Time to build frozen map with 100M keys: {:?}", end);

    frozen_map.upsert("378".as_bytes(), 1);

    let y = frozen_map.get(&"378".as_bytes()).unwrap();
    println!("{}", y);

   // std::thread::sleep(std::time::Duration::from_secs(12));

    





    






}






// verified hashmap - stores keys, values
// unverified hashmap - stores values
