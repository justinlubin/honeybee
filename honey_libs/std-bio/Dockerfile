FROM ubuntu:24.04

###############################################################################
# Install dependencies
WORKDIR /root

# Base
RUN apt-get update
RUN apt-get -y install r-base r-base-dev
RUN apt-get -y install fastqc
RUN apt-get -y install curl

RUN curl --version
# uv
RUN curl -LsSf https://astral.sh/uv/install.sh | sh


ENV PATH="/root/.local/bin:${PATH}"
COPY environment .

RUN uv sync

# RUN curl -L https://github.com/pachterlab/kallisto/releases/download/v0.52.0/kallisto_linux-v0.52.0.tar.gz > kallisto.tar.gz

RUN apt-get -y install kallisto
COPY playground playground

RUN Rscript install_dependencies.r

###############################################################################
# Set up PATH


###############################################################################
# 
# # Create aonav binary
# COPY Cargo.toml .
# COPY Cargo.lock .
# COPY src src
# RUN cargo install --root /usr/local --path .
# RUN rm -rf Cargo.toml Cargo.lock src/ target/ ~/.cargo ~/.rustup
# 
# # Set up benchmarking harness
# COPY scripts/artifact-eval.sh .
# COPY benchmark/entries entries
# COPY benchmark/analysis analysis
# RUN mkdir results
# 
# # Pre-fetch Python dependencies
# RUN cd analysis && uv sync

ENTRYPOINT ["/bin/bash"]
