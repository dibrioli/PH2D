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
// Três palavras por sonda: a cor com a altura, o CORPO (`docs/3D/29` §6) e o
// GRADIENTE da altura no objecto (o report de 01/10 da vista inclinada).
@compute @workgroup_size(64)
fn cs(@builtin(global_invocation_id) gid: vec3<u32>) {
    let i = gid.x;
    if (i >= arrayLength(&sondas)) { return; }
    let s = sondas[i];
    let r = tinta_no_ponto4(s.pi, s.p.xyz);
    saida[3u * i] = r.c;
    saida[3u * i + 1u] = vec4<f32>(r.corpo, 0.0, 0.0, 0.0);
    saida[3u * i + 2u] = vec4<f32>(r.g, 0.0);
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

/// O passo das diferenças centrais, em parâmetro da face.
const PASSO: f32 = 1e-3;

/// O ponto está a mais de dois passos de toda fronteira de célula? Dentro de
/// uma célula a altura é linear/bilinear e a diferença central é EXACTA.
fn longe_das_celulas(lado: u32, params: &[f32]) -> bool {
    params.iter().all(|&x| {
        let c = x * lado as f32;
        (c - c.round()).abs() > 2.0 * PASSO * lado as f32
    })
}

/// O gradiente da altura num TRIÂNGULO: as derivadas direccionais ao longo de
/// duas arestas, resolvidas no plano dele.
fn gradiente_tri(t: &Tinta, fi: usize, f: &[u32], bar: [f32; 3], p: [[f32; 3]; 3]) -> [f32; 3] {
    let sub = |x: [f32; 3], y: [f32; 3]| [x[0] - y[0], x[1] - y[1], x[2] - y[2]];
    let dot = |x: [f32; 3], y: [f32; 3]| x[0] * y[0] + x[1] * y[1] + x[2] * y[2];
    let h = |db: f32, dc: f32| t.altura_tri(fi, f, [bar[0] - db - dc, bar[1] + db, bar[2] + dc]);
    // `∇h·(pb − pa)` e `∇h·(pc − pa)`.
    let s1 = (h(PASSO, 0.0) - h(-PASSO, 0.0)) / (2.0 * PASSO);
    let s2 = (h(0.0, PASSO) - h(0.0, -PASSO)) / (2.0 * PASSO);
    let (d1, d2) = (sub(p[1], p[0]), sub(p[2], p[0]));
    let (g11, g12, g22) = (dot(d1, d1), dot(d1, d2), dot(d2, d2));
    let det = g11 * g22 - g12 * g12;
    let al = (s1 * g22 - s2 * g12) / det;
    let be = (s2 * g11 - s1 * g12) / det;
    [
        al * d1[0] + be * d2[0],
        al * d1[1] + be * d2[1],
        al * d1[2] + be * d2[2],
    ]
}

struct Sonda {
    pi: u32,
    p: [f32; 3],
    esperado: [f32; 3],
    /// A altura do RELEVO no mesmo ponto (`docs/3D/29`), pela mesma lei.
    altura: f32,
    /// O CORPO no mesmo ponto (`docs/3D/29` §6), pela mesma lei.
    corpo: f32,
    /// O GRADIENTE da altura no objecto, por diferenças CENTRAIS da lei da
    /// CPU — `None` quando o ponto está a menos de um passo de uma fronteira
    /// de célula, onde a altura tem um vinco e a diferença não mede nada.
    gradiente: Option<[f32; 3]>,
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
                altura: t.altura_quad(fi, f, [u, v]),
                corpo: t.espessura_quad(fi, f, [u, v])[1],
                // ⭐ A célula é o quadrado unitário ⇒ `∇h = (∂h/∂u, ∂h/∂v, 0)`.
                gradiente: longe_das_celulas(t.lado_da_face(fi), &[u, v]).then(|| {
                    let h = |du: f32, dv: f32| t.altura_quad(fi, f, [u + du, v + dv]);
                    [
                        (h(PASSO, 0.0) - h(-PASSO, 0.0)) / (2.0 * PASSO),
                        (h(0.0, PASSO) - h(0.0, -PASSO)) / (2.0 * PASSO),
                        0.0,
                    ]
                }),
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
                altura: t.altura_tri(fi, f, bar),
                corpo: t.espessura_tri(fi, f, bar)[1],
                gradiente: longe_das_celulas(t.lado_da_face(fi), &bar)
                    .then(|| gradiente_tri(t, fi, f, bar, [a, b, c])),
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

    // ⛔ **As duas fixturas GRADUADAS SAÍRAM em 2026-09-24**, com o registo de
    //   `19` palavras que as lia (ordem do dono: liberar a memória que o
    //   `Even Detail` deixou reservada). Um plano graduado DESARMA na placa, e
    //   essa lei tem gate próprio, sem adaptador: `um_plano_graduado_desarma`.
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
        // ⭐ **O RELEVO viaja pela MESMA leitura** (`docs/3D/29`), com uma
        //   altura e um corpo embaralhados DIFERENTES da cor e entre si —
        //   senão um endereço trocado entre os canais leria certo. ⚠️ A grelha de nível `0` fica
        //   SEM relevo de propósito: é o CONTROLO de que o bit desligado lê
        //   zero, e de que a cor não depende de haver relevo.
        let com_relevo = nome != "grelha plana, nível 0";
        if com_relevo {
            for i in 0..t.amostras().len() {
                let e = cor_embaralhada(i + 1_000_003);
                t.relevo_mut()[i] = [e[0] - 0.5, e[1]];
            }
        }
        assert_eq!(t.tem_relevo(), com_relevo);

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
        let mut pior_g = 0.0f32;
        let mut pior_g_onde = String::new();
        let mut com_gradiente = 0usize;
        for (s, got) in sondas.iter().zip(lido.iter()) {
            let Some(esp) = s.gradiente else { continue };
            com_gradiente += 1;
            for e in 0..3 {
                let g = got[5 + e];
                let d = (g - esp[e]).abs() / (1.0 + esp[e].abs());
                if d > pior_g {
                    pior_g = d;
                    pior_g_onde = format!("{} eixo {e}: {g} contra {}", s.onde, esp[e]);
                }
            }
        }
        // ⭐ O GRADIENTE da placa é o da LEI (report de 01/10, vista inclinada):
        //   a CPU tira-o por diferença central da `altura_*`, sem uma linha do
        //   shader. A barra é relativa — o declive vale `~L·Δh` e a diferença
        //   em `f32` perde `ulp(h)/PASSO`.
        assert!(
            pior_g <= 2e-3,
            "{nome}: o gradiente da placa e o da CPU divergem {pior_g:e} — {pior_g_onde}"
        );
        assert!(
            com_gradiente * 2 >= sondas.len(),
            "{nome}: só {com_gradiente} de {} sondas medem o gradiente",
            sondas.len()
        );
        for (s, got) in sondas.iter().zip(lido.iter()) {
            let esp4 = [
                s.esperado[0],
                s.esperado[1],
                s.esperado[2],
                s.altura,
                s.corpo,
            ];
            for (e, (g, esp)) in got.iter().zip(esp4.iter()).enumerate() {
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
) -> Vec<[f32; 8]> {
    use wgpu::util::DeviceExt as _;

    let amostras: Vec<f32> = t.amostras().iter().flat_map(|c| *c).collect();
    let idx: Vec<u32> = tris.iter().flat_map(|t| *t).collect();
    let posicoes: Vec<f32> = m.positions().iter().flat_map(|p| *p).collect();
    // ⛔⛔ **A CONFIGURAÇÃO VEM DA PORTA DO PRODUTO, e isto foi pago:** até
    //   2026-09-23 estas quatro palavras eram montadas aqui à mão, com a
    //   arrumação de então. Quando a P2 tirou o `lado` global do uniforme, o
    //   arnês continuou a escrever a arrumação ANTIGA e este gate reprovou
    //   sobre uma lei CERTA — *um arnês que constrói o uniforme em vez de o
    //   pedir mede outro programa*, a mesma família do `device()` que pedia o
    //   piso do WebGPU enquanto o produto pedia o do adaptador.
    let cfg: [u32; 4] = ph2d_mesh_render::tinta_cfg(Some(t));
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
    // ⚠️ Sem relevo o buffer é um DUMMY de uma altura, como no produto: um
    //   binding de storage não pode ter tamanho zero.
    let alturas: Vec<[f32; 2]> = t
        .relevo()
        .map_or_else(|| vec![[0.0; 2]], <[[f32; 2]]>::to_vec);
    let b_alt = buf(bytemuck::cast_slice(&alturas), st);
    let b_sondas = buf(bytemuck::cast_slice(&entrada), st);
    let n = sondas.len();
    let b_saida = device.create_buffer(&wgpu::BufferDescriptor {
        label: None,
        size: (n * 48) as u64,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
        mapped_at_creation: false,
    });
    let b_ler = device.create_buffer(&wgpu::BufferDescriptor {
        label: None,
        size: (n * 48) as u64,
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
            storage(7),
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
            wgpu::BindGroupEntry {
                binding: 7,
                resource: b_alt.as_entire_binding(),
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
    enc.copy_buffer_to_buffer(&b_saida, 0, &b_ler, 0, (n * 48) as u64);
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
        .as_chunks::<12>()
        .0
        .iter()
        .map(|c| [c[0], c[1], c[2], c[3], c[4], c[8], c[9], c[10]])
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

/// ⭐⭐⭐⭐ **GATE — UM PLANO GRADUADO DESARMA NA PLACA, e um uniforme arma.**
///
/// ⛔⛔ **É a guarda que a volta do registo de `19` para `10` palavras deixou**
/// (2026-09-24, ordem do dono): o shader lê a retícula com UM `lado`, e um plano
/// com um nível por face desenhado assim põe a tinta de umas faces no sítio das
/// outras **sem nada no ecrã a acusar**. Desarmado, ele mostra a cor por
/// vértice, que é o caso base e está certo.
///
/// ⚠️ **Corre SEM adaptador, e isso é a razão de a guarda ser uma função pura:**
/// o irmão de paridade é `#[ignore]` e nem o CI nem um arnês de mutação lhe
/// chegam.
///
/// ⚠️ **O CONTROLO vem primeiro e é a metade que torna isto uma medição:** um
/// `tinta_cfg` que devolvesse `armado = 0` sempre passaria na metade graduada —
/// e apagaria a tinta fina de toda peça.
#[test]
fn um_plano_graduado_desarma() {
    let esfera = shapes::uv_sphere(6, 8, 1.0);
    let faces: Vec<Vec<u32>> = esfera.faces().iter().map(|f| f.verts().to_vec()).collect();
    let it = || faces.iter().map(|f| &f[..]);

    // (1) CONTROLO: o plano UNIFORME arma, com o lado dele.
    let uniforme = Tinta::nova(esfera.vert_count(), it(), 3);
    let cfg = ph2d_mesh_render::tinta_cfg(Some(&uniforme));
    assert_eq!(cfg[3], 1, "CONTROLO: um plano uniforme tem de ARMAR");
    assert_eq!(cfg[0], 8, "e com o lado dele (`2^3`)");

    // (2) O GRADUADO desarma, com o `lado` mínimo de `1` (nunca zero).
    let niveis: Vec<u8> = (0..faces.len()).map(|i| (i % 4) as u8).collect();
    let graduado = Tinta::graduada(esfera.vert_count(), it(), &niveis, 0)
        .expect("a lista de níveis descreve esta malha");
    assert!(
        graduado.lado_uniforme().is_none(),
        "a fixtura não contém o fenómeno: os níveis têm de ser DIFERENTES"
    );
    assert_eq!(
        ph2d_mesh_render::tinta_cfg(Some(&graduado)),
        [1, 0, 0, 0],
        "um plano graduado tem de DESARMAR — o registo de `10` palavras não o descreve"
    );

    // (3) E sem plano nenhum, também desarmado.
    assert_eq!(ph2d_mesh_render::tinta_cfg(None), [1, 0, 0, 0]);
}
