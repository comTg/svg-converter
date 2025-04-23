use std::fs;
use std::collections::HashMap;
use std::error::Error;
use std::fmt::Write as FmtWrite;

use anyhow::{Result, anyhow};
use clap::{Parser, Subcommand};
use image;
use image::GenericImageView;
use resvg::usvg::{self, TreeParsing};
use resvg::tiny_skia;
use svg::node::element::Path as SvgPath;
use svg::Document;
use roxmltree::Document as XmlDocument;

#[derive(Parser, Debug)]
#[clap(author, version, about)]
struct Args {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// 将SVG转换为PNG
    SvgToPng {
        /// 输入SVG文件路径
        input: String,
        /// 输出PNG文件路径
        #[clap(value_name = "OUTPUT")]
        output: String,
        /// 输出宽度（可选，默认使用SVG原始宽度）
        #[clap(default_value = "0")]
        width: u32,
        /// 输出高度（可选，默认使用SVG原始高度）
        #[clap(default_value = "0")]
        height: u32,
    },
    /// 将PNG转换为SVG
    PngToSvg {
        /// 输入PNG文件路径
        input: String,
        /// 输出SVG文件路径
        #[clap(value_name = "OUTPUT")]
        output: String,
        /// 简化级别 (0-10, 0表示不简化, 10表示最大简化)
        #[clap(default_value = "3")]
        simplify: u8,
    },
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    match &args.command {
        Commands::SvgToPng { input, output, width, height } => {
            svg_to_png(input, output, *width, *height)?;
        }
        Commands::PngToSvg { input, output, simplify } => {
            png_to_svg(input, output, *simplify)?;
        }
    }

    Ok(())
}

/// 将SVG转换为PNG
fn svg_to_png(
    input: &str,
    output: &str,
    width: u32,
    height: u32,
) -> Result<(), Box<dyn Error>> {
    // 创建临时文件路径用于存储处理后的SVG
    let temp_svg = format!("{}_temp_text2path.svg", input.split('.').next().unwrap_or_default());
    
    // 尝试将文本转换为路径
    let processed_svg = convert_text_to_path(input, &temp_svg)?;
    
    // 配置SVG解析选项
    let mut opt = usvg::Options::default();
    opt.resources_dir = std::path::Path::new(&processed_svg).parent().map(|p| p.to_path_buf());
    
    // 设置字体处理选项
    opt.font_family = "Arial, Helvetica, sans-serif".to_string();
    opt.font_size = 16.0;
    opt.languages = vec!["zh-CN".to_string(), "en".to_string()]; // 支持中文和英文
    opt.shape_rendering = usvg::ShapeRendering::GeometricPrecision;
    opt.text_rendering = usvg::TextRendering::GeometricPrecision;
    opt.image_rendering = usvg::ImageRendering::OptimizeQuality;
    
    // 读取处理后的SVG
    let svg_data = std::fs::read_to_string(&processed_svg)?;
    
    // 解析SVG
    let tree = usvg::Tree::from_str(&svg_data, &opt)?;
    
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
    
    // 将像素缓冲区保存为PNG
    pixmap.save_png(output)?;
    
    // 清理临时文件
    if processed_svg != input {
        let _ = std::fs::remove_file(&processed_svg);
    }
    
    println!("成功将SVG转换为PNG：{} -> {}", input, output);
    println!("尺寸: {}x{}", width_final, height_final);
    Ok(())
}

