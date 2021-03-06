![logo](https://raw.githubusercontent.com/CodeSandwich/mocktopus/master/logo.png)

Mocking framework for Rust (currently only nightly). See documentation for more.

```rust
#[mockable]
mod hello_world {
    pub fn world() -> &'static str {
        "world"
    }

    pub fn hello_world() -> String {
        format!("Hello {}!", world())
    }
}

#[test]
fn mock_test() {
    hello_world::world.mock_safe(|| MockResult::Return("mocking"));

    assert_eq!("Hello mocking!", hello_world::hello_world());
}
```
