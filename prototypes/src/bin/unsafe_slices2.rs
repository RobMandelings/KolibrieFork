use std::slice;

fn main() {
    // Pretend this comes from FFI / some lower-level API:
    let data = [10, 20, 30, 40, 50];
    let ptr: *const i32 = data.as_ptr();
    let len: usize = data.len();

    // SAFETY: we *know* `ptr` points to `len` valid i32s (the `data` array),
    // and `data` lives at least as long as `slice` is used.
    let slice: &[i32] = unsafe { slice::from_raw_parts(ptr, len) };

    println!("slice = {:?}", slice);
}