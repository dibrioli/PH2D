//! Um binário de teste por crate (auditoria de velocidade de 2026-09-10, onda W1).
//!
//! Cada ficheiro em `tests/it/` é um MÓDULO deste binário, não um binário próprio: antes eram
//! 2 binários nesta crate, cada um a religar a closure inteira de dependências (matklad,
//! «Delete Cargo Integration Tests»: 3× menos compilação, 5× menos disco). Os nomes dos testes
//! ganham o prefixo do módulo (`ficheiro::fn`); filtros por `test(nome)` continuam a casar.
//! ⚠️ Teste novo = ficheiro novo AQUI + uma linha `mod` abaixo — nunca um `tests/*.rs` solto.

mod as_duas_caixas_cabem_na_coluna;
mod as_seccoes_arrastam_e_tem_tema;
mod every_word_this_panel_shows_comes_from_the_string_table;
mod os_cartoes;
mod panel_chrome;
mod pela_porta_do_rato;
mod seam;

/// B35.3: liga todo controlo-pai de [`ph2d_panel_wet_tuning::pais::PAIS`] (salvo o Real Oil, que é do chamador),
/// pela rota do painel — com eles a zero as filhas não se pintam.
pub(crate) fn liga_os_pais(t: &mut ph2d_tool_painter::PainterTool) {
    use ph2d_editor_core::Tool;
    use ph2d_editor_core::tool::PanelEvent;
    use ph2d_panel_wet_tuning::pais::{PAIS, Pai};
    for (_, pai) in PAIS {
        match pai {
            // O Brush dust e o Wet bloom moram no cartão Wet Paint (a rota do Tuning não os escreve).
            Pai::Knob("dust") => t.handle_panel_event(PanelEvent::SetValue(
                ph2d_tool_painter::ids::PAINTER_WETPAINT_DUST,
                1.0,
            )),
            Pai::Knob("wetOpen") => t.handle_panel_event(PanelEvent::SetValue(
                ph2d_tool_painter::ids::PAINTER_WETPAINT_WET_BLOOM,
                1.0,
            )),
            Pai::Knob(chave) => t.handle_panel_event(PanelEvent::SetValue(
                ph2d_tool_painter::ids::wet_tuning_slider_id(chave),
                1.0,
            )),
            Pai::OleoReal => continue,
        };
    }
}
