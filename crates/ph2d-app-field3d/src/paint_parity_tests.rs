//! ⭐⭐⭐ **O PASSE QUE PINTA, NOS DOIS MOTORES** — a imagem do dispositivo contra a do
//! [`ph2d_field_render::shade_render`], byte a byte.
//!
//! # ⚠️ O que este gate ISOLA, e porque isso é o ponto
//!
//! Os dois lados correm sobre a **MESMA marcha do dispositivo**: o mesmo `t`, a mesma normal, a
//! mesma sombra e a mesma oclusão. ⇒ o que sobra entre eles é **só o sombreamento** — as quatro
//! leis que atravessaram (o material, o céu, o olhar e o dono), mais a suavização da oclusão, a
//! mistura da fronteira de cor, a média da borda e a descida a 8 bits.
//!
//! ⛔ Um gate que traçasse cada lado no seu motor mediria as duas coisas somadas, e uma
//! divergência de sombreamento ficaria escondida atrás da de geometria — que já tem gate próprio
//! (`o_gbuffer_do_dispositivo_e_o_da_cpu`).

use ph2d_field::{FieldDoc, Node, NodeId, NodeKind, Op, Primitive, Xform};
use ph2d_material::{OpenPbr, Surface};

pub(crate) const W: u32 = 192;
pub(crate) const H: u32 = 108;
pub(crate) const FUNDO: [u8; 4] = [0, 0, 0, 0];

fn combina(op: Op, filhos: Vec<NodeId>) -> Node {
    Node::new(
        Xform::IDENTITY,
        NodeKind::Combine {
            op,
            children: filhos,
        },
    )
}

/// ⭐⭐ **A peça de TRÊS folhas, cada uma com o seu material** — e é assim que uma peça real é.
///
/// ⚠️ **Duas folhas não bastavam:** com duas, a rede do filtro de bolas (*«ninguém contém o
/// ponto»*) e o filtro a sério dão a mesma resposta, e o ramo que a terceira exercita nunca corre.
pub(crate) fn fixtura() -> (FieldDoc, Vec<FieldDoc>, Vec<Surface>) {
    let folhas = [
        ph2d_field_eval::leaf(
            Primitive::Sphere { radius: 0.45 },
            Xform::at(-0.30, 0.0, 0.0),
        ),
        ph2d_field_eval::leaf(
            Primitive::Sphere { radius: 0.40 },
            Xform::at(0.30, 0.10, 0.0),
        ),
        ph2d_field_eval::leaf(
            Primitive::Box {
                half: [0.60, 0.12, 0.30],
                round: 0.04,
                chamfer: 0.0,
            },
            Xform::at(0.0, -0.45, 0.0),
        ),
    ];
    let mut nos: Vec<Node> = folhas.to_vec();
    nos.push(combina(
        Op::Union(ph2d_field::Blend::Sharp),
        vec![NodeId(0), NodeId(1), NodeId(2)],
    ));
    let doc = FieldDoc::new(nos, NodeId(3)).expect("a peça de três folhas");
    // ⚠️ **Cada folha é um DOCUMENTO posto no mundo** — é isso que a `Owners` recebe, e aqui não há
    // grupo nenhum, logo a pose local já é a do mundo.
    let postas = folhas
        .iter()
        .map(|n| FieldDoc::new(vec![n.clone()], NodeId(0)).expect("a folha posta"))
        .collect();
    // ⭐ Três materiais BEM diferentes: um difuso vermelho, um metal e um verniz sobre azul. Sem a
    // diferença, trocar o dono pintaria exactamente o mesmo pixel.
    let materiais = vec![
        OpenPbr {
            base_color: [0.80, 0.12, 0.10],
            base_diffuse_roughness: 0.4,
            ..OpenPbr::default()
        }
        .prepare(),
        OpenPbr {
            base_color: [0.95, 0.75, 0.30],
            base_metalness: 1.0,
            specular_roughness: 0.22,
            ..OpenPbr::default()
        }
        .prepare(),
        OpenPbr {
            base_color: [0.10, 0.20, 0.85],
            coat_weight: 1.0,
            coat_roughness: 0.05,
            ..OpenPbr::default()
        }
        .prepare(),
    ];
    (doc, postas, materiais)
}

