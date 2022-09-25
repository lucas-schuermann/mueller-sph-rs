use std::borrow::Cow;

use glam::Vec2;
use glium::{glutin, index, uniform, Surface, VertexFormat};
use glutin::event::{ElementState, Event, KeyboardInput, StartCause, VirtualKeyCode, WindowEvent};
use log::info;

use mueller_sph_rs::Simulation;

const DAM_PARTICLES: usize = 2500;
const BLOCK_PARTICLES: usize = 250;
const MAX_PARTICLES: usize = DAM_PARTICLES + 25 * BLOCK_PARTICLES;
const POINT_SIZE: f32 = 10.0;
const WINDOW_WIDTH: u32 = 1200;
const WINDOW_HEIGHT: u32 = 800;
const VIEW_WIDTH: f32 = 1.5 * WINDOW_WIDTH as f32;
const VIEW_HEIGHT: f32 = 1.5 * WINDOW_HEIGHT as f32;

fn main() -> Result<(), String> {
    env_logger::init();

    let mut simulation = Simulation::<MAX_PARTICLES>::new(VIEW_WIDTH, VIEW_HEIGHT);
    simulation.init_dam_break(DAM_PARTICLES);

    let event_loop = glutin::event_loop::EventLoop::new();
    let size: glutin::dpi::LogicalSize<u32> = (WINDOW_WIDTH, WINDOW_HEIGHT).into();
    let wb = glutin::window::WindowBuilder::new()
        .with_inner_size(size)
        .with_resizable(false)
        .with_title("Müller SPH");
    let cb = glutin::ContextBuilder::new();
    let display = glium::Display::new(wb, cb, &event_loop)
        .map_err(|e| format!("Failed to create glium display: {}", e))?;

    let vertex_shader_src = r#"
        #version 140
        uniform mat4 matrix;
        in vec2 position;
        void main() {
            gl_Position = matrix * vec4(position, 0.0, 1.0);
        }
    "#;
    let fragment_shader_src = r#"
        #version 140
        out vec4 f_color;
        void main() {
            f_color = vec4(0.2, 0.6, 1.0, 1.0);
        }
    "#;
    let program =
        glium::Program::from_source(&display, vertex_shader_src, fragment_shader_src, None)
            .map_err(|e| format!("Failed to parse vertex shader source: {}", e))?;
    let ortho_matrix: [[f32; 4]; 4] =
        cgmath::ortho(0.0, VIEW_WIDTH, 0.0, VIEW_HEIGHT, 0.0, 1.0).into();
    let uniforms = uniform! {
        matrix: ortho_matrix
    };
    let indices = index::NoIndices(index::PrimitiveType::Points);

    // preallocate vertex buffer
    let empty_buffer = vec![Vec2::ZERO; MAX_PARTICLES];
    let bindings: VertexFormat = Cow::Owned(vec![(
        Cow::Borrowed("position"),
        2 * std::mem::size_of::<f32>(),
        0,
        glium::vertex::AttributeType::F32F32,
        false,
    )]);
    let vertex_buffer = unsafe {
        glium::VertexBuffer::new_raw_dynamic(
            &display,
            &empty_buffer,
            bindings,
            2 * std::mem::size_of::<f32>(),
        )
        .map_err(|e| format!("Failed to create vertex buffer: {}", e))?
    };

    let draw_params = glium::DrawParameters {
        polygon_mode: glium::PolygonMode::Point,
        point_size: Some(POINT_SIZE),
        ..Default::default()
    };

    event_loop.run(move |event, _, control_flow| {
        match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => {
                    *control_flow = glutin::event_loop::ControlFlow::Exit;
                    return;
                }
                WindowEvent::KeyboardInput {
                    input:
                        KeyboardInput {
                            virtual_keycode: Some(virtual_code),
                            state,
                            ..
                        },
                    ..
                } => match (virtual_code, state) {
                    (VirtualKeyCode::R, ElementState::Pressed) => {
                        vertex_buffer.invalidate();
                        vertex_buffer.write(&vec![Vec2::ZERO; MAX_PARTICLES]);
                        simulation.clear();
                        info!("Cleared simulation");
                        simulation.init_dam_break(DAM_PARTICLES);
                    }
                    (VirtualKeyCode::Space, ElementState::Pressed) => {
                        simulation.init_block(BLOCK_PARTICLES);
                    }
                    (VirtualKeyCode::Escape, ElementState::Pressed) => {
                        *control_flow = glutin::event_loop::ControlFlow::Exit;
                        return;
                    }
                    _ => (),
                },
                _ => return,
            },
            Event::NewEvents(cause) => match cause {
                StartCause::Init => (),
                StartCause::Poll => (),
                _ => return,
            },
            _ => return,
        }

        simulation.update();

        vertex_buffer
            .slice(0..simulation.num_particles)
            .unwrap()
            .write(&simulation.x); // unwrap is safe due to preallocated known length

        let mut target = display.draw();
        target.clear_color(0.9, 0.9, 0.9, 1.0);
        target
            .draw(&vertex_buffer, &indices, &program, &uniforms, &draw_params)
            .unwrap();
        target.finish().unwrap();
    });
}
