# Build

## get-base-disk

```sh
env CARGO_TARGET_DIR=/tmp/musl cross build --package get-base-disk --profile thin --target=x86_64-unknown-linux-musl -v

cp /tmp/musl/x86_64-unknown-linux-musl/thin/get-base-disk ./tmp.get-base-disk 

docker build -t reg.tmoe.me:2096/rs/get-base-disk:x64 . -f ./get-base-disk.dockerfile

docker push reg.tmoe.me:2096/rs/get-base-disk:x64
```
