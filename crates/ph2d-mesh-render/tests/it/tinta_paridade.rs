//! ⭐⭐⭐⭐ **A LEI DA RETÍCULA LIDA NOS DOIS LADOS** — a `Tinta` na CPU e o
//! gémeo em WGSL na placa, sobre a MESMA malha e o MESMO plano.
//!
//! ⛔⛔ **É este gate que faz o `tinta.wgsl` deixar de ser uma segunda
//! redacção da aritmética.** O doc da `Tinta::cor_tri` escrevia a cláusula
//! antes de o shader existir: *«ela existe para que a lei da interpolação
//! tenha UM dono; o gémeo em WGSL é conferido contra esta função»*.
//!
//! ⚠️⚠️ **E as fixturas são DUAS por uma razão de circularidade.** Num quad
//! NÃO-PLANO, o ponto `(u, v)` de um fragmento é *definido* pela conversão que
//! o próprio shader faz — escrever a expectativa com essa mesma conversão
//! deixaria o gate a concordar consigo mesmo. ⇒ a fixtura que DISCRIMINA é uma
//! **grelha plana** de quads unitários, onde `(u, v)` se lê **directamente das
//! coordenadas do mundo** e nenhuma linha é partilhada com o shader. A esfera
//! entra a seguir, pelas faces de TRIÂNGULO (onde a baricêntrica do ponto é a
//! que o construiu, também sem convenção) e para cobrir topologia a sério.
//!
//! ⛔⛔ **E há UMA mutação do gémeo que este gate NÃO pode matar, nomeada com
//! a medição:** apagar o corte do piso em `L−1` do `tinta_cor_quad`. Em `u = 1`
//! exacto o `fu` sai **exactamente zero**, logo as duas amostras de fora da
//! face entram com peso `0` e **a cor é a mesma** — o corte guarda o ENDEREÇO,
//! não a cor. Do lado da `ph2d-mesh-colors` a mesma mutação SANGRA (`M12`),
//! porque ali o endereço fora do intervalo é um pânico. *Uma mutação que
//! nenhuma régua de saída pode matar não é uma régua em falta; é uma
//! propriedade de outro eixo, e o sítio honesto para ela é este parágrafo.*
//!
//! ⚠️ **As amostras são EMBARALHADAS de propósito.** Um campo suave esconde um
//! endereço trocado — o vizinho tem quase a mesma cor. Aqui cada amostra leva
//! uma cor de um hash do índice dela, logo **qualquer** endereço errado sai
//! numa cor completamente diferente.

use ph2d_mesh::{Mesh, shapes};
use ph2d_mesh_colors::Tinta;

/// A entrada de compute que exercita a lei — o resto do módulo é o
/// [`ph2d_mesh_render::fonte::TINTA_WGSL`], sem uma linha reescrita.
const ENTRADA: &str = r#"
struct Sonda { pi: u32, _a: u32, _b: u32, _c: u32, p: vec4<f32> };
@group(0) @binding(0) var<storage, read> sondas: array<Sonda>;
@group(0) @binding(1) var<storage, read_write> saida: array<vec4<f32>>;
@compute @workgroup_size(64)
fn cs(@builtin(global_invocation_id) gid: vec3<u32>) {
    let i = gid.x;
    if (i >= arrayLength(&sondas)) { return; }
    let s = sondas[i];
    saida[i] = vec4<f32>(tinta_no_ponto(s.pi, s.p.xyz), 1.0);
}
"#;

/// Uma cor por amostra que **não** é um campo suave — ver o cabeçalho.
fn cor_embaralhada(i: usize) -> [f32; 3] {
    let h = |k: u64| {
        let mut x = (i as u64).wrapping_mul(0x9e37_79b9_7f4a_7c15) ^ k;
        x ^= x >> 31;
        x = x.wrapping_mul(0xbf58_476d_1ce4_e5b9);
        x ^= x >> 27;
        ((x >> 40) as f32) / 16_777_216.0
    };
    [h(1), h(2), h(3)]
}