/// 将SVG中的文本转换为路径
fn convert_text_to_path(input: &str, output: &str) -> Result<String, Box<dyn Error>> {
    // 读取SVG文件
    let svg_content = fs::read_to_string(input)?;
    
    // 解析XML
    let doc = XmlDocument::parse(&svg_content)?;
    
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

/// 将PNG图像转换为SVG
fn png_to_svg(input: &str, output: &str, simplify: u8) -> Result<(), Box<dyn Error>> {
    // 读取输入PNG文件
    let img = image::open(input)?;
    
    // 创建SVG文档
    let mut document = Document::new()
        .set("width", img.width())
        .set("height", img.height())
        .set("viewBox", (0, 0, img.width(), img.height()));
    
    // 颜色分离和路径追踪
    let layers = create_color_layers(&img);
    
    for (layer, color) in layers {
        let layer_paths = trace_layer(&layer);
        
        // 应用路径简化
        let simplified_paths = simplify_paths(&layer_paths, simplify);
        
        for path in simplified_paths {
            let path_element = SvgPath::new()
                .set("fill", format!("rgba({},{},{},{})", 
                     color[0], color[1], color[2], color[3] as f64 / 255.0))
                .set("stroke", "none")
                .set("d", path);
            document = document.add(path_element);
        }
    }
    
    // 保存SVG文档
    svg::save(output, &document)?;
    
    println!("SVG file saved to: {}", output);
    Ok(())
}

/// 将一个图像分解为多个颜色图层
fn create_color_layers(img: &image::DynamicImage) -> Vec<(image::RgbaImage, [u8; 4])> {
    // 获取图像尺寸
    let (width, height) = img.dimensions(); // 使用GenericImageView trait中的dimensions方法
    
    // 转换为RGBA以便于处理
    let rgba = img.to_rgba8();
    
    // 量化颜色（减少颜色数量）
    let colors = quantize_colors(&rgba, 8);
    
    // 为每个颜色创建一个图层
    let mut layers = Vec::new();
    
    for &color in &colors {
        // 创建新的空白图层，所有像素初始化为透明
        let mut layer = image::RgbaImage::new(width, height);
        
        // 复制原图中接近当前颜色的像素
        for y in 0..height {
            for x in 0..width {
                let pixel = rgba.get_pixel(x, y);
                // 只处理不透明的像素
                if pixel[3] < 128 {
                    continue;
                }
                
                // 计算当前像素与目标颜色的距离
                let distance = color_distance(pixel, color);
                
                // 如果距离小于阈值，则将此像素添加到当前图层
                if distance < 60.0 { // 阈值可调整
                    // 使用原始色彩，但保留透明度
                    let new_pixel = image::Rgba([color[0], color[1], color[2], pixel[3]]);
                    layer.put_pixel(x, y, new_pixel);
                }
            }
        }
        
        layers.push((layer, color));
    }
    
    layers
}

/// 量化颜色，将图像简化为较少的颜色
fn quantize_colors(img: &image::RgbaImage, max_colors: usize) -> Vec<[u8; 4]> {
    let (width, height) = img.dimensions();
    let mut color_counts = HashMap::new();
    
    // 对每个像素的颜色进行计数
    for y in 0..height {
        for x in 0..width {
            let pixel = img.get_pixel(x, y);
            // 忽略透明像素
            if pixel[3] < 128 {
                continue;
            }
            
            // 简化颜色（量化为16位色深）
            let simple_pixel = [
                (pixel[0] / 16) * 16,
                (pixel[1] / 16) * 16,
                (pixel[2] / 16) * 16,
                255
            ];
            
            *color_counts.entry(simple_pixel).or_insert(0) += 1;
        }
    }
    
    // 将颜色按频率排序
    let mut colors: Vec<_> = color_counts.into_iter().collect();
    colors.sort_by(|a, b| b.1.cmp(&a.1)); // 按出现频率降序排序
    
    // 取出前max_colors个颜色
    colors.iter().take(max_colors).map(|(color, _)| *color).collect()
}

/// 计算两个颜色之间的欧几里得距离
fn color_distance(p1: &image::Rgba<u8>, p2: [u8; 4]) -> f32 {
    let r1 = p1[0] as f32;
    let g1 = p1[1] as f32;
    let b1 = p1[2] as f32;
    
    let r2 = p2[0] as f32;
    let g2 = p2[1] as f32;
    let b2 = p2[2] as f32;
    
    // 计算欧几里得距离
    ((r1 - r2).powi(2) + (g1 - g2).powi(2) + (b1 - b2).powi(2)).sqrt()
}

/// 将彩色图层转换为SVG路径
fn trace_layer(layer: &image::RgbaImage) -> Vec<String> {
    let (width, height) = layer.dimensions();
    
    // 将RGBA图层转换为灰度图用于边缘检测
    let mut gray = image::GrayImage::new(width, height);
    
    // 添加填充效果，使图层边缘更平滑
    let mut dilated = image::RgbaImage::new(width, height);
    
    // 先进行轻度膨胀操作，使边缘更连贯
    let kernel_size = 2; // 膨胀核大小
    for y in 0..height {
        for x in 0..width {
            let mut has_color = false;
            
            // 检查邻域是否有不透明像素
            for dy in -kernel_size..=kernel_size {
                for dx in -kernel_size..=kernel_size {
                    let nx = x as i32 + dx;
                    let ny = y as i32 + dy;
                    
                    if nx >= 0 && nx < width as i32 && ny >= 0 && ny < height as i32 {
                        let pixel = layer.get_pixel(nx as u32, ny as u32);
                        if pixel[3] > 128 { // 不透明像素
                            has_color = true;
                            break;
                        }
                    }
                }
                if has_color {
                    break;
                }
            }
            
            if has_color {
                dilated.put_pixel(x, y, *layer.get_pixel(x, y));
            } else {
                dilated.put_pixel(x, y, image::Rgba([0, 0, 0, 0]));
            }
        }
    }
    
    // 转换为灰度图
    for y in 0..height {
        for x in 0..width {
            let pixel = dilated.get_pixel(x, y);
            if pixel[3] > 128 { // 不透明像素
                gray.put_pixel(x, y, image::Luma([255]));
            } else {
                gray.put_pixel(x, y, image::Luma([0]));
            }
        }
    }
    
    // 平滑图像以减少过度细节
    let blurred = imageproc::filter::gaussian_blur_f32(&gray, 0.8);
    
    // 使用更合适的边缘检测参数
    let edges = enhance_edges(&blurred, 10, 40);
    
    // 使用改进的轮廓追踪
    let paths = trace_contours(&edges);
    
    // 对路径进行后处理，移除太小的路径
    paths.into_iter()
        .filter(|path| {
            // 估算路径长度，忽略太短的路径
            path.len() > 20
        })
        .map(|path| path.to_string())
        .collect()
}

/// 简化SVG路径，根据简化级别调整精度
fn simplify_paths(paths: &[String], simplify_level: u8) -> Vec<String> {
    if simplify_level == 0 {
        // No simplification
        return paths.to_vec();
    }
    
    // Convert simplify_level (0-10) to a precision factor
    // Higher simplify_level means lower precision (more simplification)
    let precision = 10.0 - (simplify_level as f64 * 0.9); // Maps 1-10 to ~9.1-1.0
    
    paths.iter()
        .map(|path| {
            // Parse the SVG path and simplify it
            let simplified_path = simplify_svg_path(path, precision);
            simplified_path
        })
        .collect()
}

fn simplify_svg_path(path: &str, precision: f64) -> String {
    // Split the path into commands (M, L, Z, etc.)
    let mut result = String::new();
    let mut prev_x = 0.0;
    let mut prev_y = 0.0;
    
    // Process each command in the path
    let mut chars = path.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            'M' | 'L' => {
                result.push(c);
                // Extract the x and y coordinates
                let mut x_str = String::new();
                let mut y_str = String::new();
                
                // Skip whitespace
                while chars.peek().map_or(false, |&c| c.is_whitespace()) {
                    chars.next();
                }
                
                // Get x coordinate
                while chars.peek().map_or(false, |&c| c.is_digit(10) || c == '.' || c == '-') {
                    x_str.push(chars.next().unwrap());
                }
                
                // Skip whitespace or comma
                while chars.peek().map_or(false, |&c| c.is_whitespace() || c == ',') {
                    chars.next();
                }
                
                // Get y coordinate
                while chars.peek().map_or(false, |&c| c.is_digit(10) || c == '.' || c == '-') {
                    y_str.push(chars.next().unwrap());
                }
                
                // Parse and round the coordinates based on precision
                if let (Ok(x), Ok(y)) = (x_str.parse::<f64>(), y_str.parse::<f64>()) {
                    // Apply precision-based rounding
                    let rounded_x = (x * precision).round() / precision;
                    let rounded_y = (y * precision).round() / precision;
                    
                    // Only add the point if it's significantly different from the previous point
                    let threshold = 1.0 / precision;
                    if c == 'M' || 
                       ((rounded_x - prev_x).abs() > threshold || 
                        (rounded_y - prev_y).abs() > threshold) {
                        // Format with appropriate precision
                        let _ = write!(result, "{:.1},{:.1}", rounded_x, rounded_y);
                        prev_x = rounded_x;
                        prev_y = rounded_y;
                    } else {
                        // Skip this point as it's too close to the previous one
                        continue;
                    }
                } else {
                    // If parsing fails, just add the original coordinates
                    result.push_str(&format!("{}{}", x_str, y_str));
                }
            },
            'Z' => {
                result.push('Z');
            },
            ' ' | ',' => {
                // Skip extra whitespace and commas
                continue;
            },
            _ => {
                // Copy any other characters as is
                result.push(c);
            }
        }
    }
    
    result
}