/// A lâmpada onde a wave da §25 a põe, com a radiância do rig.
fn lampada(cam: &ph2d_field_render::Orbit) -> ph2d_field_render::PointLamp {
    let (right, up, toward_eye) = cam.basis();
    let ecra = [-0.5566703_f32, 0.6634139, 0.5];
    let r = 2.0 * cam.half_extent;
    ph2d_field_render::PointLamp {
        world: [0, 1, 2].map(|i| {
            cam.target[i] + r * (ecra[0] * right[i] + ecra[1] * up[i] + ecra[2] * toward_eye[i])
        }),
        radiance_at_one: [3.0, 3.0, 3.0],
    }
}

/// Os DOIS caminhos sobre a MESMA marcha do dispositivo — ver a nota do módulo.
pub(crate) fn dois_caminhos(
    surfaces: &ph2d_field_render::Surfaces<'_>,
    doc: &FieldDoc,
    luz: &[ph2d_field_render::PointLamp],
) -> Option<(Vec<u8>, Vec<u8>, usize)> {
    dois_caminhos_com(surfaces, doc, luz, None)
}

/// O mesmo, com o CHÃO que só recebe — ver `docs/Render3d/07`.
pub(crate) fn dois_caminhos_com(
    surfaces: &ph2d_field_render::Surfaces<'_>,
    doc: &FieldDoc,
    luz: &[ph2d_field_render::PointLamp],
    chao: Option<ph2d_field_render::Ground>,
) -> Option<(Vec<u8>, Vec<u8>, usize)> {
    let t = crate::gpu_frame::shared()?;
    let reg = ph2d_field_eval::hybrid::Registry::new();
    let cam = ph2d_field_render::Orbit::default();
    let olhar = ph2d_view_transform::Look::default();
    let mundos: Vec<[f32; 3]> = luz.iter().map(|l| l.world).collect();

    let (g, mut sh) = crate::gpu_frame::march(t, doc, &reg, &cam, &mundos, chao, W, H, true)?;
    // ⭐⭐⭐ **O RICOCHETE entra na REFERÊNCIA de CPU** (`docs/Render3d/08` §12), porque o pintor do
    // dispositivo o calcula.
    //
    // ⚠️⚠️ **Ele NÃO vem do `march`, e não podia vir:** por aquele caminho o canal chega VAZIO —
    // quem o enche é a passagem do pintor, que precisa dos materiais. ⇒ a referência calcula-o com
    // a lei da CPU (`bounce_pass`) e suaviza-o com a MESMA porta que o dispositivo usa. *É esta
    // linha que faz o gate comparar dois motores e não dois caminhos diferentes.*
    sh.set_bounce(ph2d_field_render::blur_bounce(
        &g,
        &ph2d_field_render::bounce_pass(
            doc,
            &reg,
            &cam,
            &g,
            surfaces,
            luz,
            ph2d_field_render::OCCLUSION_PASSES,
        ),
    ));
    let sem_ecra: [ph2d_field_render::Lamp; 0] = [];
    let cpu = ph2d_field_render::shade_render(
        &g,
        &cam,
        surfaces,
        &ph2d_field_render::Lighting {
            lamps: &sem_ecra,
            points: luz,
            sky: &crate::render_light::StudioSky,
            shadows: Some(&sh),
        },
        olhar,
        FUNDO,
    );
    let gpu = crate::gpu_frame::paint(
        t, doc, &reg, &cam, luz, surfaces, olhar, FUNDO, chao, W, H, true,
    )?;
    // ⚠️ **As duas contagens de borda têm de bater**, e são medidas por caminhos diferentes: a do
    // G-buffer vem da lista lida de volta, a do pintor vem do contador que decidiu o despacho.
    assert_eq!(
        gpu.edges,
        g.edges.len(),
        "as duas contagens de borda discordam"
    );
    Some((cpu, gpu.rgba, g.edges.len()))
}

