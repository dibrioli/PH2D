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

use super::paint_parity_tests::{FUNDO, H, W, fixtura};

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

    // ⚠️ **UMA lâmpada para o `march`**, que a exige (ver o `exige_luz` do
    // [`crate::gpu_frame::pedido`]) — e o matcap **não a lê**: ela existe só para o G-buffer de
    // referência atravessar o barramento. *É por isso que o lado do dispositivo passa `&[]`.*
    let mundos = [[2.0f32, 3.0, 4.0]];
    let (g, _sh) = crate::gpu_frame::march(t, &doc, &reg, &cam, &mundos, None, W, H, true)
        .expect("a marcha do dispositivo");

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

/// ⭐⭐⭐⭐ **O MATCAP MARCHA NO KERNEL MAGRO — e nunca no do Render.**
///
/// Medido (`docs/Render3d/03` §W9, «o kernel que hospeda a marcha»): o `centro_e_luz` traz o chão,
/// as lâmpadas e o ricochete, que o matcap nunca lê, e hospedar a marcha nele custava `4×`–`9×` o
/// quadro (o nó, `86` contra `10 ms` a `1920×1080`). ⚠️ **Nenhuma paridade o vê** — as duas
/// entradas dão o MESMO centro ao bit —, logo o gate afirma a ESCOLHA: um traçador novo que só
/// pintou matcap compilou a entrada magra e não a pesada. ⭐ E o CONTROLO: a marcha do G-buffer
/// (o modo Render) compila a pesada no mesmo traçador, senão o nome procurado podia simplesmente
/// não existir e a segunda metade ficaria verde por vácuo.
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
    };
    let (c, f, setup) = super::pedido(
        &doc,
        &reg,
        &cam,
        &[],
        None,
        ph2d_field_gpu::trace::MAX_LAMPS,
        super::Sonda::default(),
        W,
        H,
        None,
        false,
    )
    .expect("o pedido");
    let _ = t.matcap_frame(&f, c.sculpts(), setup, &mc, W, H);
    let depois_do_matcap = t.entradas_compiladas();
    assert!(
        depois_do_matcap.iter().any(|e| e == "centro_so"),
        "o matcap não compilou a entrada magra: {depois_do_matcap:?}"
    );
    assert!(
        !depois_do_matcap.iter().any(|e| e == "centro_e_luz"),
        "o matcap compilou a entrada do RENDER — a marcha voltou ao kernel pesado: {depois_do_matcap:?}"
    );
    // CONTROLO: o G-buffer (o modo Render) usa a pesada, no mesmo traçador.
    let _ = t.frame(&f, c.sculpts(), setup, W, H);
    assert!(
        t.entradas_compiladas().iter().any(|e| e == "centro_e_luz"),
        "CONTROLO: a marcha do G-buffer tem de compilar o `centro_e_luz`"
    );
}
