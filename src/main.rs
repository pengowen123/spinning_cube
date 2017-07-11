extern crate glutin;
#[macro_use]
extern crate gfx;
extern crate gfx_window_glutin;
extern crate cgmath;

use glutin::GlContext;
use gfx::traits::FactoryExt;
use cgmath::{Matrix4, Point3, Deg};
use gfx::{Device, texture};

use std::time::Instant;

const VERTEX_SHADER: &'static [u8] = include_bytes!("shader/v.glsl");
const FRAGMENT_SHADER: &'static [u8] = include_bytes!("shader/f.glsl");

const WIDTH: u32 = 800;
const HEIGHT: u32 = 600;

type ColorFormat = gfx::format::Srgba8;
type DepthFormat = gfx::format::DepthStencil;

gfx_defines! {
    vertex Vertex {
        pos: [f32; 4] = "a_Pos",
        uv: [f32; 2] = "a_Uv",
    }

    constant Locals {
        transform: [[f32; 4]; 4] = "u_Transform",
        rotation: [[f32; 4]; 4] = "u_Rotation",
    }

    pipeline pipe {
        vbuf: gfx::VertexBuffer<Vertex> = (),
        transform: gfx::Global<[[f32; 4]; 4]> = "u_Transform",
        locals: gfx::ConstantBuffer<Locals> = "Locals",
        color: gfx::TextureSampler<[f32; 4]> = "t_Color",
        out_color: gfx::RenderTarget<ColorFormat> = "Target0",
        out_depth: gfx::DepthTarget<DepthFormat> = gfx::preset::depth::LESS_EQUAL_WRITE,
    }
}

impl Vertex {
    fn new(pos: [i8; 3], uv: [i8; 2]) -> Self {
        Self {
            pos: [pos[0] as f32, pos[1] as f32, pos[2] as f32, 1.0],
            uv: [uv[0] as f32, uv[1] as f32],
        }
    }
}

fn create_cube<R, F>(
    factory: &mut F,
    main_color: gfx::handle::RenderTargetView<R, ColorFormat>,
    main_depth: gfx::handle::DepthStencilView<R, DepthFormat>,
) -> gfx::Bundle<R, pipe::Data<R>>
where
    R: gfx::Resources,
    F: gfx::Factory<R>,
{
    let vertex_data = vec![
        // top (0, 0, 1)
        Vertex::new([-1, -1, 1], [0, 0]),
        Vertex::new([1, -1, 1], [1, 0]),
        Vertex::new([1, 1, 1], [1, 1]),
        Vertex::new([-1, 1, 1], [0, 1]),
        // bottom (0, 0, -1)
        Vertex::new([-1, 1, -1], [1, 0]),
        Vertex::new([1, 1, -1], [0, 0]),
        Vertex::new([1, -1, -1], [0, 1]),
        Vertex::new([-1, -1, -1], [1, 1]),
        // right (1, 0, 0)
        Vertex::new([1, -1, -1], [0, 0]),
        Vertex::new([1, 1, -1], [1, 0]),
        Vertex::new([1, 1, 1], [1, 1]),
        Vertex::new([1, -1, 1], [0, 1]),
        // left (-1, 0, 0)
        Vertex::new([-1, -1, 1], [1, 0]),
        Vertex::new([-1, 1, 1], [0, 0]),
        Vertex::new([-1, 1, -1], [0, 1]),
        Vertex::new([-1, -1, -1], [1, 1]),
        // front (0, 1, 0)
        Vertex::new([1, 1, -1], [1, 0]),
        Vertex::new([-1, 1, -1], [0, 0]),
        Vertex::new([-1, 1, 1], [0, 1]),
        Vertex::new([1, 1, 1], [1, 1]),
        // back (0, -1, 0)
        Vertex::new([1, -1, 1], [0, 0]),
        Vertex::new([-1, -1, 1], [1, 0]),
        Vertex::new([-1, -1, -1], [1, 1]),
        Vertex::new([1, -1, -1], [0, 1]),
    ];

    let index_data: Vec<u16> =
        vec![
             0,  1,  2,  2,  3,  0, // top
             4,  5,  6,  6,  7,  4, // bottom
             8,  9, 10, 10, 11,  8, // right
            12, 13, 14, 14, 15, 12, // left
            16, 17, 18, 18, 19, 16, // front
            20, 21, 22, 22, 23, 20, // back
       ];

    let (vbuf, slice) =
        factory.create_vertex_buffer_with_slice(vertex_data.as_slice(), index_data.as_slice());

    let texels = [[0x20, 0xA0, 0xC0, 0x00]];
    let (_, texture_view) = factory
        .create_texture_immutable::<gfx::format::Rgba8>(
            texture::Kind::D2(1, 1, texture::AaMode::Single),
            &[&texels],
        )
        .unwrap();

    let sinfo =
        texture::SamplerInfo::new(texture::FilterMethod::Bilinear, texture::WrapMode::Clamp);

    let proj = cgmath::perspective(Deg(45.0f32), WIDTH as f32 / HEIGHT as f32, 0.1, 100.0);
    let view = Matrix4::look_at(
        Point3::new(1.5, -5.0, 3.0),
        Point3::new(0.0, 0.0, 0.0),
        cgmath::Vector3::unit_z(),
    );

    let data = pipe::Data {
        vbuf: vbuf,
        transform: (proj * view).into(),
        locals: factory.create_constant_buffer(1),
        color: (texture_view, factory.create_sampler(sinfo)),
        out_color: main_color,
        out_depth: main_depth,
    };

    let pso = factory
        .create_pipeline_simple(VERTEX_SHADER, FRAGMENT_SHADER, pipe::new())
        .unwrap();

    gfx::Bundle::new(slice, pso, data)
}

