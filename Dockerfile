FROM alpine:edge AS buildenv

RUN apk add --no-cache gcc musl-dev openssl-dev rust cargo
RUN mkdir /build
COPY . /build/
WORKDIR /build
RUN cargo build

FROM alpine:edge

RUN apk add --no-cache ca-certificates-cacert libgcc
COPY --from=buildenv /build/target/debug/fargate-stats-reporter ./fargate-stats-reporter
ENTRYPOINT ["/fargate-stats-reporter"]
