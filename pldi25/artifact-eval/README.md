# Getting Started Guide

_Estimated time to complete this section: 10 minutes._

To evaluate this artifact, please use the Docker image from the Zenodo archive, `pldi25ae-honeybee.tar.gz`.

First, you will need to [install Docker](https://docs.docker.com/get-started/get-docker/). Docker Desktop is a particularly easy path to installing Docker, but any standard installation of Docker should do.

With Docker installed, you can now "kick the tires" to make sure that everything is installed properly.

**Step 1:** Load the Docker image by running `bash LOAD_DOCKER.sh` (included in the Zenodo archive).

**Step 2:** Run the Docker image by running `bash RUN_DOCKER.sh` (included in the Zenodo archive). This command should boot you into a Bash shell in the Ubuntu environment provided by the Docker image.

**Step 3:** In the Docker-provided environment, run `./KICK_TIRES.sh` and ensure that no errors ensue.

And that's it for the kick-the-tires phase!

# Step-by-Step Instructions

_Estimated hands-on time to complete this section: 20 minutes._

_Estimated hands-off time to complete this section: between 1-3 hours (recommended setup) to approximately 1 week (full setup)._

All steps below assume that you are booted into the Docker image as in Step 2 of the Getting Started Guide.

## Step 1: Run overview example

We've included a slightly-generalized version of the example from Section 2 (Overview) in the paper to get a feel for how Honeybee works.

### Steps to take

1. Run the generalized Overview example by running `./RUN_OVERVIEW_EXAMPLE.sh`.
2. Choose any of the options that are presented to you; they all lead to a valid solution.
3. Repeat Step 2 until arriving at a valid solution.

## Step 2: Run benchmark suite

The goal of this step is to run the benchmark suite from Section 7 (Evaluation) of the paper. At a high level, each entry in the benchmark has an associated "ANY" task and ten associated "PARTICULAR" tasks (the median performance on which is reported).

As we are measuring performance, it is expected that there will be some machine-to-machine variation in the results, but the overall trends should be the same as in the paper.

_**Tip (optional):** If you want to take a look at these benchmarks, you can see them in the subdirectories of `benchmark/suites/`._

We provide two scripts to run this evaluation:

- `RUN_QUICK_EVAL.sh <NUM_CORES>`: **Quick evaluation (recommended approach).** This script runs all the experiments in the paper but with the following two changes: (1) Benchmarks are run in parallel with `<NUM_CORES>` cores. (2) Benchmarks are run with only 1 replicate and 2 "PARTICULAR" samples. Overall, this script sacrifices precision for a quicker evaluation harness.
- `RUN_FULL_EVAL.sh`: **Full evaluation.** This script runs all the experiments in the paper sequentially and exactly as described in the paper.

On our machine, running `./RUN_QUICK_EVAL.sh 8` takes about 30 minutes in Docker. The full evaluation can take close to a week. We recommend the quick evaluation approach as the performance trends are the same between the two approaches, and the quick evaluation is enough to support the claims we make in the paper.

Both of these scripts produce output in the `DOCKER_MOUNT` directory on the **host machine**. The `DOCKER_MOUNT/data` subdirectory includes the raw timing data and the `DOCKER_MOUNT/output` subdirectory contains the graphs that appear as figures in the paper. The output is organized by the start time of the evaluation run in UTC (not the host machine's timezone).

### Steps to take

1. Run _either_ `./RUN_QUICK_EVAL.sh <NUM_CORES>` (for example, `./RUN_QUICK_EVAL 8`) for the quick evaluation _or_ `./RUN_FULL_EVAL.sh` for the full evaluation.

_**Tip:** If you run the `RUN_QUICK_EVAL.sh` script and see a message from the operating system in Docker that the process was killed, it is likely because the evaluation [used more RAM than has been allocated to Docker](https://stackoverflow.com/a/50770267). Please try running the evaluation again after increasing the amount of RAM that is available to Docker and/or after decreasing the number of cores that you use for parallelism (which reduces the memory footprint)._

## Step 3: Verify the claims in the paper

We make four main empirical claims in the paper:

* **Section 7.1 (RQ1):** Honeybee solves benchmarks impossible or too large for baselines.
* **Section 7.2 (RQ2):** Honeybee scales to large problems while Naïve Enumeration and
Pruned Enumeration solve smaller problems faster.
* **Section 7.3 (RQ3):** Honeybee benefits from off-the-shelf memoization.
* **Section 7.4 (RQ4):** On the Any Task, Honeybee performs comparably to baselines.

The graphs produced from the previous step provide the evidence for these claims. (For context, the "baselines" are Naïve Enumeration and Pruned Enumeration in these graphs.) Here is how we interpret the graphs in terms of the research questions (RQs):

* **To verify RQ1:** The graph `01-fin.pdf` should show that Honeybee solves all 13/13 problems, and that the baselines do not. Moreover, the graph `02-inf.pdf` should show that Honeybee can solve 8/8 problems that the baselines cannot even represent (and thus are not plotted).
* **To verify RQ2:** The graph `03-scalability.pdf` should show that Honeybee scales linearly in depth and is roughly constant in breadth, whereas the baselines scale substantially worse. For sufficiently large problems, Honeybee should substantially outperform the baselines.
* **To verify RQ3:** Most (if not all) the points should be in the bottom-right half of the graph `04-speedup.pdf`.
* **To verify RQ4:** The Honeybee plot in the graph `05-any.pdf` should be similar to the baseline plots.

### Steps to take

1. Verify the research questions using the graphs produced in the previous step.

And that's it! Thank you so much for your service as an artifact evaluator!

# Optional: Looking at the Honeybee codebase

If you would like to take a look at implementation of Honeybee, please refer to the file `ARCHITECTURE.md` in the code repository for how to dive in! Each file in the codebase also has documentation that should help reading the code.

# Optional: Running more examples

Please take a look at the script `RUN_OVERVIEW_EXAMPLE.sh` to see an example of how to run Honeybee. You can run Honeybee interactively on any of the suites and programs available in the `benchmark/suites` directory. You can also run `cargo run -- help` from the `backend` directory for more help.
