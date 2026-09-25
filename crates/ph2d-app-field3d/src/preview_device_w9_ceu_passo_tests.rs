//! ⭐⭐⭐⭐ **OS GATES DA OCLUSÃO A PASSO NO QUADRO DE MOVIMENTO** — a cura da *«queda de resolução
//! do modelo»* ao girar (report do dono de 2026-09-24; `ph2d_field_gpu::trace::MarchSetup::ceu_passo`).
//!
//! Medido (`diag_o_ceu_a_passo`, `1920×1080`, `99 %` ociosa): a oclusão era `~75` dos `109 ms` do
//! quadro de movimento do nó de toro, e é ela que obriga o laço do movimento a encolher a tela. A
//! passo `2` o nó vai a `55,5 ms` (sem oclusão nenhuma: `34`–`37`), o vaso de `18,1` a `11,7`.
//!
//! ⛔⛔ **A economia só existe porque os representantes correm COMPACTOS** (o `ceu_meia`, uma thread
//! por célula): marchá-los no `luz_so` só nos pixels que representam não poupava nada, porque o warp
//! espera sempre pelos `48` cones de quem os marcha (nó `107 → 90 ms`).

use super::super::*;

/// Uma cena do smoke com chão — o regime do report.
fn quadro(t: &crate::gpu_frame::SharedTracer, cena: u32, passo: u32, assente: bool) -> Vec<u8> {
    let doc = crate::smoke::scene(cena);
    let reg = crate::smoke::sampled_registry();
    let cam = ph2d_field_render::Orbit::default();
    let materiais = [ph2d_material::OpenPbr::default().prepare()];
    let surfaces = ph2d_field_render::Surfaces {
        all: &materiais,
        owners: None,
    };
    let pres = ph2d_field_render::Presentation::of(ph2d_view_transform::Look::default());
    let luz = [crate::gpu_frame::tests_lampada(&cam)];
    let chao = ph2d_field_render::lowest_point(&doc, &reg)
        .map(|height| ph2d_field_render::Ground { height });
    crate::gpu_frame::paint_com(
        t,
        &doc,
        &reg,
        &cam,
        &luz,
        &surfaces,
        &pres,
        [40, 40, 40, 255],
        chao,
        LW,
        LH,
        assente,
        crate::gpu_frame::Sonda {
            ceu_passo: passo,
            ..crate::gpu_frame::Sonda::default()
        },
    )
    .expect("o pintor")
    .rgba
}

/// Quantos canais diferem mais de `8` níveis, e o pior.
fn diferenca(a: &[u8], b: &[u8]) -> (usize, usize, u8) {
    let (mut algum, mut acima, mut pior) = (0usize, 0usize, 0u8);
    for (x, y) in a.iter().zip(b) {
        let d = x.abs_diff(*y);
        pior = pior.max(d);
        if d > 0 {
            algum += 1;
        }
        if d > 8 {
            acima += 1;
        }
    }
    (algum, acima, pior)
}

/// ⭐⭐⭐⭐ **A OCLUSÃO RECONSTRUÍDA NÃO DESENHA HALO** — o quadro de movimento a passo `2` contra o
/// de passo `1`, em DUAS cenas, porque cada cerca da reconstrução só morde numa delas:
///
/// | cena | medido `> 8` · pior | sem a cerca da NORMAL | sem a do PLANO | sem o recurso |
/// |---|---:|---:|---:|---:|
/// | `=28` nó (tubos que se cruzam) | `31` · `14` | `583` · `34` | `70` · `14` | `253` · `96` |
/// | `=29` rosca (filetes paralelos a alturas diferentes) | `54` · `14` | — | `584` · `36` | — |
///
/// ⚠️ **O plano só se vê na rosca:** no nó a normal já separa as superfícies, e ali tirar o plano
/// não se nota — *uma fixtura só de tubos não contém o degrau entre duas faces paralelas*.
///
/// ⛔ **O CONTROLO vem primeiro:** as duas imagens têm de DIFERIR — se o passo não chegasse ao
/// shader (o `u32` no enchimento do `vec3`), as duas seriam o mesmo quadro e a barra passaria sobre
/// nada.
#[test]
#[ignore = "precisa de GPU"]
fn a_oclusao_a_passo_nao_desenha_halo() {
    let Some(t) = crate::gpu_frame::shared() else {
        println!("sem adaptador — saltado");
        return;
    };
    for cena in [28u32, 29] {
        let um = quadro(t, cena, 1, false);
        let dois = quadro(t, cena, 2, false);
        let (algum, acima, pior) = diferenca(&um, &dois);
        println!(
            "cena {cena}: passo 2 contra 1: {algum} canais mexidos · {acima} acima de 8 · pior {pior}"
        );
        assert!(
            algum > 1_000,
            "CONTROLO: na cena {cena} o passo 2 só mexe {algum} canais — ele não chegou ao shader, e \
             a barra abaixo não afirmaria nada"
        );
        assert!(
            acima <= ACIMA_MAX && pior <= PIOR_MAX,
            "cena {cena}: a oclusão reconstruída desenha halo: {acima} canais acima de 8 níveis \
             (tecto {ACIMA_MAX}), pior {pior} (tecto {PIOR_MAX})"
        );
    }
}

