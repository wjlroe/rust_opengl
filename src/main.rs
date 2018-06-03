extern crate cgmath;
#[macro_use]
extern crate gfx;
extern crate gfx_window_glutin;
extern crate glutin;

use cgmath::{Angle, Deg, Matrix4, Rad, Vector3, Zero};
use gfx::traits::FactoryExt;
use gfx::{format, Device};
use gfx_window_glutin as gfx_glutin;
use glutin::Api::OpenGl;
use glutin::{
    ContextBuilder, Event, EventsLoop, GlContext, GlRequest, KeyboardInput, VirtualKeyCode,
    WindowBuilder, WindowEvent,
};
use std::time::Instant;

pub type ColorFormat = format::Rgba8;
pub type DepthFormat = format::DepthStencil;

gfx_defines!{
    vertex Vertex {
        pos: [f32; 4] = "a_Pos",
        color: [f32; 3] = "a_Color",
    }

    constant Locals {
        view: [[f32; 4]; 4] = "u_View",
        projection: [[f32; 4]; 4] = "u_Projection",
    }

    pipeline pipe {
        vbuf: gfx::VertexBuffer<Vertex> = (),
        locals: gfx::ConstantBuffer<Locals> = "Locals",
        out_color: gfx::RenderTarget<ColorFormat> = "Target0",
        out_depth: gfx::DepthTarget<DepthFormat> =
            gfx::preset::depth::LESS_EQUAL_WRITE,
    }
}

impl Vertex {
    fn new(p: [f32; 3], t: [f32; 3]) -> Vertex {
        Vertex {
            pos: [p[0], p[1], p[2], 1.0],
            color: [t[0], t[1], t[2]],
        }
    }
}

fn main() {
    let mut events_loop = EventsLoop::new();
    let windowbuilder = WindowBuilder::new()
        .with_title("Triangle Example".to_string())
        .with_dimensions(640, 480);
    let contextbuilder = ContextBuilder::new()
        .with_gl(GlRequest::Specific(OpenGl, (3, 2)))
        .with_vsync(true);
    let (window, mut device, mut factory, color_view, depth_view) =
        gfx_glutin::init::<ColorFormat, DepthFormat>(windowbuilder, contextbuilder, &events_loop);
    let shade_lang = device.get_info().shading_language;
    println!("shader lang is {:?}", shade_lang);
    println!("color_view: {:?}", color_view);
    println!("depth_view: {:?}", depth_view);

    let pso = factory
        .create_pipeline_simple(
            include_bytes!("shader/anton.vert"),
            include_bytes!("shader/anton.frag"),
            pipe::new(),
        )
        .unwrap();

    let vertex_data = [
        Vertex::new([0.0, 1.0, 0.0], [1.0, 0.0, 0.0]),
        Vertex::new([0.5, 0.0, 0.0], [0.0, 1.0, 0.0]),
        Vertex::new([-0.5, 0.0, 0.0], [0.0, 0.0, 1.0]),
        Vertex::new([-0.5, 0.0, 0.0], [0.8, 0.2, 0.2]),
        Vertex::new([0.5, 0.0, 0.0], [0.2, 0.8, 0.2]),
        Vertex::new([0.0, -1.0, 0.0], [0.2, 0.2, 0.8]),
    ];

    let (vbuf, slice) = factory.create_vertex_buffer_with_slice(&vertex_data, ());

    let data = pipe::Data {
        vbuf,
        locals: factory.create_constant_buffer(1),
        out_color: color_view,
        out_depth: depth_view,
    };
    println!("out_color: {:?}", data.out_color);
    println!("out_depth: {:?}", data.out_depth);

    let mut encoder: gfx::Encoder<_, _> = factory.create_command_buffer().into();

    let mut last_frame_time = Instant::now();
    let cam_speed = 1.0;
    let cam_yaw_speed = 10.0;
    let mut cam_pos = Vector3::new(0.0, 0.0, 2.0);
    let mut cam_yaw = Deg::zero();

    let near = 0.1;
    let far = 100.0;
    let fov = Rad::from(Deg(67.0));
    let (width, height) = window
        .get_inner_size()
        .expect("Couldn't get window inner size");
    let aspect = width as f32 / height as f32;
    let range = (fov * 0.5).tan() * near;
    let s_x = (2.0 * near) / (range * aspect + range * aspect);
    let s_y = near / range;
    let s_z = -(far + near) / (far - near);
    let p_z = -(2.0 * far * near) / (far - near);

    #[cfg_attr(rustfmt, rustfmt_skip)]
    let proj_matrix = Matrix4::new(
        s_x, 0.0, 0.0, 0.0,
        0.0, s_y, 0.0, 0.0,
        0.0, 0.0, s_z, -1.0,
        0.0, 0.0, p_z, 0.0,
    );
    println!("proj_matrix: {:?}", proj_matrix);

    let mut running = true;
    while running {
        let elapsed = last_frame_time.elapsed();
        last_frame_time = Instant::now();
        let dt = elapsed.as_secs() as f32 + elapsed.subsec_nanos() as f32 * 1e-9;
        events_loop.poll_events(|event| {
            if let Event::WindowEvent { event, .. } = event {
                use VirtualKeyCode::*;
                match event {
                    WindowEvent::Closed
                    | WindowEvent::KeyboardInput {
                        input:
                            KeyboardInput {
                                virtual_keycode: Some(Escape),
                                ..
                            },
                        ..
                    } => running = false,
                    WindowEvent::KeyboardInput {
                        input:
                            KeyboardInput {
                                virtual_keycode: Some(keycode),
                                ..
                            },
                        ..
                    } => match keycode {
                        W => cam_pos[2] -= cam_speed * dt,
                        S => cam_pos[2] += cam_speed * dt,
                        A => cam_pos[0] += cam_speed * dt,
                        D => cam_pos[0] -= cam_speed * dt,
                        PageUp => cam_pos[1] -= cam_speed * dt,
                        PageDown => cam_pos[1] += cam_speed * dt,
                        Left => cam_yaw += Deg(cam_yaw_speed * dt),
                        Right => cam_yaw -= Deg(cam_yaw_speed * dt),
                        _ => {}
                    },
                    _ => {}
                }
            }
        });

        let translate = Matrix4::from_translation(-cam_pos);
        let rotate = Matrix4::from_angle_y(-cam_yaw);
        let view_matrix = rotate * translate;
        // proj_matrix: Matrix4 [[1.1331265, 0.0, 0.0, 0.0], [0.0, 1.5108353, 0.0, 0.0], [0.0, 0.0, -1.002002, -1.0], [0.0, 0.0, -0.2002002, 0.0]]
        // view_matrix: Matrix4 [[1.0, 0.0, 0.0, 0.0], [0.0, 1.0, 0.0, 0.0], [0.0, 0.0, 1.0, 0.0], [0.0, 0.0, -2.0, 1.0]]
        // println!("view_matrix: {:?}", view_matrix);

        let locals = Locals {
            projection: proj_matrix.into(),
            view: view_matrix.into(),
        };
        encoder.update_constant_buffer(&data.locals, &locals);
        encoder.clear(&data.out_color, [0.6, 0.6, 0.8, 1.0]);
        encoder.clear_depth(&data.out_depth, 1.0);
        encoder.draw(&slice, &pso, &data);
        encoder.flush(&mut device);
        window.swap_buffers().unwrap();
        device.cleanup();
    }
}
