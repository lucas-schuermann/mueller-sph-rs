extern crate glium;
use glium::{glutin, implement_vertex, uniform, Surface};
use glutin::event::{ElementState, Event, KeyboardInput, StartCause, VirtualKeyCode, WindowEvent};
use graphics;

#[derive(Copy, Clone)]
struct Vertex {
    position: [f32; 2],
}
implement_vertex!(Vertex, position);

const DAM_PARTICLES: usize = 3000;
const BLOCK_PARTICLES: usize = 250;
const POINT_SIZE: f32 = 15.0;

fn main() {
    let mut particles: Vec<graphics::Particle> = Vec::new();
    graphics::init_dam_break(&mut particles, DAM_PARTICLES);

    let event_loop = glutin::event_loop::EventLoop::new();
    let size: glutin::dpi::LogicalSize<u32> =
        (graphics::WINDOW_WIDTH, graphics::WINDOW_HEIGHT).into();
    let wb = glutin::window::WindowBuilder::new()
        .with_inner_size(size)
        .with_resizable(false)
        .with_title("Muller SPH");
    let cb = glutin::ContextBuilder::new();
    let display = glium::Display::new(wb, cb, &event_loop).unwrap();

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
            .unwrap();
    let ortho_matrix: cgmath::Matrix4<f32> = cgmath::ortho(
        0.0,
        graphics::VIEW_WIDTH,
        0.0,
        graphics::VIEW_HEIGHT,
        0.0,
        1.0,
    );
    let uniforms = uniform! {
        matrix: Into::<[[f32; 4]; 4]>::into(ortho_matrix)
    };
    let indices = glium::index::NoIndices(glium::index::PrimitiveType::Points);

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
                        particles.clear();
                        println!("Cleared simulation");
                        graphics::init_dam_break(&mut particles, DAM_PARTICLES);
                    }
                    (VirtualKeyCode::Space, ElementState::Pressed) => {
                        graphics::init_block(&mut particles, BLOCK_PARTICLES);
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

        graphics::update(&mut particles);

        // draw
        let data: Vec<Vertex> = particles
            .iter()
            .map(|p| Vertex {
                position: p.x.to_array(),
            })
            .collect();
        let vertex_buffer = glium::VertexBuffer::new(&display, &data).unwrap();

        let mut target = display.draw();
        target.clear_color(0.9, 0.9, 0.9, 1.0);
        target
            .draw(
                &vertex_buffer,
                &indices,
                &program,
                &uniforms,
                &glium::DrawParameters {
                    polygon_mode: glium::PolygonMode::Point,
                    point_size: Some(POINT_SIZE),
                    ..Default::default()
                },
            )
            .unwrap();
        target.finish().unwrap();
    });
}