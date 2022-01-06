FROM rust:1.57.0-slim-bullseye AS cargobase

ARG user=procrustes project=procrustes src=src

RUN apt-get update && \
    apt-get install -y libtagc0-dev && \
    useradd -ms /bin/bash "$user"
# Non-root user.
USER $user
WORKDIR /home/$user

# Project.
RUN mkdir /home/$user/$project
WORKDIR /home/$user/$project
COPY $src ./$src/
COPY Cargo.toml Cargo.lock README.rst ./

# Build.
RUN cargo build --release

FROM debian:bullseye-slim

ARG user=procrustes project=procrustes

RUN apt-get update && \
    apt-get install -y libtagc0-dev && \
    useradd -ms /bin/bash "$user"
# Non-root user.
USER $user
WORKDIR /home/$user
ENV PATH=/home/$user/.local/bin:$PATH

RUN echo 'alias dp=procrustes' >> .bashrc && \
    mkdir -p .local/bin
COPY --from=cargobase /home/$user/$project/target/release/procrustes .local/bin

CMD ["bash"]

