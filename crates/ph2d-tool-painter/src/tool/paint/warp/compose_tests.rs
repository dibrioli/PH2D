//! **A lei do Reshape** — como a lista de dabs é dobrada, e a prova de que o `apply.rs` a segue.
//!
//! ⚠️ Os dois primeiros gates nasceram VERMELHOS contra a soma que shipava e ficaram verdes com a
//! travessia do [ADR-0156] (2026-08-08). Eles julgam a **LEI** (`compose_at`); quem julga o **PRODUTO** é
//! `the_product_composes_the_dab_list_instead_of_summing_it`, e a distinção é a lição inteira do W0:
//! *uma suíte pode ficar verde sobre a lei e cega ao arquivo que o artista usa*.
//!
//! ## O oráculo é GEOMÉTRICO, e isso é o ponto
//!
//! O primeiro gate não afirma um número nosso: afirma que **uma rotação é uma rotação**. Girar um ponto
//! de raio `r` em torno do centro não pode deslocá-lo mais que **`2r`** — o diâmetro, alcançado a 180°,
//! e nenhuma composição de rotações passa disso porque rotações formam um GRUPO. Um oráculo que
//! comparasse o mapa novo com o mapa antigo seria *razão entre dois doentes*; este não pode ser satisfeito
//! por um bug que ande junto nos dois lados.
//!
//! ## O que o produto FAZIA, reproduzido termo a termo
//!
//! Até 2026-08-08 o [`super::apply`] avaliava `field.at([dx, dy])` no pixel de DESTINO — fixo — e somava:
//! `d += a`. Logo o campo dele era `Σ_k f_k(p)`, e é exatamente isso que [`summed_at`] computa. Não é uma
//! paráfrase: é a mesma aritmética, na mesma ordem, com os mesmos dabs — e é por isso que ela fica, como
//! **rota de ablação** que dá sentido às mutações e como o número que as mensagens de falha imprimem.
//!
//! Somar as cordas `R(θ)v − v` N vezes dá `N·(corda)` — uma **reta tangente**, que cresce sem limite.
//! Compor dá `R(Nθ)`, que é limitado. Somar É composição exata **para translação e para mais nada**, e é
//! por isso que só o **Push** parecia bom.
//!
//! ⚠️ **A mutação que prova os gates da lei:** troque [`compose_at`] por [`summed_at`]. A do gate do
//! PRODUTO é outra e mora no `apply.rs`: fazer a leitura no destino (`win.sample(dx, dy)`) em vez do ponto
//! retro-traçado — que É a soma. Todas sangram, e as mensagens imprimem o número ao lado do teto
//! geométrico.
//!
//! [ADR-0156]: ../../../../../../docs/architecture/decisions/0156-liquify-is-an-authored-dab-list-cooked-on-the-device-never-a-stored-dense-field.md

use super::apply::bilinear_clamped;
use super::field::{DabField, DeformMode, compose_at};

const SIDE: u32 = 128;
const CENTRE: [f32; 2] = [64.0, 64.0];
/// Raio do PINCEL. O ponto sondado fica bem dentro dele, onde o falloff é forte — é ali que a divergência
/// aparece, e é ali que a arte do artista está.
const BRUSH_R: f32 = 100.0;
/// O raio SONDADO: a distância do centro em que o teto `2r` é afirmado.
const PROBE_R: f32 = 30.0;

/// A lista de dabs de um Twist mantido no lugar — o gesto do report do Enio (*"Twist nas imagens: veja
/// linhas sumindo"*), que é como um artista de fato usa a ferramenta: ele insiste.
fn twist_dabs(n: usize) -> Vec<DabField> {
    (0..n)
        .map(|k| {
            DabField::new(
                DeformMode::Twist,
                CENTRE,
                BRUSH_R,
                [0.0, 0.0],
                [0.0, 0.0],
                1.0, // strength no máximo — o regime que o artista alcança com o slider
                0.8, // pressão default
                0.0,
                0.0,
                k as u64 + 1,
            )
        })
        .collect()
}

/// **A lei que SHIPAVA**, isolada: `Σ_k f_k(p)`, cada dab avaliado no pixel de destino fixo. Não é uma
/// segunda porta — é a rota de ablação que dá sentido às mutações, e ela existe só sob `cfg(test)`.
fn summed_at(dabs: &[DabField], p: [f32; 2]) -> [f32; 2] {
    let mut d = [0.0_f32, 0.0];
    for f in dabs {
        let v = f.at(p);
        d[0] += v[0];
        d[1] += v[1];
    }
    d
}

