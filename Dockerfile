FROM ubuntu:latest

# Install base system components
RUN apt-get update
RUN apt-get -y install build-essential
RUN apt-get -y install curl
RUN apt-get -y install vim

# Install rust
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y

# Install uv
RUN curl -LsSf https://astral.sh/uv/install.sh | sh

# Copy over files
WORKDIR /home/ubuntu/
COPY . .
RUN rm -rf benchmark/data
RUN rm -rf benchmark/analysis/output
RUN mkdir -p benchmark/data
RUN mkdir -p benchmark/analysis/output

# Load bash in home directory
CMD ["bash"]
