//! Os gates do **chão que só recebe** — ver [`crate::ground`] e `docs/Render3d/07`.

use crate::{
    Ground, Lighting, Orbit, PointLamp, Screen, Shadows, Surfaces, lowest_point, refine_hemisphere,
    shade_render, shadow_pass, shadow_pass_on, trace,
};
use ph2d_field::{FieldDoc, Node, NodeId, NodeKind, Op, Primitive, Xform};
use ph2d_field_eval::hybrid::Registry;
use ph2d_view_transform::Look;

/// O fundo do modelador: TRANSPARENTE — a peça é composta sobre o canvas.
const FUNDO: [u8; 4] = [0, 0, 0, 0];

struct CeuUniforme(f32);

impl ph2d_material::Environment for CeuUniforme {
    fn radiance(&self, _dir: [f32; 3], _alpha: f32) -> [f32; 3] {
        [self.0; 3]
    }
    fn irradiance(&self, _n: [f32; 3]) -> [f32; 3] {
        [self.0; 3]
    }
}

fn esfera_em(c: [f32; 3], r: f32) -> FieldDoc {
    FieldDoc::new(
        vec![ph2d_field_eval::leaf(
            Primitive::Sphere { radius: r },
            Xform::at(c[0], c[1], c[2]),
        )],
        NodeId(0),
    )
    .expect("a esfera")
}

// ── a lei do raio ─────────────────────────────────────────────────────────────────────────────

#[test]
fn o_raio_toca_o_chao_so_vindo_de_cima() {
    let chao = Ground { height: 0.5 };
    let q = chao
        .hit([1.0, 2.0, 3.0], [0.6, -0.8, 0.0])
        .expect("um raio que desce, de cima, toca");
    assert!((q[0] - 2.125).abs() < 1e-5 && q[2] == 3.0, "{q:?}");
    assert_eq!(
        q[1], 0.5,
        "o y do ponto é a ALTURA, escrita, e não um arredondamento dela"
    );
    for (nome, o, d) in [
        ("raio a subir", [0.0, 2.0, 0.0], [0.0, 1.0, 0.0]),
        ("raio horizontal", [0.0, 2.0, 0.0], [1.0, 0.0, 0.0]),
        // ⚠️ Estes dois são recusados pelo `t ≤ 0`, e não por uma cerca própria — ver [`Ground::hit`].
        ("olho abaixo do chão", [0.0, 0.0, 0.0], [0.0, -1.0, 0.0]),
        ("olho NO chão", [0.0, 0.5, 0.0], [0.0, -1.0, 0.0]),
        ("NaN", [0.0, f32::NAN, 0.0], [0.0, -1.0, 0.0]),
    ] {
        assert_eq!(
            chao.hit(o, d),
            None,
            "{nome}: um chão invisível visto assim não é nada"
        );
    }
}

// ── onde o chão está ──────────────────────────────────────────────────────────────────────────

/// ⭐ A esfera: o ponto mais baixo é `c.y − r`, e a busca acha-o à tolerância do olhar fino.
#[test]
fn o_ponto_mais_baixo_de_uma_esfera() {
    let doc = esfera_em([0.2, 0.3, -0.1], 0.25);
    let y = lowest_point(&doc, &Registry::new()).expect("a esfera tem geometria");
    assert!(
        (y - 0.05).abs() <= 1e-4,
        "o ponto mais baixo saiu {y}, e é 0,05"
    );
}

/// A pose que põe um cubo de pé numa quina (a diagonal na vertical).
fn de_pe_na_quina(t: [f32; 3]) -> Xform {
    Xform {
        translation: t,
        rotation: ph2d_field::xform::quat_mul(
            ph2d_field::xform::quat_axis_angle([1.0, 0.0, 0.0], 0.615_479_7),
            ph2d_field::xform::quat_axis_angle([0.0, 0.0, 1.0], std::f32::consts::FRAC_PI_4),
        ),
        scale: 1.0,
    }
}

