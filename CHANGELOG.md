# Changelog

All notable changes to this project will be documented in this file.

This project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.0.0] - 2023-11-07

A quick note to any packages. The generated shell completions and man page are
now in the `gen` directory of the repo. They're also included in the pre-built
release artifacts on the releases page.

### Improvements

- #115 Do not replace symlink with output file (@SimplyDanny)
  - Fixes an issue where a symlink would be replaced with a regular file
- #124 Fix tests (@Linus789)
  - Removed displaying the file path when passing the `--preview` flag and fixed
    how text coloring was handled in tests

### Breaking

- #192 Rename `--string-mode` to `--fixed-strings` (@CosmicHorrorDev)
  - Renamed `-s` `--string-mode` to `-f` `--fixed-strings` to better match
    similar tools
  - `-s` and `--string-mode` will still continue to work for backwards
    compatibility, but are no longer documented
- #258 Error on `$<num><non_num>` capture replacement names (@CosmicHorrorDev)
  - Previously when you tried to use a numbered capture group right before some
    letters in the replacement text (e.g. `$1foo`) then it would be considered
    the impossible-to-use `1foo` capture. The correct way to pass the numbered
    capture group in this case would be to surround the number with curly braces
    like so `${1}foo`. The error just detects this case and informs the user of
    the issue

### Docs

- #93 Add note about in-place file modification to --help output (@jchook)
- #148 Doc: nitpick `--` has no special meaning to shells (@hexagonrecursion)
- #181 Fix man page -f flag help text (@ulope)
  - Fixed copy-pasted text in the man page's `-f` flag's help text
- #186 Improve error message for failed replacements (@CosmicHorrorDev)
- #187 Freshen up README (@CosmicHorrorDev)
  - Added a repology badge to document different installation methods
  - Improved the formatting of the benchmarks
- #207 Documenting `$` escape (@yahkbar)
  - Adds a section in the README that covers that `$$` is a literal `$` in the
    replacement text
- #227 Improve README readability (@vassudanagunta)
  - Various formatting improvements
- #231 Use `clap_mangen` and `roff` to generate manpage (@nc7s)
  - This change ensures the man page contents stay in sync with the CLI
    automatically, and fixes some broken rendering of the existing manpage
- #243 Exclude unsupported packages from the repology badge (@CosmicHorrorDev)

### Pre-built Releases

- (11295fb) Add ARM target (@chmln)
  - Added the `arm-unknown-linux-gnueabihf` target to CI and releases
- #114 Adding `aarch64-apple-darwin` target (@yahkbar)
- #143  Fix paths to release binary in "publish" action (@skrattaren)
- #179 Build Adjustments (@yahkbar)
  - `strip`ed release binaries and added the `aarch64-ubuntu-linux-musl` target
- #204 Adding `armv7-unknown-linux-gnueabihf` target (@yahkbar)
  - Added the `armv7-unknown-linux-gnueabihf` target to the list of targets to
    build in CI and for each release
- #205 Resolving broken `aarch64-apple-darwin` tests (@yahkbar)
  - Switched `aarch64-apple-darwin` to only try building the executable without
    running the tests since there seems to be no easy way to test for ARM Apple
    targets
- #206 Adding Windows builds back (@yahkbar)
  - Added the `x86_64-pc-windows-gnu` and `x86_64-windows-musl` targets back to
    the list of targets to build in CI and for each release

### Internal

- #118 Fix master (@SimplyDanny)
  - Fixes several cross-compilation issues that effected different targets in CI
- #182 `cargo update` (@CosmicHorrorDev)
  - Bumps dependencies to their latest compatible versions
- #183 Switch `memmap` -> `memmap2` (@CosmicHorrorDev)
  - Switches away from an unmaintained crate
- #184 Add editor config file matching rustfmt config (@CosmicHorrorDev)
  - Adds an `.editorconfig` file matching the settings listed in the
    `.rustfmt.toml` file
- #185 Fix warnings and clippy lints (@CosmicHorrorDev)
- #188 Switch `atty` for `is-terminal` (@CosmicHorrorDev)
  - Switches away from an unmaintained crate
