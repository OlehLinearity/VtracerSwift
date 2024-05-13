// Copyright 2023 Tsang Hao Fung. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

uniffi::include_scaffolding!("vtracer");

mod config;
mod converter;
mod svg;

use std::fmt;

pub use config::*;
pub use converter::*;
use image::io::Reader;
pub use svg::*;
use visioncortex::Color;
pub use visioncortex::ColorImage;
use visioncortex::CompoundPath;
use visioncortex::CompoundPathElement;
use visioncortex::Path;
use visioncortex::PathF64;
use visioncortex::PathI32;
use visioncortex::PathSimplifyMode;
use visioncortex::PointF64;
use visioncortex::PointI32;
use visioncortex::PointType;
use visioncortex::Spline;
use std::io::Cursor;
use std::alloc;
use cap::Cap;


// #[global_allocator]
// static ALLOCATOR: Cap<alloc::System> = Cap::new(alloc::System, usize::max_value());

#[derive(Debug, thiserror::Error)]
pub enum SvgError {
    #[error("{0}")]
    ConversionError(String),
}
pub struct LSvgFile {
    paths: Vec<LSvgPath>,
    width: u32,
    height: u32,
    path_precision: Option<u32>,
}
pub struct LSvgPath {
    path: LCompoundPath,
    color: Color,
}

pub struct LCompoundPath {
    paths: Vec<LCompoundPathElement>,
}

pub enum LCompoundPathElement {
    PathI32 { path: PathI32 },
    PathF64 { path: PathF64 },
    Spline { spline: Spline },
}

impl LSvgFile {
    pub fn from_svg_file(svg_file: SvgFile) -> Self {
        LSvgFile {
            paths: svg_file
                .paths
                .into_iter()
                .map(LSvgPath::from_svg_path)
                .collect(),
            width: svg_file.width as u32,
            height: svg_file.height as u32,
            path_precision: svg_file.path_precision,
        }
    }
}

impl LSvgPath {
    pub fn from_svg_path(svg_path: SvgPath) -> Self {
        LSvgPath {
            path: LCompoundPath::from_compound_path(svg_path.path),
            color: svg_path.color,
        }
    }
}

impl LCompoundPath {
    pub fn from_compound_path(compound_path: CompoundPath) -> Self {
        LCompoundPath {
            paths: compound_path
                .paths
                .into_iter()
                .map(|el| match el {
                    CompoundPathElement::PathI32(p) => LCompoundPathElement::PathI32 { path: p },
                    CompoundPathElement::PathF64(p) => LCompoundPathElement::PathF64 { path: p },
                    CompoundPathElement::Spline(p) => LCompoundPathElement::Spline { spline: p },
                })
                .collect(),
        }
    }

    pub fn to_svg_string<P>(&self, close: bool, offset: P, precision: Option<u32>) -> (String, P)
    where
        P: PointType + std::ops::Sub<Output = P>,
    {
        let origin = if !self.paths.is_empty() {
            match &self.paths[0] {
                LCompoundPathElement::PathI32 { path } => P::default() - path[0].to::<P>(),
                LCompoundPathElement::PathF64 { path } => P::default() - path[0].to::<P>(),
                LCompoundPathElement::Spline { spline } => {
                    P::default() - spline.points[0].to::<P>()
                }
            }
        } else {
            P::default()
        };

        let string = self
            .paths
            .iter()
            .map(|p| match p {
                LCompoundPathElement::PathI32 { path } => {
                    path.to_svg_string(close, &origin.to_point_i32(), precision)
                }
                LCompoundPathElement::PathF64 { path } => {
                    path.to_svg_string(close, &origin.to_point_f64(), precision)
                }
                LCompoundPathElement::Spline { spline } => {
                    spline.to_svg_string(close, &origin.to_point_f64(), precision)
                }
            })
            .collect::<String>();

        (string, offset - origin)
    }
}
trait ConvertToPath64 {
    fn convert_to_path_f64(&self) -> PathF64;
}

impl ConvertToPath64 for CompoundPathElement {
    fn convert_to_path_f64(&self) -> PathF64 {
        match self {
            CompoundPathElement::PathI32(p) => p.to_path_f64(),
            CompoundPathElement::PathF64(p) => p.clone(),
            CompoundPathElement::Spline(p) => Path {
                path: p.points.clone(),
            },
        }
    }
}

impl fmt::Display for LSvgFile {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, r#"<?xml version="1.0" encoding="UTF-8"?>"#)?;
        writeln!(
            f,
            r#"<!-- Generator: visioncortex VTracer {} -->"#,
            env!("CARGO_PKG_VERSION")
        )?;
        writeln!(
            f,
            r#"<svg version="1.1" xmlns="http://www.w3.org/2000/svg" width="{}" height="{}">"#,
            self.width, self.height
        )?;

        for path in &self.paths {
            path.fmt_with_precision(f, self.path_precision)?;
        }

        writeln!(f, "</svg>")
    }
}

impl fmt::Display for LSvgPath {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.fmt_with_precision(f, None)
    }
}

impl LSvgPath {
    fn fmt_with_precision(&self, f: &mut fmt::Formatter, precision: Option<u32>) -> fmt::Result {
        let (string, offset) = self
            .path
            .to_svg_string(true, PointF64::default(), precision);
        writeln!(
            f,
            "<path d=\"{}\" fill=\"{}\" transform=\"translate({},{})\"/>",
            string,
            self.color.to_hex_string(),
            offset.x,
            offset.y
        )
    }
}

pub fn convert_image_to_svg_with_preset(
    preset: Preset,
    image_data: Vec<u8>,
    width: u32,
    height: u32,
) -> Result<LSvgFile, SvgError> {
    let config = Config::from_preset(preset);
    convert_image_to_svg(config, image_data, width, height)
}

pub fn convert_image_to_svg(
    config: Config,
    image_data: Vec<u8>,
    width: u32,
    height: u32,
) -> Result<LSvgFile, SvgError> {
    // ALLOCATOR.set_limit(400 * 1024 * 1024).unwrap();
    let image_result = Reader::new(Cursor::new(image_data))
        .with_guessed_format()
        .unwrap()
        .decode()
        .map_err(|e| SvgError::ConversionError(e.to_string()))?;
    let pixels = image_result.to_rgba8().into_vec();
    
    let color_image = ColorImage {
        pixels: pixels,
        width: width as usize,
        height: height as usize,
    };

    let result = match convert(color_image, config) {
        Ok(svg_file) => Ok(LSvgFile::from_svg_file(svg_file)),
        Err(error) => Err(SvgError::ConversionError(error)),
    };

    result
}

pub fn make_svg_string(svg_file: LSvgFile) -> String {
    svg_file.to_string()
}