/// ⭐⭐ **Um cubo de pé numa quina** — a quina é um ponto de medida nula, e o olhar amostra o centro
/// dos pixels: é o caso que pede o TERCEIRO olhar. O esperado sai dos oito cantos, rodados pela
/// mesma pose que o avaliador usa.
#[test]
fn o_ponto_mais_baixo_de_um_cubo_de_pe_na_quina() {
    let pose = de_pe_na_quina([0.1, 0.4, 0.0]);
    let h = 0.2f32;
    let doc = FieldDoc::new(
        vec![ph2d_field_eval::leaf(
            Primitive::Box {
                half: [h; 3],
                round: 0.0,
                chamfer: 0.0,
            },
            pose,
        )],
        NodeId(0),
    )
    .expect("o cubo");
    let mut esperado = f32::INFINITY;
    for sx in [-h, h] {
        for sy in [-h, h] {
            for sz in [-h, h] {
                esperado = esperado.min(pose.apply([sx, sy, sz])[1]);
            }
        }
    }
    let y = lowest_point(&doc, &Registry::new()).expect("o cubo tem geometria");
    assert!(
        (y - esperado).abs() <= 1e-4,
        "a quina está a {esperado} e a busca achou {y}"
    );
}

/// ⭐⭐ **O CONTROLO que diz porque a busca existe: um cilindro INCLINADO, onde a caixa mente.**
///
/// A borda mais baixa de um cilindro de eixo `u` (raio `ρ`, meia-altura `H`) está a
/// `H·|u_y| + ρ·√(1 − u_y²)` abaixo do centro; a caixa rodada soma os dois eixos transversais
/// separadamente (`ρ·(|x_y| + |z_y|)`), e desce abaixo dela. ⚠️ Num cubo a caixa é EXACTA (a quina é
/// um canto dela) — a 1.ª redacção deste controlo usava o cubo e reprovou sobre isso.
#[test]
fn num_cilindro_inclinado_a_caixa_desce_abaixo_da_peca_e_a_busca_nao() {
    let pose = de_pe_na_quina([0.0, 0.3, 0.0]);
    let (rho, hh) = (0.1f32, 0.25f32);
    let doc = FieldDoc::new(
        vec![ph2d_field_eval::leaf(
            Primitive::Cylinder {
                radius: rho,
                half_height: hh,
                round: 0.0,
                chamfer: 0.0,
            },
            pose,
        )],
        NodeId(0),
    )
    .expect("o cilindro");
    // ⚠️ O eixo do `Cylinder` é o `z` (medido: o interior está em `(0, 0, ±h)`).
    let u_y = pose.apply_dir([0.0, 0.0, 1.0])[1];
    let esperado = 0.3 - (hh * u_y.abs() + rho * (1.0 - u_y * u_y).max(0.0).sqrt());
    let reg = Registry::new();
    let y = lowest_point(&doc, &reg).expect("o cilindro tem geometria");
    assert!(
        (y - esperado).abs() <= 1e-4,
        "a borda está a {esperado} e a busca achou {y}"
    );
    let caixa = ph2d_field_eval::bounds::bounding_ball(&doc, &reg)
        .expect("a bola")
        .aabb()
        .0[1];
    assert!(
        caixa < esperado - 0.005,
        "a caixa ({caixa}) já não mente sobre a borda ({esperado}) — o controlo deixou de controlar"
    );
}

#[test]
fn o_ponto_mais_baixo_de_duas_pecas_e_o_da_de_baixo() {
    let folha = |c: [f32; 3], r: f32| {
        ph2d_field_eval::leaf(Primitive::Sphere { radius: r }, Xform::at(c[0], c[1], c[2]))
    };
    let doc = FieldDoc::new(
        vec![
            folha([-0.3, 0.5, 0.0], 0.2),
            folha([0.3, 0.1, 0.2], 0.15),
            Node {
                xform: Xform::IDENTITY,
                kind: NodeKind::Combine {
                    op: Op::Union(ph2d_field::Blend::Sharp),
                    children: vec![NodeId(0), NodeId(1)],
                },
                mods: Vec::new(),
                verb: None,
            },
        ],
        NodeId(2),
    )
    .expect("as duas");
    let y = lowest_point(&doc, &Registry::new()).expect("geometria");
    assert!(
        (y + 0.05).abs() < 3e-4,
        "a de baixo toca em −0,05 e saiu {y}"
    );
}

// ── o que se pinta ─────────────────────────────────────────────────────────────────────────────

/// A câmera de omissão, a olhar para o sítio onde a esfera pousa.
fn camara() -> Orbit {
    Orbit {
        target: [0.0, 0.2, 0.0],
        ..Orbit::default()
    }
}

/// Uma lâmpada a `~45°` de elevação — uma sombra DESLOCADA pela altura, que é o que a régua mede.
const LAMPADA: PointLamp = PointLamp {
    world: [-1.2, 1.4, 0.4],
    radiance_at_one: [4.0, 4.0, 4.0],
};

const CHAO: Ground = Ground { height: 0.0 };

