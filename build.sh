#!/bin/bash

mkdir -p ./release \
  && cd ./server \
  && cargo build --target=x86_64-unknown-linux-musl --release \
  && cp ../target/x86_64-unknown-linux-musl/release/server ../release