fn len(v: [f32; 2]) -> f32 {
    (v[0] * v[0] + v[1] * v[1]).sqrt()
}

/// **Gate 1 — um Twist é uma ROTAÇÃO, não um cisalhamento divergente.**
///
/// Nasceu VERMELHO contra a soma: a mensagem imprime os dois números lado a lado, e o dela passa do teto
/// por múltiplos.
#[test]
fn a_twist_is_a_rotation_not_a_runaway_shear() {
    let ceiling = 2.0 * PROBE_R;
    let probe = [CENTRE[0] + PROBE_R, CENTRE[1]];
    for n in [1usize, 5, 20, 60, 200] {
        let dabs = twist_dabs(n);
        let composed = len(compose_at(&dabs, probe));
        assert!(
            composed <= ceiling,
            "com {n} dabs a composição deslocou {composed:.2} px num raio de {PROBE_R} — uma rotação \
             não passa de {ceiling:.0} px (o diâmetro, a 180°). A soma que shipava dava {:.2} px.",
            len(summed_at(&dabs, probe))
        );
    }
}

/// **E o teto não é alcançado por acidente de escala.** Um gate que só dissesse `<= 2r` ficaria verde com
/// um campo IDENTICAMENTE ZERO — a ferramenta desligada. Este exige que ela ainda DEFORME.
#[test]
fn the_bounded_twist_still_turns_the_picture() {
    let probe = [CENTRE[0] + PROBE_R, CENTRE[1]];
    let d = compose_at(&twist_dabs(60), probe);
    assert!(
        len(d) > 1.0,
        "60 dabs de Twist têm de girar algo visível; deslocou {:.3} px",
        len(d)
    );
}

/// **A tabela do [ADR-0156], saindo da MESMA fixture dos gates.**
///
/// ⚠️ Ela existe porque o ADR nasceu citando números de uma sonda exploratória com fixture PRÓPRIA
/// (158,55 px · 3,4%), e um fato medido duas vezes com duas fixtures é um fato que ninguém consegue
/// reproduzir depois. Aqui o gate e a tabela partilham `twist_dabs`, então **concordam por construção**.
///
/// Rodar: `cargo test -p ph2d-tool-painter --lib warp::compose -- --ignored --nocapture`
#[test]
#[ignore = "probe: measures, does not assert"]
fn measure_the_divergence_of_the_sum() {
    let probe = [CENTRE[0] + PROBE_R, CENTRE[1]];
    let src = line_canvas();
    let before = ink(&src) as f64;
    println!(
        "\n=== o preço da SOMA (pincel r={BRUSH_R}, sonda r={PROBE_R}, teto geométrico {:.0} px) ===",
        2.0 * PROBE_R
    );
    println!(
        "{:>6} {:>14} {:>14} {:>12} {:>12}",
        "dabs", "|D| soma", "|D| composto", "tinta soma", "tinta comp."
    );
    for n in [1usize, 5, 20, 60, 200] {
        let dabs = twist_dabs(n);
        let s = ink(&warp_with(&src, |p| summed_at(&dabs, p))) as f64 / before * 100.0;
        let c = ink(&warp_with(&src, |p| compose_at(&dabs, p))) as f64 / before * 100.0;
        println!(
            "{n:>6} {:>14.2} {:>14.2} {:>11.1}% {:>11.1}%",
            len(summed_at(&dabs, probe)),
            len(compose_at(&dabs, probe)),
            s,
            c
        );
    }
}

/// Uma tela com sessão de warp aberta — a porta do PRODUTO, não um banco de teste.
fn tool_with_session(side: u32) -> crate::tool::PainterTool {
    use ph2d_editor_core::tool::RasterEditTool as _;
    let mut t = crate::tool::PainterTool::default();
    t.set_source(vec![255u8; (side * side * 4) as usize], side, side);
    assert!(t.ensure_warp_session(), "a sessão de warp tem de abrir");
    t
}

