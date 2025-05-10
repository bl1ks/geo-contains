use geo::{Contains, Point, Polygon};
use h3o::{CellIndex, Resolution};
use h3o::geom::{self, PolyfillConfig, ToCells};
use rayon::prelude::*;

use crate::error::Result;

pub struct SpatialIndex {
    polygons: Vec<Polygon>,
    indexed: bool,
    h3_cells: Vec<CellIndex>,
    resolution: Resolution,
}

impl SpatialIndex {
    pub fn new(polygons: Vec<Polygon>) -> Self {
        Self {
            polygons,
            indexed: false,
            h3_cells: Vec::new(),
            resolution: Resolution::Nine, // デフォルト解像度
        }
    }

    pub fn with_resolution(mut self, resolution: Resolution) -> Self {
        self.resolution = resolution;
        self
    }

    pub fn build_index(&mut self) -> Result<()> {
        if self.indexed {
            return Ok(());
        }

        let config = PolyfillConfig::new(self.resolution);
        
        self.h3_cells = self
            .polygons
            .par_iter()
            .flat_map(|polygon| {
                match geom::Polygon::from_degrees(polygon.clone()) {
                    Ok(h3_polygon) => {
                        h3_polygon.to_cells(config).collect::<Vec<_>>()
                    },
                    Err(_) => Vec::new(),
                }
            })
            .collect();

        self.indexed = true;
        Ok(())
    }

    pub fn contains(&self, lat: f64, lng: f64) -> bool {
        let point = Point::new(lng, lat);

        if !self.indexed {
            return self.polygons.iter().any(|polygon| polygon.contains(&point));
        }

        let latlng = match h3o::LatLng::new(lat, lng) {
            Ok(latlng) => latlng,
            Err(_) => return false,
        };
        
        let cell = latlng.to_cell(self.resolution);
        
        self.h3_cells.contains(&cell) || self.polygons.iter().any(|polygon| polygon.contains(&point))
    }

    pub fn contains_batch(&self, points: &[(f64, f64)]) -> Vec<bool> {
        points
            .par_iter()
            .map(|(lat, lng)| self.contains(*lat, *lng))
            .collect()
    }

    pub fn index_size(&self) -> usize {
        self.h3_cells.len()
    }

    pub fn polygon_count(&self) -> usize {
        self.polygons.len()
    }
}