/// ⭐⭐⭐ **A IMAGEM DO DISPOSITIVO É A DA CPU.**
///
/// # A barra, e de onde ela sai
///
/// A saída é um **byte de 8 bits depois da curva sRGB**. O que separa os dois motores é
/// arredondamento: um valor a meio caminho entre dois bytes cai para um lado num e para o outro no
/// outro, e uma soma de `f32` reassociada move o último bit.
///
/// ⇒ a barra é **`1` nível** na esmagadora maioria dos canais e **`2`** no pior. ⛔ Uma barra em
/// RMS esconderia exactamente o que interessa: um punhado de pixels muito errados numa fronteira.
#[test]
#[ignore = "precisa de GPU"]
fn a_imagem_do_dispositivo_e_a_da_cpu() {
    if crate::gpu_frame::shared().is_none() {
        println!("sem adaptador — saltado");
        return;
    }
    let (doc, postas, materiais) = fixtura();
    let reg = ph2d_field_eval::hybrid::Registry::new();
    // ⚠️ A margem é a tolerância de acerto da marcha, que é como a tabela de materiais a deriva.
    let owners = ph2d_field_eval::owners::Owners::new(
        &postas,
        &reg,
        ph2d_field_render::hit_tolerance(
            ph2d_field_render::Orbit::default().half_extent,
            f32::from(u16::try_from(W.min(H)).expect("a tela cabe")),
        ),
    );
    assert_eq!(owners.len(), 3, "a lei do dono tem de ver as três folhas");
    let surfaces = ph2d_field_render::Surfaces {
        all: &materiais,
        owners: Some(&owners),
    };
    let (cpu, gpu, bordas) = dois_caminhos(
        &surfaces,
        &doc,
        &[lampada(&ph2d_field_render::Orbit::default())],
    )
    .expect("o dispositivo tem de tomar esta peça");

    // ⭐ **A POPULAÇÃO vem primeiro**: sem ela, duas imagens de fundo vazio leriam `0` de desvio.
    let pintados = cpu.as_chunks::<4>().0.iter().filter(|p| p[3] > 0).count();
    assert!(
        pintados > 3_000,
        "só {pintados} pixels foram pintados — a fixtura não enche a tela, e o gate não afirma nada"
    );
    // ⭐⭐ **E a BORDA tem de existir**, senão a segunda entrada do pintor (`pinta_bordas`) podia
    // estar inteira partida com a imagem a ler `100 %`. *Um gate que não sabe se o sujeito dele
    // correu não afirma nada sobre ele.*
    assert!(
        bordas > 200,
        "só {bordas} pixels de borda — a metade re-amostrada do pintor ficou por exercitar"
    );

    let mut hist = [0usize; 256];
    let mut pior = (0u8, 0usize, 0usize);
    for (i, (a, b)) in cpu.iter().zip(gpu.iter()).enumerate() {
        let d = a.abs_diff(*b);
        hist[d as usize] += 1;
        if d > pior.0 {
            pior = (d, i / 4 % W as usize, i / 4 / W as usize);
        }
    }
    #[allow(clippy::cast_precision_loss)]
    let total = hist.iter().sum::<usize>() as f64;
    #[allow(clippy::cast_precision_loss)]
    let fraccao = hist[..2].iter().sum::<usize>() as f64 / total;
    #[allow(clippy::cast_precision_loss)]
    let media = hist
        .iter()
        .enumerate()
        .map(|(d, n)| d as f64 * *n as f64)
        .sum::<f64>()
        / total;
    println!(
        "pintura · {total} canais · |Δ| medio {media:.4} · ≤1 nivel em {:.3} % · pior {} em ({}, {})",
        fraccao * 100.0,
        pior.0,
        pior.1,
        pior.2
    );
    assert!(
        fraccao >= 0.995,
        "só {:.3} % dos canais estão a ≤1 nível — a lei do pintor divergiu",
        fraccao * 100.0
    );
    assert!(
        pior.0 <= 2,
        "o pior canal diverge {} níveis em ({}, {}) — a quantização admite 2",
        pior.0,
        pior.1,
        pior.2
    );
}

