use std::cell::RefCell;
use std::rc::Rc;

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::MouseEvent;
use web_sys::WebGlProgram;
use web_sys::WebGlShader;
use web_sys::{HtmlCanvasElement, WebGlRenderingContext as GL};

const VERT_SHADER: &'static str = r#"
        attribute vec2 coordinates;
        void main(void) {
            gl_Position = vec4(coordinates, 0.0, 1.0);
            gl_PointSize = 10.0;
        }
    "#;

const FRAG_SHADER: &'static str = r#"
        void main(void) {
            gl_FragColor = vec4(1.0, 0.0, 0.0, 1.0);
        }
    "#;

#[wasm_bindgen]
pub fn render(canvas_id: &str) {
    Rustex::new(canvas_id);
}

pub struct Rustex {
    pub gl: GL,
    pub canvas: HtmlCanvasElement,
    pub vertex_buffer: Vec<f32>,
    pub mouse_x: f32,
    pub mouse_y: f32,
}

impl Rustex {
    pub fn new(canvas_id: &str) {
        let document = web_sys::window().unwrap().document().unwrap();
        let canvas = document
            .get_element_by_id(canvas_id)
            .unwrap()
            .dyn_into::<HtmlCanvasElement>()
            .unwrap();

        let x_offset = canvas.get_bounding_client_rect().left() as i32;
        let y_offset = canvas.get_bounding_client_rect().top() as i32;

        let c_width = canvas.width() + 2;
        let c_height = canvas.height() + 2;

        let gl = canvas
            .get_context("webgl")
            .unwrap()
            .unwrap()
            .dyn_into::<GL>()
            .unwrap();
        gl.viewport(0, 0, canvas.width() as i32, canvas.height() as i32);

        let rustex = Rc::new(RefCell::new(Rustex {
            gl,
            canvas,
            vertex_buffer: vec![],
            mouse_x: 0.0,
            mouse_y: 0.0,
        }));
        {
            let click_clone = rustex.clone();
            let closure = Closure::wrap(Box::new(move |event: MouseEvent| {
                let x = ((event.client_x() - x_offset) as f32 / c_width as f32) * 2.0 - 1.0;
                let y = ((event.client_y() - y_offset) as f32 / c_height as f32) * -2.0 + 1.0;
                log(&format!("x: {x}, y: {y}"));
                place_vertex(&click_clone, x, y);
            }) as Box<dyn FnMut(_)>);

            rustex
                .borrow()
                .canvas
                .add_event_listener_with_callback("click", closure.as_ref().unchecked_ref())
                .unwrap();
            closure.forget();
        }
        {
            let move_clone = rustex.clone();
            let mouse_closure = Closure::wrap(Box::new(move |event: MouseEvent| {
                let x = (event.client_x() as f32 / c_width as f32) * 2.0 - 1.0;
                let y = (event.client_y() as f32 / c_height as f32) * -2.0 + 1.0;
                //log(&format!("x: {x}, y: {y}"));
            }) as Box<dyn FnMut(_)>);

            rustex
                .borrow()
                .canvas
                .add_event_listener_with_callback(
                    "mousemove",
                    mouse_closure.as_ref().unchecked_ref(),
                )
                .unwrap();
            mouse_closure.forget();
        }
    }
}

fn prepare_vertex_buffer(gl: &GL) {
    let vert_shader = compile_shader(&gl, GL::VERTEX_SHADER, VERT_SHADER).unwrap();
    let frag_shader = compile_shader(&gl, GL::FRAGMENT_SHADER, FRAG_SHADER).unwrap();

    let program = link_program(&gl, &vert_shader, &frag_shader).unwrap();
    gl.use_program(Some(&program));

    let buffer = gl.create_buffer().ok_or("Failed to create buffer").unwrap();
    gl.bind_buffer(GL::ARRAY_BUFFER, Some(&buffer));

    let coord = gl.get_attrib_location(&program, "coordinates") as u32;
    gl.enable_vertex_attrib_array(coord);
    gl.vertex_attrib_pointer_with_i32(coord, 2, GL::FLOAT, false, 0, 0);
}

fn compile_shader(gl: &GL, shader_type: u32, source: &str) -> Result<WebGlShader, JsValue> {
    let shader = gl
        .create_shader(shader_type)
        .ok_or_else(|| JsValue::from_str("Unable to create shader object"))?;
    gl.shader_source(&shader, source);
    gl.compile_shader(&shader);
    if gl
        .get_shader_parameter(&shader, GL::COMPILE_STATUS)
        .as_bool()
        .unwrap_or(false)
    {
        Ok(shader)
    } else {
        Err(JsValue::from_str(
            &gl.get_shader_info_log(&shader)
                .unwrap_or_else(|| "Unknown error creating shader".into()),
        ))
    }
}

fn link_program(
    gl: &GL,
    vert_shader: &WebGlShader,
    frag_shader: &WebGlShader,
) -> Result<WebGlProgram, JsValue> {
    let shader_program = gl.create_program().unwrap();
    gl.attach_shader(&shader_program, &vert_shader);
    gl.attach_shader(&shader_program, &frag_shader);
    gl.link_program(&shader_program);

    if gl
        .get_program_parameter(&shader_program, GL::LINK_STATUS)
        .as_bool()
        .unwrap_or(false)
    {
        gl.use_program(Some(&shader_program));
        Ok(shader_program)
    } else {
        return Err(JsValue::from_str(
            &gl.get_program_info_log(&shader_program)
                .unwrap_or_else(|| "Unknown error linking program".into()),
        ));
    }
}

fn place_vertex(rustex: &Rc<RefCell<Rustex>>, x: f32, y: f32) {
    prepare_vertex_buffer(&rustex.borrow().gl);
    rustex.borrow_mut().vertex_buffer.push(x);
    rustex.borrow_mut().vertex_buffer.push(y);
    unsafe {
        let vertex_array = js_sys::Float32Array::view(&rustex.borrow().vertex_buffer);
        rustex.borrow().gl.buffer_data_with_array_buffer_view(
            GL::ARRAY_BUFFER,
            &vertex_array,
            GL::STATIC_DRAW,
        );
        rustex
            .borrow()
            .gl
            .draw_arrays(GL::POINTS, 0, (vertex_array.length() / 2) as i32);
    }
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}
