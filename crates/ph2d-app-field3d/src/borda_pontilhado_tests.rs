//! ⭐⭐⭐ **O CONTORNO PONTILHADO — a luz de um pixel de silhueta vem do MESMO pixel que o ponto**
//! (foto do dono de 2026-09-25, cena `=28`, medido e curado em 2026-09-29).
//!
//! # O defeito
//!
//! O passe da borda re-amostra cada pixel de silhueta em quatro sub-amostras. Num pixel cujo
//! CENTRO falha a peça, o ponto já era emprestado do primeiro vizinho de cruz que acerta
//! (`pixel_da_borda`), mas a oclusão do céu, a sombra da lâmpada e o ricochete continuavam a ser
//! lidos no índice do CENTRO — um pixel de FUNDO, com céu aberto e lâmpada sem sombra. As
//! sub-amostras que acertam acendiam-se com essa luz, e cada degrau da silhueta de um tubo escuro
//! pintava um ponto claro isolado: a linha tracejada da foto. Medido no dispositivo pela sonda
//! `diag_o_contorno_pontilhado`: `597` pontos claros isolados antes, `94` depois, `25` sem a borda.
//!
//! # ⛔ Porque a régua NÃO é a da foto
//!
//! A 1.ª redacção deste gate contava os pontos claros isolados no quadro de CPU, com e sem a
//! borda — e a PROVA DE MUTAÇÃO derrubou-a: com a cura revertida ela lia **o mesmo `11` contra
//! `21`**, ao bit. No caminho de CPU desta cena a luz de um pixel de fundo e a do vizinho que
//! acerta quase não diferem (a oclusão do céu não é assada aqui), logo *a fixtura não contém o
//! fenómeno* e o gate ficava verde a afirmar nada.
//!
//! ⇒ a régua mede a LEI e não o sintoma: **envenenam-se os canais de luz dos pixels de FUNDO** e
//! exige-se que nenhuma sub-amostra que ACERTA os veja. Para isolar a lei, as sub-amostras de cada
//! pixel de borda cujo centro falha passam todas a «acertar» (com a normal do vizinho) — assim o
//! caminho do fundo, que lê os pixels de fundo DE PROPÓSITO (o chão e a luz que ele devolve), não
//! entra na conta. Com a cura, o veneno não move um byte; com ela revertida, move a borda inteira.

use ph2d_field_render::{EdgePixel, Gbuffer, Shadows};

/// A cena do report.
const CENA: u32 = 28;
const W: u32 = 480;
const H: u32 = 270;

/// O veneno: luz que nenhum pixel de peça tem — céu fechado, lâmpada tapada e um ricochete
/// magenta que nenhuma cena produz.
const VENENO_CEU: f32 = 0.0;
const VENENO_LAMPADA: f32 = 0.0;
const VENENO_RICOCHETE: [f32; 3] = [4.0, 0.0, 4.0];

/// O vizinho de cruz que acerta, na ordem do produto (esquerda, direita, cima, baixo).
fn vizinho_que_acerta(g: &Gbuffer, i: usize) -> Option<usize> {
    let (w, h) = (g.width as usize, g.height as usize);
    let (x, y) = (i % w, i / w);
    [
        (x > 0).then(|| i - 1),
        (x + 1 < w).then(|| i + 1),
        (y > 0).then(|| i - w),
        (y + 1 < h).then(|| i + w),
    ]
    .into_iter()
    .flatten()
    .find(|&j| g.hit[j])
}

/// Os canais de luz do passe, com os pixels de FUNDO envenenados (ou não).
fn luz(g: &Gbuffer, sh: &Shadows, lampadas: usize, envenena: bool) -> Shadows {
    let n = g.hit.len();
    // ⚠️ Um CLONE e não um `default()`: o chão e o campo dele viajam no mesmo valor.
    let mut out = sh.clone();
    let fundo = |i: usize| envenena && !g.hit[i];
    out.set_ambient(
        (0..n)
            .map(|i| {
                if fundo(i) {
                    VENENO_CEU
                } else {
                    sh.ambient_at(i)
                }
            })
            .collect(),
    );
    for l in 0..lampadas {
        out.set_lamp(
            l,
            (0..n)
                .map(|i| {
                    if fundo(i) {
                        VENENO_LAMPADA
                    } else {
                        sh.at(l, i)
                    }
                })
                .collect(),
        );
    }
    out.set_bounce(
        (0..n)
            .map(|i| {
                if fundo(i) {
                    VENENO_RICOCHETE
                } else {
                    sh.bounce_at(i)
                }
            })
            .collect(),
    );
    out
}