/// **O GATE DA TRAVESSIA.** Os dois gates acima julgam a LEI (`compose_at`); este julga o `apply.rs`, que
/// é onde a soma morava — e a distinção é a lição inteira do W0: *uma suíte pode ficar verde sobre a lei e
/// cega ao produto*.
///
/// ⚠️ **A mutação que tem de sangrar** é restaurar `disp[gi] = disp_old[gi] + v` (a soma que shipava).
#[test]
fn the_product_composes_the_dab_list_instead_of_summing_it() {
    const SIDE: u32 = 128;
    let ceiling = 2.0 * PROBE_R;
    let mut t = tool_with_session(SIDE);
    for f in &twist_dabs(60) {
        t.warp_apply_dab(f, CENTRE, BRUSH_R);
    }
    let gi = (CENTRE[1] as u32 * SIDE + (CENTRE[0] + PROBE_R) as u32) as usize;
    let d = t.paint.warp.disp[gi];
    assert!(
        len(d) <= ceiling,
        "60 dabs de Twist deslocaram {:.2} px no MAPA DO PRODUTO, num raio de {PROBE_R} — uma rotação \
         não passa de {ceiling:.0} px. A soma que shipava dava {:.2}.",
        len(d),
        len(summed_at(&twist_dabs(60), [CENTRE[0] + PROBE_R, CENTRE[1]]))
    );
}

/// **O cache INCREMENTAL deriva da lei exata, e este é o número — o achado que torna a lista de dabs
/// obrigatória em vez de arquitetura de gosto.**
///
/// O produto avança o mapa incrementalmente (`D_k(p) = v_k(p) + D_{k−1}(p − v_k(p))`, lendo o mapa antigo
/// por **bilinear**); [`compose_at`] desenrola a recursão avaliando cada `v_j` no ponto retro-traçado
/// EXATO. São a mesma lei por dois caminhos — e o que os separa é a reamostragem do MAPA, que num campo de
/// ROTAÇÃO (fortemente curvo) não é benigna: o erro de um passo entra na POSIÇÃO de leitura do seguinte, e
/// amplifica.
///
/// **Medido (256², Twist parado, r=100, pressão 0,8):**
///
/// | N dabs | pior deriva | \|D\| no probe |
/// |---:|---:|---:|
/// | 1 | **0,0000 px** | 3,47 |
/// | 60 | 1,8709 | 19,28 |
/// | 200 | **41,4538** | 50,61 |
///
/// ⚠️ **A 200 dabs a deriva tem a ORDEM do próprio sinal** — e 200 dabs é um *hold* normal, o gesto que o
/// Enio reportou (*"ele insiste"*). Ou seja: a travessia mata o cisalhamento divergente (o `2r` do gate
/// acima passa no produto), **e o cache incremental não substitui o re-cook exato**. É por isso que o
/// [ADR-0156] põe a LISTA como estado e o campo denso como cache: não é preferência de arquitetura, é a
/// única forma que não acumula. E o re-cook exato só é pagável no **device** (0,008 ns por nó·dab,
/// `cook_gpu`), o que é o §0 outra vez — *o caminho lento não define o produto*.
///
/// ⚠️ **N=1 é EXATO (0,0000), e essa metade é o teste da FIAÇÃO**: qualquer erro de janela, de sinal ou de
/// ordem apareceria ali. O que sobra depois dela é deriva de reamostragem, não bug.
///
/// [ADR-0156]: ../../../../../../docs/architecture/decisions/0156-liquify-is-an-authored-dab-list-cooked-on-the-device-never-a-stored-dense-field.md
#[test]
fn the_incremental_cache_drifts_from_the_exact_walk_and_this_is_the_number() {
    const SIDE: u32 = 256;
    const C: [f32; 2] = [128.0, 128.0];
    const R: f32 = 100.0;
    /// Um dab tem de sair EXATO — o mapa antigo é zero, então a leitura cai em coordenada inteira, onde a
    /// bilinear devolve o texel. Qualquer outro número aqui é bug de fiação, não deriva.
    const WIRING_TOL: f32 = 0.0;
    /// 60 dabs: medido 1,8709 px. Guarda de regressão com ~1,6× de folga.
    const DRIFT_60: f32 = 3.0;
    /// 200 dabs: medido 41,45 px. ⚠️ Este é um piso, não um teto — o gate afirma que o PROBLEMA EXISTE,
    /// para que ninguém apague a nota sem ter mexido no mecanismo (o padrão do
    /// `the_documented_hardening_is_still_there_and_this_is_its_number` da máscara).
    const DRIFT_200_FLOOR: f32 = 10.0;

    let worst_for = |n: u16| -> (f32, (u32, u32), usize) {
        let dabs: Vec<DabField> = (0..n)
            .map(|k| {
                DabField::new(
                    DeformMode::Twist,
                    C,
                    R,
                    [0.0, 0.0],
                    [0.0, 0.0],
                    1.0,
                    0.8,
                    0.0,
                    0.0,
                    u64::from(k) + 1,
                )
            })
            .collect();
        let mut t = tool_with_session(SIDE);
        for f in &dabs {
            t.warp_apply_dab(f, C, R);
        }
        let (mut worst, mut at, mut moved) = (0.0_f32, (0u32, 0u32), 0usize);
        for y in 0..SIDE {
            for x in 0..SIDE {
                let exact = compose_at(&dabs, [x as f32, y as f32]);
                let cached = t.paint.warp.disp[(y * SIDE + x) as usize];
                if len(exact) > 0.5 {
                    moved += 1;
                }
                let e = (exact[0] - cached[0])
                    .abs()
                    .max((exact[1] - cached[1]).abs());
                if e > worst {
                    worst = e;
                    at = (x, y);
                }
            }
        }
        (worst, at, moved)
    };

    let (w1, _, m1) = worst_for(1);
    let (w60, at60, m60) = worst_for(60);
    let (w200, _, _) = worst_for(200);
    println!("deriva do cache incremental: N=1 {w1:.4} px · N=60 {w60:.4} · N=200 {w200:.4}");
    assert!(
        m1 > 2000 && m60 > 2000,
        "a fixture mal deforma ({m1}/{m60})"
    );
    assert!(
        w1 <= WIRING_TOL,
        "UM dab tem de sair exato e saiu {w1:.6} px — isto é fiação, não deriva"
    );
    assert!(
        w60 <= DRIFT_60,
        "a deriva a 60 dabs subiu para {w60:.4} px em {at60:?} (guarda {DRIFT_60})"
    );
    assert!(
        w200 >= DRIFT_200_FLOOR,
        "a deriva a 200 dabs caiu para {w200:.4} px — se o mecanismo mudou, MEÇA e reescreva a tabela \
         deste doc-comment em vez de baixar o piso"
    );
}

