# Context for this Dockerfile is the Nix build's `result/` output
# (a fully static musl binary at bin/server, no shared libs needed).
FROM scratch
COPY bin/server /server
EXPOSE 3000
ENTRYPOINT ["/server"]
