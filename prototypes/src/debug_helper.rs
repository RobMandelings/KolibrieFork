pub fn push_tracking<T>(v: &mut Vec<T>, value: T, tag: &str) {
    let old_cap = v.capacity();
    v.push(value);
    let new_cap = v.capacity();

    if new_cap != old_cap {
        println!(
            "[{}] Vec grew: len={} old_cap={} new_cap={}",
            tag,
            v.len(),
            old_cap,
            new_cap
        );
    }
}