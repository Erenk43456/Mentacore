extern crate alloc;

use alloc::{
    boxed::Box,
    string::String,
    vec::Vec,
};

use super::framework::TestRunner;

pub fn run(runner: &mut TestRunner) {
    runner.run(
        b"heap::basic_allocation",
        test_basic_allocation,
    );

    runner.run(
        b"heap::multi_page_allocation",
        test_multi_page_allocation,
    );
}

fn test_basic_allocation() -> bool {
    let mut values = Vec::new();

    values.push(10u64);
    values.push(20u64);
    values.push(30u64);

    let boxed = Box::new(1234u64);

    let text = String::from("Mentacore heap");

    values.len() == 3
        && values[0] == 10
        && values[1] == 20
        && values[2] == 30
        && *boxed == 1234
        && text.as_bytes() == b"Mentacore heap"
}

fn test_multi_page_allocation() -> bool {
    let mut multi_page = Vec::with_capacity(2048);

    for i in 0..2048u64 {
        multi_page.push(i);
    }

    multi_page.len() == 2048
        && multi_page[0] == 0
        && multi_page[1024] == 1024
        && multi_page[2047] == 2047
}