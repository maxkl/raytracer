
use std::error::Error;
use std::fmt;
use std::fmt::{Display, Formatter};

use crate::mesh::{MeshData, IndexedTriangle};

#[derive(Debug)]
pub enum ObjParseError {
    NotEnoughArguments(usize, String),
    TooManyArguments(usize, String),
    MultipleObjects(usize),
    InvalidFloat(usize),
    InvalidKeyword(usize, String),
    InvalidVertexReference(usize, String),
    IndexOutOfBounds(String),
}

impl Display for ObjParseError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            ObjParseError::NotEnoughArguments(line_number, keyword) => write!(f, "Not enough arguments to '{}' in line {}", keyword, line_number),
            ObjParseError::TooManyArguments(line_number, keyword) => write!(f, "Too many arguments to '{}' in line {}", keyword, line_number),
            ObjParseError::MultipleObjects(line_number) => write!(f, "More than one object (second object starts in line {})", line_number),
            ObjParseError::InvalidFloat(line_number) => write!(f, "Invalid float in line {}", line_number),
            ObjParseError::InvalidKeyword(line_number, keyword) => write!(f, "Invalid keyword '{}' in line {}", keyword, line_number),
            ObjParseError::InvalidVertexReference(line_number, msg) => write!(f, "Invalid vertex reference in line {}: {}", line_number, msg),
            ObjParseError::IndexOutOfBounds(name) => write!(f, "Vertex {} index out of bounds", name),
        }
    }
}

impl Error for ObjParseError {}

fn parse_multiple<I: Iterator, R, E, P: FnMut(I::Item) -> Result<R, E>>(it: I, parse_fn: P) -> Result<Vec<R>, E> {
    it.map(parse_fn)
        .collect::<Result<_, _>>()
}

fn parse_multiple_float<'s, I: Iterator>(it: I, line_number: usize) -> Result<Vec<f32>, ObjParseError>
    where
        I: Iterator<Item=&'s str>
{
    parse_multiple(it, str::parse)
        .map_err(|_| ObjParseError::InvalidFloat(line_number))
}

fn parse_vertex_ref(s: &str, line_number: usize) -> Result<(usize, Option<usize>, Option<usize>), ObjParseError> {
    let mut parts = s.split('/');

    let pos_index: usize = parts.next()
        .ok_or_else(|| ObjParseError::InvalidVertexReference(line_number, "missing position index".to_string()))?
        .parse()
        .map_err(|_| ObjParseError::InvalidVertexReference(line_number, "invalid position index".to_string()))?;
    let tex_coord_index: Option<usize> = parts.next()
        .map(|s| s.parse())
        .transpose()
        .map_err(|_| ObjParseError::InvalidVertexReference(line_number, "invalid texture coordinate index".to_string()))?;
    let normal_index: Option<usize> = parts.next()
        .map(|s| s.parse())
        .transpose()
        .map_err(|_| ObjParseError::InvalidVertexReference(line_number, "invalid normal index".to_string()))?;

    if parts.next().is_some() {
        return Err(ObjParseError::InvalidVertexReference(line_number, "too many slashes".to_string()));
    }

    // Indices in .obj start at 1
    let pos_index_0 = pos_index - 1;
    let tex_coord_index_0 = tex_coord_index.map(|i| i - 1);
    let normal_index_0 = normal_index.map(|i| i - 1);

    Ok((pos_index_0, tex_coord_index_0, normal_index_0))
}

pub struct ObjParser {}