/// **UM dab avança o mapa por exatamente UMA composição** — `disp_new(p) = v(p) + disp_old(p − v(p))`,
/// julgado no `apply.rs` contra o mapa que o produto de fato tinha.
///
/// ⚠️ **E este gate registra um NÃO-achado que vale tanto quanto ele:** eu o escrevi para tornar a margem
/// da janela (`reach`) load-bearing, e **a mutação `reach → 0` sobrevive aos dois gates** — ao do Twist
/// parado *e* a este, com um Push de 25 px. O mecanismo é geométrico: **todo modo multiplica por `f`, que
/// vale 1 no centro e 0 na borda**, então `|v|` é máximo onde o dab está mais fundo dentro do próprio bbox
/// e some exatamente onde o bbox acaba — o retro-traço é sempre para DENTRO, e o `+2` fixo da bilinear
/// basta. A margem fica como defesa em camada (o precedente do ADR-0145: *documentada em vez de gateada,
/// porque no regime que shipa não é observável*), e a invariante de que ela depende está dita: um modo
/// futuro cujo `v` NÃO desapareça na borda torna-a necessária na hora.
#[test]
fn one_dab_advances_the_map_by_exactly_one_composition() {
    const SIDE: u32 = 256;
    const C: [f32; 2] = [128.0, 128.0];
    const R: f32 = 100.0;
    const TOL_PX: f32 = 0.05;
    let mut dabs: Vec<DabField> = (0..30u16)
        .map(|k| {
            DabField::new(
                DeformMode::Twist,
                C,
                R,
                [0.0, 0.0],
                [0.0, 0.0],
                1.0,
                0.8,
                0.0,
                0.0,
                u64::from(k) + 1,
            )
        })
        .collect();
    dabs.push(DabField::new(
        DeformMode::Push,
        C,
        R,
        [25.0, 0.0], // o retro-traço que exige a margem
        [0.0, 0.0],
        0.0,
        1.0,
        0.0,
        0.0,
        99,
    ));
    let mut t = tool_with_session(SIDE);
    let last = dabs.len() - 1;
    for f in &dabs[..last] {
        t.warp_apply_dab(f, C, R);
    }
    // ⚠️ O oráculo é o mapa que o PRODUTO tinha, não o exato: a deriva das 30 primeiras é assunto do gate
    // irmão, e misturá-la aqui mediria duas coisas de uma vez. O que este gate julga é UM passo.
    let map30: Vec<[f32; 2]> = t.paint.warp.disp.as_ref().clone();
    t.warp_apply_dab(&dabs[last], C, R);

    // A leitura de REFERÊNCIA é a mesma função com uma janela que não pode clampar (a tela inteira) —
    // reescrever a bilinear aqui seria a segunda resposta que o `MapWindow` existe para não haver.
    let mut ref_buf: Vec<[f32; 2]> = Vec::new();
    let full = ph2d_painter_brush::MapWindow::snapshot(
        &mut ref_buf,
        &map30,
        SIDE,
        SIDE,
        (0, 0, SIDE, SIDE),
        0.0,
    );
    let mut worst = 0.0_f32;
    let mut at = (0u32, 0u32);
    for y in 0..SIDE {
        for x in 0..SIDE {
            let p = [x as f32, y as f32];
            let v = dabs[last].at(p);
            let before = full.sample(p[0] - v[0], p[1] - v[1]);
            let want = [v[0] + before[0], v[1] + before[1]];
            let got = t.paint.warp.disp[(y * SIDE + x) as usize];
            let e = (want[0] - got[0]).abs().max((want[1] - got[1]).abs());
            if e > worst {
                worst = e;
                at = (x, y);
            }
        }
    }
    assert!(
        worst <= TOL_PX,
        "o último dab leu o mapa {worst:.4} px fora do lugar em {at:?} — a janela não cobriu o retro-traço"
    );
}

