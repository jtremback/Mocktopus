#![feature(extern_absolute_paths, proc_macro)]
#![::mocktopus::macros::mockable]

extern crate mocktopus;

use mocktopus::mocking::{Mockable, MockResult};

fn function() -> &'static str {
    "not mocked"
}

#[test]
fn when_not_mocked_then_runs_normally() {
    assert_eq!("not mocked", function());
}

#[test]
fn when_mocked_then_runs_mock() {
    function.mock_safe(|| MockResult::Return("mocked"));

    assert_eq!("mocked", function());
}