/// ⭐⭐ **O CONTROLO: a régua responde.**
///
/// Sem ele, um arnês que pintasse os dois lados pelo mesmo caminho leria `0` e passaria para
/// sempre. Aqui a peça é a MESMA e a lei do dono é **apagada** de um dos lados — que é a diferença
/// entre uma peça de três materiais e uma de um.
#[test]
#[ignore = "precisa de GPU"]
fn a_regua_da_pintura_acusa_a_lei_do_dono_apagada() {
    let Some(t) = crate::gpu_frame::shared() else {
        println!("sem adaptador — saltado");
        return;
    };
    let (doc, postas, materiais) = fixtura();
    let reg = ph2d_field_eval::hybrid::Registry::new();
    let cam = ph2d_field_render::Orbit::default();
    let owners = ph2d_field_eval::owners::Owners::new(
        &postas,
        &reg,
        ph2d_field_render::hit_tolerance(
            cam.half_extent,
            f32::from(u16::try_from(W.min(H)).expect("a tela cabe")),
        ),
    );
    let luz = [lampada(&cam)];
    let olhar = ph2d_view_transform::Look::default();
    let com = crate::gpu_frame::paint(
        t,
        &doc,
        &reg,
        &cam,
        &luz,
        &ph2d_field_render::Surfaces {
            all: &materiais,
            owners: Some(&owners),
        },
        olhar,
        FUNDO,
        None,
        W,
        H,
        true,
    )
    .expect("o dispositivo tem de tomar esta peça")
    .rgba;
    let sem = crate::gpu_frame::paint(
        t,
        &doc,
        &reg,
        &cam,
        &luz,
        &ph2d_field_render::Surfaces {
            all: &materiais,
            owners: None,
        },
        olhar,
        FUNDO,
        None,
        W,
        H,
        true,
    )
    .expect("o dispositivo tem de tomar esta peça")
    .rgba;
    let fora = com
        .iter()
        .zip(sem.iter())
        .filter(|(a, b)| a.abs_diff(**b) > 2)
        .count();
    assert!(
        fora > 2_000,
        "apagar a lei do dono moveu só {fora} canais — a régua não está a olhar"
    );
}

