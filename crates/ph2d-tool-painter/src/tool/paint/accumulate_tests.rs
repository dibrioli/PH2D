//! Gates do **Accumulate** — a metade que AFIRMA, ao lado da sonda que MEDE
//! ([`super::accumulate_probe`], que é dona da fixture e dos helpers: um oráculo só, para o gate e a
//! medição que o motivou nunca discordarem sobre o que "o traço" significa).
//!
//! O estudo com os números e o mecanismo é [`docs/Painter/35_accumulate_vs_blender.md`].

use super::accumulate_probe::{alpha, one_stroke, soft_tool};
use ph2d_editor_core::ids as core_ids;
use ph2d_editor_core::tool::{PanelEvent, Tool};

/// A razão entre o espaçamento mais denso e o mais esparso, num mesmo CAMINHO.
fn spacing_ratio(accumulate: bool, atten: bool) -> f32 {
    let (mut lo, mut hi) = (f32::MAX, 0.0f32);
    for &sp in &[0.05f32, 0.10, 0.20, 0.40] {
        let mut t = soft_tool(0.5, accumulate);
        if atten {
            t.handle_panel_event(PanelEvent::Click(core_ids::PAINTER_BRUSH_SPACE_ATTEN));
        }
        t.handle_panel_event(PanelEvent::SetValue(
            core_ids::PAINTER_BRUSH_SPACING,
            f64::from(sp),
        ));
        one_stroke(&mut t, 1);
        let a = alpha(&t);
        lo = lo.min(a);
        hi = hi.max(a);
    }
    hi / lo
}

/// **"Adjust Strength for Spacing" não pode PIORAR o que o nome promete.**
///
/// O knob é o compensador da lei que empilha (Accumulate ON). Sob o teto por texel nada empilha, e
/// ele entrava em `coverage`, que É o teto — então atenuar só abaixava o teto, por um número função
/// do espaçamento: medido **8,17×** onde a lei capada sozinha dá **1,02×** (doc 35 §3.3).
///
/// **Mutação que tem de sangrar:** tirar o `&& self.accumulate` de
/// [`ph2d_painter_brush::BrushSpec::space_overlap_factor`] ⇒ esta razão volta a ~8×.
#[test]
fn the_spacing_knob_never_makes_a_capped_stroke_spacing_dependent() {
    let plain = spacing_ratio(false, false);
    let attenuated = spacing_ratio(false, true);
    assert!(
        plain < 1.15,
        "controle: a lei capada JA e independente do espacamento, e a fixture tem de mostrar isso \
         (razao {plain:.2}x)"
    );
    assert!(
        attenuated <= plain + 0.05,
        "o knob de espacamento PIOROU um traco capado: {attenuated:.2}x contra {plain:.2}x sem ele"
    );
}

/// E o CONTROLE do outro lado: no modo que o knob de fato serve, ele continua servindo — sem esta
/// metade, "desligar a atenuação em todo lugar" passaria no gate acima e mataria a feature.
#[test]
fn the_spacing_knob_still_flattens_the_law_that_piles_up() {
    let raw = spacing_ratio(true, false);
    let fixed = spacing_ratio(true, true);
    assert!(
        raw > 2.0,
        "a lei que empilha DEPENDE do espacamento — a fixture precisa conter isso (razao {raw:.2}x)"
    );
    assert!(
        fixed < raw * 0.6,
        "com a atenuacao a lei que empilha tem de ficar bem mais plana: {fixed:.2}x contra {raw:.2}x"
    );
}

/// ⚠️ **O que a medição REFUTOU, pinado para ninguém "consertar" de volta.**
///
/// O doc 35 recomendava, na 1ª escrita, tirar a cláusula `strength < 1.0` de
/// `stroke_cover_wanted` — a teoria era que o teto passaria a existir em força máxima e o ombro
/// pararia de endurecer. **Construí a cura e medi: byte-idêntica.** A álgebra diz por quê: com
/// `cap = 1` o passo do teto é `m += w·(1 − m)` e o chamador compõe em `a = add/(1 − m) = w`, que é
/// **exatamente** source-over por dab — a lei do Accumulate ON. As duas leis COINCIDEM em
/// `strength = 1`, então a cláusula é uma **otimização** (pular o buffer onde ele provadamente não
/// faz nada), não um defeito.
///
/// Este gate afirma a coincidência, que é o fato durável.
#[test]
fn at_full_strength_the_two_laws_are_the_same_law() {
    for &n in &[1usize, 15] {
        let mut off = soft_tool(1.0, false);
        let mut on = soft_tool(1.0, true);
        one_stroke(&mut off, n);
        one_stroke(&mut on, n);
        assert_eq!(
            off.canvas_rgba, on.canvas_rgba,
            "em strength 1.0 o teto vale 1 ⇒ o cap reduz a source-over por dab; n={n}"
        );
    }
}

// ─────────────────────────────────────────────────────────────────────────────────────────────────
// D3 — o Accumulate alcança o RELEVO (doc 35 §6/D3). Os gates de comportamento; a aritmética tem os
// dela em `ph2d-painter-brush::height_tests`.
// ─────────────────────────────────────────────────────────────────────────────────────────────────

use super::accumulate_probe::{impasto_tool, relief};
use ph2d_editor_core::tool::{CanvasPaintTool, PointerPhase};