/// A imagem de uma esfera de raio `0,25` a `folga` acima do chão, e as sombras dela.
fn pinta(folga: f32, chao: Option<Ground>, lampadas: &[PointLamp], w: u32, h: u32) -> Vec<u8> {
    let doc = esfera_em([0.0, 0.25 + folga, 0.0], 0.25);
    let reg = Registry::new();
    let cam = camara();
    let g = trace(&doc, &reg, &cam, w, h);
    let mundos: Vec<[f32; 3]> = lampadas.iter().map(|l| l.world).collect();
    let sh = shadow_pass_on(&doc, &reg, &cam, &g, &mundos, chao);
    pinta_com(&g, &cam, lampadas, &sh)
}

fn pinta_com(g: &crate::Gbuffer, cam: &Orbit, lampadas: &[PointLamp], sh: &Shadows) -> Vec<u8> {
    let mat = [ph2d_material::OpenPbr::default().prepare()];
    shade_render(
        g,
        cam,
        &Surfaces {
            all: &mat,
            owners: None,
        },
        &Lighting {
            lamps: &[],
            points: lampadas,
            sky: &CeuUniforme(0.3),
            shadows: Some(sh),
        },
        Look::default(),
        FUNDO,
    )
}

/// Os alfas dos pixels de FUNDO (o fundo é transparente: o alfa de um pixel de fundo é a sombra).
///
/// ⛔⛔ **Os pixels de BORDA ficam de fora, e a 1.ª redacção não os tirava:** um pixel de silhueta
/// cujo centro falha a peça leva tinta das sub-amostras que a acertam — ele tem alfa por COBERTURA,
/// não por sombra. Com ele dentro, o `sem_lampadas_o_contacto_escurece_pela_oclusao` lia `128` de
/// contacto com a oclusão do chão **ignorada** (a verdade é `224`), e a mutação sobrevivia.
fn alfas_de_fundo(g: &crate::Gbuffer, rgba: &[u8]) -> Vec<(usize, u8)> {
    rgba.as_chunks::<4>()
        .0
        .iter()
        .enumerate()
        .filter(|(i, _)| {
            !g.hit[*i]
                && g.edges
                    .binary_search_by_key(&(*i as u32), |e| e.pixel)
                    .is_err()
        })
        .map(|(i, px)| (i, px[3]))
        .collect()
}

/// ⭐ **Sem chão, o quadro é o de sempre, ao byte** — a porta nova com `None` é a porta velha.
#[test]
fn sem_chao_a_imagem_e_a_de_sempre() {
    let doc = esfera_em([0.0, 0.25, 0.0], 0.25);
    let reg = Registry::new();
    let cam = camara();
    let g = trace(&doc, &reg, &cam, 160, 90);
    let velha = shadow_pass(&doc, &reg, &cam, &g, &[LAMPADA.world]);
    let nova = shadow_pass_on(&doc, &reg, &cam, &g, &[LAMPADA.world], None);
    assert_eq!(velha, nova, "com `None` o passe novo tem de ser o velho");
    assert_eq!(
        pinta_com(&g, &cam, &[LAMPADA], &velha),
        pinta_com(&g, &cam, &[LAMPADA], &nova)
    );
    let rgba = pinta_com(&g, &cam, &[LAMPADA], &nova);
    for (i, a) in alfas_de_fundo(&g, &rgba) {
        assert_eq!(a, 0, "sem chão um pixel de fundo não pode ter tinta ({i})");
    }
}

/// ⭐⭐⭐ **A esfera pousada deita sombra no chão — e longe dela o fundo é o de sempre, AO BYTE.**
#[test]
fn a_esfera_pousada_deita_sombra_e_longe_dela_o_fundo_e_intacto() {
    let (w, h) = (320, 180);
    let doc = esfera_em([0.0, 0.25, 0.0], 0.25);
    let reg = Registry::new();
    let cam = camara();
    let g = trace(&doc, &reg, &cam, w, h);
    let sh = shadow_pass_on(&doc, &reg, &cam, &g, &[LAMPADA.world], Some(CHAO));
    let rgba = pinta_com(&g, &cam, &[LAMPADA], &sh);
    let fundo = alfas_de_fundo(&g, &rgba);
    let sombra = fundo.iter().filter(|(_, a)| *a > 20).count();
    let escura = fundo.iter().map(|(_, a)| *a).max().unwrap_or(0);
    assert!(
        sombra > 800,
        "a sombra no chão tem de ser uma MANCHA, e tem {sombra} pixels de fundo com tinta"
    );
    // ⚠️ O miolo NÃO é preto, e está certo: sem a oclusão (que refina depois) o céu continua a
    // iluminar a mancha, e a razão fica `céu / (céu + lâmpada)` — `~0,25` nesta fixtura.
    assert!(
        escura > 150,
        "o miolo da sombra tem de ser escuro, e o mais escuro lê {escura}"
    );
    // ⚠️ Os cantos de cima vêem o chão ao longe (a câmera olha de cima) ou o céu: nos dois casos os
    // bytes são os do fundo.
    for i in [0, (w - 1) as usize, (w as usize) * 10 + 5] {
        assert!(!g.hit[i]);
        assert_eq!(
            &rgba[i * 4..i * 4 + 4],
            &FUNDO,
            "o pixel {i} longe da peça mudou"
        );
    }
    // ⭐ E a sombra não é do chão inteiro: a maior parte dele fica sem marca VISÍVEL. ⚠️ Não «intacto»:
    // o céu do chão escurece de leve uma coroa larga à volta da peça (como a referência de cones).
    let marcados = fundo.iter().filter(|(_, a)| *a > 20).count();
    assert!(
        marcados * 2 < fundo.len(),
        "{marcados} de {} pixels de fundo com marca visível — a sombra tapou o chão inteiro?",
        fundo.len()
    );
}

