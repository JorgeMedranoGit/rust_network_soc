FROM rust:slim-bookworm
RUN apt-get update && apt-get install -y pkg-config libssl-dev libpcap-dev && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY . .
CMD ["cargo", "run"]