/// ⭐⭐⭐ **A LÂMPADA ENCOSTADA À PEÇA** — o ramo do piso, que a fixtura de cima não exercita.
///
/// # ⛔⛔ Porque ele é um gate PRÓPRIO e não mais uma célula
///
/// Uma prova de mutação sobre o pintor pôs `cru > piso` a `cru > 0` e **SOBREVIVEU**: na cena de
/// cima a lâmpada está a duas meias-extensões da peça, e nenhum pixel chega a `0,05` dela. *Um
/// corpus no ponto NEUTRO de um knob não testa esse knob.*
///
/// ⇒ a lâmpada vai para **cima da superfície** do primeiro pólo. Abaixo do piso a direcção passa a
/// ser a NORMAL — e a alternativa (normalizar o vector ZERO) pinta o pixel de **PRETO**, que é a
/// metade do defeito que o [`ph2d_field_render::POINT_LAMP_MIN_DISTANCE`] descreve por escrito.
#[test]
#[ignore = "precisa de GPU"]
fn a_lampada_encostada_a_peca_concorda_nos_dois_motores() {
    if crate::gpu_frame::shared().is_none() {
        println!("sem adaptador — saltado");
        return;
    }
    let (doc, postas, materiais) = fixtura();
    let reg = ph2d_field_eval::hybrid::Registry::new();
    let cam = ph2d_field_render::Orbit::default();
    let owners = ph2d_field_eval::owners::Owners::new(
        &postas,
        &reg,
        ph2d_field_render::hit_tolerance(
            cam.half_extent,
            f32::from(u16::try_from(W.min(H)).expect("a tela cabe")),
        ),
    );
    let surfaces = ph2d_field_render::Surfaces {
        all: &materiais,
        owners: Some(&owners),
    };
    // O pólo da primeira esfera — `radius = 0,45` em `x = −0,30`, virado ao olho.
    let luz = [ph2d_field_render::PointLamp {
        world: [-0.30, 0.0, 0.45],
        radiance_at_one: [0.05, 0.05, 0.05],
    }];
    let (cpu, gpu, _) =
        dois_caminhos(&surfaces, &doc, &luz).expect("o dispositivo tem de tomar esta peça");

    // ⭐ **A POPULAÇÃO da pergunta**: quantos pixels estão de facto ABAIXO do piso. Sem ela este
    // gate seria uma segunda cópia do de cima.
    let (g, _) = crate::gpu_frame::march(
        crate::gpu_frame::shared().expect("o adaptador"),
        &doc,
        &reg,
        &cam,
        &[luz[0].world],
        None,
        W,
        H,
        true,
    )
    .expect("a marcha");
    let piso =
        ph2d_field_render::POINT_LAMP_MIN_DISTANCE * ph2d_field_render::POINT_LAMP_MIN_DISTANCE;
    let colados = (0..g.point.len())
        .filter(|&i| {
            g.hit[i] && {
                let d = [0, 1, 2].map(|c| luz[0].world[c] - g.point[i][c]);
                d[0] * d[0] + d[1] * d[1] + d[2] * d[2] <= piso
            }
        })
        .count();
    assert!(
        colados > 0,
        "nenhum pixel está abaixo do piso — a lâmpada não encostou, e o ramo fica por exercitar"
    );

    let pior = cpu
        .iter()
        .zip(gpu.iter())
        .map(|(a, b)| a.abs_diff(*b))
        .max()
        .unwrap_or(0);
    println!("lâmpada encostada · {colados} pixels abaixo do piso · pior |Δ| {pior}");
    assert!(
        pior <= 2,
        "o pior canal diverge {pior} níveis com a lâmpada encostada"
    );
}

