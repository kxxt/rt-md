// Archimedes' Method
// Credit: https://rust-analyzer.github.io/blog/2020/12/04/measuring-memory-usage-in-rust.html

use dns_exf_detect::method::bfcms::Bfcms;

fn memory_usage() -> usize {
    unsafe { nix::libc::mallinfo() }.uordblks as usize
}

fn main() {
    let stack = size_of::<Bfcms>();
    let mut heap = usize::MAX;
    for _ in 0..10 {
        let before = memory_usage();
        let mut bfcms = Bfcms::new(0).unwrap();
        for _ in 0..100 {
            bfcms
                .top
                .push("jadwfkagfjkhajfkgjkhafghakjhgafkg.com".to_string(), &0);
        }
        let after = memory_usage();
        let delta = after - before;
        heap = heap.min(delta);
    }
    println!("Memory usage per bfcms:");
    println!("Stack: {stack} bytes");
    println!("Heap: {heap} bytes");
    println!("Total: {} bytes", heap + stack);
}
