//! ⭐⭐⭐⭐ **O MATCAP NOS DOIS MOTORES** — a imagem do dispositivo contra a do
//! [`ph2d_field_render::shade_with`], byte a byte.
//!
//! # ⚠️ O que este gate ISOLA, e porque isso é o ponto
//!
//! Os dois lados correm sobre a **MESMA marcha do dispositivo** (o gate irmão do pintor de material
//! usa a mesma disciplina): o mesmo `t` e a mesma normal. ⇒ o que sobra entre eles é **só a lei do
//! matcap** — a amostra bilinear, a convenção `uv = n.xy·0,5 + 0,5`, o olhar e a descida a 8 bits,
//! mais a média das quatro sub-amostras da silhueta.
//!
//! ⛔ Um gate que traçasse cada lado no seu motor mediria as duas coisas somadas, e uma divergência
//! da AMOSTRA ficaria escondida atrás da de geometria — que já tem gate próprio
//! (`o_gbuffer_do_dispositivo_e_o_da_cpu`).

use ph2d_field::{FieldDoc, Node, NodeId, NodeKind, Op, Primitive, Xform};
use ph2d_material::{OpenPbr, Surface};

// ⚠️ A fixtura de TRÊS folhas mudou-se do `paint_parity_tests` (o pintor de material do Render
// traçado, retirado em 03/10) para aqui, que é quem a lê.
pub(crate) const W: u32 = 192;
pub(crate) const H: u32 = 108;
pub(crate) const FUNDO: [u8; 4] = [0, 0, 0, 0];

pub(crate) fn combina(op: Op, filhos: Vec<NodeId>) -> Node {
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

/// O lado da fotografia de teste. ⚠️ **Ímpar de propósito**: com um lado par e uma normal simétrica
/// as coordenadas caem nos centros dos texels e a bilinear degenera no vizinho-mais-próximo —
/// *uma fixtura que não interpola não testa interpolação nenhuma*.
const LADO: u32 = 63;

/// ⭐⭐⭐ **Uma fotografia que DISCRIMINA a lei de amostragem.**
///
/// # ⛔ Porque ela não é um gradiente suave
///
/// Um gradiente linear é **reproduzido exactamente** por bilinear **e** por vizinho-mais-próximo a
/// menos de meio texel, e as duas leituras ficariam dentro de qualquer barra. ⇒ ela leva um
/// **tabuleiro de xadrez de um texel** somado a uma rampa: ali a bilinear devolve a média dos
/// quatro vizinhos e o vizinho-mais-próximo devolve o extremo, e a diferença é `~0,5`.
///
/// ⚠️ **Os valores ficam em `0..=1`** — um matcap é uma fotografia, e é nesse domínio que o
/// `Look::default()` é a identidade ao bit.
fn fotografia() -> (u32, Vec<f32>) {
    let lado = LADO as usize;
    let mut rgb = Vec::with_capacity(lado * lado * 3);
    for y in 0..lado {
        for x in 0..lado {
            #[allow(clippy::cast_precision_loss)]
            let (fx, fy) = (x as f32 / lado as f32, y as f32 / lado as f32);
            let xadrez = if (x + y) % 2 == 0 { 0.35 } else { 0.0 };
            rgb.push((fx * 0.6 + xadrez).clamp(0.0, 1.0));
            rgb.push((fy * 0.6 + xadrez).clamp(0.0, 1.0));
            rgb.push((0.5 - 0.4 * fx + xadrez).clamp(0.0, 1.0));
        }
    }
    (LADO, rgb)
}

/// ⭐ A chave é derivada do conteúdo, como no produto — ver [`crate::smoke_state::MatcapTexels`].
fn chave(side: u32, rgb: &[f32]) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    let mut come = |b: u8| {
        h ^= u64::from(b);
        h = h.wrapping_mul(0x0000_0100_0000_01b3);
    };
    for b in side.to_le_bytes() {
        come(b);
    }
    for v in rgb {
        for b in v.to_le_bytes() {
            come(b);
        }
    }
    h
}

