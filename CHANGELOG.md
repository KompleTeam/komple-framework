# Changelog

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- **Custom Module Template** for kickstarting development of custom modules. This contract includes the common functions and messages found in the framework modules. ([#65](https://github.com/KompleTeam/komple-framework/pull/65))
- **Custom Permission Template** for kickstarting development of custom permissions. This contract includes the common functions and messages found in the framework permissions. ([#65](https://github.com/KompleTeam/komple-framework/pull/66))

## [1.0.1-beta] - 2022-12-05

### Changed

- Collection minting lock is now saved as `false` by default in `create_collection` message in Mint Module ([#56](https://github.com/KompleTeam/komple-framework/pull/56))
- `info.sender` is now used instead of `env.contract.address` for querying contract admin on Token Module instantiate message. ([#57](https://github.com/KompleTeam/komple-framework/pull/57))

## [1.0.0-beta] - 2022-11-17

- Initial version for the framework. Release can be found [here](https://github.com/KompleTeam/komple-framework/releases/tag/v1.0.0-beta) 