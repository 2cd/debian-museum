# syntax=docker/dockerfile:1
FROM curlimages/curl-base:8.6.0 as curl
RUN <<'EOF'
#!/bin/sh -ex
url="http://azure.archive.ubuntu.com/ubuntu/pool/main/d/debootstrap/"
deb_file=$(curl -L "$url" | awk -F 'href=' '/debootstrap.*_all\.deb/ {html = $2}  END{ split(html, val, "\""); print val[2] }')
curl -LO "$url/$deb_file"
install -Dm 644 "$deb_file" /app/pkg.deb
EOF

FROM ghcr.io/2moe/zsh-static
COPY --from=curl /app/pkg.deb /app/
WORKDIR /app
