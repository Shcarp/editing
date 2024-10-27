use std::borrow::Cow;
use pathfinder_canvas::{vec2i, Canvas, CanvasFontContext, CanvasImageSource, ImageData as PathfinderImageData,  Vector2F};
use pathfinder_color::ColorU;
use pathfinder_content::pattern::Pattern;
use wasm_bindgen::JsCast;
use web_sys::{HtmlCanvasElement, HtmlImageElement, ImageData, WebGl2RenderingContext};

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
}

pub struct Image<'a>(ImageDataSource<'a>);

impl<'a> Image<'a> {
    pub fn new<T: Into<ImageDataSource<'a>>>(source: T) -> Self {
        Image(source.into())
    }

    pub fn as_html_image_element(&self) -> HtmlImageElement {
        match &self.0 {
            ImageDataSource::HtmlImage(img) => img.clone().into_owned(),
            ImageDataSource::HtmlCanvas(canvas) => {
                canvas.clone().into_owned().into_html_image_element()
            } // 处理其他类型...
        }
    }

    pub fn as_html_canvas_element(&self) -> HtmlCanvasElement {
        match &self.0 {
            ImageDataSource::HtmlImage(img) => img.clone().into_owned().into_html_canvas_element(),
            ImageDataSource::HtmlCanvas(canvas) => canvas.clone().into_owned(),
            // 处理其他类型...
        }
    }

    pub fn into_pattern(self) -> Result<Pattern, String> {
        match &self.0 {
            ImageDataSource::HtmlImage(img) => Ok(img.clone().into_owned().into_pattern()),
            ImageDataSource::HtmlCanvas(canvas) => Ok(canvas.clone().into_owned().into_pattern()),
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
        let canvas = document
            .create_element("canvas")
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
        context
            .draw_image_with_html_image_element(&self, 0.0, 0.0)
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
        let image = document
            .create_element("img")
            .unwrap()
            .dyn_into::<HtmlImageElement>()
            .unwrap();

        // 将画布内容转换为 data URL
        let data_url = self.to_data_url().unwrap();

        // 设置图像的 src 为画布的 data URL
        image.set_src(&data_url);

        image
    }
}

// Define a new trait that wraps CanvasImageSource for HtmlImageElement
pub trait HtmlImageCanvasSource: Sized {
    fn to_pattern(
        self,
        dest_context: &mut pathfinder_canvas::CanvasRenderingContext2D,
        transform: pathfinder_canvas::Transform2F,
    ) -> pathfinder_content::pattern::Pattern;

    fn into_pattern(self) -> Pattern;
}

impl HtmlImageCanvasSource for HtmlCanvasElement {
    fn to_pattern(
        self,
        dest_context: &mut pathfinder_canvas::CanvasRenderingContext2D,
        transform: pathfinder_canvas::Transform2F,
    ) -> Pattern {
       let canvas: HtmlCanvasElement = self.dyn_into::<HtmlCanvasElement>().unwrap();
       let context = canvas
           .get_context("webgl2")
           .unwrap()
           .unwrap()
           .dyn_into::<WebGl2RenderingContext>()
           .unwrap();

       let framebuffer_size = vec2i(canvas.width() as i32, canvas.height() as i32);
       
       let font_context = CanvasFontContext::from_system_source();
       let new_canvas = Canvas::new(framebuffer_size.to_f32());
       
       let mut pixel_data = vec![0u8; (framebuffer_size.x() * framebuffer_size.y() * 4) as usize];
       context.read_pixels_with_opt_u8_array(
           0,
           0,
           framebuffer_size.x() as i32,
           framebuffer_size.y() as i32,
           WebGl2RenderingContext::RGBA,
           WebGl2RenderingContext::UNSIGNED_BYTE,
           Some(&mut pixel_data),
       ).unwrap();

       let image_data = PathfinderImageData {
           size: framebuffer_size,
           data: pixel_data.into_iter().map(|c| ColorU::new(c, c, c, c)).collect(),
       };

       let pattern = Pattern::from_image(image_data.into_image());

       let location = Vector2F::zero();

       let mut new_context = new_canvas.get_context_2d(font_context);
       new_context.set_transform(&transform);
       new_context.draw_image(pattern, location);

       dest_context.create_pattern_from_canvas(new_context.into_canvas(), transform)
    }

    fn into_pattern(self) -> Pattern {
        let canvas: HtmlCanvasElement = self.dyn_into::<HtmlCanvasElement>().unwrap();
        let context = canvas
            .get_context("2d")
            .unwrap()
            .unwrap()
            .dyn_into::<web_sys::CanvasRenderingContext2d>()
            .unwrap();

        let width = canvas.width() as i32;
        let height = canvas.height() as i32;
        let framebuffer_size = vec2i(width, height);
        
        // 使用 2D context 获取像素数据
        let image_data = context
            .get_image_data(0.0, 0.0, width as f64, height as f64)
            .unwrap();
        let pixel_data = image_data.data().0;

        let image_data = PathfinderImageData {
            size: framebuffer_size,
            data: pixel_data
                .chunks(4)
                .map(|chunk| ColorU::new(chunk[0], chunk[1], chunk[2], chunk[3]))
                .collect(),
        };

        Pattern::from_image(image_data.into_image())
    }
}

impl HtmlImageCanvasSource for HtmlImageElement {
    fn to_pattern(
        self,
        dest_context: &mut pathfinder_canvas::CanvasRenderingContext2D, 
        transform: pathfinder_canvas::Transform2F,
    ) -> pathfinder_content::pattern::Pattern {
        let canvas: HtmlCanvasElement = self.dyn_into::<HtmlCanvasElement>().unwrap();
        canvas.to_pattern(dest_context, transform)
    }

    fn into_pattern(self) -> Pattern {
        let canvas: HtmlCanvasElement = self.into_canvas();
        canvas.into_pattern()
    }
}

impl<'a> CanvasImageSource for Image<'a> {
    fn to_pattern(self,
        dest_context: &mut pathfinder_canvas::CanvasRenderingContext2D, 
        transform: pathfinder_canvas::Transform2F,) -> Pattern {
        self.as_html_canvas_element().to_pattern(dest_context, transform)
    }
}

// Define a new trait for converting to PathfinderImageData
pub trait ToPathfinderImageData {
    fn to_pathfinder_image_data(&self) -> PathfinderImageData;
}

// Implement the trait for ImageData
impl ToPathfinderImageData for ImageData {
    fn to_pathfinder_image_data(&self) -> PathfinderImageData {
        let width = self.width() as i32;
        let height = self.height() as i32;
        let data: Vec<u8> = self.data().0.to_vec();
        PathfinderImageData {
            size: vec2i(width, height),
            data: data.into_iter().map(|c| ColorU::new(c, c, c, 255)).collect(),
        }
    }
}