/// Uma tela branca com uma linha preta HORIZONTAL de 3 px pelo meio — a figura da foto do Enio.
fn line_canvas() -> Vec<u8> {
    let mut px = vec![255u8; (SIDE * SIDE) as usize * 4];
    for y in 63..66 {
        for x in 0..SIDE {
            let b = ((y * SIDE + x) * 4) as usize;
            px[b] = 0;
            px[b + 1] = 0;
            px[b + 2] = 0;
        }
    }
    px
}

/// Quantos texels ainda carregam tinta escura — a régua de *"as linhas somem"*.
fn ink(px: &[u8]) -> usize {
    px.chunks_exact(4).filter(|c| c[0] < 128).count()
}

/// O gather REAL do produto ([`bilinear_clamped`]), dirigido por um campo dado. Uma reamostragem por
/// texel, como o `apply` faz — reescrevê-lo aqui seria a segunda resposta a *"como um warp lê a fonte?"*.
fn warp_with(src: &[u8], field: impl Fn([f32; 2]) -> [f32; 2]) -> Vec<u8> {
    let mut out = vec![0u8; src.len()];
    for y in 0..SIDE {
        for x in 0..SIDE {
            let p = [x as f32, y as f32];
            let d = field(p);
            let c = bilinear_clamped(src, SIDE, SIDE, p[0] - d[0], p[1] - d[1]);
            let b = ((y * SIDE + x) * 4) as usize;
            out[b..b + 4].copy_from_slice(&c);
        }
    }
    out
}

/// **Gate 2 — a linha fina SOBREVIVE ao swirl.**
///
/// Uma rotação move tinta; ela não a apaga. Sob a soma, cada destino busca a fonte longe demais, a linha
/// é esticada até virar fio translúcido e some no branco — os arcos finos da foto. Nasce VERMELHO.
#[test]
fn the_thin_line_survives_a_twist() {
    let src = line_canvas();
    let before = ink(&src);
    let dabs = twist_dabs(60);

    let kept = ink(&warp_with(&src, |p| compose_at(&dabs, p)));
    let pct = kept as f64 / before as f64 * 100.0;
    let summed_pct = ink(&warp_with(&src, |p| summed_at(&dabs, p))) as f64 / before as f64 * 100.0;
    assert!(
        pct >= 80.0,
        "a linha tem de sobreviver ao Twist: restaram {pct:.1}% da tinta ({kept} de {before} texels). \
         A soma que shipava deixava {summed_pct:.1}%."
    );
}
