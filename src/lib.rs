use wasm_bindgen::prelude::*;
use image::DynamicImage;
use anyhow::Result;

mod core;
mod utils;

// 当wasm发生panic时使用console.error输出错误
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn error(s: &str);
}

// 设置panic钩子
#[wasm_bindgen(start)]
pub fn start() {
    std::panic::set_hook(Box::new(|info| {
        error(&format!("panic: {:?}", info));
    }));
}

#[wasm_bindgen]
pub struct SvgConverter {
    // 保存最近一次操作的结果
    last_result: Option<Vec<u8>>,
}

#[wasm_bindgen]
impl SvgConverter {
    /// 创建一个新的SVG转换器实例
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        SvgConverter {
            last_result: None,
        }
    }

    /// 将SVG转换为PNG
    /// 
    /// @param svg_data - SVG数据（字符串）
    /// @param width - 输出宽度（可选，设为0使用原始宽度）
    /// @param height - 输出高度（可选，设为0使用原始高度）
    /// @returns Base64编码的PNG数据
    #[wasm_bindgen]
    pub fn svg_to_png(&mut self, svg_data: &str, width: u32, height: u32) -> Result<String, JsValue> {
        match core::svg2png::convert_svg_to_png(svg_data, width, height) {
            Ok(png_data) => {
                self.last_result = Some(png_data.clone());
                // 使用工具模块处理Base64编码
                let data_url = utils::encode_to_base64_data_url(&png_data, "image/png");
                Ok(data_url)
            },
            Err(e) => Err(JsValue::from_str(&format!("Error converting SVG to PNG: {}", e))),
        }
    }

    /// 将PNG转换为SVG
    /// 
    /// @param png_data_base64 - Base64编码的PNG数据（需要包含MIME前缀）
    /// @param simplify - 简化级别 (0-10, 0表示不简化, 10表示最大简化)
    /// @returns SVG数据（字符串）
    #[wasm_bindgen]
    pub fn png_to_svg(&mut self, png_data_base64: &str, simplify: u8) -> Result<String, JsValue> {
        // 从Base64解码PNG数据
        let png_data = utils::extract_base64_data(png_data_base64)
            .map_err(|e| JsValue::from_str(&format!("Invalid base64 data: {}", e)))?;
        
        // 加载图像
        let img = image::load_from_memory(&png_data)
            .map_err(|e| JsValue::from_str(&format!("Failed to load image: {}", e)))?;
        
        // 使用核心模块将PNG转换为SVG
        match core::png2svg::convert_png_to_svg(&img, simplify) {
            Ok(svg_data) => {
                self.last_result = Some(svg_data.clone().into_bytes());
                Ok(svg_data)
            },
            Err(e) => Err(JsValue::from_str(&format!("Error converting PNG to SVG: {}", e))),
        }
    }

    /// 获取最后生成的文件作为字节数组
    #[wasm_bindgen]
    pub fn get_last_result(&self) -> Option<Box<[u8]>> {
        self.last_result.clone().map(|data| data.into_boxed_slice())
    }
} 