/// ⭐⭐⭐⭐ **A LEI DO MATCAP É A MESMA NOS DOIS MOTORES.**
///
/// ⚠️ **A barra é o PIOR BYTE e não uma fracção**, e a razão é medida: em 2026-09-19 uma paridade
/// escrita como fracção leu `99,579 %` contra uma barra de `99,5` — *passava por `0,079`* — com o
/// pior byte a `12`, sobre um dispositivo que **não estava a pedir o canal**. *Uma fracção afoga um
/// fenómeno que ocupa um décimo da imagem.*
#[test]
#[ignore = "precisa de adaptador de GPU"]
fn o_matcap_e_o_mesmo_nos_dois_motores() {
    let Some(t) = crate::gpu_frame::shared() else {
        eprintln!("[matcap-parity] sem adaptador — skip");
        return;
    };
    let (doc, _folhas, _sup) = fixtura();
    let reg = ph2d_field_eval::hybrid::Registry::new();
    let cam = ph2d_field_render::Orbit::default();
    let (lado, rgb) = fotografia();
    let look = ph2d_view_transform::Look::default();

    let g =
        crate::gpu_frame::march(t, &doc, &reg, &cam, W, H, true).expect("a marcha do dispositivo");

    // ⭐ **A REFERÊNCIA: a lei da CPU sobre a marcha do dispositivo.**
    let cpu = ph2d_field_render::shade_with(
        &g,
        &ph2d_field_render::Matcap {
            side: lado,
            rgb_linear: &rgb,
        },
        look,
        FUNDO,
    );

    // ⭐ **O PRODUTO: o passe do dispositivo, pela porta que o quadro usa.**
    let dev = crate::gpu_frame::pinta_matcap(
        t,
        &doc,
        &reg,
        &cam,
        &ph2d_field_gpu::matcap::MatcapSetup {
            rgb_linear: &rgb,
            side: lado,
            chave: chave(lado, &rgb),
            stops: look.exposure_stops,
            view: ph2d_view_transform::wgsl::view_code(look.view),
            background: FUNDO,
            entrega: None,
        },
        W,
        H,
    )
    .expect("o passe do matcap no dispositivo");

    assert_eq!(
        dev.rgba.len(),
        cpu.len(),
        "as duas imagens têm o mesmo tamanho"
    );
    assert_eq!(
        dev.edges,
        g.edges.len(),
        "as duas marchas re-amostraram a MESMA silhueta — sem isto o gate compararia dois quadros \
         com anti-serrilhado diferente e leria a diferença como defeito de LEI"
    );

    // ⚠️ **O CONTROLO vem primeiro:** uma imagem toda de fundo passaria em qualquer barra. A peça
    // tem de estar lá, nos DOIS lados.
    let acertos = |v: &[u8]| {
        v.as_chunks::<4>()
            .0
            .iter()
            .filter(|p| p[3] > FUNDO[3])
            .count()
    };
    let (a_cpu, a_dev) = (acertos(&cpu), acertos(&dev.rgba));
    assert!(
        a_cpu > (W * H / 20) as usize,
        "CONTROLO: a fixtura tem de cobrir píxeis — {a_cpu} de {}",
        W * H
    );
    assert_eq!(a_cpu, a_dev, "os dois lados cobrem os MESMOS píxeis");

    let mut pior = 0u8;
    let mut fora = 0usize;
    let mut pior_px = 0usize;
    for (i, (c, d)) in cpu
        .as_chunks::<4>()
        .0
        .iter()
        .zip(dev.rgba.as_chunks::<4>().0)
        .enumerate()
    {
        let delta = (0..4).map(|k| c[k].abs_diff(d[k])).max().unwrap_or(0);
        if delta > 0 {
            fora += 1;
        }
        if delta > pior {
            pior = delta;
            pior_px = i;
        }
    }
    // ⭐⭐⭐ **A barra é `1` byte** — o arredondamento a 8 bits de duas somas em `f32` que a placa
    // pode contrair num `fma`. ⛔ Ela NÃO é um número escolhido: o texto do empacotamento é o MESMO
    // dos dois lados do dispositivo (ver [`ph2d_field_gpu::empacota_wgsl`]) e a lei da amostra é a
    // da CPU linha a linha, logo tudo acima disto é uma DIVERGÊNCIA de lei e não de última casa.
    assert!(
        pior <= 1,
        "o matcap divergiu entre os motores: pior byte {pior} no pixel {pior_px} \
         ({fora} píxeis fora de {})",
        W * H
    );
}

