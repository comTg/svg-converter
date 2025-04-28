use std::error::Error;

use clap::{Parser, Subcommand};

mod core;
mod utils;

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
            // 使用新的核心模块
            core::svg2png::convert_svg_file_to_png_file(input, output, *width, *height)?;
            println!("成功将SVG转换为PNG：{} -> {}", input, output);
        }
        Commands::PngToSvg { input, output, simplify } => {
            // 使用新的核心模块
            core::png2svg::convert_png_file_to_svg_file(input, output, *simplify)?;
            println!("成功将PNG转换为SVG：{} -> {}", input, output);
        }
    }

    Ok(())
}
