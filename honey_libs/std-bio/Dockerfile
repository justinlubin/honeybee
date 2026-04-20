FROM ubuntu:24.04

WORKDIR /root

# R dependencies
RUN apt-get update
RUN apt-get -y install r-base r-base-dev
COPY dependencies/deps.r .
RUN Rscript dependencies/deps.r

# uv dependencies
RUN apt-get -y install curl
RUN curl -LsSf https://astral.sh/uv/install.sh | sh
ENV PATH="/root/.local/bin:${PATH}"
COPY dependencies/pyproject.toml .
RUN uv sync

# Other dependencies
RUN apt-get -y install fastqc
RUN apt-get -y install kallisto

# Final setup
COPY playground playground
COPY environment .

ENTRYPOINT ["/bin/bash"]
