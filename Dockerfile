FROM rust:1.53.0 as build

WORKDIR /usr/src/project
COPY . .

RUN cargo install --path .

FROM gcr.io/distroless/cc-debian10

COPY --from=build /usr/local/cargo/bin/github-wiki-see /usr/local/bin/github-wiki-see

CMD ["github-wiki-see"]