/// 增强图像边缘
fn enhance_edges(img: &image::GrayImage, low_threshold: u8, high_threshold: u8) -> image::GrayImage {
    let (width, height) = img.dimensions();
    let mut result = image::GrayImage::new(width, height);
    
    // 简单的Sobel边缘检测
    let sobel_x = [[-1, 0, 1], [-2, 0, 2], [-1, 0, 1]];
    let sobel_y = [[-1, -2, -1], [0, 0, 0], [1, 2, 1]];
    
    for y in 1..height-1 {
        for x in 1..width-1 {
            let mut gx = 0;
            let mut gy = 0;
            
            // 应用Sobel算子
            for i in 0..3 {
                for j in 0..3 {
                    let img_x = x as i32 + (j as i32 - 1);
                    let img_y = y as i32 + (i as i32 - 1);
                    
                    if img_x >= 0 && img_x < width as i32 && img_y >= 0 && img_y < height as i32 {
                        let pixel = img.get_pixel(img_x as u32, img_y as u32)[0] as i32;
                        gx += pixel * sobel_x[i as usize][j as usize];
                        gy += pixel * sobel_y[i as usize][j as usize];
                    }
                }
            }
            
            // 计算梯度大小
            let g = ((gx.pow(2) + gy.pow(2)) as f32).sqrt();
            
            // 应用双阈值
            if g >= high_threshold as f32 {
                result.put_pixel(x, y, image::Luma([255]));
            } else if g >= low_threshold as f32 {
                result.put_pixel(x, y, image::Luma([128]));
            } else {
                result.put_pixel(x, y, image::Luma([0]));
            }
        }
    }
    
    // 滞后阈值处理 - 连接边缘
    let mut final_result = result.clone();
    for y in 1..height-1 {
        for x in 1..width-1 {
            if result.get_pixel(x, y)[0] == 128 {
                // 检查是否与强边缘相连
                let mut is_connected = false;
                for dy in -1..=1 {
                    for dx in -1..=1 {
                        if dx == 0 && dy == 0 {
                            continue;
                        }
                        
                        let nx = x as i32 + dx;
                        let ny = y as i32 + dy;
                        
                        if nx >= 0 && nx < width as i32 && ny >= 0 && ny < height as i32 {
                            if result.get_pixel(nx as u32, ny as u32)[0] == 255 {
                                is_connected = true;
                                break;
                            }
                        }
                    }
                    if is_connected {
                        break;
                    }
                }
                
                if is_connected {
                    final_result.put_pixel(x, y, image::Luma([255]));
                } else {
                    final_result.put_pixel(x, y, image::Luma([0]));
                }
            }
        }
    }
    
    final_result
}

