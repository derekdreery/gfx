// Copyright 2015 The Gfx-rs Developers.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

#[macro_use]
extern crate gfx;
extern crate gfx_window_glutin;
extern crate glutin;

extern crate image;

use gfx::Device;
use gfx::format::Rgba8;
use gfx::traits::{EncoderFactory, Factory, FactoryExt};

gfx_structure!( Vertex {
    pos: [f32; 2] = "a_Pos",
    uv: [f32; 2] = "a_Uv",
});

impl Vertex {
    fn new(p: [f32; 2], u: [f32; 2]) -> Vertex {
        Vertex {
            pos: p,
            uv: u,
        }
    }
}

gfx_pipeline_init!( PipeData PipeMeta PipeInit {
    vbuf: gfx::VertexBuffer<Vertex> = gfx::PER_VERTEX,
    lena: gfx::ResourceView<Rgba8> = "t_Lena",
    lena_sampler: gfx::Sampler = "t_Lena",
    tint: gfx::ResourceView<Rgba8> = "t_Tint",
    tint_sampler: gfx::Sampler = "t_Tint",
    blend: gfx::Global<i32> = "i_Blend",
    out: gfx::RenderTarget<Rgba8> = ("o_Color", gfx::state::MASK_ALL),
});

fn load_texture<R, F>(factory: &mut F, data: &[u8])
                -> Result<gfx::handle::ShaderResourceView<R, Rgba8>, String> where
                R: gfx::Resources, F: gfx::Factory<R> {
    use std::io::Cursor;
    use gfx::core::factory::Phantom;
    use gfx::tex as t;
    let img = image::load(Cursor::new(data), image::PNG).unwrap().to_rgba();
    let (width, height) = img.dimensions();
    //TODO: cast the slice
    //let (_, view) = factory.create_texture_2d_const(width as Size, height as Size, &img, false).unwrap();
    let desc = t::Descriptor {
        kind: t::Kind::D2(width as t::Size, height as t::Size, t::AaMode::Single),
        levels: 1,
        format: gfx::format::SurfaceType::R8_G8_B8_A8,
        bind: gfx::core::factory::SHADER_RESOURCE,
    };
    let raw = factory.create_new_texture_with_data(desc,
        gfx::format::ChannelType::UintNormalized, &img).unwrap();
    let tex = Phantom::new(raw);
    let view = factory.view_texture_as_shader_resource(&tex, (0, 0)).unwrap();
    Ok(view)
}

pub fn main() {
    let builder = glutin::WindowBuilder::new()
            .with_title("Blending example".to_string())
            .with_dimensions(800, 600);
    let (window, mut device, mut factory, main_color, _) =
        gfx_window_glutin::init_new::<Rgba8>(builder);
    let mut encoder = factory.create_encoder();

    // fullscreen quad
    let vertex_data = [
        Vertex::new([-1.0, -1.0], [0.0, 1.0]),
        Vertex::new([ 1.0, -1.0], [1.0, 1.0]),
        Vertex::new([ 1.0,  1.0], [1.0, 0.0]),

        Vertex::new([-1.0, -1.0], [0.0, 1.0]),
        Vertex::new([ 1.0,  1.0], [1.0, 0.0]),
        Vertex::new([-1.0,  1.0], [0.0, 0.0]),
    ];
    let (vbuf, slice) = factory.create_vertex_buffer(&vertex_data);

    let lena_texture = load_texture(&mut factory, &include_bytes!("image/lena.png")[..]).unwrap();
    let tint_texture = load_texture(&mut factory, &include_bytes!("image/tint.png")[..]).unwrap();
    let sampler = factory.create_sampler_linear();

    let shaders = factory.create_shader_set(
        include_bytes!("shader/blend_150.glslv"),
        include_bytes!("shader/blend_150.glslf")
        ).unwrap();

    let pso = factory.create_pipeline_state(&shaders,
        gfx::Primitive::TriangleList, Default::default(), &PipeInit::new()
        ).unwrap();

    // we pass a integer to our shader to show what blending function we want
    // it to use. normally you'd have a shader program per technique, but for
    // the sake of simplicity we'll just branch on it inside the shader.

    // each index correspond to a conditional branch inside the shader
    let blends = [
        (0, "Screen"),
        (1, "Dodge"),
        (2, "Burn"),
        (3, "Overlay"),
        (4, "Multiply"),
        (5, "Add"),
        (6, "Divide"),
        (7, "Grain Extract"),
        (8, "Grain Merge")
    ];
    let mut blends_cycle = blends.iter().cycle();
    let blend_func = blends_cycle.next().unwrap();

    println!("Using '{}' blend equation", blend_func.1);

    let mut data = PipeData {
        vbuf: vbuf,
        lena: lena_texture,
        lena_sampler: sampler.clone(),
        tint: tint_texture,
        tint_sampler: sampler,
        blend: blend_func.0,
        out: main_color,
    };

    'main: loop {
        encoder.reset();
        // quit when Esc is pressed.
        for event in window.poll_events() {
            match event {
                glutin::Event::KeyboardInput(glutin::ElementState::Pressed, _, Some(glutin::VirtualKeyCode::B)) => {
                    let blend_func = blends_cycle.next().unwrap();
                    println!("Using '{}' blend equation", blend_func.1);
                    data.blend = blend_func.0;
                },
                glutin::Event::KeyboardInput(_, _, Some(glutin::VirtualKeyCode::Escape)) |
                glutin::Event::Closed => break 'main,
                _ => {},
            }
        }

        encoder.clear_target(&data.out, [0.0; 4]);
        encoder.draw_pipeline(&slice, &pso, &data);
        device.submit(encoder.as_buffer());
        window.swap_buffers().unwrap();
        device.cleanup();
    }
}
