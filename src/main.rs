use clap::{Parser, Subcommand};
use geo_contains::{
    error::{GeoContainsError, Result},
    geojson_utils::{load_geojson, point_in_geojson, extract_polygons},
    spatial_index::SpatialIndex,
};
use h3o::Resolution;
use rayon::prelude::*;
use std::path::{Path, PathBuf};
use std::time::Instant;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Check {
        #[arg(short = 'a', long)]
        lat: f64,

        #[arg(short = 'o', long)]
        lng: f64,

        #[arg(short, long)]
        file: PathBuf,

        #[arg(short, long, default_value_t = false)]
        use_index: bool,

        #[arg(short, long, default_value_t = 9)]
        resolution: u8,
    },

    BatchCheck {
        #[arg(short = 'a', long)]
        lat: f64,

        #[arg(short = 'o', long)]
        lng: f64,

        #[arg(short, long)]
        dir: PathBuf,

        #[arg(short, long, default_value_t = false)]
        use_index: bool,

        #[arg(short, long, default_value_t = 9)]
        resolution: u8,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Check {
            lat,
            lng,
            file,
            use_index,
            resolution,
        } => {
            check_point(*lat, *lng, file, *use_index, *resolution)?;
        }
        Commands::BatchCheck {
            lat,
            lng,
            dir,
            use_index,
            resolution,
        } => {
            batch_check_point(*lat, *lng, dir, *use_index, *resolution)?;
        }
    }

    Ok(())
}

fn check_point(lat: f64, lng: f64, file: &Path, use_index: bool, resolution: u8) -> Result<()> {
    println!("GeoJSONファイル '{}' を処理中...", file.display());
    
    let start = Instant::now();
    let geojson = load_geojson(file)?;
    println!("GeoJSONファイルの読み込み: {:?}", start.elapsed());

    if use_index {
        let start = Instant::now();
        let polygons = extract_polygons(&geojson)?;
        println!("ポリゴンの抽出: {:?}", start.elapsed());
        println!("ポリゴン数: {}", polygons.len());

        let start = Instant::now();
        let resolution = Resolution::try_from(resolution).unwrap_or(Resolution::Nine);
        let mut index = SpatialIndex::new(polygons).with_resolution(resolution);
        index.build_index()?;
        println!("インデックスの構築: {:?}", start.elapsed());
        println!("インデックスサイズ: {}", index.index_size());

        let start = Instant::now();
        let result = index.contains(lat, lng);
        println!("点の判定: {:?}", start.elapsed());
        
        println!(
            "点 ({}, {}) は{}ポリゴン内にあります",
            lat,
            lng,
            if result { "" } else { "**ではなく、" }
        );
    } else {
        let start = Instant::now();
        let result = point_in_geojson(lat, lng, &geojson)?;
        println!("点の判定: {:?}", start.elapsed());
        
        println!(
            "点 ({}, {}) は{}ポリゴン内にあります",
            lat,
            lng,
            if result { "" } else { "**ではなく、" }
        );
    }

    Ok(())
}

fn batch_check_point(lat: f64, lng: f64, dir: &Path, use_index: bool, resolution: u8) -> Result<()> {
    println!("ディレクトリ '{}' 内のGeoJSONファイルを処理中...", dir.display());
    
    let entries = std::fs::read_dir(dir)?;
    let geojson_files: Vec<PathBuf> = entries
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let path = entry.path();
            if path.extension()?.to_str()? == "geojson" {
                Some(path)
            } else {
                None
            }
        })
        .collect();

    println!("GeoJSONファイル数: {}", geojson_files.len());

    if geojson_files.is_empty() {
        return Err(GeoContainsError::Other("GeoJSONファイルが見つかりませんでした".to_string()));
    }

    let results: Vec<(PathBuf, bool)> = if use_index {
        geojson_files
            .par_iter()
            .filter_map(|file| {
                let geojson = load_geojson(file).ok()?;
                let polygons = extract_polygons(&geojson).ok()?;
                let resolution = Resolution::try_from(resolution).unwrap_or(Resolution::Nine);
                let mut index = SpatialIndex::new(polygons).with_resolution(resolution);
                index.build_index().ok()?;
                let result = index.contains(lat, lng);
                Some((file.clone(), result))
            })
            .collect()
    } else {
        geojson_files
            .par_iter()
            .filter_map(|file| {
                let geojson = load_geojson(file).ok()?;
                let result = point_in_geojson(lat, lng, &geojson).ok()?;
                Some((file.clone(), result))
            })
            .collect()
    };

    for (file, result) in &results {
        println!(
            "ファイル '{}': 点 ({}, {}) は{}ポリゴン内にあります",
            file.display(),
            lat,
            lng,
            if *result { "" } else { "**ではなく、" }
        );
    }

    let contained_count = results.iter().filter(|(_, result)| *result).count();
    println!(
        "結果: 点 ({}, {}) は {} 個のGeoJSONファイル内に含まれています（全 {} ファイル中）",
        lat, lng, contained_count, results.len()
    );

    Ok(())
}
