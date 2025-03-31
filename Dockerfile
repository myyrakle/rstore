FROM rust:1.85.0-alpine AS builder
COPY . /home/server 
WORKDIR /home/server
# Do something for ready to build
RUN apk add musl-dev
RUN cargo build --release
RUN mv /home/server/target/release/rstore /home/server/bin

FROM alpine:3.21.0 AS deployer
RUN mkdir /app
COPY --from=builder /home/server/bin /app/server
RUN ls -lah /app
ENTRYPOINT [ "/bin/sh", "-c", "/app/server" ]
EXPOSE 13535