/// A grelha `2×2` de quads unitários — a fixtura que DISCRIMINA.
fn grelha() -> (Mesh, Vec<Vec<u32>>) {
    let pos: Vec<[f32; 3]> = (0..3)
        .flat_map(|y| (0..3).map(move |x| [x as f32, y as f32, 0.0]))
        .collect();
    let faces = vec![
        vec![0u32, 1, 4, 3],
        vec![1, 2, 5, 4],
        vec![3, 4, 7, 6],
        vec![4, 5, 8, 7],
    ];
    let caras: Vec<ph2d_mesh::Face> = faces
        .iter()
        .map(|f| ph2d_mesh::Face::quad(f[0], f[1], f[2], f[3]))
        .collect();
    (
        Mesh::from_parts(pos, caras).expect("a grelha é válida"),
        faces,
    )
}

struct Sonda {
    pi: u32,
    p: [f32; 3],
    esperado: [f32; 3],
    onde: String,
}

/// As sondas da grelha: `(u, v)` lido do MUNDO, sem uma linha do shader.
fn sondas_da_grelha(m: &Mesh, faces: &[Vec<u32>], t: &Tinta, origem: &[u32]) -> Vec<Sonda> {
    let mut out = Vec::new();
    for (fi, f) in faces.iter().enumerate() {
        let a = m.positions()[f[0] as usize];
        // ⚠️⚠️ **As quatro últimas estão na BORDA da face, e não é acabamento:**
        //   uma mutação que apagava o corte do piso em `L−1` **SOBREVIVEU** às
        //   quatro do miolo — ali `floor(u·L)` nunca chega a `L`, logo o ramo
        //   que o corte protege não é cruzado. *Uma sonda no meio de uma face
        //   não diz nada sobre a beira dela*, e a beira é onde a leitura erra
        //   sem que ninguém veja, porque o vizinho tem quase a mesma cor.
        //   ⭐ E elas caem nos quatro lados: `v = 0` (a→b), `u = 1` (b→c),
        //   `v = 1` (c→d, que anda para trás) e `u = 0` (d→a, idem).
        for (u, v) in [
            (0.70f32, 0.20f32),
            (0.90, 0.35),
            (0.20, 0.70),
            (0.35, 0.90),
            (0.60, 0.00),
            (1.00, 0.40),
            (0.40, 1.00),
            (0.00, 0.60),
        ] {
            // ⭐ O sub-triângulo é geometria e não convenção: a diagonal é
            //   `a–c`, logo `v < u` é a metade `(a,b,c)` e `v > u` a `(a,c,d)`.
            let sub = u32::from(v > u);
            let alvo = ((fi as u32) << 1) | sub;
            let pi = origem
                .iter()
                .position(|&o| o == alvo)
                .expect("o triângulo desta metade existe") as u32;
            out.push(Sonda {
                pi,
                // ⭐ A célula é o quadrado unitário ⇒ o mundo É o `(u, v)`.
                p: [a[0] + u, a[1] + v, 0.0],
                esperado: t.cor_quad(fi, f, [u, v]),
                onde: format!("grelha face {fi} sub {sub} ({u}, {v})"),
            });
        }
    }
    out
}

/// As sondas da esfera: só as faces de TRIÂNGULO, onde a baricêntrica do ponto
/// é a que o construiu — também sem convenção partilhada.
fn sondas_da_esfera(m: &Mesh, t: &Tinta, origem: &[u32]) -> Vec<Sonda> {
    let mut out = Vec::new();
    for (fi, face) in m.faces().iter().enumerate() {
        let f = face.verts();
        if f.len() != 3 {
            continue;
        }
        let alvo = (fi as u32) << 1;
        let Some(pi) = origem.iter().position(|&o| o == alvo) else {
            continue;
        };
        let (a, b, c) = (
            m.positions()[f[0] as usize],
            m.positions()[f[1] as usize],
            m.positions()[f[2] as usize],
        );
        for bar in [[0.6f32, 0.3, 0.1], [0.2, 0.5, 0.3], [0.34, 0.33, 0.33]] {
            let p = [
                bar[0] * a[0] + bar[1] * b[0] + bar[2] * c[0],
                bar[0] * a[1] + bar[1] * b[1] + bar[2] * c[1],
                bar[0] * a[2] + bar[1] * b[2] + bar[2] * c[2],
            ];
            out.push(Sonda {
                pi: pi as u32,
                p,
                esperado: t.cor_tri(fi, f, bar),
                onde: format!("esfera face {fi} {bar:?}"),
            });
        }
    }
    out
}

