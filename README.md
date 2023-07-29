# TOML

This is a simple library for working with TOML files.

```rust
use toml::from_str;

let toml = from_str(r#"
[groceries]
fruit.apples=3
fruit.bananas=5
pasta.sauce="marinara"
pasta.noodles="spaghetti"
cash=true
"#).unwrap();

assert_eq!(toml["groceries"]["fruit"]["apples"].as_int(), 3);
assert_eq!(toml["groceries"]["fruit"]["bananas"].as_int(), 5);
assert_eq!(toml["groceries"]["pasta"]["sauce"].as_str(), "marinara");
assert_eq!(toml["groceries"]["pasta"]["noodles"].as_str(), "spaghetti");
assert_eq!(toml["groceries"]["cash"].as_bool(), true);
```