impl ObjParser {
    pub fn parse(obj_str: &str) -> Result<MeshData, ObjParseError> {
        let mut object_name = None;
        let mut vertex_positions = Vec::new();
        let mut vertex_normals = Vec::new();
        let mut vertex_tex_coords = Vec::new();
        let mut triangles = Vec::new();

        for (i, line) in obj_str.lines().enumerate() {
            let line_number = i + 1;

            let line = line.trim_start();
            if line.starts_with("#") {
                // Ignore comments
            } else {
                let mut parts = line.split_whitespace();
                let keyword = parts.next();
                if let Some(keyword) = keyword {
                    match keyword {
                        "mtllib" | "usemtl" => {
                            // Materials not supported
                        }
                        "s" => {
                            // Smoothing groups not supported
                        }
                        "o" => {
                            let name = parts.next()
                                .ok_or_else(|| ObjParseError::NotEnoughArguments(line_number, "o".to_string()))?;

                            if parts.next().is_some() {
                                return Err(ObjParseError::TooManyArguments(line_number, "o".to_string()))
                            }

                            if object_name.is_some() {
                                return Err(ObjParseError::MultipleObjects(line_number));
                            }

                            object_name = Some(name.to_string());
                        }
                        "v" => {
                            // v <x> <y> <z> [w=1.0]
                            let parts_parsed = parse_multiple_float(parts, line_number)?;
                            if parts_parsed.len() < 3 {
                                return Err(ObjParseError::NotEnoughArguments(line_number, "v".to_string()));
                            } else if parts_parsed.len() > 4 {
                                return Err(ObjParseError::TooManyArguments(line_number, "v".to_string()));
                            }

                            let x = parts_parsed[0];
                            let y = parts_parsed[1];
                            let z = parts_parsed[2];

                            vertex_positions.push((x, y, z));
                        }
                        "vn" => {
                            // vn <x> <y> <z>
                            let parts_parsed = parse_multiple_float(parts, line_number)?;
                            if parts_parsed.len() < 3 {
                                return Err(ObjParseError::NotEnoughArguments(line_number, "vn".to_string()));
                            } else if parts_parsed.len() > 3 {
                                return Err(ObjParseError::TooManyArguments(line_number, "vn".to_string()));
                            }

                            let x = parts_parsed[0];
                            let y = parts_parsed[1];
                            let z = parts_parsed[2];

                            let mag = (x.powi(2) + y.powi(2) + z.powi(2)).sqrt();

                            vertex_normals.push((x / mag, y / mag, z / mag));
                        }
                        "vt" => {
                            // vt <u> [v=0] [w=0]
                            let parts_parsed = parse_multiple_float(parts, line_number)?;
                            if parts_parsed.len() < 1 {
                                return Err(ObjParseError::NotEnoughArguments(line_number, "vt".to_string()));
                            } else if parts_parsed.len() > 3 {
                                return Err(ObjParseError::TooManyArguments(line_number, "vt".to_string()));
                            }

                            let u = parts_parsed[0];
                            let v = parts_parsed.get(1).cloned().unwrap_or(0.0);

                            vertex_tex_coords.push((u, v));
                        }
                        "f" => {
                            // f <v0> <v1> <v2>
                            let parts_parsed = parse_multiple(parts, |part| parse_vertex_ref(part, line_number))?;
                            if parts_parsed.len() < 3 {
                                return Err(ObjParseError::NotEnoughArguments(line_number, "f".to_string()));
                            }

                            let has_tex_coords = parts_parsed[0].1.is_some();
                            let has_normals = parts_parsed[0].2.is_some();

                            for part in &parts_parsed {
                                if part.1.is_some() != has_tex_coords {
                                    return Err(ObjParseError::InvalidVertexReference(line_number, "only some vertices have texture coordinates".to_string()));
                                }

                                if part.2.is_some() != has_normals {
                                    return Err(ObjParseError::InvalidVertexReference(line_number, "only some vertices have normals".to_string()));
                                }
                            }

                            for i in 2..parts_parsed.len() {
                                let vert0 = parts_parsed[0];
                                let vert1 = parts_parsed[i - 1];
                                let vert2 = parts_parsed[i];

                                let position_indices = (vert0.0, vert1.0, vert2.0);
                                let tex_coords_indices = if has_tex_coords {
                                    Some((vert0.1.unwrap(), vert1.1.unwrap(), vert2.1.unwrap()))
                                } else {
                                    None
                                };
                                let normal_indices = if has_normals {
                                    Some((vert0.2.unwrap(), vert1.2.unwrap(), vert2.2.unwrap()))
                                } else {
                                    None
                                };

                                triangles.push(IndexedTriangle {
                                    position_indices,
                                    normal_indices,
                                    tex_coords_indices,
                                });
                            }
                        }
                        keyword => return Err(ObjParseError::InvalidKeyword(line_number, keyword.to_string()))
                    }
                }
            }
        }

        let indices_exist = |indices: &(usize, usize, usize), len: usize| {
            indices.0 < len && indices.1 < len && indices.2 < len
        };

        for triangle in &triangles {
            if !indices_exist(&triangle.position_indices, vertex_positions.len()) {
                return Err(ObjParseError::IndexOutOfBounds("position".to_string()));
            }

            if let Some(tex_coords_indices) = &triangle.tex_coords_indices {
                if !indices_exist(tex_coords_indices, vertex_tex_coords.len()) {
                    return Err(ObjParseError::IndexOutOfBounds("texture coordinates".to_string()));
                }
            }

            if let Some(normal_indices) = &triangle.normal_indices {
                if !indices_exist(normal_indices, vertex_normals.len()) {
                    return Err(ObjParseError::IndexOutOfBounds("normal".to_string()));
                }
            }
        }

        Ok(MeshData {
            vertex_positions,
            vertex_normals,
            vertex_tex_coords,
            triangles,
        })
    }
}
