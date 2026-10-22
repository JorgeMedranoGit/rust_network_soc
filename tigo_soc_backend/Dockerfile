# * * * IMAGEN BASE Y COMPONENTES DE COMPILACIÓN CERRADA PARA RUST Y C++ * * *
FROM rust:slim-bookworm
RUN apt-get update && apt-get install -y \
    build-essential \
    make \
    pkg-config \
    libssl-dev \
    libpcap-dev \
    clang \
    libclang-dev \
    libomp-dev \
    curl \
    tar \
    && rm -rf /var/lib/apt/lists/* \
    && curl -L https://github.com/Kitware/CMake/releases/download/v3.31.5/cmake-3.31.5-linux-x86_64.tar.gz | tar xz --strip-components=1 -C /usr/local
WORKDIR /app
COPY . .
# * * * EJECUCIÓN DEL SERVIDOR EN MODO RELEASE DE MÁXIMO RENDIMIENTO * * *
CMD ["cargo", "run", "--release", "--", "server"]
