use thiserror::Error;

#[derive(Error, Debug)]
pub enum GeoContainsError {
    #[error("IOエラー: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSONパースエラー: {0}")]
    Json(#[from] serde_json::Error),

    #[error("GeoJSONパースエラー: {0}")]
    GeoJson(String),

    #[error("無効な座標: {0}")]
    InvalidCoordinate(String),

    #[error("無効なジオメトリ: {0}")]
    InvalidGeometry(String),

    #[error("その他のエラー: {0}")]
    Other(String),
}

pub type Result<T> = std::result::Result<T, GeoContainsError>;
