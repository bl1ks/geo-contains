
pub mod error;
pub mod geojson_utils;
pub mod spatial_index;

pub use error::{GeoContainsError, Result};
pub use geojson_utils::{extract_polygons, load_geojson, point_in_geojson, point_in_polygons};
pub use spatial_index::SpatialIndex;
