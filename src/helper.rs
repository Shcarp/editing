use nalgebra as na;
use once_cell::sync::Lazy;
use rand::Rng;
use std::collections::HashSet;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;
use wasm_bindgen::{prelude::Closure, JsCast};
use web_sys::js_sys::Date;
use web_sys::{Document, SvgMatrix, SvgsvgElement};

use crate::log;

pub fn create_svg_matrix() -> Result<SvgMatrix, String> {
    // 获取当前文档
    let document = web_sys::window()
        .ok_or("Failed to get window")?
        .document()
        .ok_or("Failed to get document")?;

    // 创建一个临时的 SVG 元素
    let svg = create_temporary_svg(&document)?;

    // 使用 SVG 元素创建矩阵
    let matrix = svg.create_svg_matrix();

    // 清理：从文档中移除临时 SVG 元素
    document
        .body()
        .ok_or("Failed to get body")?
        .remove_child(&svg.dyn_into::<web_sys::Element>().unwrap())
        .map_err(|_| "Failed to remove temporary SVG element")?;

    Ok(matrix)
}

fn create_temporary_svg(document: &Document) -> Result<SvgsvgElement, String> {
    let svg = document
        .create_element_ns(Some("http://www.w3.org/2000/svg"), "svg")
        .map_err(|_| "Failed to create SVG element")?
        .dyn_into::<SvgsvgElement>()
        .map_err(|_| "Failed to cast to SvgsvgElement")?;

    document
        .body()
        .ok_or("Failed to get body")?
        .append_child(&svg)
        .map_err(|_| "Failed to append SVG to body")?;

    Ok(svg)
}

pub fn request_animation_frame(f: &Closure<dyn FnMut()>) -> i32 {
    web_sys::window()
        .unwrap()
        .request_animation_frame(f.as_ref().unchecked_ref())
        .expect("should register `requestAnimationFrame` OK")
}

// 生成id
static COUNTER: AtomicU64 = AtomicU64::new(0);

pub fn generate_id() -> String {
    let timestamp = Date::new_0().get_time();
    let random_part: u32 = rand::thread_rng().gen();
    let counter = COUNTER.fetch_add(1, Ordering::Relaxed);
    format!("{}-{:x}-{:x}", timestamp as u64, random_part, counter)
}

static USED_COLORS: Lazy<Mutex<HashSet<String>>> = Lazy::new(|| Mutex::new(HashSet::new()));

pub fn generate_color_id() -> [i32; 4] {
    loop {
        let color: [i32; 4] = generate_random_color();
        let color_string = color_to_string(&color);
        let mut used_colors = USED_COLORS.lock().unwrap();
        if !used_colors.contains(&color_string) {
            used_colors.insert(color_string);
            return color;
        }
    }
}

fn generate_random_color() -> [i32; 4] {
    let mut rng = rand::thread_rng();
    [
        rng.gen_range(0..256),
        rng.gen_range(0..256),
        rng.gen_range(0..256),
        255,
    ]
}

fn color_to_string(color: &[i32; 4]) -> String {
    format!("{}-{}-{}-{}", color[0], color[1], color[2], color[3])
}

pub fn normalize_if_needed(mut matrix: na::Matrix1x6<f64>) -> na::Matrix1x6<f64> {
    const NORMALIZATION_THRESHOLD: f64 = 0.9999;
    let det = matrix[0] * matrix[3] - matrix[1] * matrix[2];
    if (det - 1.0).abs() > NORMALIZATION_THRESHOLD {
        let scale = det.sqrt();
        for i in 0..4 {
            matrix[i] /= scale;
        }
    }

    matrix
}

pub fn normalize_3x3_if_needed(mut matrix: na::Matrix3<f64>) -> na::Matrix3<f64> {
    const NORMALIZATION_THRESHOLD: f64 = 0.9999;

    let det = matrix[(0, 0)] * matrix[(1, 1)] - matrix[(0, 1)] * matrix[(1, 0)];

    if (det - 1.0).abs() > NORMALIZATION_THRESHOLD {
        let scale = det.sqrt();

        for i in 0..2 {
            for j in 0..3 {
                matrix[(i, j)] /= scale;
            }
        }
    }

    matrix[(2, 0)] = 0.0;
    matrix[(2, 1)] = 0.0;
    matrix[(2, 2)] = 1.0;

    matrix
}

// 将 1 * 6 转为 3 * 3
pub fn convert_1x6_to_3x3(matrix: na::Matrix1x6<f64>) -> na::Matrix3<f64> {
    na::Matrix3::new(
        matrix[0], matrix[2], matrix[4], 
        matrix[1], matrix[3], matrix[5], 
        0.0, 0.0, 1.0,
    )
} 

pub fn convert_3x3_to_1x6(matrix: na::Matrix3<f64>) -> na::Matrix1x6<f64> {
    na::Matrix1x6::new(
        matrix[(0, 0)],
        matrix[(1, 0)],
        matrix[(0, 1)],
        matrix[(1, 1)],
        matrix[(0, 2)],
        matrix[(1, 2)],
    )
}

pub fn get_rotation_matrix(angle_radians: f64) -> na::Matrix3<f64> {
    const EPSILON: f64 = 1e-6;
    if angle_radians.abs() < EPSILON {
        let sin = angle_radians;
        let cos = 1.0 - 0.5 * angle_radians * angle_radians;
        na::Matrix3::new(
            cos, -sin, 0.0, 
            sin, cos, 0.0,
            0.0, 0.0, 1.0
        )
    } else {
        let (sin, cos) = angle_radians.sin_cos();
        na::Matrix3::new(
            cos, -sin, 0.0, 
            sin, cos, 0.0, 
            0.0, 0.0, 1.0
        )
    }
}

pub fn multiply_transform_matrices(a: na::Matrix1x6<f64>, b: na::Matrix1x6<f64>, is_2x2: bool) -> na::Matrix1x6<f64> {
    let mut result = na::Matrix1x6::zeros();

    // 计算 2x2 或 3x3 部分
    result[0] = a[0] * b[0] + a[2] * b[1];
    result[1] = a[1] * b[0] + a[3] * b[1];
    result[2] = a[0] * b[2] + a[2] * b[3];
    result[3] = a[1] * b[2] + a[3] * b[3];

    // 如果不是 2x2 矩阵，计算平移部分
    if !is_2x2 {
        result[4] = a[0] * b[4] + a[2] * b[5] + a[4];
        result[5] = a[1] * b[4] + a[3] * b[5] + a[5];
    }

    result
}

pub fn print_matrice(name: &str, matrix: na::Matrix1x6<f64>) {
    log(&format!("{} offset {},{}, {}, {}, {}, {}", name, matrix[0], matrix[1], matrix[2], matrix[3], matrix[4], matrix[5]));
}

pub fn print_matrix_3x3(name: &str, matrix: na::Matrix3<f64>) {
    log(&format!(
        "{} matrix:\n[{:.6}, {:.6}, {:.6}\n {:.6}, {:.6}, {:.6}\n {:.6}, {:.6}, {:.6}]",
        name,
        matrix[(0, 0)], matrix[(0, 1)], matrix[(0, 2)],
        matrix[(1, 0)], matrix[(1, 1)], matrix[(1, 2)],
        matrix[(2, 0)], matrix[(2, 1)], matrix[(2, 2)]
    ));
}
