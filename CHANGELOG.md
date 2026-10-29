# Change Log

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

<!--
Check [Keep a Changelog](http://keepachangelog.com/) for recommendations on how to structure this file.

    Added -- for new features.
    Changed -- for changes in existing functionality.
    Deprecated -- for soon-to-be removed features.
    Removed -- for now removed features.
    Fixed -- for any bug fixes.
    Security -- in case of vulnerabilities.
-->
## [Unreleased]

### Changed
- Script templates are now Handlebars templates organized in YAML files
- The sweep script now configures each channel directly instead of including a generic
  channel-configuration library, making generated scripts much shorter

### Fixed
- Waiting on more than two other channels of the same role no longer skips one of them
- Measurement buffers of MPSU channels are now referenced correctly in generated scripts

## [0.1.7]

## [0.1.6]

## [0.1.5]

### Fixed
- Fix issue where backend server can become unresponsive and use too much RAM

## [0.1.4]

## [0.1.3]

## [0.1.2]

## [0.1.1]

### Security
- Updated 3rd-party dependencies

## [0.1.0]

### Added
- Script generation feature
- Step to Sweep Delay adds a delay before starting the sweep curve
- Measure Range has been added to the generated TSP script
- Make all the plot x axis timeline same
- Power envelope validation check has been implemented for MPSU50-2ST
- Step to Sweep Delay implementation in script
- Timing requirements for MP5000 module
- Added Time Per Step above step plot

### Changed
- Allow overrange MPSU50-2ST source voltage values
- Allow overrange MSMU60-2 source voltage/current values
- Source and Measure Range AUTO by default for bias, step, and sweep
- Change X-axis of preview plots to show engineering units for improved readability

### Fixed
- Step and sweep plot y axis ticks incorrect for list mode
- Sweep plot x axis ticks cutoff when number of points increase
- Disable Start/Stop/Style controls when List checkbox is check and Disable Edit when unchecked
- Fix expander chevron position when bias, step and/or sweep is/are empty
- Cursor for plots moving to top
- Don't move cursor when user starts making any UI change
- Fix 3+ channel configurations in the generated script
- Remove range settings for MPSU modules since they don't have any
- Improve source limits for MSMU modules
- Fix over-range support on MPSU modules
- Resolve issue with script not writing in OneDrive folders
- Resolve issue with list sweeps points resetting with unrelated changes
- Resolve issue with list sweep points editing only on hitting enter key
- SMU limit validation

<!-- Version Comparison Links -->
[Unreleased]: https://github.com/tektronix/tsp-toolkit-script-gen/compare/v0.1.7...HEAD
[0.1.7]: https://github.com/tektronix/tsp-toolkit-script-gen/releases/tag/v0.1.7
[0.1.6]: https://github.com/tektronix/tsp-toolkit-script-gen/releases/tag/v0.1.6
[0.1.5]: https://github.com/tektronix/tsp-toolkit-script-gen/releases/tag/v0.1.5
[0.1.4]: https://github.com/tektronix/tsp-toolkit-script-gen/releases/tag/v0.1.4
[0.1.3]: https://github.com/tektronix/tsp-toolkit-script-gen/releases/tag/v0.1.3
[0.1.2]: https://github.com/tektronix/tsp-toolkit-script-gen/releases/tag/v0.1.2
[0.1.1]: https://github.com/tektronix/tsp-toolkit-script-gen/releases/tag/v0.1.1
[0.1.0]: https://github.com/tektronix/tsp-toolkit-script-gen/releases/tag/v0.1.0