/// ⏱️⭐⭐⭐ **O QUE O PINTOR DO DISPOSITIVO COMPRA** — os dois caminhos, o mesmo quadro assente.
///
/// ⚠️ **Ela imprime o `/proc/loadavg` ao lado de cada número.** Nenhuma leitura de relógio desta
/// máquina vale nada acima de `load ~5`, e uma régua sem a carga ao lado não se pode desmentir.
///
/// ⚠️ **O MÍNIMO de N corridas, com a mediana ao lado** — a carga de fundo desta workstation não
/// desce abaixo de `~7`, e «esperar pela calma» nunca chega.
#[test]
#[ignore = "medição — precisa de GPU e de máquina calma"]
fn mede_o_que_o_pintor_do_dispositivo_compra() {
    let Some(t) = crate::gpu_frame::shared() else {
        println!("sem adaptador — saltado");
        return;
    };
    const LW: u32 = 1920;
    const LH: u32 = 1080;
    const CORRIDAS: usize = 7;

    let (doc, postas, materiais) = fixtura();
    let reg = ph2d_field_eval::hybrid::Registry::new();
    let cam = ph2d_field_render::Orbit::default();
    let owners = ph2d_field_eval::owners::Owners::new(
        &postas,
        &reg,
        ph2d_field_render::hit_tolerance(
            cam.half_extent,
            f32::from(u16::try_from(LW.min(LH)).expect("a tela cabe")),
        ),
    );
    let surfaces = ph2d_field_render::Surfaces {
        all: &materiais,
        owners: Some(&owners),
    };
    let luz = [lampada(&cam)];
    let mundos: Vec<[f32; 3]> = luz.iter().map(|l| l.world).collect();
    let olhar = ph2d_view_transform::Look::default();
    let sem_ecra: [ph2d_field_render::Lamp; 0] = [];

    let mede = |mut f: Box<dyn FnMut()>| -> (f64, f64) {
        let mut v: Vec<f64> = Vec::with_capacity(CORRIDAS);
        for _ in 0..CORRIDAS {
            let t0 = std::time::Instant::now();
            f();
            v.push(t0.elapsed().as_secs_f64() * 1e3);
        }
        v.sort_by(f64::total_cmp);
        (v[0], v[CORRIDAS / 2])
    };

    // ⚠️ **Uma corrida de aquecimento fora da conta** — a primeira compila o pipeline (`6`–`49 ms`).
    let _ = crate::gpu_frame::march(t, &doc, &reg, &cam, &mundos, None, LW, LH, true);
    let _ = crate::gpu_frame::paint(
        t, &doc, &reg, &cam, &luz, &surfaces, olhar, FUNDO, None, LW, LH, true,
    );

    // ⭐ **A fatia que é SÓ o dispositivo mais a leitura do G-buffer** — sem ela, a diferença entre
    // os dois caminhos lê-se como um número só e não se sabe quanto dela é o barramento.
    let (m_min, m_med) = mede(Box::new(|| {
        let (g, _) = crate::gpu_frame::march(t, &doc, &reg, &cam, &mundos, None, LW, LH, true)
            .expect("a marcha");
        std::hint::black_box(g.hit.len());
    }));
    let (a_min, a_med) = mede(Box::new(|| {
        let (g, sh) = crate::gpu_frame::march(t, &doc, &reg, &cam, &mundos, None, LW, LH, true)
            .expect("a marcha");
        let px = ph2d_field_render::shade_render(
            &g,
            &cam,
            &surfaces,
            &ph2d_field_render::Lighting {
                lamps: &sem_ecra,
                points: &luz,
                sky: &crate::render_light::StudioSky,
                shadows: Some(&sh),
            },
            olhar,
            FUNDO,
        );
        std::hint::black_box(px.len());
    }));
    let (b_min, b_med) = mede(Box::new(|| {
        let p = crate::gpu_frame::paint(
            t, &doc, &reg, &cam, &luz, &surfaces, olhar, FUNDO, None, LW, LH, true,
        )
        .expect("o pintor");
        std::hint::black_box(p.rgba.len());
    }));

    let carga = std::fs::read_to_string("/proc/loadavg").unwrap_or_default();
    println!("\n  {LW}×{LH} · load {}", carga.trim());
    println!("  caminho                                 min      mediana");
    println!("  marcha + G-buffer de volta (49,8 MB) {m_min:6.2}  {m_med:7.2} ms");
    println!("  … mais o pintor na CPU              {a_min:7.2}  {a_med:7.2} ms");
    println!("  marcha + PINTOR no dispositivo      {b_min:7.2}  {b_med:7.2} ms");
    println!(
        "  ganho                               {:7.2}×",
        a_min / b_min
    );
    println!(
        "  barramento                          {:.1} MB → {:.1} MB",
        f64::from(LW) * f64::from(LH) * 24.0 / 1e6,
        f64::from(LW) * f64::from(LH) * 4.0 / 1e6
    );
}

/// ⭐⭐⭐ **Os gates das LÂMPADAS** — ver [`lamps`].
#[path = "paint_lamps_tests.rs"]
mod lamps;

