# SVG转换器

一个用于在SVG和PNG格式之间进行转换的工具。

## 功能

- SVG转PNG：将SVG文件转换为PNG图像，支持自定义尺寸
- PNG转SVG：将PNG图像转换为SVG矢量图形，支持简化级别调整

## 命令行工具使用方法

### 构建项目

```bash
cargo build --release
```

### SVG转PNG

```bash
./target/release/svg-converter svg-to-png input.svg output.png [宽度] [高度]
```

参数：
- `input.svg`：输入SVG文件路径
- `output.png`：输出PNG文件路径
- `[宽度]`：可选，输出PNG的宽度，默认使用SVG的原始宽度
- `[高度]`：可选，输出PNG的高度，默认使用SVG的原始高度

### PNG转SVG

```bash
./target/release/svg-converter png-to-svg input.png output.svg [简化级别]
```

参数：
- `input.png`：输入PNG文件路径
- `output.svg`：输出SVG文件路径
- `[简化级别]`：可选，SVG路径简化级别(0-10)，默认为3
  - 0: 不简化
  - 10: 最大简化

## WebAssembly版本

此项目也可以编译为WebAssembly，在浏览器中运行。

### 构建WebAssembly

1. 首先安装wasm-pack：

```bash
cargo install wasm-pack
```

2. 构建WebAssembly包：

```bash
wasm-pack build --target web
```

这将在`./pkg`目录下生成WebAssembly模块和相应的JavaScript绑定文件。

### 在浏览器中使用

我们提供了一个示例HTML文件，展示如何在浏览器中使用此工具：

1. 查看`examples/index.html`文件中的示例代码
2. 使用本地服务器运行示例：

```bash
# 使用Python的简单HTTP服务器
python -m http.server
# 或使用Node.js的http-server
npx http-server
```

3. 在浏览器中访问`http://localhost:8000/examples/`即可使用SVG/PNG转换工具

### JavaScript API

#### SVG转PNG

```javascript
import init, { SvgConverter } from './pkg/svg_converter.js';

// 初始化WebAssembly模块
await init();

// 创建转换器实例
const converter = new SvgConverter();

// SVG转PNG
const svgContent = '<svg>...</svg>'; // SVG内容
const width = 800; // 可选，设置为0使用原始尺寸
const height = 600; // 可选，设置为0使用原始尺寸

// 返回Base64编码的PNG数据URL
const pngDataUrl = converter.svg_to_png(svgContent, width, height);
// 可以直接用于<img>标签的src属性
```

#### PNG转SVG

```javascript
import init, { SvgConverter } from './pkg/svg_converter.js';

// 初始化WebAssembly模块
await init();

// 创建转换器实例
const converter = new SvgConverter();

// PNG转SVG
const pngBase64 = 'data:image/png;base64,...'; // PNG的Base64编码数据URL
const simplifyLevel = 3; // 简化级别 (0-10)

// 返回SVG字符串
const svgContent = converter.png_to_svg(pngBase64, simplifyLevel);
```

## 技术细节

- 使用`resvg`库渲染SVG
- 使用`image`库处理PNG图像
- 使用自定义边缘检测和轮廓追踪算法将PNG转换为SVG
- WebAssembly支持通过`wasm-bindgen`实现

## 安装

确保您已安装了Rust和Cargo。然后克隆此仓库并构建项目：

```bash
git clone https://github.com/yourusername/svg-converter.git
cd svg-converter
cargo build --release
```

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