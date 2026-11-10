//! P13 L: a placa escolhe o MESMO degrau que o gabarito CPU (`ph2d_pixel_rampa::luz`), em todo
//! caso (rampa × degrau × luz). A lei WGSL é a de `ph2d_render::luz_rampa::LUZ_RAMPA_WGSL`.

use ph2d_pixel_core::{PaletteRamps, Ramp, RampId};
use ph2d_pixel_rampa::luz::{LutDaLuz, degrau_aceso};
use ph2d_pixel_rampa::{ConfigRampa, gera_rampa};
use ph2d_render::luz_rampa::{FORMATO_DA_LUT, LUZ_RAMPA_WGSL};
use wgpu::util::DeviceExt;

use crate::comum_luz::gpu;

const KERNEL: &str = "
struct Caso { rampa: u32, pos: u32, p0: u32, p1: u32, luz: vec4<f32> }
@group(0) @binding(0) var lut: texture_2d<f32>;
@group(0) @binding(1) var<storage, read> casos: array<Caso>;
@group(0) @binding(2) var<storage, read_write> saida: array<vec4<f32>>;
@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let i = id.x;
    if (i >= arrayLength(&casos)) { return; }
    let c = casos[i];
    saida[i] = luz_receber_rampa(lut, c.rampa, c.pos, c.luz.xyz);
}
";

fn paleta_rampas() -> (Vec<[u8; 4]>, PaletteRamps) {
    let mut pal = Vec::new();
    let mut ramps = Vec::new();
    for (id, (base, n)) in [
        ([0xd9, 0xa0, 0x66, 255], 5),
        ([0x5b, 0x6e, 0xe1, 255], 4),
        ([0x6a, 0xbe, 0x30, 255], 6),
    ]
    .into_iter()
    .enumerate()
    {
        let first = pal.len();
        let cfg = ConfigRampa {
            passos: n,
            desvio_matiz_graus: 20.0,
            curva_croma: 0.6,
            faixa_l: [0.25, 0.92],
        };
        pal.extend(gera_rampa(base, cfg));
        ramps.push(Ramp {
            id: RampId(id as u32 + 1),
            name: format!("r{id}"),
            indices: (first..pal.len()).map(|i| i as u8).collect(),
        });
    }
    let r = PaletteRamps {
        ramps,
        cycles: Vec::new(),
        luz_na_paleta: true,
    };
    (pal, r)
}

#[test]
fn a_placa_e_o_gabarito_concordam_na_rampa() {
    let Some(gpu) = gpu() else {
        eprintln!("sem GPU — gate saltado");
        return;
    };
    let (pal, ramps) = paleta_rampas();
    let lut = LutDaLuz::da_paleta(&pal, &ramps);
    let mut casos = Vec::new();
    let mut bytes = Vec::new();
    for (s, r) in lut.rampas.iter().enumerate() {
        for pos in 0..r.len() {
            for tinta in [[1.0, 1.0, 1.0], [1.0, 0.6, 0.3], [0.2, 0.5, 1.0]] {
                for i in 0..=200 {
                    let luz = tinta.map(|t: f32| t * i as f32 * 0.01);
                    casos.push((s, pos, luz));
                    for u in [s as u32, pos as u32, 0, 0] {
                        bytes.extend_from_slice(&u.to_le_bytes());
                    }
                    for f in [luz[0], luz[1], luz[2], 0.0] {
                        bytes.extend_from_slice(&f.to_le_bytes());
                    }
                }
            }
        }
    }
    let dev = &gpu.device;
    let (w, h) = (lut.largura() as u32, lut.rampas.len() as u32);
    let texeis: Vec<u8> = lut
        .texeis()
        .iter()
        .flatten()
        .flat_map(|f| f.to_le_bytes())
        .collect();
    let tex = dev.create_texture_with_data(
        &gpu.queue,
        &wgpu::TextureDescriptor {
            label: Some("lut da luz na rampa"),
            size: wgpu::Extent3d {
                width: w,
                height: h,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: FORMATO_DA_LUT,
            usage: wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        },
        wgpu::util::TextureDataOrder::LayerMajor,
        &texeis,
    );
    let entrada = dev.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("casos"),
        contents: &bytes,
        usage: wgpu::BufferUsages::STORAGE,
    });
    let tam = (casos.len() * 16) as u64;
    let saida = dev.create_buffer(&wgpu::BufferDescriptor {
        label: Some("saida"),
        size: tam,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
        mapped_at_creation: false,
    });
    let leitura = dev.create_buffer(&wgpu::BufferDescriptor {
        label: Some("leitura"),
        size: tam,
        usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    let fonte = format!("{LUZ_RAMPA_WGSL}\n{KERNEL}");
    let modulo = dev.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("luz na rampa"),
        source: wgpu::ShaderSource::Wgsl(fonte.into()),
    });
    let pipe = dev.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: Some("luz na rampa"),
        layout: None,
        module: &modulo,
        entry_point: Some("main"),
        compilation_options: wgpu::PipelineCompilationOptions::default(),
        cache: None,
    });
    let vista = tex.create_view(&wgpu::TextureViewDescriptor::default());
    let grupo = dev.create_bind_group(&wgpu::BindGroupDescriptor {
        label: None,
        layout: &pipe.get_bind_group_layout(0),
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&vista),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: entrada.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: saida.as_entire_binding(),
            },
        ],
    });
    let mut enc = dev.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
    {
        let mut p = enc.begin_compute_pass(&wgpu::ComputePassDescriptor::default());
        p.set_pipeline(&pipe);
        p.set_bind_group(0, &grupo, &[]);
        p.dispatch_workgroups((casos.len() as u32).div_ceil(64), 1, 1);
    }
    enc.copy_buffer_to_buffer(&saida, 0, &leitura, 0, tam);
    gpu.queue.submit([enc.finish()]);
    let fatia = leitura.slice(..);
    fatia.map_async(wgpu::MapMode::Read, |_| {});
    dev.poll(wgpu::PollType::wait_indefinitely()).expect("poll");
    let m = fatia.get_mapped_range().expect("mapa lido");
    let f: Vec<f32> = m
        .as_chunks::<4>()
        .0
        .iter()
        .map(|b| f32::from_le_bytes([b[0], b[1], b[2], b[3]]))
        .collect();
    let mut trocas = 0;
    for (k, &(s, pos, luz)) in casos.iter().enumerate() {
        let esperado = degrau_aceso(&lut.rampas[s], pos, luz);
        let d = lut.rampas[s][esperado];
        let rgb = d.cor.map(|c| f32::from(c) / 255.0);
        let g = &f[k * 4..k * 4 + 4];
        assert_eq!(
            [g[0], g[1], g[2], g[3]],
            [d.l, rgb[0], rgb[1], rgb[2]],
            "rampa {s} degrau {pos} luz {luz:?}: a placa escolheu outro degrau que o gabarito ({esperado})"
        );
        trocas += usize::from(esperado != pos);
    }
    eprintln!(
        "luz na rampa: {} casos, {trocas} trocam de degrau, placa = gabarito em todos",
        casos.len()
    );
    assert!(
        trocas > 1000,
        "a varredura quase não troca o degrau ({trocas})"
    );
}
