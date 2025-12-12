# Honeybee

Honeybee is a programming tool scientists can use to help them write Python
code to analyze experimental data. Its goal is to enable scientists to write
the programs they need with _only_ domain expertise, _not_ programming
expertise.

The frontend for Honeybee—which currently targets experimental biology—can be
accessed via the
[Honeybee Editor](https://editor.honeybee-lang.org).

Honeybee is based on our recent work on
[Programming by Navigation](https://dl.acm.org/doi/10.1145/3729264)
appearing in PLDI 2025.

## Honeybee Editor video demo

https://github.com/user-attachments/assets/e75636f7-d6ea-4498-9753-a0bf89e3447f

## Repository structure

The main directories are:

- `backend`: Programming by Navigation synthesis (all the algorithms behind
  Honeybee). Written in Rust.
- `editor`: Frontend code for the Honeybee Editor. Written in Elm.
- `library-generator`: An ergonomic Python API to define building blocks for
  Honeybee and a working library for experimental biology. Written in Python.

The `artifact-eval` directory contains materials for the PLDI 2025 artifact
evaluation and the `benchmark` directory contains the benchmarks we used for
our PLDI 2025 publication.

## Beyond Honeybee

Honeybee is part of a broader project we’re working on called SciInterop to
enable scientists across a variety of scientific domains to write code with
only domain expertise, not programming expertise. Honeybee and SciInterop are
based on our recent work on
[Programming by Navigation](https://dl.acm.org/doi/10.1145/3729264),
which is agnostic to the underlying scientific domain. SciInterop can therefore
support domains that look very different from biology, like geospatial data
analysis. For example, for geospatial data analysis, we’re incorporating
bread-and-butter computational analyses such as the normalized difference
vegetation index (NDVI). Beyond additional domains, we are also working on
capabilities for SciInterop to help debug experimental design issues before
time-consuming and costly experiments get run as well as functionality to
import existing scientific publications into a Honeybee-like navigation
interface. Stay tuned to our website
[honeybee-lang.org](https://honeybee-lang.org)
and our
[GitHub repository](https://github.com/justinlubin/honeybee)
to stay up-to-date on our project! 
