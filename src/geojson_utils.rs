use geo::{Contains, Coord, Point, Polygon};
use geojson::{GeoJson, Geometry, Value};
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

use crate::error::Result;

pub fn load_geojson<P: AsRef<Path>>(path: P) -> Result<GeoJson> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let geojson = GeoJson::from_reader(reader)?;
    Ok(geojson)
}

pub fn extract_polygons(geojson: &GeoJson) -> Result<Vec<Polygon>> {
    let mut polygons = Vec::new();

    match geojson {
        GeoJson::FeatureCollection(collection) => {
            for feature in &collection.features {
                if let Some(geometry) = &feature.geometry {
                    extract_polygons_from_geometry(geometry, &mut polygons)?;
                }
            }
        }
        GeoJson::Feature(feature) => {
            if let Some(geometry) = &feature.geometry {
                extract_polygons_from_geometry(geometry, &mut polygons)?;
            }
        }
        GeoJson::Geometry(geometry) => {
            extract_polygons_from_geometry(geometry, &mut polygons)?;
        }
    }

    Ok(polygons)
}

fn extract_polygons_from_geometry(geometry: &Geometry, polygons: &mut Vec<Polygon>) -> Result<()> {
    match &geometry.value {
        Value::Polygon(coords) => {
            if let Some(polygon) = convert_coordinates_to_polygon(coords) {
                polygons.push(polygon);
            }
        }
        Value::MultiPolygon(multi_coords) => {
            for coords in multi_coords {
                if let Some(polygon) = convert_coordinates_to_polygon(coords) {
                    polygons.push(polygon);
                }
            }
        }
        Value::GeometryCollection(geometries) => {
            for geometry in geometries {
                extract_polygons_from_geometry(geometry, polygons)?;
            }
        }
        _ => {}
    }

    Ok(())
}

fn convert_coordinates_to_polygon(coords: &Vec<Vec<Vec<f64>>>) -> Option<Polygon> {
    if coords.is_empty() || coords[0].is_empty() {
        return None;
    }

    let exterior: Vec<Coord> = coords[0]
        .iter()
        .map(|coord| {
            if coord.len() >= 2 {
                Coord {
                    x: coord[0],
                    y: coord[1],
                }
            } else {
                Coord { x: 0.0, y: 0.0 }
            }
        })
        .collect();

    let interiors: Vec<geo::LineString> = coords
        .iter()
        .skip(1)
        .map(|ring| {
            let coords: Vec<Coord> = ring.iter()
                .map(|coord| {
                    if coord.len() >= 2 {
                        Coord {
                            x: coord[0],
                            y: coord[1],
                        }
                    } else {
                        Coord { x: 0.0, y: 0.0 }
                    }
                })
                .collect();
            coords.into()
        })
        .collect();

    Some(Polygon::new(exterior.into(), interiors))
}

pub fn point_in_polygons(lat: f64, lng: f64, polygons: &[Polygon]) -> bool {
    let point = Point::new(lng, lat);
    polygons.iter().any(|polygon| polygon.contains(&point))
}

pub fn point_in_geojson(lat: f64, lng: f64, geojson: &GeoJson) -> Result<bool> {
    let polygons = extract_polygons(geojson)?;
    Ok(point_in_polygons(lat, lng, &polygons))
}
