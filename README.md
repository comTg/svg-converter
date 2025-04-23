# SVG-Converter

一个用Rust编写的SVG和PNG格式转换工具。

## 功能

- 将SVG图像转换为PNG格式
- 将PNG图像转换为SVG格式（使用边缘检测）

## 安装

确保您已安装了Rust和Cargo。然后克隆此仓库并构建项目：

```bash
git clone https://github.com/yourusername/svg-converter.git
cd svg-converter
cargo build --release
```

## 使用方法

### SVG转PNG

```bash
cargo run -- svg-to-png --input input.svg --output output.png [--width <宽度>] [--height <高度>]
```

参数说明：
- `--input, -i`: 输入SVG文件路径
- `--output, -o`: 输出PNG文件路径
- `--width, -w`: (可选) 输出PNG的宽度
- `--height, -h`: (可选) 输出PNG的高度

如果不指定宽度和高度，将使用SVG的原始尺寸。如果只指定宽度或高度，将按原始宽高比自动计算另一个维度。

### PNG转SVG

```bash
cargo run -- png-to-svg --input input.png --output output.svg [--threshold-low <低阈值>] [--threshold-high <高阈值>]
```

参数说明：
- `--input, -i`: 输入PNG文件路径
- `--output, -o`: 输出SVG文件路径
- `--threshold-low`: (可选) Canny边缘检测的低阈值，默认为50
- `--threshold-high`: (可选) Canny边缘检测的高阈值，默认为150

阈值参数影响边缘检测的灵敏度。值越低，检测的边缘越多，但可能会包含更多噪点。

## 示例

### 将SVG转换为PNG

```bash
cargo run -- svg-to-png --input examples/sample.svg --output examples/sample.png --width 800
```

### 将PNG转换为SVG

```bash
cargo run -- png-to-svg --input examples/photo.png --output examples/photo.svg --threshold-low 30 --threshold-high 100
```

## 依赖库

- resvg: SVG渲染库
- usvg: SVG解析和简化
- tiny-skia: 光栅化库
- image: 图像处理库
- svg: SVG生成
- imageproc: 图像处理算法（用于边缘检测）
- clap: 命令行参数解析
- anyhow: 错误处理

## 许可证

MIT 