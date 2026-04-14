use std::slice;

fn main() {
    let ids: Vec<i32> = (0..10).collect();

    let ptr: *const i32 = ids.as_ptr();
    let len: usize = ids.len();

    // Call a function with raw ptr + len
    print_from_raw(ptr, len);

    // Call a closure with raw ptr + len
    let print_closure = |p: *const i32, l: usize| {
        // SAFETY: caller must ensure `p` is valid for `l` elements,
        // and the backing storage outlives this use.
        let slice: &[i32] = unsafe { slice::from_raw_parts(p, l) };
        println!("from closure: {:?}", slice);
    };

    print_closure(ptr, len);
}

fn print_from_raw(ptr: *const i32, len: usize) {
    // SAFETY: same contract – the caller guarantees that `ptr`
    // is valid for `len` elements, properly aligned, and lives
    // long enough for this slice to be used.
    let slice: &[i32] = unsafe { slice::from_raw_parts(ptr, len) };

    println!("from function: {:?}", slice);
}