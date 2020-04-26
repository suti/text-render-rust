#  文字渲染引擎rust版

## deploy server:

`./build.sh -> build docker`

在mac上交叉编辑linux需要：
```$xslt
brew install FiloSottile/musl-cross/musl-cross
brew install mingw-w64 
echo "[target.x86_64-unknown-linux-musl]" >> ~/.cargo/config
echo "linker = 'x86_64-linux-musl-gcc'" >> ~/.cargo/config
rustup target add x86_64-unknown-linux-musl
```

## deploy wasm for browser:

`wasm-pack build --target web --scope text-render-rust --release --out-name typesetting`

## deploy wasm for node:

`wasm-pack build --target nodejs --scope text-render-rust --release --out-name typesetting-node`

## deploy asm.js:

在**deploy wasm for browser**基础上执行

`wasm2js --pedantic --output pkg/typesetting.wasm.js pkg/typesetting_bg.wasm`

`wasm2js`在binaryen工具链( https://github.com/WebAssembly/binaryen )中，同时依赖emsdk( https://emscripten.org/ )

同时需要手动修改`typesetting.wasm.js`中的引用

