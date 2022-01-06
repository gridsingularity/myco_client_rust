FROM rust:1.49 as build

# create a new empty shell project
RUN USER=root cargo new --bin myco_client_rust
WORKDIR /myco_client_rust

# copy manifests
COPY ./Cargo.toml ./Cargo.toml

# cache dependencies
RUN cargo build --release
RUN rm src/*.rs

# copy source tree
COPY ./src ./src

# build for release
RUN rm ./target/release/deps/myco_client_rust*
RUN cargo build --release

# final base
FROM rust:1.49

# copy the build artifact from the build stage
COPY --from=build /myco_client_rust/target/release/myco_client_rust .

# set the startup command to run your binary
CMD ["./myco_client_rust"]