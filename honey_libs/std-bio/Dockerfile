FROM ubuntu:24.04

WORKDIR /root

RUN apt-get update

# R

RUN apt-get install -y r-base r-base-dev
RUN apt-get install -y build-essential libcurl4-gnutls-dev libxml2-dev
RUN apt-get install -y libssl-dev libuv1-dev libfontconfig1-dev
# RUN apt-get install -y libharfbuzz-dev libfribidi-dev
RUN Rscript -e 'install.packages("BiocManager")'
RUN Rscript -e 'BiocManager::install(version = "3.18")'
RUN Rscript -e 'BiocManager::install("tximport")'
RUN Rscript -e 'BiocManager::install("DESeq2")'
RUN Rscript -e 'BiocManager::install("rhdf5")'
RUN Rscript -e 'install.packages("devtools")'
RUN Rscript -e 'BiocManager::install("biomaRt")'
RUN Rscript -e 'install.packages("optparse")'
# RUN Rscript -e 'devtools::install_github("pachterlab/sleuth")'

# System packages

RUN apt-get update
RUN apt-get -y install fastqc
RUN apt-get -y install less
RUN apt-get -y install python3-dev
RUN apt-get -y install wget
RUN apt-get -y install cmake
RUN apt-get install -y aria2

# kallisto

RUN wget https://github.com/pachterlab/kallisto/archive/refs/tags/v0.52.0.zip
RUN unzip v0.52.0.zip && \
    cd kallisto-0.52.0 && \
    mkdir build && \
    cd build && \
    cmake .. && \
    make
RUN mkdir -p /root/.local/bin/
RUN mv kallisto-0.52.0/build/src/kallisto /root/.local/bin/kallisto
RUN rm v0.52.0.zip
RUN rm -rf kallisto-0.52.0

RUN apt-get install -y libharfbuzz-dev libfribidi-dev
RUN apt-get install -y libfreetype6-dev libpng-dev libtiff5-dev libjpeg-dev libwebp-dev
RUN Rscript -e 'install.packages("tidyverse")'


# uv

RUN apt-get -y install curl
RUN curl -LsSf https://astral.sh/uv/install.sh | sh
ENV PATH="/root/.local/bin:${PATH}"
RUN uv init \
  --name "hb-stdbio-docker" \
  --description "Honeybee Standard Bio Docker Image" \
  --no-readme \
  --python 3.12
RUN uv add cutadapt
RUN uv add multiqc
RUN uv add polars
RUN uv add --dev jupyter

RUN uv sync
RUN rm main.py

RUN uv run jupyter labextension disable "@jupyterlab/apputils-extension:announcements"

# Final setup

COPY environment environment
COPY edit-warning.md DO_NOT_EDIT_HERE.md

COPY entrypoint.sh entrypoint.sh
RUN chmod +x entrypoint.sh

ARG VERSION
ARG COMMIT
ARG USER_FILES

ENV HONEYBEE_VERSION=${VERSION}+${COMMIT}
ENV HONEYBEE_USER_ENVIRONMENT=${USER_FILES}/environment

ENTRYPOINT ["/root/entrypoint.sh"]
