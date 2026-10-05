use super::*;

fn populate(cache: &mut AlmaWeightCache, site: CallSiteId, length: usize) {
    assert!(
        cache
            .prepare(site, length, 0.85, 6.0, false, true)
            .is_none()
    );
    assert!(
        cache
            .prepare(site, length, 0.85, 6.0, false, true)
            .is_some()
    );
}

#[test]
fn cold_and_new_not_ready_sites_do_not_allocate_cache_roots() {
    let mut cache = AlmaWeightCache::default();
    assert!(
        cache
            .prepare(CallSiteId(1), 1_000_000_000, 0.85, 6.0, false, false)
            .is_none()
    );
    assert!(cache.entries.is_none());
    assert_eq!(cache.capacity(), 0);
    assert_eq!(cache.site_count(), 0);

    assert!(
        cache
            .prepare(CallSiteId(1), 4, 0.85, 6.0, false, true)
            .is_none()
    );
    let checkpoint = cache.clone();
    assert!(
        cache
            .prepare(CallSiteId(2), 1_000_000_000, 0.85, 6.0, false, false)
            .is_none()
    );
    assert!(Arc::ptr_eq(
        cache.entries.as_ref().unwrap(),
        checkpoint.entries.as_ref().unwrap()
    ));
    assert_eq!(cache.site_count(), 1);
    assert_eq!(cache.capacity(), 0);
}

#[test]
fn second_ready_key_builds_once_and_hits_share_root() {
    let mut cache = AlmaWeightCache::default();
    let site = CallSiteId(1);
    assert!(cache.prepare(site, 4, 0.85, 6.0, false, true).is_none());
    let pending = cache.clone();
    let first = cache.prepare(site, 4, 0.85, 6.0, false, true).unwrap();
    let kernel = first as *const AlmaWeights;
    let values = first.values().as_ptr();
    let sum = first.weight_sum().to_bits();
    assert_eq!(cache.capacity(), 4);
    assert_eq!(pending.capacity(), 0);
    assert!(!Arc::ptr_eq(
        cache.entries.as_ref().unwrap(),
        pending.entries.as_ref().unwrap()
    ));
    let checkpoint = cache.clone();
    let hit = cache.prepare(site, 4, 0.85, 6.0, false, true).unwrap();
    assert_eq!(hit as *const AlmaWeights, kernel);
    assert_eq!(hit.values().as_ptr(), values);
    assert_eq!(hit.weight_sum().to_bits(), sum);
    assert!(cache.prepare(site, 4, 0.85, 6.0, false, false).is_none());
    assert_eq!(cache.capacity(), 4);
    assert!(Arc::ptr_eq(
        cache.entries.as_ref().unwrap(),
        checkpoint.entries.as_ref().unwrap()
    ));
}

#[test]
fn changed_parameters_use_admission_and_preserve_checkpoint_kernel() {
    let mut cache = AlmaWeightCache::default();
    let site = CallSiteId(1);
    populate(&mut cache, site, 4);
    for (length, offset, sigma, floor) in [
        (4, 0.0, 6.0, false),
        (4, -0.0, 6.0, false),
        (4, -0.0, 7.0, false),
        (4, -0.0, 7.0, true),
        (3, -0.0, 7.0, true),
    ] {
        let checkpoint = cache.clone();
        let old_capacity = checkpoint.capacity();
        let old_weights = Arc::downgrade(
            checkpoint
                .entries
                .as_ref()
                .unwrap()
                .get(&site)
                .unwrap()
                .weights
                .as_ref()
                .unwrap(),
        );
        assert!(
            cache
                .prepare(site, length, offset, sigma, floor, true)
                .is_none()
        );
        assert_eq!(cache.capacity(), 0);
        assert_eq!(cache.site_count(), 1);
        assert_eq!(checkpoint.capacity(), old_capacity);
        assert!(old_weights.upgrade().is_some());
        assert!(
            cache
                .prepare(site, length, offset, sigma, floor, true)
                .is_some()
        );
        assert_eq!(cache.capacity(), length);
        drop(checkpoint);
        assert!(old_weights.upgrade().is_none());
    }
}

#[test]
fn not_ready_key_change_discards_only_its_site_and_releases_last_root() {
    let mut cache = AlmaWeightCache::default();
    let first = CallSiteId(1);
    let second = CallSiteId(2);
    populate(&mut cache, first, 8);
    populate(&mut cache, second, 4);
    let checkpoint = cache.clone();
    assert!(cache.prepare(first, 2, 0.85, 6.0, false, false).is_none());
    assert_eq!(cache.site_count(), 1);
    assert_eq!(cache.capacity(), 4);
    assert_eq!(checkpoint.capacity(), 12);
    let surviving = cache
        .entries
        .as_ref()
        .unwrap()
        .get(&second)
        .unwrap()
        .weights
        .as_ref()
        .unwrap();
    let old_surviving = checkpoint
        .entries
        .as_ref()
        .unwrap()
        .get(&second)
        .unwrap()
        .weights
        .as_ref()
        .unwrap();
    assert!(Arc::ptr_eq(surviving, old_surviving));

    let root = Arc::downgrade(cache.entries.as_ref().unwrap());
    let last_checkpoint = cache.clone();
    assert!(cache.prepare(second, 1, 0.85, 6.0, false, false).is_none());
    assert!(cache.entries.is_none());
    assert_eq!(cache.capacity(), 0);
    assert_eq!(cache.site_count(), 0);
    assert!(root.upgrade().is_some());
    drop(last_checkpoint);
    assert!(root.upgrade().is_none());
    assert_eq!(checkpoint.capacity(), 12);
}
