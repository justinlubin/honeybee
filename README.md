# Getting Started Guide

_Estimated time to complete this section: 10 minutes._

To evaluate this artifact, we recommend using the included Docker image `pldi25ae-honeybee.tar.gz`.

First, you will need to [install Docker](https://docs.docker.com/get-started/get-docker/). Docker Desktop is a particularly easy path to installing Docker, but any standard installation of Docker should do.

Second, here is how to "kick the tires" to make sure that everything is installed properly:

1. Load the Docker image by running `docker load -i pldi25ae-honeybee.tar.gz`.
2. Run the Docker image by running `./RUN_DOCKER.sh`. This command should boot you into a Bash shell in the Ubuntu environment provided by the Docker image.
3. Run `./KICK_TIRES.sh` and ensure that no errors ensue.

And that's it!

_**Tip:** If Docker does not work well with your system, it is also easy (but not recommended) to work directly from the source repository; the only external dependencies are [Rust](https://www.rust-lang.org/) and [uv](https://docs.astral.sh/uv/). If you follow this approach, simply start at Step 3 below to "kick the tires."_


# Step-by-Step Instructions