/// 从边缘图像跟踪轮廓并生成SVG路径
fn trace_contours(edges: &image::GrayImage) -> Vec<String> {
    let (width, height) = edges.dimensions();
    let mut visited = vec![vec![false; width as usize]; height as usize];
    let mut paths = Vec::new();
    
    // 查找起始点并跟踪轮廓
    for y in 0..height {
        for x in 0..width {
            if edges.get_pixel(x, y)[0] == 255 && !visited[y as usize][x as usize] {
                let mut contour = Vec::new();
                let mut current_x = x;
                let mut current_y = y;
                
                // 跟踪当前轮廓
                loop {
                    visited[current_y as usize][current_x as usize] = true;
                    contour.push((current_x as f32, current_y as f32));
                    
                    // 查找下一个轮廓点
                    let mut found_next = false;
                    for dy in -1..=1 {
                        for dx in -1..=1 {
                            if dx == 0 && dy == 0 {
                                continue;
                            }
                            
                            let nx = current_x as i32 + dx;
                            let ny = current_y as i32 + dy;
                            
                            if nx >= 0 && nx < width as i32 && ny >= 0 && ny < height as i32 {
                                let nx = nx as u32;
                                let ny = ny as u32;
                                
                                if edges.get_pixel(nx, ny)[0] == 255 && !visited[ny as usize][nx as usize] {
                                    current_x = nx;
                                    current_y = ny;
                                    found_next = true;
                                    break;
                                }
                            }
                        }
                        if found_next {
                            break;
                        }
                    }
                    
                    if !found_next || contour.len() > 10000 { // 防止无限循环
                        break;
                    }
                }
                
                // 创建SVG路径
                if contour.len() > 2 {
                    let mut path_data = String::new();
                    path_data.push('M');
                    path_data.push_str(&format!("{:.1},{:.1}", contour[0].0, contour[0].1));
                    
                    for i in 1..contour.len() {
                        path_data.push_str(&format!(",{:.1},{:.1}", contour[i].0, contour[i].1));
                    }
                    
                    path_data.push('Z');
                    paths.push(path_data);
                }
            }
        }
    }
    
    paths
}