#[test]
#[ignore = "precisa de adaptador"]
fn a_lei_da_reticula_le_o_mesmo_na_placa_e_na_cpu() {
    let Some((device, queue)) = super::device_de_teste::device() else {
        panic!("sem adaptador — este gate não é verde por skip");
    };

    let (grade, faces_grade) = grelha();
    let esfera = shapes::uv_sphere(6, 8, 1.0);
    let faces_esfera: Vec<Vec<u32>> = esfera.faces().iter().map(|f| f.verts().to_vec()).collect();

    let mut total_sondas = 0usize;
    for (nome, m, faces, nivel) in [
        ("grelha plana", &grade, &faces_grade, 2u8),
        ("esfera", &esfera, &faces_esfera, 2),
        ("grelha plana, nível 0", &grade, &faces_grade, 0),
        ("esfera, nível 3", &esfera, &faces_esfera, 3),
    ] {
        let it = || faces.iter().map(|f| &f[..]);
        let mut t = Tinta::nova(m.vert_count(), it(), nivel);
        for i in 0..t.amostras().len() {
            t.amostras_mut()[i] = cor_embaralhada(i);
        }

        let mut pay = Vec::new();
        assert!(
            t.topologia().payload(it(), &mut pay),
            "a porta recusou as faces de que o plano nasceu"
        );
        let mut tris = Vec::new();
        let mut origem = Vec::new();
        m.triangle_indices_com_origem(&mut tris, Some(&mut origem));

        let sondas = if std::ptr::eq(m, &grade) {
            sondas_da_grelha(m, faces, &t, &origem)
        } else {
            sondas_da_esfera(m, &t, &origem)
        };
        assert!(!sondas.is_empty(), "{nome}: nenhuma sonda");
        total_sondas += sondas.len();

        let lido = corre_na_placa(&device, &queue, m, &t, &pay, &tris, &origem, &sondas);

        let mut pior = 0.0f32;
        let mut pior_onde = String::new();
        for (s, got) in sondas.iter().zip(lido.iter()) {
            for (e, (g, esp)) in got.iter().zip(s.esperado.iter()).enumerate() {
                let d = (g - esp).abs();
                if d > pior {
                    pior = d;
                    pior_onde = format!("{} canal {e}: {g} contra {esp}", s.onde);
                }
            }
        }
        // ⚠️ A barra é a da resolução das baricêntricas: o shader RECUPERA-as
        //   de uma posição interpolada em `f32`, e a CPU recebe-as prontas.
        assert!(
            pior <= 2e-5,
            "{nome}: a placa e a CPU divergem {pior:e} — {pior_onde}"
        );
    }
    assert!(
        total_sondas >= 40,
        "só {total_sondas} sondas — a régua está magra"
    );
}

