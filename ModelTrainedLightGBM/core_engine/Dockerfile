# --- ETAPA DE CONSTRUCCIÓN ---
FROM rust:latest as builder

# Instalar dependencias esenciales
RUN apt-get update && apt-get install -y \
    libomp-dev \
    clang \
    curl \
    tar \
    && rm -rf /var/lib/apt/lists/*

# Instalar CMake 3.31 manualmente
RUN curl -L https://github.com/Kitware/CMake/releases/download/v3.31.5/cmake-3.31.5-linux-x86_64.tar.gz | tar xz --strip-components=1 -C /usr/local

WORKDIR /usr/src/app
COPY . .

# Compilar para release
RUN cargo build --release

# --- ETAPA FINAL (Usamos Ubuntu para garantizar librerías modernas) ---
FROM ubuntu:24.04

# Evitar prompts interactivos
ENV DEBIAN_FRONTEND=noninteractive

# Instalar librerías de ejecución necesarias
RUN apt-get update && apt-get install -y \
    libomp5 \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copiar el binario desde el builder
COPY --from=builder /usr/src/app/target/release/core_engine /app/

# Copiar activos necesarios
COPY --from=builder /usr/src/app/models /app/models
COPY --from=builder /usr/src/app/fonts /app/fonts
COPY --from=builder /usr/src/app/dashboard_live.html /app/
COPY --from=builder /usr/src/app/reporte_interactivo.html /app/

# Configuración de red para Render
ENV PORT=3000
EXPOSE 3000

# Ejecutar el servidor
CMD ["./core_engine", "server"]
