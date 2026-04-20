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
RUN Rscript -e 'devtools::install_github("pachterlab/sleuth")'
ENTRYPOINT ["/bin/bash"]

# # uv dependencies
# RUN apt-get -y install curl
# RUN curl -LsSf https://astral.sh/uv/install.sh | sh
# ENV PATH="/root/.local/bin:${PATH}"
# COPY image-dependencies/pyproject.toml .
# RUN uv sync

# # Other dependencies
# RUN apt-get -y install fastqc
# RUN apt-get -y install kallisto

# # Final setup
# COPY image-environment .
# COPY playground playground

# ENTRYPOINT ["/bin/bash"]