/// ⭐⭐⭐ **A RÉGUA DA `W4`: um objecto a `0`, `1` e `10 cm` do chão dá três sombras diferentes.**
///
/// ⚠️ Com a lâmpada de lado, a altura DESLOCA a sombra (o centro dela afasta-se do pé da peça) e
/// ALARGA-LHE a penumbra (o estimador `k·d/t` amolece com a distância ao oclusor). O gate mede as
/// duas, e as três alturas têm de se distinguir nas duas.
#[test]
fn tres_alturas_tres_sombras() {
    let (w, h) = (320, 180);
    let cam = camara();
    let screen = Screen::new(w, h, cam.half_extent);
    let mut centros = Vec::new();
    let mut penumbras = Vec::new();
    for folga in [0.0f32, 0.01, 0.10] {
        let doc = esfera_em([0.0, 0.25 + folga, 0.0], 0.25);
        let g = trace(&doc, &Registry::new(), &cam, w, h);
        let rgba = pinta(folga, Some(CHAO), &[LAMPADA], w, h);
        let (mut sx, mut sy, mut sw) = (0.0f64, 0.0f64, 0.0f64);
        let mut meias = 0usize;
        for (i, a) in alfas_de_fundo(&g, &rgba) {
            if a == 0 {
                continue;
            }
            let peso = f64::from(a);
            sx += peso * (i % w as usize) as f64;
            sy += peso * (i / w as usize) as f64;
            sw += peso;
            if a > 10 && a < 240 {
                meias += 1;
            }
        }
        assert!(sw > 0.0, "[{folga}] sem sombra nenhuma");
        centros.push((sx / sw, sy / sw));
        penumbras.push(meias);
    }
    // O pé da peça no ecrã, para dizer em que direcção a sombra anda.
    let pe = cam
        .project([0.0, 0.0, 0.0], screen)
        .expect("o pé está à frente")
        .0;
    let longe = |c: (f64, f64)| (c.0 - f64::from(pe[0])).hypot(c.1 - f64::from(pe[1]));
    assert!(
        longe(centros[0]) < longe(centros[1]) && longe(centros[1]) < longe(centros[2]),
        "a sombra tem de se AFASTAR do pé com a altura: {centros:?} (pé em {pe:?})"
    );
    assert!(
        longe(centros[2]) - longe(centros[0]) > 3.0,
        "a 10 cm a sombra tem de andar pixels visíveis: {centros:?}"
    );
    assert!(
        penumbras[0] < penumbras[2],
        "a penumbra tem de ALARGAR com a altura: {penumbras:?}"
    );
    assert!(
        centros[0] != centros[1] && penumbras[0] != penumbras[1],
        "0 e 1 cm têm de dar sombras diferentes: {centros:?} {penumbras:?}"
    );
}