/// ⭐⭐⭐ **Uma sub-amostra que acerta a peça nunca lê a luz de um pixel de FUNDO.**
#[test]
fn a_luz_de_uma_subamostra_que_acerta_nunca_vem_do_fundo() {
    let cam = ph2d_field_render::Orbit::default();
    let doc = crate::smoke::scene(CENA);
    let reg = crate::smoke::sampled_registry();
    let materiais = [ph2d_material::OpenPbr::default().prepare()];
    let surfaces = ph2d_field_render::Surfaces {
        all: &materiais,
        owners: None,
    };
    let lampada = [crate::gpu_frame::tests_lampada(&cam)];
    let mundos: Vec<[f32; 3]> = lampada.iter().map(|l| l.world).collect();
    let chao = ph2d_field_render::lowest_point(&doc, &reg)
        .map(|height| ph2d_field_render::Ground { height });
    let mut g = ph2d_field_render::trace(&doc, &reg, &cam, W, H);
    let sh = ph2d_field_render::shadow_pass_on(&doc, &reg, &cam, &g, &mundos, chao);

    // O isolamento: toda sub-amostra de uma borda cujo centro falha passa a acertar, com a normal
    // do vizinho de quem o produto empresta o ponto.
    let mut populacao = Vec::new();
    let mut bordas = std::mem::take(&mut g.edges);
    for e in &mut bordas {
        let i = e.pixel as usize;
        if g.hit[i] {
            continue;
        }
        let Some(j) = vizinho_que_acerta(&g, i) else {
            continue;
        };
        *e = EdgePixel {
            pixel: e.pixel,
            hit: [true; 4],
            normal: [g.normal[j]; 4],
        };
        populacao.push(i);
    }
    g.edges = bordas;
    // A população: sem bordas de centro FALHADO o gate não afirmaria nada.
    assert!(
        populacao.len() > 500,
        "só {} bordas com o centro fora da peça — a fixtura não as tem",
        populacao.len()
    );

    let sem_ecra: [ph2d_field_render::Lamp; 0] = [];
    let pinta = |s: &Shadows| {
        ph2d_field_render::shade_render(
            &g,
            &cam,
            &surfaces,
            &ph2d_field_render::Lighting {
                lamps: &sem_ecra,
                points: &lampada,
                sky: &crate::render_light::StudioSky,
                shadows: Some(s),
            },
            &ph2d_field_render::Presentation::of(ph2d_view_transform::Look::default()),
            [40, 40, 40, 255],
        )
    };
    let limpo = pinta(&luz(&g, &sh, lampada.len(), false));
    let envenenado = pinta(&luz(&g, &sh, lampada.len(), true));

    // ⚠️ CONTROLO: o veneno tem de ser VISÍVEL onde é lido de propósito — no fundo, que o chão
    // pinta com a sombra dos pixels de fundo. Sem isto o veneno podia não chegar a pixel nenhum e
    // a igualdade abaixo seria vácua.
    let fundo_mudou = (0..g.hit.len())
        .filter(|&i| !g.hit[i] && limpo[4 * i..4 * i + 3] != envenenado[4 * i..4 * i + 3])
        .count();
    assert!(
        fundo_mudou > 1_000,
        "o veneno só moveu {fundo_mudou} pixels de fundo — ele não chega ao quadro, e o gate não mede nada"
    );

    let mut pior = 0u8;
    let mut mexidos = 0usize;
    for &i in &populacao {
        let d = (0..3)
            .map(|c| limpo[4 * i + c].abs_diff(envenenado[4 * i + c]))
            .max()
            .unwrap_or(0);
        pior = pior.max(d);
        mexidos += usize::from(d > 0);
    }
    println!(
        "cena {CENA} {W}×{H} · bordas de centro falhado {} · fundo movido pelo veneno {fundo_mudou} · \
         bordas movidas {mexidos} · pior byte {pior}",
        populacao.len()
    );
    assert_eq!(
        mexidos, 0,
        "{mexidos} pixels de borda mudaram com o fundo envenenado (pior byte {pior}) — a luz de uma \
         sub-amostra que ACERTA está a ser lida no pixel de FUNDO"
    );
}
