# setup builder container
FROM rust:1.62-buster as builder

WORKDIR /usr/src/cardstock
# copies in the current directory
COPY . .

RUN cargo build --release 
 
# set up runner container
FROM debian:buster-slim

# this is a hack to install the prerequisites for doing https requests. you might not need it
RUN apt-get update && \
    apt-get dist-upgrade -y && \
    apt-get install wget -y

COPY --from=builder /usr/src/cardstock/target/release/cardstock .
# you can copy other files here:
COPY data/games.json ./data/games.json
COPY data/idols.json ./data/idols.json
COPY data/teams.json ./data/teams.json

# USER 1000
CMD ["./cardstock"]
