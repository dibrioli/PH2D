//! **Qual MÓDULO é dono deste nível de smoke** — irmão do [`super::build_smoke`] pelo teto de 600
//! LOC da shell (HR-18), cortado por RESPONSABILIDADE.
//!
//! O `build_smoke.rs` fazia duas coisas: ROTEAR os níveis cujas cenas moram noutros módulos, e
//! HOSPEDAR as cenas que ainda vivem nele. A primeira é uma tabela de despacho que cresce a cada
//! wave (foi a cena `=34`, a lei de mistura, que a levou ao teto); a segunda é código de cena. São
//! assuntos diferentes, e agora são arquivos diferentes.
//!
//! ⚠️ **A ORDEM é load-bearing e não mudou.** Os níveis específicos vêm ANTES dos genéricos: um
//! nível roteado aqui nunca chegava ao `match f` do irmão, e um que não é roteado nunca entrava
//! nestes braços. Devolver `true` significa *"já tratei, pare"*.

/// Roteia `level` para o módulo dono dele. `true` = tratado (o chamador retorna).
pub(crate) fn route(app: &mut crate::App, f: u32, level: u32) -> bool {
    // As cenas do ENVELOPE (níveis 11 e 12) vivem no módulo irmão `envelope_smoke` — teto de
    // LOC. Elas só usam os frames 3 e 4 e nenhum braço compartilhado, então sair do `match`
    // aqui é a MESMA sequência de antes: um nível fora de 11/12 nunca entrava nesses braços, e
    // 11/12 nunca chegavam aos genéricos (os específicos vinham primeiro).
    if matches!(level, 11 | 12 | 27) {
        crate::envelope_smoke::frame(app, f, level);
        return true;
    }
    // A cena da PILHA de efeitos (ADR-0132), no módulo irmão `fx_smoke` — mesma razão de
    // LOC, e mesma disciplina: os níveis 13/14 nunca tocam um braço genérico abaixo. 13 é a
    // pilha animada; 14 é a cena do Apply / Convert (estática).
    // A cena do CONTOUR (pesquisa `20_*` #9), no módulo irmão `contour_smoke` — mesma razão
    // de LOC e mesma disciplina: o nível 25 nunca toca um braço genérico abaixo.
    if level == 25 {
        match f {
            3 => app.smoke_contour_build(),
            4 => app.smoke_contour_arm(),
            _ => {}
        }
        return true;
    }
    // A cena da família WARP (Arc/Bulge/Wave/Fisheye/Rise) — irmão `warp_smoke`, mesma razão
    // de LOC: cinco retângulos armados + um pelado para autorar pela seção Effects.
    if level == 26 {
        crate::warp_smoke::frame(app, f);
        return true;
    }
    // A cena do FALLOFF (o campo escalar que modula a força do deformador seguinte) — irmão
    // `falloff_smoke`, mesma razão de LOC.
    if level == 28 {
        crate::falloff_smoke::frame(app, f);
        return true;
    }
    // A cena do TWIST (o remoinho, e o Falloff a modulá-lo) — irmão `twist_smoke`, mesma razão.
    if level == 29 {
        crate::twist_smoke::frame(app, f);
        return true;
    }
    // A cena do KNOT (o entrelace celta over/under) — irmão `knot_smoke`, mesma razão.
    if level == 30 {
        crate::knot_smoke::frame(app, f);
        return true;
    }
    // As cenas do SKETCH (=31) e do HATCH (=32) — irmão `sketch_hatch_smoke`, mesma razão.
    if level == 31 || level == 32 {
        crate::sketch_hatch_smoke::frame(app, f, level);
        return true;
    }
    // A cena do FX RASTER (=33 — Blur/Glow/Drop Shadow, plano 24) — irmão `fx_raster_smoke`.
    // NÃO confundir com `fx_smoke` (=13/14), que é a pilha de deformadores vetoriais (ADR-0132).
    if level == 33 {
        crate::fx_raster_smoke::frame(app, f);
        return true;
    }
    // A LEI DE MISTURA por degrau (=34, plano 24 W6) — irmã da =33, e separada dela porque é
    // um A/B (o mesmo degrau sob duas leis), não um catálogo.
    if level == 34 {
        crate::fx_blend_smoke::frame(app, f);
        return true;
    }
    // A TURBULÊNCIA (=35, plano 24 W6b) — irmã da =34 e no mesmo molde: quatro pares, um knob de
    // diferença em cada.
    if level == 35 {
        crate::fx_turbulence_smoke::frame(app, f);
        return true;
    }
    // GROW / SHRINK (=36, plano 24 W7) — a mesma família: quatro pares, uma coisa de diferença.
    if level == 36 {
        crate::fx_morphology_smoke::frame(app, f);
        return true;
    }
    if matches!(level, 13 | 14) {
        crate::fx_smoke::frame(app, f, level);
        return true;
    }
    // O UNDO da pilha de efeitos, AUTO-DIRIGIDO (o report do Enio, 3×) — irmão `fx_undo_smoke`.
    if level == 20 {
        crate::fx_undo_smoke::frame(app, f);
        return true;
    }
    // A cena da W0 (texto + pilha de efeitos sobrevive ao re-cook) — irmão `text_fx_smoke`.
    if level == 21 {
        crate::text_fx_smoke::frame(app, f);
        return true;
    }
    // A cena da W3 (o texto cavalga o caminho) — irmão `text_path_smoke`.
    if level == 22 {
        crate::text_path_smoke::frame(app, f);
        return true;
    }
    // A cena do GESTO (o artista prende o texto pelo painel) — irmão
    // `text_path_gesture_smoke`. Irmã da 22: aquela mostra o motor, esta o caminho até ele.
    if level == 23 {
        crate::text_path_gesture_smoke::frame(app, f);
        return true;
    }
    // A cena do GESTO do Pattern Along Path (plano 23, W3): motivo + guia selecionados, o
    // artista prende pelo painel; daí o `pattern_live::recook` -> `dispatch` desenha as cópias.
    if level == 24 {
        crate::pattern_path_smoke::frame(app, f);
        return true;
    }
    false
}
