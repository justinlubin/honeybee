FROM ubuntu:24.04

WORKDIR /root

RUN apt-get update

# R dependencies
RUN apt-get -y install r-base r-base-dev
RUN Rscript -e 'install.packages("BiocManager")'
RUN Rscript -e 'BiocManager::install(version = "3.18")'
RUN Rscript -e 'BiocManager::install("tximport")'
RUN Rscript -e 'BiocManager::install("DESeq2")'
RUN Rscript -e 'BiocManager::install("rhdf5")'
RUN Rscript -e 'install.packages("devtools")'

# RUN Rscript -e 'devtools::install_github("pachterlab/sleuth")'
#ENTRYPOINT ["/bin/bash"]

# # Other dependencies
RUN apt-get update
RUN apt-get -y install fastqc
RUN apt-get -y install less
RUN apt-get -y install python3-dev
RUN apt-get -y install wget
RUN apt-get -y install cmake

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

# # uv dependencies
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
# RUN uv add --dev jupyterlab-lsp
# RUN uv add --dev jupyter-lsp
# RUN uv add --dev 'python-lsp-server'
RUN uv sync
RUN rm main.py

RUN uv run jupyter labextension disable "@jupyterlab/apputils-extension:announcements"

RUN apt-get install -y aria2
RUN apt-get install -y build-essential libcurl4-gnutls-dev libxml2-dev libssl-dev
RUN Rscript -e 'BiocManager::install("biomaRt")'

# # Final setup
COPY environment environment
COPY environment/_DO_NOT_EDIT.md .
# COPY jupyter /root/.jupyter
