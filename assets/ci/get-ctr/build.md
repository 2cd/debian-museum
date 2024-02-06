# Build

## get-ctr

```sh
env CARGO_TARGET_DIR=/tmp/musl cross build --package get-ctr --profile thin --target=x86_64-unknown-linux-musl -v

cp /tmp/musl/x86_64-unknown-linux-musl/thin/get-ctr ./tmp.get-ctr 

docker build -t reg.tmoe.me:2096/rs/get-ctr:x64 .

docker push reg.tmoe.me:2096/rs/get-ctr:x64
```
