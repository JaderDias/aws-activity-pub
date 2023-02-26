FROM amazonlinux:2

# gcc is required to build ring, one of aws-config dependencies
# openssl-devel and pkgconfig are required to build openssl, a direct dependency
RUN yum update -y \
    && yum install -y \
        gcc-7.3.1-15.amzn2  \
        openssl-devel-1.0.2k-24.amzn2.0.6 \
        pkgconfig-0.27.1-4.amzn2.0.2 \
    && yum clean all

# https://rust-lang.github.io/rustup/concepts/profiles.html
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | /bin/sh -s -- --profile minimal -y

WORKDIR /volume
