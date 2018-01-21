use mocktopus_aliased::macros::*;

#[mockable]
mod another;

pub use self::another::function;
