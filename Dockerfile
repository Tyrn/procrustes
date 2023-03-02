FROM rust:1.57.0-slim-bullseye AS cargobase

ARG cuser=procrustes cproject=procrustes src=src

RUN apt-get update && \
    apt-get install -y libtagc0-dev && \
    useradd -ms /bin/bash "$cuser"
# Non-root user.
USER $cuser
WORKDIR /home/$cuser

# Project.
RUN mkdir /home/$cuser/$cproject
WORKDIR /home/$cuser/$cproject
COPY $src ./$src/
COPY Cargo.toml Cargo.lock README.rst ./

# Build.
RUN cargo build --release

FROM debian:bullseye-slim

ARG cuser=procrustes cproject=procrustes

RUN apt-get update && \
    apt-get install -y libtagc0-dev && \
    apt-get install -y tree && \
    apt-get install -y less && \
    apt-get install -y zoxide && \
    useradd -ms /bin/bash "$cuser"
# Non-root user.
USER $cuser
WORKDIR /home/$cuser
ENV PATH=/home/$cuser/.local/bin:$PATH

RUN echo 'alias ll="ls -lh"' >> .bashrc && \
    echo 'alias lls="ls -lh --color=always | less -r"' >> .bashrc && \
    echo 'alias lss="ls --color=always | less -r"' >> .bashrc && \
    echo 'alias dp=procrustes' >> .bashrc && \
    echo 'eval "$(zoxide init bash --hook=prompt)"' >> .bashrc && \
    echo 'alias cd=z' >> .bashrc && \
    mkdir -p .local/bin
COPY --from=cargobase /home/$cuser/$cproject/target/release/$cproject .local/bin

CMD ["bash"]