fn handle_events(events: &mut glutin::EventsLoop, running: &mut bool) {
    events.poll_events(|e| if let glutin::Event::WindowEvent {
        event,
        window_id: _,
    } = e
    {
        use glutin::{WindowEvent, VirtualKeyCode};
        match event {
            WindowEvent::Closed => *running = false,
            WindowEvent::KeyboardInput {
                device_id: _,
                input,
            } => {
                if input.virtual_keycode == Some(VirtualKeyCode::Escape) {
                    *running = false;
                }
            }
            _ => {}
        }
    });

}

fn update_delta_time(dt: &mut f64, previous_time: &mut Instant) {
    let now = Instant::now();
    let elapsed = now - *previous_time;
    *previous_time = now;

    *dt = (elapsed.as_secs() * 1_000_000_000 + elapsed.subsec_nanos() as u64) as f64 /
        1_000_000_000.0;
}

fn get_rot_matrix(angle: Deg<f32>) -> Matrix4<f32> {
    Matrix4::from_angle_z(angle)
}

fn main() {
    let mut events = glutin::EventsLoop::new();
    let window_builder = glutin::WindowBuilder::new()
        .with_title("Spinning Cube")
        .with_dimensions(WIDTH, HEIGHT);

    let context_builder = glutin::ContextBuilder::new();

    let (window, mut device, mut factory, main_color, main_depth) =
        gfx_window_glutin::init::<ColorFormat, DepthFormat>(
            window_builder,
            context_builder,
            &events,
        );
    let mut encoder: gfx::Encoder<_, _> = factory.create_command_buffer().into();

    let bundle = create_cube(&mut factory, main_color.clone(), main_depth.clone());

    let mut running = true;
    let mut cube_angle = 0.0;
    let mut dt = 1.0;
    let mut time = Instant::now();

    while running {
        update_delta_time(&mut dt, &mut time);

        cube_angle += 150.0 * dt;

        if cube_angle > 360.0 {
            cube_angle -= 360.0;
        }

        encoder.clear(&main_color, [0.1, 0.2, 0.3, 1.0]);
        encoder.clear_depth(&main_depth, 1.0);

        encoder.update_constant_buffer(
            &bundle.data.locals,
            &Locals {
                transform: bundle.data.transform,
                rotation: get_rot_matrix(Deg(cube_angle as f32)).into(),
            },
        );

        bundle.encode(&mut encoder);

        encoder.flush(&mut device);
        window.swap_buffers().unwrap();
        device.cleanup();

        handle_events(&mut events, &mut running);
    }
}
