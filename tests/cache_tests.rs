use realm::cache::CacheManager;
use serde::{Deserialize, Serialize};
use std::thread;
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct TestData {
  value: String,
  count: u32,
}

#[test]
fn test_cache_manager_new() {
  let result = CacheManager::new();
  assert!(result.is_ok(), "Failed to create CacheManager");

  let manager = result.unwrap();
  assert!(manager.cache_dir().exists(), "Cache directory should exist");
}

#[test]
fn test_cache_set_and_get() {
  let manager = CacheManager::new().unwrap();

  let test_data = vec!["3.12.6".to_string(), "3.11.9".to_string()];

  let result = manager.set("test_versions", &test_data);
  assert!(result.is_ok(), "Failed to set cache");

  let cached: Option<Vec<String>> = manager.get("test_versions").unwrap();
  assert!(cached.is_some(), "Cache should return data");
  assert_eq!(cached.unwrap(), test_data);

  let _ = manager.clear("test_versions");
}

#[test]
fn test_cache_get_nonexistent() {
  let manager = CacheManager::new().unwrap();

  let cached: Option<Vec<String>> = manager.get("nonexistent_key").unwrap();
  assert!(cached.is_none(), "Should return None for nonexistent key");
}

#[test]
fn test_cache_expiration() {
  let manager = CacheManager::new().unwrap().with_ttl(1); // 1 second TTL

  let test_data = vec!["test".to_string()];
  manager.set("expiring_data", &test_data).unwrap();

  // Should exist immediately
  let cached: Option<Vec<String>> = manager.get("expiring_data").unwrap();
  assert!(cached.is_some(), "Cache should exist immediately");

  // Wait for expiration
  thread::sleep(Duration::from_secs(2));

  // Should be expired
  let expired: Option<Vec<String>> = manager.get("expiring_data").unwrap();
  assert!(expired.is_none(), "Cache should be expired after TTL");

  let _ = manager.clear("expiring_data");
}

#[test]
fn test_cache_get_stale() {
  let manager = CacheManager::new().unwrap().with_ttl(1); // 1 second TTL

  let test_data = vec!["stale".to_string()];
  manager.set("stale_data_test", &test_data).unwrap();

  // Wait for expiration
  thread::sleep(Duration::from_secs(2));

  // get() should return None
  let expired: Option<Vec<String>> = manager.get("stale_data_test").unwrap();
  assert!(
    expired.is_none(),
    "get() should return None for expired data"
  );

  // get_stale() should still return data
  let stale: Option<Vec<String>> = manager.get_stale("stale_data_test").unwrap();
  assert!(
    stale.is_some(),
    "get_stale() should return expired data, but got None"
  );
  assert_eq!(stale.unwrap(), test_data);

  let _ = manager.clear("stale_data_test");
}

#[test]
fn test_cache_clear() {
  let manager = CacheManager::new().unwrap();

  let test_data = vec!["data".to_string()];
  manager.set("clear_test", &test_data).unwrap();

  // Verify it exists
  let cached: Option<Vec<String>> = manager.get("clear_test").unwrap();
  assert!(cached.is_some());

  // Clear it
  manager.clear("clear_test").unwrap();

  // Should be gone
  let cleared: Option<Vec<String>> = manager.get("clear_test").unwrap();
  assert!(cleared.is_none(), "Cache should be cleared");
}

#[test]
#[ignore] // Run separately to avoid interfering with parallel tests: cargo test -- --ignored
fn test_cache_clear_all() {
  let manager = CacheManager::new().unwrap();

  // Use unique keys to avoid interfering with other tests
  let key1 = "clear_all_item1";
  let key2 = "clear_all_item2";
  let key3 = "clear_all_item3";

  // Set multiple cache entries
  manager.set(key1, vec!["a".to_string()]).unwrap();
  manager.set(key2, vec!["b".to_string()]).unwrap();
  manager.set(key3, vec!["c".to_string()]).unwrap();

  // Verify they exist
  assert!(manager.get::<Vec<String>>(key1).unwrap().is_some());
  assert!(manager.get::<Vec<String>>(key2).unwrap().is_some());
  assert!(manager.get::<Vec<String>>(key3).unwrap().is_some());

  // Clear all
  manager.clear_all().unwrap();

  // All should be gone
  assert!(manager.get::<Vec<String>>(key1).unwrap().is_none());
  assert!(manager.get::<Vec<String>>(key2).unwrap().is_none());
  assert!(manager.get::<Vec<String>>(key3).unwrap().is_none());
}

#[test]
fn test_cache_with_complex_data() {
  let manager = CacheManager::new().unwrap();

  let test_data = TestData {
    value: "test".to_string(),
    count: 42,
  };

  manager.set("complex_data", &test_data).unwrap();

  let cached: Option<TestData> = manager.get("complex_data").unwrap();
  assert!(cached.is_some());
  assert_eq!(cached.unwrap(), test_data);

  let _ = manager.clear("complex_data");
}

#[test]
fn test_cache_overwrite() {
  let manager = CacheManager::new().unwrap();

  let data1 = vec!["first".to_string()];
  let data2 = vec!["second".to_string()];

  manager.set("overwrite_test", &data1).unwrap();

  let cached: Option<Vec<String>> = manager.get("overwrite_test").unwrap();
  assert_eq!(cached.unwrap(), data1);

  // Overwrite
  manager.set("overwrite_test", &data2).unwrap();

  let updated: Option<Vec<String>> = manager.get("overwrite_test").unwrap();
  assert_eq!(updated.unwrap(), data2);

  let _ = manager.clear("overwrite_test");
}
