FROM ubuntu:20.04

ARG DEBIAN_FRONTEND='noninteractive'

ADD dockerfile.d/01_apt.sh /root
RUN bash /root/01_apt.sh

ADD ./dockerfile.d/03_sdk.sh /root
RUN bash /root/03_sdk.sh

ARG RUST_TOOLCHAIN='nightly-2021-07-03'
ADD ./dockerfile.d/05_rust.sh /root
RUN bash /root/05_rust.sh

WORKDIR /root

# ====== build phala ======

RUN mkdir phala-blockchain
ADD . phala-blockchain

RUN mkdir prebuilt

RUN cd phala-blockchain && \
    PATH="$PATH:$HOME/.cargo/bin" cargo build --release && \
    cp ./target/release/pherry /root/prebuilt && \
    cp ./target/release/phala-node /root/prebuilt && \
    PATH="$PATH:$HOME/.cargo/bin" cargo clean && \
    rm -rf /root/.cargo/registry && \
    rm -rf /root/.cargo/git

RUN cd phala-blockchain/standalone/pruntime && \
    PATH="$PATH:$HOME/.cargo/bin" SGX_SDK="/opt/sgxsdk" SGX_MODE=SW make && \
    cp ./bin/app /root/prebuilt && \
    cp ./bin/enclave.signed.so /root/prebuilt && \
    cp ./bin/Rocket.toml /root/prebuilt && \
    PATH="$PATH:$HOME/.cargo/bin" make clean && \
    rm -rf /root/.cargo/registry && \
    rm -rf /root/.cargo/git

# ====== clean up ======

RUN rm -rf phala-blockchain
ADD dockerfile.d/cleanup.sh .
RUN bash cleanup.sh

# ====== start phala ======

ADD dockerfile.d/console.sh ./console.sh
ADD dockerfile.d/startup.sh ./startup.sh
ADD dockerfile.d/api.nginx.conf /etc/nginx/sites-enabled/default
CMD bash ./startup.sh

EXPOSE 8000
EXPOSE 9933
EXPOSE 9944
EXPOSE 30333
