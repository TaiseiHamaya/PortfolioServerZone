FROM amazonlinux:2023.9.20250929.0

# 開発に必要なパッケージのインストール
RUN dnf update -y && \
    dnf install -y --allowerasing \
    tar gzip \
    git gcc gcc-c++ make pkgconfig openssl-devel net-tools iputils \
    rust cargo rust-src rustfmt

# 作業ディレクトリ
WORKDIR /workspace
