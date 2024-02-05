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
pub use svg::*;
use uniffi::FfiConverter;
use visioncortex::Color;
pub use visioncortex::ColorImage;
use visioncortex::CompoundPath;
use visioncortex::CompoundPathElement;
use visioncortex::Path;
use visioncortex::PathF64;
use visioncortex::PathSimplifyMode;
use visioncortex::PointF64;
use visioncortex::PointI32;
#[derive(Debug, thiserror::Error)]
enum SvgError {
    #[error("{0}")]
    ConversionError(String),
}
struct LSvgFile {
    paths: Vec<LSvgPath>,
    width: u32,
    height: u32,
    path_precision: Option<u32>,
}
struct LSvgPath {
    path: LCompoundPath,
    color: Color,
}
struct LCompoundPath {
    paths: Vec<PathF64>,
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
                .map(|el| el.convert_to_path_f64())
                .collect(),
        }
    }

    fn to_svg_string(
        &self,
        close: bool,
        offset: PointF64,
        precision: Option<u32>,
    ) -> (String, PointF64) {
        let origin = if !self.paths.is_empty() {
            PointF64::default() - self.paths[0].path[0]
        } else {
            PointF64::default()
        };

        let string = self
            .paths
            .iter()
            .map(|p| p.to_svg_string(close, &origin, precision))
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

fn process_image(image_data: Vec<u8>, width: u32, height: u32) -> Result<LSvgFile, SvgError> {
    let mut hardcoded_config = Config::from_preset(Preset::Photo);
    hardcoded_config.mode = PathSimplifyMode::Polygon;
    let color_image = ColorImage {
        pixels: image_data,
        width: width as usize,
        height: height as usize,
    };

    let result = match convert(color_image, hardcoded_config) {
        Ok(svg_file) => Ok(LSvgFile::from_svg_file(svg_file)),
        Err(error) => Err(SvgError::ConversionError(error)),
    };

    if let Ok(ref svg_file) = result {
        print!("{svg_file}")
    }

    result
}
