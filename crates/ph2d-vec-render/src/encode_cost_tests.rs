//! **O custo de re-encode por frame tem um ORÇAMENTO** — arquivo irmão de `lib_tests.rs`.
//!
//! O spike de escala (ADR-0108 §5) mediu 10.000 formas em **0,77 ms/frame** e o kill-criterion do
//! módulo apoia-se nesse número — mas ele vivia só num `println!` de um teste `#[ignore]`, então
//! ninguém o vigiava: em 2026-07-29 a auditoria mediu **1,323 ms** para a mesma cena, uma regressão
//! de **1,7× que atravessou meses sem um único gate piscar**.
//!
//! A causa era estrutural: o [`crate::draw_path`] construía `build_bezpath` **incondicionalmente** e
//! depois `build_fill_bezpath`. Numa forma só-preenchida — a arte comum, e a cena inteira do spike —
//! o primeiro era construído e **jogado fora**; numa forma preenchida-e-traçada sem contorno aberto,
//! os dois eram o MESMO desenho. E `cooked()` (a pilha de Live Path Effects + o arredondamento de
//! quina) corria **uma vez por construção**, logo duas vezes por forma.
//!
//! # Por que estes gates CONTAM em vez de cronometrar
//!
//! ⚠️ **A 1ª versão deles era uma RAZÃO de tempo e não discriminava — a mutação SOBREVIVEU.** A ideia
//! era comparar a mesma geometria só-preenchida contra só-traçada, já que as duas pedem um desenho
//! por forma. Medido com o defeito reinstalado: `fill = 4,862 ms` · `stroke = 5,255 ms` ⇒ razão
//! **0,93×**. Um encode de FILL e um de STROKE **não fazem o mesmo trabalho** no Vello (o traço paga
//! a expansão), e essa diferença é maior que a construção de caminho inteira que o gate queria
//! isolar: *uma razão entre duas coisas que não fazem o mesmo trabalho não isola a terceira*.
//!
//! Então o oráculo passou a ser o **NÚMERO de construções e de cozimentos** — exato, sem relógio,
//! sem flake, e é literalmente a propriedade em questão (o precedente é o gate do ADR-0120 no áudio,
//! que CONTA quantas vezes o caminho rápido dispara). Os contadores são `#[cfg(test)]`: eles não
//! existem no binário do produto.
//!
//! A sonda de tempo FICA (`measure_encode_by_style`), porque medir continua valendo — só não como
//! gate.

use super::*;
use ph2d_vec_scene::{Paint, Rgba8, StrokeSpec, VecScene};

/// Formas na cena de medição. Grande o bastante para o custo por-forma dominar o fixo do
/// `dispatch`, pequeno o bastante para a suíte não pesar.
const N: usize = 4_000;

/// Rodadas cronometradas por cena (a 1ª é descartada — *first touch* das alocações).
const ITERS: u32 = 12;

/// Uma grade de `N` formas fechadas, com o estilo que o teste pedir.
fn grid(fill: bool, stroke: bool) -> VecScene {
    let mut scene = VecScene::demo_grid(N);
    for p in scene.paths_mut() {
        p.fill = fill.then(|| Paint::solid(Rgba8::new(90, 150, 230, 255)));
        p.stroke = stroke.then(|| StrokeSpec::new(Rgba8::new(20, 20, 20, 255), 0.01));
    }
    scene
}

/// Mediana de `ITERS` re-encodes completos da cena, em milissegundos.
fn encode_ms(scene: &VecScene) -> f64 {
    let xf = VecXforms::new();
    let view = VecViewState::default();
    let live = LiveGeometry::new();
    let fx = FxImages::new();
    let mut target = VectorScene::new();
    let mut samples: Vec<f64> = Vec::with_capacity(ITERS as usize);
    for i in 0..=ITERS {
        target.reset();
        let t = std::time::Instant::now();
        dispatch(scene, &view, &xf, &live, &fx, Affine::IDENTITY, &mut target);
        // A 1ª rodada paga o *first touch* dos buffers do target — ela não é o produto em regime.
        if i > 0 {
            samples.push(t.elapsed().as_secs_f64() * 1000.0);
        }
    }
    samples.sort_by(f64::total_cmp);
    samples[samples.len() / 2]
}

/// Os contadores do frame, zerados na leitura. `#[cfg(test)]` — não existem no produto.
mod counters {
    use std::cell::Cell;
    thread_local! {
        static BUILDS: Cell<u32> = const { Cell::new(0) };
        static COOKS: Cell<u32> = const { Cell::new(0) };
    }
    pub(super) fn bump_build() {
        BUILDS.with(|c| c.set(c.get() + 1));
    }
    pub(super) fn bump_cook() {
        COOKS.with(|c| c.set(c.get() + 1));
    }
    /// `(construções, cozimentos)` desde a última leitura.
    pub(super) fn take() -> (u32, u32) {
        (BUILDS.with(|c| c.replace(0)), COOKS.with(|c| c.replace(0)))
    }
}

