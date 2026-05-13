FROM ubuntu:24.04

WORKDIR /root

RUN apt-get update

# R

RUN apt-get install -y r-base r-base-dev
RUN apt-get install -y build-essential libcurl4-gnutls-dev libxml2-dev
RUN apt-get install -y libssl-dev libuv1-dev libfontconfig1-dev libgit2-dev
RUN apt-get update
RUN apt-get install -y libharfbuzz-dev
RUN apt-get install -y libfribidi-dev
RUN apt-get install -y libfreetype6-dev
RUN apt-get install -y libpng-dev libtiff5-dev libjpeg-dev libwebp-dev

RUN Rscript -e 'install.packages("optparse")'
RUN Rscript -e 'install.packages("devtools")'
RUN Rscript -e 'install.packages("remotes")'
RUN Rscript -e 'install.packages("tidyverse")'
RUN Rscript -e 'install.packages("BiocManager")'

RUN Rscript -e 'BiocManager::install("biomaRt")'
RUN Rscript -e 'BiocManager::install(version = "3.18")'
RUN Rscript -e 'BiocManager::install("tximport")'
RUN Rscript -e 'BiocManager::install("DESeq2")'
RUN Rscript -e 'BiocManager::install("rhdf5")'

RUN Rscript -e 'devtools::install_github("pachterlab/sleuth")'

# System packages

RUN apt-get install -y fastqc
RUN apt-get install -y less
RUN apt-get install -y python3-dev
RUN apt-get install -y wget
RUN apt-get install -y cmake
RUN apt-get install -y aria2
RUN apt-get install -y salmon
RUN apt-get install -y libhdf5-dev
RUN apt-get install -y autoconf
RUN apt-get install -y zlib1g-dev

# kallisto

RUN wget https://github.com/pachterlab/kallisto/archive/refs/tags/v0.52.0.zip
RUN unzip v0.52.0.zip && \
    cd kallisto-0.52.0 && \
    cd ext/htslib && \
    autoheader && \
    autoconf && \
    cd ../../ && \
    mkdir build && \
    cd build && \
    cmake -DUSE_HDF5=ON -DHDF5_ROOT=/usr/include/hdf5/serial -DHDF5_LIBRARIES=/usr/lib/aarch64-linux-gnu/hdf5/serial/libhdf5.so -DCMAKE_CXX_FLAGS="-I/usr/include/hdf5/serial" .. && \
    make 
RUN mkdir -p /root/.local/bin/
RUN mv kallisto-0.52.0/build/src/kallisto /root/.local/bin/kallisto
RUN rm v0.52.0.zip
RUN rm -rf kallisto-0.52.0

# (Temporary: some more apt packages)
RUN apt-get install -y bwa
RUN apt-get install -y bowtie2
RUN apt-get install -y samtools
RUN apt-get install -y minimap2

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
RUN uv add macs3
RUN uv add "altair[all]"
RUN uv add ipython
RUN uv add pysam

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