/// ⭐⭐ **Sem lâmpada nenhuma, o contacto escurece pela OCLUSÃO** — o céu é uma fonte, e a peça tapa-o
/// junto do pé. É o *contact shadow* que faz uma peça pousar mesmo sob luz difusa.
#[test]
fn sem_lampadas_o_contacto_escurece_pela_oclusao() {
    let (w, h) = (160, 90);
    let doc = esfera_em([0.0, 0.25, 0.0], 0.25);
    let reg = Registry::new();
    let cam = camara();
    let g = trace(&doc, &reg, &cam, w, h);
    let mut sh = shadow_pass_on(&doc, &reg, &cam, &g, &[], Some(CHAO));
    assert_eq!(
        sh.ground(),
        Some(CHAO),
        "sem lâmpadas o chão tem de ficar declarado"
    );
    let sem_cena = Surfaces {
        all: &[],
        owners: None,
    };
    let passagens = refine_hemisphere(&doc, &reg, &cam, &g, &sem_cena, &[], &mut sh, |_, _| true);
    assert_eq!(passagens, crate::OCCLUSION_PASSES);
    let rgba = pinta_com(&g, &cam, &[], &sh);
    let fundo = alfas_de_fundo(&g, &rgba);
    let contacto = fundo.iter().map(|(_, a)| *a).max().unwrap_or(0);
    let sombreados = fundo.iter().filter(|(_, a)| *a > 20).count();
    // ⚠️ **As duas metades**: quão escuro fica o contacto, e quanto chão ele cobre. Com a oclusão do
    // chão ignorada isto lê `128` e `39` — tudo pixels de silhueta (ver [`alfas_de_fundo`]).
    assert!(
        contacto > 150 && sombreados > 1_000,
        "junto do pé o céu é tapado: o mais escuro lê {contacto} e {sombreados} pixels de chão têm sombra"
    );
    assert_eq!(&rgba[0..4], &FUNDO, "longe da peça o céu chega inteiro");
}

/// ⚠️ **Um olho debaixo do chão não vê sombra nenhuma** — o chão só existe visto de cima.
#[test]
fn um_olho_debaixo_do_chao_nao_ve_sombra() {
    let (w, h) = (160, 90);
    let doc = esfera_em([0.0, 0.25, 0.0], 0.25);
    let reg = Registry::new();
    let cam = Orbit {
        target: [0.0, -0.6, 0.0],
        ..Orbit::from_yaw_pitch(0.3, -0.5)
    };
    let g = trace(&doc, &reg, &cam, w, h);
    let sh = shadow_pass_on(&doc, &reg, &cam, &g, &[LAMPADA.world], Some(CHAO));
    let com = pinta_com(&g, &cam, &[LAMPADA], &sh);
    let sem = pinta_com(
        &g,
        &cam,
        &[LAMPADA],
        &shadow_pass_on(&doc, &reg, &cam, &g, &[LAMPADA.world], None),
    );
    assert_eq!(com, sem, "visto de baixo, o chão invisível não é nada");
}

/// ⭐⭐ **A silhueta de baixo não tem um FIO CLARO** entre a peça e a sombra de contacto.
///
/// ⚠️ As sub-amostras de uma borda que falham a peça vêem o CHÃO, não o fundo limpo. A propriedade é
/// exacta: um pixel de borda é `cobertura + (1 − cobertura)·sombra`, logo **nunca é mais
/// transparente do que o chão escuro ao lado dele**. Com o fundo limpo ele sairia com o alfa da
/// cobertura só — um anel claro à volta do pé, exactamente onde a sombra é mais escura.
#[test]
fn a_silhueta_de_baixo_nao_tem_fio_claro() {
    let (w, h) = (320, 180);
    let doc = esfera_em([0.0, 0.25, 0.0], 0.25);
    let reg = Registry::new();
    let cam = camara();
    let g = trace(&doc, &reg, &cam, w, h);
    let sh = shadow_pass_on(&doc, &reg, &cam, &g, &[LAMPADA.world], Some(CHAO));
    let rgba = pinta_com(&g, &cam, &[LAMPADA], &sh);
    let wu = w as usize;
    let alfa = |i: usize| rgba[i * 4 + 3];
    let mut julgados = 0;
    for e in &g.edges {
        let i = e.pixel as usize;
        if !g.hit[i] || i < wu || i + wu >= g.hit.len() {
            continue;
        }
        let fundo: Vec<u8> = [i - 1, i + 1, i - wu, i + wu]
            .into_iter()
            .filter(|j| !g.hit[*j])
            .map(alfa)
            .collect();
        let Some(&mais_claro) = fundo.iter().min() else {
            continue;
        };
        if mais_claro < 120 {
            continue;
        }
        julgados += 1;
        assert!(
            u16::from(alfa(i)) + 2 >= u16::from(mais_claro),
            "o pixel de borda {i} lê alfa {} ao lado de uma sombra de {mais_claro} — um fio claro",
            alfa(i)
        );
    }
    assert!(
        julgados > 5,
        "a fixtura tem de ter bordas encostadas à sombra de contacto, e tem {julgados}"
    );
}

