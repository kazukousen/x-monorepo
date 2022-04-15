use crate::{println, virtio};

pub fn run_tests() {
    type TestSuite = fn() -> &'static [(&'static str, fn())];
    let suites = [("virtio", virtio::tests::tests as TestSuite)];

    for (name, suite) in &suites {
        let tests = suite();
        println!("  {}", name);
        for (name, f) in tests {
            println!("      {}", name);
            f();
        }
    }
    println!("\x1b[0;32mall tests passed!\x1b[0m");
}
