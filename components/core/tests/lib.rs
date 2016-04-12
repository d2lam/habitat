// Copyright:: Copyright (c) 2015-2016 The Habitat Maintainers
//
// The terms of the Evaluation Agreement (Habitat) between Chef Software Inc.
// and the party accessing this file ("Licensee") apply to Licensee's use of
// the Software until such time that the Software is made available under an
// open source license such as the Apache 2.0 License.

extern crate habitat_core as hcore;
extern crate time;

use std::env;
use std::fs;

// call a closure in a loop until it returns Ok(()),
// or the 30 second timeout
pub fn wait_until_ok<F>(some_fn: F) -> bool
    where F: Fn() -> Result<(), hcore::error::Error>
{
    let wait_duration = time::Duration::seconds(30);
    let current_time = time::now_utc().to_timespec();
    let stop_time = current_time + wait_duration;
    while time::now_utc().to_timespec() < stop_time {
        if let Ok(_) = some_fn() {
            return true;
        }
    }
    false
}

#[test]
fn generate_key_revisions_test() {
    let key_dir = "/tmp/habitat_test_keys";
    let _ = fs::remove_dir_all(&key_dir);
    fs::create_dir_all(&key_dir).unwrap();

    // override the location where Habitat wants to store keys
    env::set_var("HAB_CACHE_KEY_PATH", &key_dir);

    let test_key_name = "habitat123";

    // there aren't any keys, but it should crash. It should
    // return an empty Vec
    match hcore::crypto::get_key_revisions(test_key_name) {
        Ok(revs) => assert!(revs.len() == 0),
        Err(e) => panic!("Can't get key revisions {}", e),
    }

    // generate a single key
    if let Err(e) = hcore::crypto::generate_origin_sig_key(test_key_name) {
        panic!("Error generating keys {}", e)
    };

    // we should only see a single revision
    let first_rev = match hcore::crypto::get_key_revisions(test_key_name) {
        Ok(revs) => {
            assert!(revs.len() == 1);
            revs.first().unwrap().clone()
        }
        Err(e) => panic!("Can't get key revisions {}", e),
    };

    // We can't generate more than 1 key with the same name per second,
    // otherwise, the keys would have the same revision. Call
    // generate_origin_sig_key in a loop, and wait until it returns Ok(())
    // we generate another key to see if get_key_revisions() returns 2
    if !wait_until_ok(|| hcore::crypto::generate_origin_sig_key(test_key_name)) {
        panic!("Failed to generate another key after 30 seconds");
    }

    let second_rev = match hcore::crypto::get_key_revisions(test_key_name) {
        Ok(revs) => {
            assert!(revs.len() == 2);
            revs.first().unwrap().clone()
        }
        Err(e) => panic!("Can't get key revisions {}", e),
    };
    assert!(first_rev != second_rev);
}