// ── o céu do chão ──────────────────────────────────────────────────────────────────────────────

/// ⭐ **A REFERÊNCIA**: os cones da peça, CONVERGIDOS (`total` direcções), num ponto do chão — sem a
/// cerca da bola (num cone ela é outra descontinuidade: o eixo pode falhar a bola e o cone tocar a
/// peça). É contra isto que as constantes da [`crate::ground::ground_sky`] foram ajustadas.
fn ceu_por_cones(
    doc: &FieldDoc,
    reg: &Registry,
    cam: &Orbit,
    g: &crate::Gbuffer,
    pontos: &[Option<[f32; 3]>],
    total: u32,
) -> Vec<f32> {
    let shape = ph2d_field_eval::hybrid::Hybrid::new(doc, reg);
    let scene = crate::march::Scene {
        shape: &shape,
        cam,
        basis: cam.basis(),
        sharp: crate::Sharpness::for_frame(cam.half_extent, g.width.min(g.height) as usize),
        clip: None,
        step: ph2d_field_eval::safe_march_step(doc),
        shrink: ph2d_field_eval::field_shrink(doc, reg),
        stencil: crate::Stencil::Tetra4,
    };
    let lift = scene.sharp.hit * crate::march::BIAS;
    let alcance = crate::OCCLUSION_REACH * cam.half_extent;
    let (mut quem, mut origens, mut dirs, mut cercas, mut durezas) =
        (Vec::new(), Vec::new(), Vec::new(), Vec::new(), Vec::new());
    let mut peso = vec![0.0f32; pontos.len()];
    for (i, q) in pontos.iter().enumerate() {
        let Some(q) = q else { continue };
        for k in 0..total {
            let d = crate::cone_dir(k, total);
            if d[1] <= 0.0 {
                continue;
            }
            peso[i] += d[1];
            quem.push((i, d[1]));
            origens.push([q[0], q[1] + lift, q[2]]);
            dirs.push(d);
            cercas.push(alcance);
            durezas.push(1.0 / d[1]);
        }
    }
    let v = crate::march::march_cone_to(&scene, &origens, &dirs, &cercas, &durezas);
    let mut soma = vec![0.0f32; pontos.len()];
    for (j, &(i, c)) in quem.iter().enumerate() {
        soma[i] += c * v[j];
    }
    soma.iter()
        .zip(&peso)
        .map(|(s, w)| if *w > 0.0 { s / w } else { 1.0 })
        .collect()
}

/// As duas cenas: a do ajuste (esfera e cubo) e uma que NÃO entrou nele (três cilindros cruzados,
/// um por eixo, pousados pelo ponto mais baixo).
///
/// ⚠️ **O eixo do `Cylinder` é o `z`**, e a 1.ª redacção desta fixtura supôs `y`: rodar em `z` não o
/// move, e a «cruz» tinha dois cilindros sobrepostos.
fn cenas_do_ceu() -> Vec<(&'static str, FieldDoc)> {
    let folha = ph2d_field_eval::leaf;
    let uniao = |filhos: Vec<NodeId>| Node {
        xform: Xform::IDENTITY,
        kind: NodeKind::Combine {
            op: Op::Union(ph2d_field::Blend::Sharp),
            children: filhos,
        },
        mods: Vec::new(),
        verb: None,
    };
    let s = std::f32::consts::FRAC_1_SQRT_2;
    let cil = |rot: [f32; 4]| {
        folha(
            Primitive::Cylinder {
                radius: 0.08,
                half_height: 0.3,
                round: 0.02,
                chamfer: 0.0,
            },
            Xform {
                translation: [0.0, 0.3, 0.0],
                rotation: rot,
                scale: 1.0,
            },
        )
    };
    vec![
        (
            "esfera e cubo",
            FieldDoc::new(
                vec![
                    folha(
                        Primitive::Sphere { radius: 0.22 },
                        Xform::at(-0.3, 0.22, 0.0),
                    ),
                    folha(
                        Primitive::Box {
                            half: [0.15; 3],
                            round: 0.02,
                            chamfer: 0.0,
                        },
                        Xform::at(0.3, 0.15, 0.1),
                    ),
                    uniao(vec![NodeId(0), NodeId(1)]),
                ],
                NodeId(2),
            )
            .expect("esfera e cubo"),
        ),
        (
            "cruz",
            FieldDoc::new(
                vec![
                    cil([0.0, 0.0, 0.0, 1.0]),
                    cil([s, 0.0, 0.0, s]),
                    cil([0.0, s, 0.0, s]),
                    uniao(vec![NodeId(0), NodeId(1), NodeId(2)]),
                ],
                NodeId(3),
            )
            .expect("a cruz"),
        ),
    ]
}

