use web_sys::{HtmlCanvasElement, HtmlImageElement};
use wasm_bindgen::JsCast;
use std::borrow::Cow;

pub trait ImageSource {
    fn into_html_image_element(self) -> HtmlImageElement;
    fn into_html_canvas_element(self) -> HtmlCanvasElement;
}

impl ImageSource for HtmlImageElement {
    fn into_html_image_element(self) -> HtmlImageElement {
        self
    }

    fn into_html_canvas_element(self) -> HtmlCanvasElement {
        self.into_canvas()
    }
}

impl ImageSource for HtmlCanvasElement {
    fn into_html_image_element(self) -> HtmlImageElement {
        self.into_image()
    }

    fn into_html_canvas_element(self) -> HtmlCanvasElement {
        self
    }
}

pub enum ImageDataSource<'a> {
    HtmlImage(Cow<'a, HtmlImageElement>),
    HtmlCanvas(Cow<'a, HtmlCanvasElement>),
    // 可以添加更多类型，比如:
    // Svg(Cow<'a, SvgElement>),
    // Blob(Cow<'a, Blob>),
}

pub struct Image<'a>(ImageDataSource<'a>);

impl<'a> Image<'a> {
    pub fn new<T: Into<ImageDataSource<'a>>>(source: T) -> Self {
        Image(source.into())
    }

    pub fn as_html_image_element(&self) -> HtmlImageElement {
        match &self.0 {
            ImageDataSource::HtmlImage(img) => img.clone().into_owned(),
            ImageDataSource::HtmlCanvas(canvas) => canvas.clone().into_owned().into_html_image_element(),
            // 处理其他类型...
        }
    }

    pub fn as_html_canvas_element(&self) -> HtmlCanvasElement {
        match &self.0 {
            ImageDataSource::HtmlImage(img) => img.clone().into_owned().into_html_canvas_element(),
            ImageDataSource::HtmlCanvas(canvas) => canvas.clone().into_owned(),
            // 处理其他类型...
        }
    }
}

// 实现 From trait 以支持不同类型的转换
impl<'a> From<HtmlImageElement> for ImageDataSource<'a> {
    fn from(img: HtmlImageElement) -> Self {
        ImageDataSource::HtmlImage(Cow::Owned(img))
    }
}

impl<'a> From<&'a HtmlImageElement> for ImageDataSource<'a> {
    fn from(img: &'a HtmlImageElement) -> Self {
        ImageDataSource::HtmlImage(Cow::Borrowed(img))
    }
}

impl<'a> From<HtmlCanvasElement> for ImageDataSource<'a> {
    fn from(canvas: HtmlCanvasElement) -> Self {
        ImageDataSource::HtmlCanvas(Cow::Owned(canvas))
    }
}

impl<'a> From<&'a HtmlCanvasElement> for ImageDataSource<'a> {
    fn from(canvas: &'a HtmlCanvasElement) -> Self {
        ImageDataSource::HtmlCanvas(Cow::Borrowed(canvas))
    }
}

// 可以为其他类型实现类似的 From trait

pub trait IntoCanvas {
    fn into_canvas(self) -> HtmlCanvasElement;
}

impl IntoCanvas for HtmlImageElement {
    fn into_canvas(self) -> HtmlCanvasElement {
        let document = web_sys::window().unwrap().document().unwrap();
        let canvas = document.create_element("canvas")
            .unwrap()
            .dyn_into::<HtmlCanvasElement>()
            .unwrap();

        // 设置画布大小与图像一致
        canvas.set_width(self.natural_width());
        canvas.set_height(self.natural_height());

        // 获取 2D 渲染上下文
        let context = canvas
            .get_context("2d")
            .unwrap()
            .unwrap()
            .dyn_into::<web_sys::CanvasRenderingContext2d>()
            .unwrap();

        // 将图像绘制到画布上
        context.draw_image_with_html_image_element(&self, 0.0, 0.0)
            .expect("Failed to draw image on canvas");

        canvas
    }
}


// 定义新的 trait
pub trait IntoImage {
    fn into_image(self) -> HtmlImageElement;
}

// 为 HtmlCanvasElement 实现 IntoImage trait
impl IntoImage for HtmlCanvasElement {
    fn into_image(self) -> HtmlImageElement {
        let document = web_sys::window().unwrap().document().unwrap();
        let image = document.create_element("img").unwrap().dyn_into::<HtmlImageElement>().unwrap();
        
        // 将画布内容转换为 data URL
        let data_url = self.to_data_url().unwrap();
        
        // 设置图像的 src 为画布的 data URL
        image.set_src(&data_url);
        
        image
    }
}
