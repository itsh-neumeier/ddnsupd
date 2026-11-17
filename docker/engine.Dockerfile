FROM golang:1.26-trixie AS build
WORKDIR /src
COPY provider-engine/ ./
RUN go test ./... && CGO_ENABLED=0 go build -trimpath -ldflags='-s -w' -o /engine .
FROM scratch
COPY --from=build /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/ca-certificates.crt
COPY --from=build /engine /app/engine
COPY provider-engine/LICENSE.upstream /app/LICENSE.upstream
COPY LICENSE /app/LICENSE
USER 10001:10001
EXPOSE 8090
HEALTHCHECK --interval=15s --timeout=5s --retries=3 CMD ["/app/engine", "healthcheck"]
ENTRYPOINT ["/app/engine"]
