# Change Log: Jaffi

All notable changes to this project will be documented in this file.
This project adheres to [Semantic Versioning](https://semver.org/).

All notes should be prepended with the location of the change, e.g. `(jaffi)` or `(jaffi_support)`.

## 0.2.0

### Added

- Wrap Java Exceptions and also throw Exceptions on panics #9
- Github actions for testing #4

### Fixed

- dedup method names in Rust especially for overloaded methods from Java #8

### Changed

- Moved from `TinyTemplate` to use proc_macro tools, e.g. `quote` #3
- Rewrite Java syntactic names to Rust style using `heck` #5
- Escape Rust keywords as necessary when Java names are invalid #6

## 0.1.0

### Added

- Basic template of Java -> Rust support
- Initial Commit!