- #189 Replace structopt with clap v4 (@CosmicHorrorDev)
  - Switches away from a defacto deprecated crate
- #190 Change how all shell variants are expressed (@CosmicHorrorDev)
  - Tiny tidying up PR
- #196 Move generating static assets to a `cargo-xtask` task (@CosmicHorrorDev)
  - Moves the generation of the man page and shell completions from a build
    script to a [`cargo-xtask`](https://github.com/matklad/cargo-xtask) task
- #197 Add a release checklist (@CosmicHorrorDev)
- #209 Dependency updates (@yahkbar)
- #235 Update generated assets (@CosmicHorrorDev)
- #236 Tone down dependabot (@CosmicHorrorDev)
- #245 Update sd to 2021 edition (@CosmicHorrorDev)
  - Updates `sd` to the Rust 2021 edition
- #248 Misc Cargo.toml tweaks (@CosmicHorrorDev)
  - Switches to use workspace edition and dependencies where appropriate
- #249 Resolve CI warnings (@CosmicHorrorDev)
  - Switched from `actions-rs` actions to `dtolnay@rust-toolchain`
  - Switched from using `::set-output` to `$GITHUB_ENV`
- #251 Update dependencies (@CosmicHorrorDev)
- A lot of sad CI tweaking:
    - #252 Fix build target usage in CI (@CosmicHorrorDev)
    - #253 Improve publishing CI job (@CosmicHorrorDev)
    - #256 More CI tweaks (@CosmicHorrorDev)
    - #257 Fix publish action (@CosmicHorrorDev)
- #267 Rework the replacements flag (@CosmicHorrorDev)
- #269 Make modified text blue instead of green (@CosmicHorrorDev)
- #271 Fix release checklist indentation (@CosmicHorrorDev)
- #272 Remove outdated release checklist step (@CosmicHorrorDev)
- #274 Prepare 1.0.0-beta.0 release (@CosmicHorrorDev)
- #275 Update `sd` version in lockfile (@CosmicHorrorDev)

## (History listed in here is missing from v0.6.3 - v0.7.6)

## [0.6.2]

- Fixed pre-allocated memmap buffer size
- Fixed failing tests

## [0.6.0] - 2019-06-15

### Improvements

- `sd` now uses memory-mapped files, allowing replacement on files of any size
- `-p`/`--preview` flag is now added to preview changes
  - as of right now, results are simply emitted to stdout
  - in a future version, the output will be changed to contain only relevant information
- a `w` regex flag is added to match full words only, e.g. `sd -f w foo bar file.txt`

### Deprecations

- `--in-place` is now deprecated and assumed whenever a list of files is given

## [0.5.0] - 2019-02-22

### Improvements

- Windows support (thanks to @ErichDonGubler)

## [0.4.2] - 2019-01-02

### Improvements

- Support for unicode and special characters (like `\n`) in replacement expressions
  - Only in regex mode
- Fixed edge-cases when replacing content containing unescaped characters

## [0.4.1] - 2019-01-01

### Improvements

- Significant performance improvements (see benchmarks in README)

## [0.4.0] - 2018-12-30

### Added

- Option to set regex flags via `-f` or `--flags`:
  - `m` (multi-line)
  - `c` (case-sensitive)
  - `i` (case-insensitive)
- Smart case-sensitivity is used by default with regular expressions

### Improvements

- You may now pass multiple files to `sd`
  - this is now valid: `sd -i "\n" "," *.txt`

## [0.3.0] - 2018-12-29

**Breaking Change**: the `-i`/`--input` is revamped per [#1](https://github.com/chmln/sd/issues/1). The file path now comes at the end, instead of `-i`. 

Transforming the file in-place:
- Before: `sd -s 'str' '' -i file.txt'`
- Now: `sd -i -s 'str' '' file.txt`
- Future: `sd -i -s 'str' '' *.txt`

To reflect this change, `--input` is also renamed to `--in-place`. This is the first and most likely the last breaking change in this project.

### Improvements

- Files are now written to [atomically](https://github.com/chmln/sd/issues/3)


