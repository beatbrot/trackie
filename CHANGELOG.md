# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- New `status` command that shows which project is currently tracked and for how long (thanks [/u/radarevada](https://www.reddit.com/r/programming/comments/ozxrd4/trackie_is_a_private_daemonless_time_tracker/h84sukr))
  - This enables various shell integrations e.g. for [starship](https://starship.rs) or [Oh My Posh](https://ohmyposh.dev/).
- New `resume` command starts tracking the last tracked project
- A custom path for the `trackie.json` file can be specified via the `TRACKIE_CONFIG` environment variable.
- Add possibility to dump the report as JSON via the `--json` flag

### Changed
- The `trackie.json` is now saved in `%APPDATA%/trackie/trackie.json` in Windows and `$XDG_DATA_HOME/trackie/trackie.json` on Unix. 
  - Automatic migration code was added.

## [0.1.0] - 2021-08-07

- Initial release

[Unreleased]: https://github.com/beatbrot/trackie/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/beatbrot/trackie/releases/tag/v0.1.0