/// **O corpo da tinta engrossa quando se esfrega** — o report que abriu esta frente. E o CONTROLE
/// vive no mesmo gate: com o flag desligado o envelope `max` mantém uma passada valendo uma
/// espessura, por mais que a mão vá e volte.
///
/// **Mutação que tem de sangrar:** `accum_step` devolver `None` sempre.
#[test]
fn scrubbing_builds_the_body_up_only_with_accumulate_on() {
    let (mut off1, mut off15) = (impasto_tool(false), impasto_tool(false));
    super::accumulate_probe::one_stroke(&mut off1, 1);
    super::accumulate_probe::one_stroke(&mut off15, 15);
    let (a, b) = (relief(&off1), relief(&off15));
    assert!(
        b < a * 1.2,
        "CONTROLE: sob o envelope 15 passadas nao podem empilhar ({a:.4} -> {b:.4})"
    );

    let (mut on1, mut on15) = (impasto_tool(true), impasto_tool(true));
    super::accumulate_probe::one_stroke(&mut on1, 1);
    super::accumulate_probe::one_stroke(&mut on15, 15);
    let (c, d) = (relief(&on1), relief(&on15));
    assert!(
        d > c * 5.0,
        "com Accumulate 15 passadas tem de engrossar de verdade ({c:.4} -> {d:.4})"
    );
}

/// **I1 — o relevo é fato do CAMINHO, nunca de quão fino o motor amostrou o caminho.** É a doença
/// que esta linha curou três vezes (a cápsula, a mordida do bow wave, o campo do Smear), e a lei
/// nova tem de nascer imune: dobrar a densidade de dabs dobra a contagem e divide `Δs` por dois.
///
/// **Mutação que tem de sangrar:** somar `perfil` em vez de `perfil · Δs` (tirar o `* step`).
#[test]
fn the_accumulated_body_is_a_fact_of_the_path_not_of_the_spacing() {
    use ph2d_editor_core::ids as core_ids;
    use ph2d_editor_core::tool::{PanelEvent, Tool};
    let (mut lo, mut hi) = (f32::MAX, 0.0f32);
    for &sp in &[0.05f32, 0.10, 0.20] {
        let mut t = impasto_tool(true);
        t.handle_panel_event(PanelEvent::SetValue(
            core_ids::PAINTER_BRUSH_SPACING,
            f64::from(sp),
        ));
        super::accumulate_probe::one_stroke(&mut t, 1);
        let h = relief(&t);
        lo = lo.min(h);
        hi = hi.max(h);
    }
    let ratio = hi / lo;
    assert!(
        ratio < 1.15,
        "a lei do arco tem de ser independente do espacamento: razao {ratio:.2}x"
    );
}

/// **O toggle não muda o traço SIMPLES** — é o que a norma `2·∫₀^ρ perfil` compra, e o gate mais
/// forte que esta feature pode ter: ligar o Accumulate não pode repintar a arte de quem só passa o
/// pincel uma vez.
///
/// **Mutação que tem de sangrar:** trocar a norma por uma constante (p.ex. `1.0`).
#[test]
fn one_straight_pass_lays_the_same_body_in_both_laws() {
    let mut out = Vec::new();
    for &acc in &[false, true] {
        let mut t = impasto_tool(acc);
        t.on_canvas_pointer(super::accumulate_probe::cp(
            [20.0, 32.0],
            PointerPhase::Down,
        ));
        t.on_canvas_pointer(super::accumulate_probe::cp(
            [44.0, 32.0],
            PointerPhase::Move,
        ));
        t.on_canvas_pointer(super::accumulate_probe::cp([44.0, 32.0], PointerPhase::Up));
        out.push(relief(&t));
    }
    let (off, on) = (out[0], out[1]);
    assert!(
        (on - off).abs() / off < 0.10,
        "uma passada reta tem de dar o mesmo corpo nas duas leis: off={off:.4} on={on:.4}"
    );
}

/// ⚠️ **O preço da decisão (i), MEDIDO e não escondido:** um TAP sob Accumulate deposita *uma
/// unidade de espaçamento*, que é bem mais fino que sob o envelope. A lei pura do arco daria ZERO
/// (um toque não percorre nada); o piso nominal existe para o pincel não deixar de carimbar parado.
#[test]
fn a_tap_still_lays_body_under_the_arc_law_and_this_is_its_price() {
    let mut out = Vec::new();
    for &acc in &[false, true] {
        let mut t = impasto_tool(acc);
        t.on_canvas_pointer(super::accumulate_probe::cp(
            [32.0, 32.0],
            PointerPhase::Down,
        ));
        t.on_canvas_pointer(super::accumulate_probe::cp([32.0, 32.0], PointerPhase::Up));
        out.push(relief(&t));
    }
    let (off, on) = (out[0], out[1]);
    assert!(on > 0.0, "um TAP nao pode depositar ZERO: {on}");
    assert!(
        on < off,
        "e o preco e' ele ser mais FINO que sob o envelope: off={off:.4} on={on:.4}"
    );
}

/// **I2 — o re-carimbo é idempotente.** Os shape editors re-carimbam a figura INTEIRA a cada quadro,
/// e sem isto o relevo cresceria enquanto o artista apenas OLHA para a curva aberta. A lei do arco o
/// garante por ser função do caminho; o mecanismo que a torna executável é o `reset_stroke_height`,
/// que zera o plano de carga antes de cada re-carimbo — este gate pina os dois.
#[test]
fn a_re_stamp_starts_the_accumulated_load_over() {
    let mut t = impasto_tool(true);
    super::accumulate_probe::one_stroke(&mut t, 5);
    t.reset_stroke_height();
    assert!(
        t.paint.relief.stroke_paint.is_empty(),
        "o re-carimbo tem de comecar a carga do zero, senao a figura engorda enquanto o artista olha"
    );
}
