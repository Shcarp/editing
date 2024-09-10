use wasm_bindgen::JsCast;
use web_sys::{Document, SvgsvgElement, SvgMatrix};

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
    document.body()
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
    
    document.body()
        .ok_or("Failed to get body")?
        .append_child(&svg)
        .map_err(|_| "Failed to append SVG to body")?;
    
    Ok(svg)
}