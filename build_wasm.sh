#!/bin/bash

# 检查是否安装了wasm-pack
if ! command -v wasm-pack &> /dev/null; then
    echo "wasm-pack 未安装，正在安装..."
    cargo install wasm-pack
fi

# 构建WebAssembly
echo "正在构建WebAssembly包..."
wasm-pack build --target web

# 检查是否成功
if [ $? -eq 0 ]; then
    echo "构建成功！WebAssembly包已生成在 ./pkg 目录下"
    echo "要查看演示，请启动一个本地HTTP服务器，例如:"
    echo "python -m http.server"
    echo "然后访问: http://localhost:8000/examples/"
else
    echo "构建失败，请查看上面的错误信息"
fi 