#[allow(clippy::too_many_arguments)]
fn corre_na_placa(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    m: &Mesh,
    t: &Tinta,
    pay: &[u32],
    tris: &[[u32; 3]],
    origem: &[u32],
    sondas: &[Sonda],
) -> Vec<[f32; 3]> {
    use wgpu::util::DeviceExt as _;

    let amostras: Vec<f32> = t.amostras().iter().flat_map(|c| *c).collect();
    let idx: Vec<u32> = tris.iter().flat_map(|t| *t).collect();
    let posicoes: Vec<f32> = m.positions().iter().flat_map(|p| *p).collect();
    let cfg: [u32; 4] = [
        t.lado_uniforme().expect("a fixtura e' uniforme"),
        t.topologia().verts() as u32,
        t.topologia().arestas() as u32,
        1,
    ];
    let entrada: Vec<f32> = sondas
        .iter()
        .flat_map(|s| {
            let pi = f32::from_bits(s.pi);
            [pi, 0.0, 0.0, 0.0, s.p[0], s.p[1], s.p[2], 1.0]
        })
        .collect();

    let buf = |dados: &[u8], uso: wgpu::BufferUsages| {
        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: dados,
            usage: uso,
        })
    };
    let st = wgpu::BufferUsages::STORAGE;
    let b_amostras = buf(bytemuck::cast_slice(&amostras), st);
    let b_topo = buf(bytemuck::cast_slice(pay), st);
    let b_origem = buf(bytemuck::cast_slice(origem), st);
    let b_idx = buf(bytemuck::cast_slice(&idx), st);
    let b_pos = buf(bytemuck::cast_slice(&posicoes), st);
    let b_cfg = buf(bytemuck::cast_slice(&cfg), wgpu::BufferUsages::UNIFORM);
    let b_sondas = buf(bytemuck::cast_slice(&entrada), st);
    let n = sondas.len();
    let b_saida = device.create_buffer(&wgpu::BufferDescriptor {
        label: None,
        size: (n * 16) as u64,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
        mapped_at_creation: false,
    });
    let b_ler = device.create_buffer(&wgpu::BufferDescriptor {
        label: None,
        size: (n * 16) as u64,
        usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    let entrada_bgl = layout(device, &[(0, false), (1, true)]);
    let tinta_bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: None,
        entries: &[
            storage(1),
            storage(2),
            storage(3),
            storage(4),
            storage(5),
            wgpu::BindGroupLayoutEntry {
                binding: 6,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
        ],
    });

    let fonte = format!("{}\n{ENTRADA}", ph2d_mesh_render::fonte::TINTA_WGSL);
    let modulo = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("tinta paridade"),
        source: wgpu::ShaderSource::Wgsl(fonte.into()),
    });
    let pl = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: None,
        bind_group_layouts: &[Some(&entrada_bgl), Some(&tinta_bgl)],
        immediate_size: 0,
    });
    let pipe = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: None,
        layout: Some(&pl),
        module: &modulo,
        entry_point: Some("cs"),
        compilation_options: wgpu::PipelineCompilationOptions::default(),
        cache: None,
    });

    let bg0 = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: None,
        layout: &entrada_bgl,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: b_sondas.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: b_saida.as_entire_binding(),
            },
        ],
    });
    let bg1 = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: None,
        layout: &tinta_bgl,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 1,
                resource: b_amostras.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: b_topo.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 3,
                resource: b_origem.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 4,
                resource: b_idx.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 5,
                resource: b_pos.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 6,
                resource: b_cfg.as_entire_binding(),
            },
        ],
    });

    let mut enc = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
    {
        let mut cp = enc.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: None,
            timestamp_writes: None,
        });
        cp.set_pipeline(&pipe);
        cp.set_bind_group(0, &bg0, &[]);
        cp.set_bind_group(1, &bg1, &[]);
        cp.dispatch_workgroups(n.div_ceil(64) as u32, 1, 1);
    }
    enc.copy_buffer_to_buffer(&b_saida, 0, &b_ler, 0, (n * 16) as u64);
    queue.submit([enc.finish()]);

    let fatia = b_ler.slice(..);
    fatia.map_async(wgpu::MapMode::Read, |_| {});
    device
        .poll(wgpu::PollType::wait_indefinitely())
        .expect("poll");
    let dados = fatia.get_mapped_range();
    // ⛔ Esta crate PROÍBE `unsafe`; o `bytemuck` já é dependência dela.
    let quatro: &[f32] = bytemuck::cast_slice(&dados);
    let out = quatro
        .as_chunks::<4>()
        .0
        .iter()
        .map(|c| [c[0], c[1], c[2]])
        .collect();
    drop(dados);
    b_ler.unmap();
    out
}

fn storage(binding: u32) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility: wgpu::ShaderStages::COMPUTE,
        ty: wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Storage { read_only: true },
            has_dynamic_offset: false,
            min_binding_size: None,
        },
        count: None,
    }
}

fn layout(device: &wgpu::Device, quais: &[(u32, bool)]) -> wgpu::BindGroupLayout {
    let entries: Vec<_> = quais
        .iter()
        .map(|&(binding, escreve)| wgpu::BindGroupLayoutEntry {
            binding,
            visibility: wgpu::ShaderStages::COMPUTE,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Storage {
                    read_only: !escreve,
                },
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        })
        .collect();
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: None,
        entries: &entries,
    })
}
