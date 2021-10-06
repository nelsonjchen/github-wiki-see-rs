ARG RUST_BUILD_IMAGE=rust:1.54.0
FROM ${RUST_BUILD_IMAGE} as build

WORKDIR /usr/src/project
COPY . .

RUN cargo install --path .

FROM gcr.io/distroless/cc-debian10

COPY --from=build /usr/local/cargo/bin/github-wiki-see /usr/local/bin/github-wiki-see

ENV ROCKET_PORT=8080

CMD ["github-wiki-see"]