/// O produto chama estas duas nas ÚNICAS portas que constroem e que cozem.
pub(crate) fn count_build() {
    counters::bump_build();
}
pub(crate) fn count_cook() {
    counters::bump_cook();
}

/// Desenha UMA forma pelo caminho do produto e devolve `(construções, cozimentos)`.
fn draws_of(fill: bool, stroke: bool, open_subpath: bool) -> (u32, u32) {
    let mut scene = VecScene::demo_grid(1);
    {
        let p = &mut scene.paths_mut()[0];
        p.fill = fill.then(|| Paint::solid(Rgba8::new(90, 150, 230, 255)));
        p.stroke = stroke.then(|| StrokeSpec::new(Rgba8::new(20, 20, 20, 255), 0.01));
        if open_subpath {
            // Uma linha de construção: contorno ABERTO, que o preenchimento ignora e o traço leva.
            p.subpaths.push(ph2d_vec_scene::Contour {
                verts: vec![
                    ph2d_vec_scene::VecVertex::corner([0.0, 0.0]),
                    ph2d_vec_scene::VecVertex::corner([0.5, 0.5]),
                ],
                closed: false,
            });
        }
    }
    let mut target = VectorScene::new();
    let _ = counters::take(); // zera o que a montagem tenha contado
    dispatch(
        &scene,
        &VecViewState::default(),
        &VecXforms::new(),
        &LiveGeometry::new(),
        &FxImages::new(),
        Affine::IDENTITY,
        &mut target,
    );
    counters::take()
}

/// **Uma forma SÓ-PREENCHIDA custa UMA construção e UM cozimento.**
///
/// Era o caso da cena inteira do spike de escala, e era onde o desperdício doía: o caminho completo
/// era construído e jogado fora, porque não havia traço para o ler.
///
/// ⚠️ Mutação que tem de sangrar: construir o caminho completo incondicionalmente (**o código que
/// shipava**) ⇒ 2 construções.
#[test]
fn a_fill_only_shape_is_built_once_and_cooked_once() {
    assert_eq!(
        draws_of(true, false, false),
        (1, 1),
        "uma forma so'-preenchida tem de custar UMA construcao de caminho e UM cozimento"
    );
}

/// **Uma forma SÓ-TRAÇADA custa UMA construção e UM cozimento.** O controle do de cima: ele já era
/// verdade antes da cura, e é o que garante que a cura não trocou um desperdício por outro.
#[test]
fn a_stroke_only_shape_is_built_once_and_cooked_once() {
    assert_eq!(draws_of(false, true, false), (1, 1));
}

/// **Preenchida E traçada, sem contorno aberto: os dois COMPARTILHAM uma construção.**
///
/// Sem contorno aberto o desenho do traço **é** o do preenchimento. Cozer duas vezes aqui era o item
/// 3 da auditoria: a pilha de Live Path Effects corria duas vezes por forma por frame.
///
/// ⚠️ Mutação que tem de sangrar: dar ao traço construção própria ⇒ 2 construções.
#[test]
fn a_closed_shape_filled_and_stroked_shares_one_build() {
    assert_eq!(
        draws_of(true, true, false),
        (1, 1),
        "o traco de uma forma fechada desenha o MESMO caminho do preenchimento — construir duas          vezes e' o desperdicio, cozinhar duas vezes e' a pilha de efeitos a correr em dobro"
    );
}

/// **Com contorno ABERTO são DOIS desenhos — e duas construções é o trabalho honesto.**
///
/// O preenchimento ignora o contorno aberto (ele não tem interior) e o traço o leva, então os dois
/// caminhos DIFEREM. O cozimento continua sendo **um**: é o mesmo cozido a servir os dois.
///
/// ⚠️ É este gate que impede a "otimização" de compartilhar sempre — que reintroduziria o triângulo
/// escuro no cubo isométrico (o bug que criou o `build_fill_bezpath`).
#[test]
fn an_open_subpath_forces_two_builds_but_still_one_cook() {
    assert_eq!(
        draws_of(true, true, true),
        (2, 1),
        "com contorno ABERTO o preenchimento e o traco desenham caminhos diferentes (duas          construcoes, trabalho honesto), mas o COZIDO e' um so'"
    );
}

/// Sonda: os três números que os gates acima comparam.
/// `cargo test -p ph2d-vec-render --release measure_encode_by_style -- --ignored --nocapture`
#[test]
#[ignore = "sonda de medição; rode com --nocapture"]
fn measure_encode_by_style() {
    let f = encode_ms(&grid(true, false));
    let s = encode_ms(&grid(false, true));
    let b = encode_ms(&grid(true, true));
    println!(
        "\nN={N}  fill={f:.3} ms  stroke={s:.3} ms  both={b:.3} ms   fill/stroke={:.2}x  both/fill={:.2}x",
        f / s,
        b / f
    );
}