/// ⭐⭐⭐ **O CHÃO DO DISPOSITIVO É O DA CPU** (`docs/Render3d/07`) — a sombra, a oclusão e a
/// escurecida do fundo, nos dois motores.
///
/// ⚠️ **A população vem primeiro, e ela é a DIFERENÇA**: quantos bytes o chão muda contra o mesmo
/// quadro sem chão. Sem isso, dois fundos transparentes iguais leriam `100 %` de paridade sobre um
/// chão que nenhum dos dois desenhou.
#[test]
#[ignore = "precisa de GPU"]
fn o_chao_do_dispositivo_e_o_da_cpu() {
    if crate::gpu_frame::shared().is_none() {
        println!("sem adaptador — saltado");
        return;
    }
    let folha = |p: ph2d_field::Primitive, x: ph2d_field::Xform| ph2d_field_eval::leaf(p, x);
    let doc = FieldDoc::new(
        vec![
            folha(
                ph2d_field::Primitive::Sphere { radius: 0.25 },
                ph2d_field::Xform::at(-0.25, 0.25, 0.0),
            ),
            folha(
                ph2d_field::Primitive::Box {
                    half: [0.16; 3],
                    round: 0.02,
                    chamfer: 0.0,
                },
                ph2d_field::Xform::at(0.3, 0.16, 0.1),
            ),
            combina(
                Op::Union(ph2d_field::Blend::Sharp),
                vec![NodeId(0), NodeId(1)],
            ),
        ],
        NodeId(2),
    )
    .expect("a peça pousada");
    let reg = ph2d_field_eval::hybrid::Registry::new();
    let materiais = [ph2d_material::OpenPbr::default().prepare()];
    let surfaces = ph2d_field_render::Surfaces {
        all: &materiais,
        owners: None,
    };
    let cam = ph2d_field_render::Orbit::default();
    let luz = [lampada(&cam)];
    let chao = ph2d_field_render::lowest_point(&doc, &reg)
        .map(|height| ph2d_field_render::Ground { height });
    assert!(chao.is_some(), "a peça tem chão");
    let (cpu_sem, _, _) =
        dois_caminhos_com(&surfaces, &doc, &luz, None).expect("o dispositivo toma a peça");
    let (cpu, gpu, bordas) =
        dois_caminhos_com(&surfaces, &doc, &luz, chao).expect("o dispositivo toma a peça");

    let mudados = cpu.iter().zip(&cpu_sem).filter(|(a, b)| a != b).count();
    assert!(
        mudados > 4_000,
        "o chão só mudou {mudados} bytes — a fixtura não o mostra, e o gate não afirma nada"
    );
    assert!(
        bordas > 100,
        "só {bordas} pixels de borda — a metade da borda ficou por exercitar"
    );

    let mut hist = [0usize; 256];
    let mut pior = (0u8, 0usize, 0usize);
    for (i, (a, b)) in cpu.iter().zip(gpu.iter()).enumerate() {
        let d = a.abs_diff(*b);
        hist[d as usize] += 1;
        if d > pior.0 {
            pior = (d, i / 4 % W as usize, i / 4 / W as usize);
        }
    }
    #[allow(clippy::cast_precision_loss)]
    let total = hist.iter().sum::<usize>() as f64;
    #[allow(clippy::cast_precision_loss)]
    let fraccao = hist[..2].iter().sum::<usize>() as f64 / total;
    println!(
        "chão · {mudados} bytes mudados pelo chão · ≤1 nivel em {:.3} % · pior {} em ({}, {})",
        fraccao * 100.0,
        pior.0,
        pior.1,
        pior.2
    );
    assert!(
        fraccao >= 0.995,
        "só {:.3} % dos canais estão a ≤1 nível — a lei do chão divergiu entre os motores",
        fraccao * 100.0
    );
    assert!(
        pior.0 <= 2,
        "o pior canal diverge {} níveis em ({}, {})",
        pior.0,
        pior.1,
        pior.2
    );
}