/// O tecto dos canais acima de `8` níveis — no vale entre o medido (`≤ 54`) e as mutações (`≥ 253`).
const ACIMA_MAX: usize = 150;
/// O tecto do pior canal — no vale entre o medido (`14`) e as mutações (`≥ 34`).
const PIOR_MAX: u8 = 24;

/// ⭐⭐⭐⭐ **O QUADRO ASSENTE IGNORA O PASSO** — ele é o que a paridade com a CPU mede, e a lei W73
/// (*grosso a mexer, nítido ao assentar*) põe a oclusão inteira nele. Byte a byte.
#[test]
#[ignore = "precisa de GPU"]
fn o_quadro_assente_ignora_o_passo() {
    let Some(t) = crate::gpu_frame::shared() else {
        println!("sem adaptador — saltado");
        return;
    };
    let (algum, _, pior) = diferenca(&quadro(t, 28, 1, true), &quadro(t, 28, 2, true));
    assert_eq!(
        algum, 0,
        "o quadro assente mudou com o passo da oclusão ({algum} canais, pior {pior})"
    );
}

/// ⭐⭐⭐⭐ **O QUADRO DE MOVIMENTO DO PRODUTO USA O PASSO** — a porta de fábrica (`paint`, com a
/// `Sonda::default()`) dá o quadro a passo [`crate::preview::PASSO_DO_CEU_A_MEXER`] e NÃO o de passo
/// `1`. ⛔ Sem esta metade a cura podia existir só na sonda.
#[test]
#[ignore = "precisa de GPU"]
fn o_quadro_de_movimento_do_produto_usa_o_passo() {
    let Some(t) = crate::gpu_frame::shared() else {
        println!("sem adaptador — saltado");
        return;
    };
    if std::env::var("PH2D_FIELD_CEU_PASSO").is_ok() {
        println!("PH2D_FIELD_CEU_PASSO definido — a fábrica não é a que corre; saltado");
        return;
    }
    const { assert!(crate::preview::PASSO_DO_CEU_A_MEXER > 1) };
    let doc = crate::smoke::scene(28);
    let reg = crate::smoke::sampled_registry();
    let cam = ph2d_field_render::Orbit::default();
    let materiais = [ph2d_material::OpenPbr::default().prepare()];
    let surfaces = ph2d_field_render::Surfaces {
        all: &materiais,
        owners: None,
    };
    let pres = ph2d_field_render::Presentation::of(ph2d_view_transform::Look::default());
    let luz = [crate::gpu_frame::tests_lampada(&cam)];
    let chao = ph2d_field_render::lowest_point(&doc, &reg)
        .map(|height| ph2d_field_render::Ground { height });
    let produto = crate::gpu_frame::paint(
        t,
        &doc,
        &reg,
        &cam,
        &luz,
        &surfaces,
        &pres,
        [40, 40, 40, 255],
        chao,
        LW,
        LH,
        false,
    )
    .expect("o pintor")
    .rgba;
    let (com_fabrica, ..) = diferenca(
        &produto,
        &quadro(t, 28, crate::preview::PASSO_DO_CEU_A_MEXER, false),
    );
    let (com_um, ..) = diferenca(&produto, &quadro(t, 28, 1, false));
    assert_eq!(
        com_fabrica, 0,
        "o quadro de movimento do produto não é o do passo de fábrica ({com_fabrica} canais)"
    );
    assert!(
        com_um > 0,
        "o quadro de movimento do produto é o de passo 1 — a oclusão a passo não está ligada"
    );
}
