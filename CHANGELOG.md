# Changelog

This changelog tracks the notable changes for downstream users in each version
of Honeybee. 

## [Unreleased]

### Added

- Add "use hints" that allow for quick comparison between tools
  ([#101](https://github.com/justinlubin/honeybee/pull/101))
- Add /unstable route for accessing latest pushed unreleased version
- Add hover text for complex concepts
- Add "Undo" button to frontend
- Add Honeybee citation to output script
- Add citation backreferences via Google Scholar
- Add tool search via Google and DuckDuckGo
- Add reference to PubMed entry (PMID) for each tool
  ([#102](https://github.com/justinlubin/honeybee/pull/102))
- Add GoatCounter private analytics
- Add website (https://honeybee-lang.org), including video demo

### Changed

- Overhaul glue annotations into the Honey language, resulting in much
  more ergnomic definitions for building blocks, better codegen, and a
  smoother editor experience, including auto-selecting the "functions" that
  declare the user's workflow
  ([#95](https://github.com/justinlubin/honeybee/pull/95))
- Move "Next CHOICE" button (and new "Undo" button) to bottom of screen
- Make variable names in generated code nicer (e.g. `TRANSCRIPT_MATRICES`,
  not `TRANSCRIPTMATRICES`)
- Make "Go to download" button simply a "Download" button
- Make code preview nicer (shows holes as circles)
  ([#107](https://github.com/justinlubin/honeybee/pull/107))
- Made the menu bar sticky
  ([#103](https://github.com/justinlubin/honeybee/pull/103),
  [#108](https://github.com/justinlubin/honeybee/pull/108))
