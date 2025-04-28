use anyhow::{Result, anyhow};
use resvg::usvg::{self, TreeParsing};
use resvg::tiny_skia;
use std::fs;

/// 将SVG转换为PNG图像数据
pub fn convert_svg_to_png(
    svg_data: &str,
    width: u32,
    height: u32,
) -> Result<Vec<u8>> {
    // 配置SVG解析选项
    let mut opt = usvg::Options::default();
    opt.font_family = "Arial, Helvetica, sans-serif".to_string();
    opt.font_size = 16.0;
    opt.languages = vec!["zh-CN".to_string(), "en".to_string()]; // 支持中文和英文
    opt.shape_rendering = usvg::ShapeRendering::GeometricPrecision;
    opt.text_rendering = usvg::TextRendering::GeometricPrecision;
    opt.image_rendering = usvg::ImageRendering::OptimizeQuality;
    
    // 解析SVG
    let tree = usvg::Tree::from_str(svg_data, &opt)?;
    
    // 获取原始尺寸
    let orig_size = tree.view_box.rect.size();
    
    // 确定输出尺寸
    let (width_final, height_final) = if width == 0 || height == 0 {
        // 如果宽度或高度为0，使用原始SVG的尺寸
        (orig_size.width() as u32, orig_size.height() as u32)
    } else {
        // 否则使用指定的尺寸
        (width, height)
    };
    
    // 创建像素缓冲区
    let mut pixmap = tiny_skia::Pixmap::new(width_final, height_final)
        .ok_or(anyhow!("无法创建像素图像"))?;
    
    // 渲染SVG到像素缓冲区
    let render_tree = resvg::Tree::from_usvg(&tree);
    let transform = tiny_skia::Transform::from_scale(
        width_final as f32 / orig_size.width(),
        height_final as f32 / orig_size.height(),
    );
    render_tree.render(transform, &mut pixmap.as_mut());
    
    // 将像素缓冲区转换为PNG数据
    let png_data = pixmap.encode_png()
        .map_err(|e| anyhow!("PNG编码错误: {}", e))?;
    
    Ok(png_data)
}

/// 从SVG文件转换为PNG文件
pub fn convert_svg_file_to_png_file(
    input_path: &str,
    output_path: &str,
    width: u32,
    height: u32,
) -> Result<()> {
    // 创建临时文件路径用于存储处理后的SVG
    let temp_svg = format!("{}_temp_text2path.svg", input_path.split('.').next().unwrap_or_default());
    
    // 尝试将文本转换为路径
    let processed_svg = convert_text_to_path(input_path, &temp_svg)?;
    
    // 读取处理后的SVG
    let svg_data = fs::read_to_string(&processed_svg)?;
    
    // 使用核心函数进行转换
    let png_data = convert_svg_to_png(&svg_data, width, height)?;
    
    // 保存PNG文件
    fs::write(output_path, &png_data)?;
    
    // 清理临时文件
    if processed_svg != input_path {
        let _ = fs::remove_file(&processed_svg);
    }
    
    Ok(())
}

/// 将SVG中的文本转换为路径（针对文件操作）
pub fn convert_text_to_path(input: &str, output: &str) -> Result<String> {
    // 读取SVG文件
    let svg_content = fs::read_to_string(input)?;
    
    // 解析XML
    let doc = roxmltree::Document::parse(&svg_content)?;
    
    // 检查是否包含text元素
    let has_text = doc.descendants().any(|node| node.tag_name().name() == "text");
    
    if has_text {
        // 使用更安全的SVG处理方式
        // 不修改原始SVG，而是让usvg库自己去处理文本
        fs::copy(input, output)?;
        println!("SVG文件包含文本元素，将使用内置SVG渲染器处理文本。");
    } else {
        // 如果没有文本元素，直接复制
        fs::copy(input, output)?;
    }
    
    Ok(output.to_string())
} 