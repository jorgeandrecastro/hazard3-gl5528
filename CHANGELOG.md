# Changelog

All notable changes to this project are documented in this file.

## [0.2.2] - Unreleased

### Changed
- Renamed the package and library to `hazard3-gl5528` and `hazard3_gl5528` in the independent Hazard3 Async ecosystem.
- Made `hazard3-rp` the default HAL for RP235xa, with upstream `embassy-rp` as an alternative.
- Added explicit HAL and chip selection; `rp235x` remains an alias for `rp235xa`.
- Documented migration and corrected the ADC resolution and voltage divider formula.

# [0.2.0] - 2026-05-02

- Replaced compatibility ranges with explicit versions for more stable and reproducible builds.

### Dependencies

```toml
[dependencies.embassy-rp]
version = "0.10"
features = ["unstable-pac"]

```


# [0.1.3] - 2026-04-21
### Changed
- **README overhaul**: Added details and clearer documentation.
- **Cargo.toml optimization**: Expanded compatibility with `embassy-rp` (v0.4 to v0.10+).
- **RP235x support**: Documented the Raspberry Pi Pico 2 feature in the README.

## [0.1.2] - 2026-04-21

### Added
- Official support for the **Raspberry Pi Pico 2 (RP235x)** through the `rp235x` feature.
- Updated documentation to include RP235x compatibility.

### Changed
- **Dependencies**: Expanded the version range for `embassy-rp` (`>=0.4, <0.11`).
- This change allows recent crates.io versions (v0.10.0+) without dependency conflicts.
- Updated the internal RP235x feature to `rp235xa` to match Embassy v0.10 conventions.

### Fixed
- Resolved "links" conflicts when using the driver with a recent Embassy version.