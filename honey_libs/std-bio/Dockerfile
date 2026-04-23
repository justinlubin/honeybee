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
RUN apt-get -y install kallisto
RUN apt-get -y install less
RUN apt-get -y install python3-dev

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

# # Final setup
COPY environment environment
COPY environment/_DO_NOT_EDIT.md .
# COPY jupyter /root/.jupyter