/// O céu do chão de uma cena, pela lei e pela referência, sobre os pixels de chão.
fn ceu_das_duas_leis(doc: &FieldDoc, w: u32, h: u32, cones: u32) -> (Vec<f32>, Vec<f32>) {
    let reg = Registry::new();
    let cam = Orbit {
        target: [0.0, 0.15, 0.0],
        ..Orbit::default()
    };
    let g = trace(doc, &reg, &cam, w, h);
    let chao = lowest_point(doc, &reg).map(|height| Ground { height });
    let pontos = crate::ground::ground_points(&cam, &g, chao);
    let lei = crate::ground::ground_sky(doc, &reg, &cam, &pontos);
    let referencia = ceu_por_cones(doc, &reg, &cam, &g, &pontos, cones);
    let (a, b): (Vec<f32>, Vec<f32>) = pontos
        .iter()
        .zip(lei.iter().zip(&referencia))
        .filter(|(q, _)| q.is_some())
        .map(|(_, (l, r))| (*l, *r))
        .unzip();
    (a, b)
}

/// ⭐⭐⭐ **A LEI DO CÉU DO CHÃO SEGUE OS CONES CONVERGIDOS — e numa cena que NÃO entrou no ajuste.**
///
/// Medido (`160×90`, `1 024` cones), sobre os pixels de chão com a referência abaixo de `0,98`:
///
/// | cena | pixels | erro quadrático | p90 | pior |
/// |---|---:|---:|---:|---:|
/// | esfera e cubo (a do ajuste) | `6 172` | `0,044` | `0,071` | `0,238` |
/// | cruz de três eixos (fora do ajuste) | `5 770` | `0,028` | `0,043` | `0,114` |
///
/// ⚠️ As barras ficam acima das duas com folga de ruído, e a da cruz é a que diz que as constantes
/// não decoraram a cena em que nasceram.
#[test]
fn a_oclusao_do_chao_segue_os_cones_convergidos() {
    for (nome, doc) in cenas_do_ceu() {
        let (lei, referencia) = ceu_das_duas_leis(&doc, 160, 90, 1024);
        let mut perto: Vec<f32> = lei
            .iter()
            .zip(&referencia)
            .filter(|(_, r)| **r < 0.98)
            .map(|(l, r)| (l - r).abs())
            .collect();
        assert!(
            perto.len() > 3000,
            "[{nome}] a fixtura tem de ter chão ocluído: {}",
            perto.len()
        );
        perto.sort_by(f32::total_cmp);
        #[allow(clippy::cast_precision_loss)]
        let rmse = (perto.iter().map(|e| e * e).sum::<f32>() / perto.len() as f32).sqrt();
        let p90 = perto[perto.len() * 9 / 10];
        let pior = perto[perto.len() - 1];
        assert!(
            rmse <= 0.06 && p90 <= 0.10 && pior <= 0.30,
            "[{nome}] a lei afastou-se da referência: rmse {rmse:.4} · p90 {p90:.4} · pior {pior:.4}"
        );
    }
}

/// Quantas vezes a luz do chão muda de sentido ao longo das linhas da metade de baixo da imagem —
/// a régua dos ANÉIS (um anel é um degrau, e cada degrau dá dois extremos).
fn extremos_do_chao(ceu: &[f32], w: usize, h: usize) -> usize {
    let mut n = 0;
    for y in h / 2..h {
        let linha = &ceu[y * w..(y + 1) * w];
        let mut sinal = 0i8;
        for x in 1..w {
            let d = linha[x] - linha[x - 1];
            if d.abs() < 1e-4 {
                continue;
            }
            let s = if d > 0.0 { 1 } else { -1 };
            if sinal != 0 && s != sinal {
                n += 1;
            }
            sinal = s;
        }
    }
    n
}

