use std::path::PathBuf;

use anyhow::{Result, anyhow};
use clap::{Parser, Subcommand};
use image;
use imageproc::edges::canny;
use resvg::usvg::{self, TreeParsing};
use resvg::tiny_skia;
use svg::node::element::{path::Data, Path};
use svg::Document;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// 将SVG转换为PNG
    SvgToPng {
        /// 输入SVG文件路径
        #[arg(short, long)]
        input: PathBuf,

        /// 输出PNG文件路径
        #[arg(short, long)]
        output: PathBuf,

        /// 输出图像的宽度
        #[arg(short, long)]
        width: Option<u32>,

        /// 输出图像的高度
        #[arg(short = 'H', long)]
        height: Option<u32>,
    },
    /// 将PNG转换为SVG
    PngToSvg {
        /// 输入PNG文件路径
        #[arg(short, long)]
        input: PathBuf,

        /// 输出SVG文件路径
        #[arg(short, long)]
        output: PathBuf,

        /// 边缘检测阈值（较低阈值）
        #[arg(long, default_value = "50")]
        threshold_low: u8,

        /// 边缘检测阈值（较高阈值）
        #[arg(long, default_value = "150")]
        threshold_high: u8,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::SvgToPng { input, output, width, height } => {
            svg_to_png(input, output, *width, *height)
        },
        Commands::PngToSvg { input, output, threshold_low, threshold_high } => {
            png_to_svg(input, output, *threshold_low, *threshold_high)
        },
    }
}

/// 将SVG转换为PNG
fn svg_to_png(
    input: &PathBuf,
    output: &PathBuf,
    width: Option<u32>,
    height: Option<u32>,
) -> Result<()> {
    // 读取SVG文件内容
    let svg_data = std::fs::read_to_string(input)?;
    
    // 解析SVG
    let opt = usvg::Options::default();
    let tree = usvg::Tree::from_str(&svg_data, &opt)?;
    
    // 获取尺寸
    let orig_size = tree.view_box.rect.size();
    
    // 确定输出尺寸
    let (width_final, height_final) = match (width, height) {
        (Some(w), Some(h)) => (w, h),
        (Some(w), None) => {
            let aspect_ratio = orig_size.height() / orig_size.width();
            (w, (w as f32 * aspect_ratio) as u32)
        },
        (None, Some(h)) => {
            let aspect_ratio = orig_size.width() / orig_size.height();
            ((h as f32 * aspect_ratio) as u32, h)
        },
        (None, None) => (orig_size.width() as u32, orig_size.height() as u32),
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
    
    println!("成功将SVG转换为PNG：{} -> {}", input.display(), output.display());
    Ok(())
}

/// 将PNG转换为SVG
fn png_to_svg(
    input: &PathBuf,
    output: &PathBuf,
    threshold_low: u8,
    threshold_high: u8,
) -> Result<()> {
    // 读取PNG图像
    let img = image::open(input)?;
    
    // 将图像转换为灰度
    let gray_img = img.to_luma8();
    
    // 使用Canny边缘检测
    let edges = canny(&gray_img, threshold_low as f32, threshold_high as f32);
    
    // 创建SVG文档
    let (width, height) = (edges.width() as i32, edges.height() as i32);
    let mut document = Document::new()
        .set("width", width.to_string())
        .set("height", height.to_string())
        .set("viewBox", format!("0 0 {} {}", width, height));
    
    // 遍历图像中的边缘点并创建路径
    let mut paths = Vec::new();
    let mut current_path = Vec::new();
    let mut in_path = false;
    
    for y in 0..height {
        for x in 0..width {
            let pixel = edges.get_pixel(x as u32, y as u32);
            
            if pixel[0] > 0 {
                if !in_path {
                    // 开始新路径
                    current_path.clear();
                    current_path.push((x, y));
                    in_path = true;
                } else {
                    // 继续当前路径
                    current_path.push((x, y));
                }
            } else if in_path {
                // 结束当前路径
                if current_path.len() > 1 {
                    paths.push(current_path.clone());
                }
                in_path = false;
            }
        }
        
        // 行尾结束当前路径
        if in_path {
            if current_path.len() > 1 {
                paths.push(current_path.clone());
            }
            in_path = false;
        }
    }
    
    // 将路径添加到SVG文档
    for path_points in paths {
        if path_points.len() < 2 {
            continue;
        }
        
        let mut data = Data::new();
        let first = path_points[0];
        data = data.move_to((first.0, first.1));
        
        for point in &path_points[1..] {
            data = data.line_to((point.0, point.1));
        }
        
        let path = Path::new()
            .set("fill", "none")
            .set("stroke", "black")
            .set("stroke-width", "1")
            .set("d", data);
        
        document = document.add(path);
    }
    
    // 保存SVG文件
    svg::save(output, &document)?;
    
    println!("成功将PNG转换为SVG：{} -> {}", input.display(), output.display());
    Ok(())
}
