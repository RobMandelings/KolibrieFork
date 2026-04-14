fn main() {
    // Our original Vec of IDs
    let ids: Vec<u32> = (0..10).collect();

    // 1) Safe slice: from index 5 to the end
    let tail: &[u32] = &ids[5..];
    println!("tail (safe slice) = {:?}", tail);

    // 2) Build a slice from a raw pointer + length (unsafe)
    //    Here we start at index 5 and take len = 3 elements.
    let start_index = 5;
    let len = 3;

    // Get a raw pointer to the first element of the Vec
    let base_ptr: *const u32 = ids.as_ptr();

    // Pointer to ids[start_index]
    let raw_ptr_to_start: *const u32 = unsafe { base_ptr.add(start_index) };

    // SAFETY: we must ensure that:
    // - raw_ptr_to_start is valid for `len` elements,
    // - ids is not dropped or mutated in a way that invalidates the pointer.
    let unsafe_slice: &[u32] = unsafe {
        std::slice::from_raw_parts(raw_ptr_to_start, len)
    };

    println!("unsafe_slice = {:?}", unsafe_slice);
}