/// ⭐⭐ **O CHÃO NÃO TEM ANÉIS** — a razão de a lei não ser a dos cones da peça.
///
/// O controlo são os `48` cones do produto no mesmo chão (os anéis que a primeira imagem mostrou), e a
/// barra é uma FRACÇÃO deles: a lei contínua só muda de sentido onde a peça muda de forma.
#[test]
fn o_ceu_do_chao_nao_tem_aneis() {
    let (w, h) = (320u32, 180u32);
    let (_, doc) = cenas_do_ceu().remove(0);
    let reg = Registry::new();
    let cam = Orbit {
        target: [0.0, 0.15, 0.0],
        ..Orbit::default()
    };
    let g = trace(&doc, &reg, &cam, w, h);
    let chao = lowest_point(&doc, &reg).map(|height| Ground { height });
    let pontos = crate::ground::ground_points(&cam, &g, chao);
    let so_chao = |v: Vec<f32>| -> Vec<f32> {
        v.iter()
            .zip(&pontos)
            .map(|(c, q)| if q.is_some() { *c } else { 1.0 })
            .collect()
    };
    let lei = so_chao(crate::ground::ground_sky(&doc, &reg, &cam, &pontos));
    let cones = so_chao(ceu_por_cones(
        &doc,
        &reg,
        &cam,
        &g,
        &pontos,
        crate::OCCLUSION_PASSES,
    ));
    let (a, b) = (
        extremos_do_chao(&lei, w as usize, h as usize),
        extremos_do_chao(&cones, w as usize, h as usize),
    );
    println!("extremos: lei {a} · 48 cones {b}");
    assert!(
        b > 200,
        "o controlo tem de mostrar os anéis dos cones, e mostra {b}"
    );
    assert!(
        a * 4 <= b,
        "o chão da lei tem {a} extremos contra {b} dos cones — anéis?"
    );
}

/// ⭐⭐⭐ **A SOMBRA DO CHÃO NÃO TEM DEGRAU LONGE DA PEÇA** — o gate da cerca alargada.
///
/// ⛔⛔ Com a bola SIMPLES a penumbra era cortada numa elipse dura: um raio que passava a raspar a
/// ponta de um cilindro, fora da bola, não marchava e lia `1`; o vizinho que a tocava lia `0,4`.
/// Medido nesta fixtura (a cruz pousada), sobre pares de vizinhos cujo chão está a mais de
/// `raio + 0,15` do centro da peça — isto é, **fora do contacto**, onde a sombra é penumbra e tem de
/// ser suave:
///
/// | cerca | pior salto | saltos `> 0,2` |
/// |---|---:|---:|
/// | a bola simples | `0,968` | `87` |
/// | **+ `distância à luz / dureza`** | **`0,093`** | **`0`** |
///
/// ⚠️ **A régua exclui o contacto de propósito**: ali a sombra muda mesmo de repente (a peça toca o
/// chão), e um máximo global lê `1,0` nas duas cercas — *era a primeira redacção, e não separava nada*.
#[test]
fn a_sombra_do_chao_nao_tem_degrau_longe_da_peca() {
    let (w, h) = (320u32, 180u32);
    let (_, doc) = cenas_do_ceu().remove(1);
    let reg = Registry::new();
    let cam = Orbit {
        target: [0.0, 0.15, 0.0],
        ..Orbit::default()
    };
    let g = trace(&doc, &reg, &cam, w, h);
    let chao = lowest_point(&doc, &reg).map(|height| Ground { height });
    let sh = shadow_pass_on(&doc, &reg, &cam, &g, &[LAMPADA.world], chao);
    let pontos = crate::ground::ground_points(&cam, &g, chao);
    let bola = ph2d_field_eval::bounds::bounding_ball(&doc, &reg).expect("a bola");
    let longe = |i: usize| {
        pontos[i].is_some_and(|q| {
            let d = [q[0] - bola.center[0], q[2] - bola.center[2]];
            d[0].hypot(d[1]) > bola.radius + 0.15
        })
    };
    let wu = w as usize;
    let (mut pior, mut saltos, mut julgados) = (0.0f32, 0usize, 0usize);
    let mut sombreados = 0usize;
    for y in 0..h as usize {
        for x in 1..wu {
            let (i, j) = (y * wu + x, y * wu + x - 1);
            if !longe(i) || !longe(j) {
                continue;
            }
            julgados += 1;
            if sh.at(0, i) < 0.9 {
                sombreados += 1;
            }
            let d = (sh.at(0, i) - sh.at(0, j)).abs();
            if d > 0.2 {
                saltos += 1;
            }
            pior = pior.max(d);
        }
    }
    // ⭐ A população: pares julgados, e pares de facto SOMBREADOS — sem os segundos, um chão todo
    // iluminado leria «sem degrau» sobre uma sombra que não existe.
    assert!(julgados > 10_000, "só {julgados} pares longe da peça");
    assert!(
        sombreados > 500,
        "só {sombreados} pares sombreados longe da peça"
    );
    assert!(
        saltos == 0 && pior <= 0.15,
        "a penumbra longe da peça tem degraus: pior {pior:.4}, {saltos} saltos acima de 0,2"
    );
}