/// ⭐⭐⭐⭐ **O MATCAP MARCHA NO KERNEL MAGRO.**
///
/// Medido (`docs/Render3d/03` §W9, «o kernel que hospeda a marcha»): o kernel que trazia o chão, as
/// lâmpadas e o ricochete do Render traçado, que o matcap nunca lê, custava `4×`–`9×` o quadro (o
/// nó, `86` contra `10 ms` a `1920×1080`). Esse kernel saiu em 03/10 com o Render traçado; o gate
/// afirma que nenhuma entrada de luz ou de céu volta a ser compilada — nem pelo matcap nem pela
/// marcha do G-buffer.
#[test]
#[ignore = "precisa de adaptador de GPU"]
fn o_matcap_marcha_no_kernel_magro() {
    let Some(mut t) = ph2d_field_gpu::trace::Tracer::new() else {
        eprintln!("[matcap-magro] sem adaptador — skip");
        return;
    };
    let (doc, _folhas, _sup) = fixtura();
    let reg = ph2d_field_eval::hybrid::Registry::new();
    let cam = ph2d_field_render::Orbit::default();
    let (lado, rgb) = fotografia();
    let look = ph2d_view_transform::Look::default();
    let mc = ph2d_field_gpu::matcap::MatcapSetup {
        rgb_linear: &rgb,
        side: lado,
        chave: chave(lado, &rgb),
        stops: look.exposure_stops,
        view: ph2d_view_transform::wgsl::view_code(look.view),
        background: FUNDO,
        entrega: None,
    };
    let (c, f, setup) =
        super::pedido(&doc, &reg, &cam, super::Sonda::default(), W, H).expect("o pedido");
    let _ = t.matcap_frame(&f, c.sculpts(), setup, &mc, W, H);
    let depois_do_matcap = t.entradas_compiladas();
    assert!(
        depois_do_matcap.iter().any(|e| e == "centro_so"),
        "o matcap não compilou a entrada magra: {depois_do_matcap:?}"
    );
    assert!(
        !depois_do_matcap
            .iter()
            .any(|e| e.contains("luz") || e.contains("ceu")),
        "o matcap compilou uma entrada de LUZ ou de CÉU — a marcha voltou a pagar o que ele não \
         lê: {depois_do_matcap:?}"
    );
    // ⭐⭐⭐ **A borda re-amostra-se COMPACTA** (`docs/Render3d/03` §W9, «a borda que esperava pelas
    // vizinhas»): a detecção e a re-amostragem são dois despachos, e o segundo é uma thread por
    // sub-amostra. ⚠️ O `setup` é o de omissão, que re-amostra a silhueta em todo quadro.
    assert!(
        setup.antialias && depois_do_matcap.iter().any(|e| e == "bordas_marcha"),
        "a borda não foi re-amostrada pelo despacho compacto: {depois_do_matcap:?}"
    );
    // CONTROLO: a marcha do G-buffer (a porta das paridades) usa o MESMO kernel magro — a luz do
    // Render traçado saiu em 03/10 e com ela o kernel pesado.
    let _ = t.frame(&f, c.sculpts(), setup, W, H);
    let depois_da_marcha = t.entradas_compiladas();
    assert!(
        depois_da_marcha.iter().all(|e| !e.contains("luz")),
        "CONTROLO: a marcha do G-buffer compilou uma entrada de luz: {depois_da_marcha:?}"
    );
}
