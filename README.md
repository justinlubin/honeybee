# Getting Started Guide

_Estimated time to complete this section: 10 minutes._

To evaluate this artifact, we recommend using the included Docker image `pldi25ae-honeybee.tar.gz`.

First, you will need to [install Docker](https://docs.docker.com/get-started/get-docker/). Docker Desktop is a particularly easy path to installing Docker, but any standard installation of Docker should do.

Second, here is how to "kick the tires" to make sure that everything is installed properly.

**Step 1:** Load the Docker image by running `docker load -i pldi25ae-honeybee.tar.gz`.

**Step 2:** Run the Docker image by running `./RUN_DOCKER.sh`. This command should boot you into a Bash shell in the Ubuntu environment provided by the Docker image.

**Step 3:** Run `./KICK_TIRES.sh` and ensure that no errors ensue.

And that's it!

_**Tip:** If Docker does not work well with your system, it is also easy (but not recommended) to work directly from the source repository; the only external dependencies are [Rust](https://www.rust-lang.org/) and [uv](https://docs.astral.sh/uv/). If you follow this approach, simply start at Step 3 below to "kick the tires."_


# Step-by-Step Instructions

_Estimated hands-on time to complete this section: 20 minutes._

_Estimated hands-off time to complete this section: between 1-2 hours (recommended setup) to approximately 1 week (full setup)._

All steps below assume that you are booted into the Docker image as in Step 2 of the Getting Started Guide. (You may also run these steps below without Docker if you have the required dependencies mentioned above, but this is not recommended.)

## Step 1: Run overview example

1. Run the example from the Section 2 (Overview) of the paper by running `RUN_OVERVIEW_EXAMPLE.sh`.
2. Verify that the program synthesizer lets you interactively construct a Python program.

## Step 2: Run benchmark suite


The goal of this step is to run the benchmark suite from Section 7 (Evaluation) of the paper. At a high level, each entry in the benchmark has an associated "ANY" task and ten associated "PARTICULAR" tasks (the median performance on which is reported). As we are measuring performance, it is expected that there will be some machine-to-machine variation in the results, but the overall trends should be the same as in the paper.

_**Tip (optional):** If you want to take a look at these benchmarks, you can see them in the subdirectories of `benchmark/suites/`._

We provide two scripts to run this evaluation:

- `RUN_QUICK_EVAL.sh <NUM_CORES>`: **Quick evaluation (recommended approach).** This script runs all the experiments in the paper but with the following two changes: (1) Benchmarks are run in parallel with `<NUM_CORES>` cores. (2) Benchmarks are run with only 1 replicate and 2 "PARTICULAR" samples. Overall, this script sacrifices precision for a quicker evaluation harness.
- `RUN_FULL_EVAL.sh`: **Full evaluation.** This script runs all the experiments in the paper sequentially and exactly as described in the paper.

On our machine, running `./RUN_QUICK_EVAL.sh 8` takes about 30 minutes. The full evaluation can take close to a week. We recommend the quick evaluation approach as the performance trends are the same between the two approaches, and the quick evaluation is enough to support the claims we make in the paper.

Both of these scripts produce output in the `DOCKER_MOUNT` directory on the **host machine**. The `DOCKER_MOUNT/data` subdirectory includes the raw timing data and the `DOCKER_MOUNT/output` subdirectory contains the graphs that appear as figures in the paper.


### Steps to take

1. Run _either_ `./RUN_QUICK_EVAL.sh <NUM_CORES>` (for example, `./RUN_QUICK_EVAL 8`) for the quick evaluation _or_ `./RUN_FULL_EVAL.sh` for the full evaluation.

## Step 3: Verify the claims in the paper

We make four main empirical claims in the paper:

* **Section 7.1 (RQ1):** Honeybee solves benchmarks impossible or too large for baselines.
* **Section 7.2 (RQ2):** Honeybee scales to large problems while Naïve Enumeration and
Pruned Enumeration solve smaller problems faster.
* **Section 7.3 (RQ3):** Honeybee benefits from off-the-shelf memoization.
* **Section 7.4 (RQ4):** On the Any Task, Honeybee performs comparably to baselines.

The graphs produced from the previous step provide the evidence for these claims. (For context, the "baselines" are Naïve Enumeration and Pruned Enumeration in these graphs.) Here is how we interpret the graphs in terms of the research questions (RQs):

* **To verify RQ1:** The graph `01-fin.pdf` should show that Honeybee solves all 13/13 problems, and that the baselines do not. Moreover, the graph `02-inf.pdf` should show that Honeybee can solve 8/8 problems that the baselines cannot even represent (and thus are not plotted).
* **To verify RQ2:** The graph `03-scalability.pdf` should show that Honeybee scales linearly in depth and is roughly constant in breadth, whereas the scale substantially worse. For sufficiently large problems, Honeybee should substantially outperform the baselines.
* **To verify RQ3:** Most (if not all) the points should be in the bottom-right quadrant of the graph `04-speedup.pdf`.
* **To verify RQ4:** The Honeybee plot in the graph `05-any.pdf` should be similar to the baselines.

### Steps to take

1. Verify the research questions using the graphs produced in the previous step.

And that's it! Thank you so much for your service as an artifact evaluator!

# Optional: Looking at the Honeybee codebase

If you would like to take a look at implementation of Cobbler, please refer to the file `ARCHITECTURE.md` for how to dive in!

# Optional: Running more examples

Please take a look at the script