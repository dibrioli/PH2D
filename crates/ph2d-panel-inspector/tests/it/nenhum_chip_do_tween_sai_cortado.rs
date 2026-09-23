//! ⭐⭐⭐ **NENHUM CHIP DA SECÇÃO TWEEN SAI CORTADO** (suplente #22, W10).
//!
//! ⛔⛔ **Report do dono, 2026-09-19, com FOTO:** o grupo dos CANAIS mostrava `Silhoue...`,
//! `Positio...` e `Positio...` — e os dois últimos são **`Position X` e `Position Y`**, que o
//! artista passa a ler **iguais**. *Duas opções que se leem igual são uma escolha que ele não
//! consegue fazer* — a lei que o chip da vigia do contador já escreve.
//!
//! # ⛔⛔⛔ A 1.ª RÉGUA DESTA MEDIÇÃO MEDIU OUTRO PROGRAMA
//!
//! Ela pesou os rótulos à **`Sm`** — a fonte da *legenda* de um grupo — e leu **«nenhum cortado»**
//! sobre a foto do dono. O botão pinta o rótulo à **`Base`**
//! ([`ph2d_editor_core::widget::Button::label_font_px`]), e à `Base` a mesma régua acusa os três,
//! ao pixel:
//!
//! | rótulo | mede | chip (4 por fileira, painel de omissão) |
//! |---|---|---|
//! | `Silhouette` | `63,3` | `61,0` |
//! | `Position X` | `61,6` | `61,0` |
//! | `Position Y` | `61,3` | `61,0` |
//!
//! ⇒ *quem pergunta «este rótulo cabe?» lê a fonte da PORTA DO BOTÃO*, e a porta nasceu com esta
//! medição para que ninguém volte a adivinhá-la.
//!
//! # ⭐ E a cura é DERIVAR, não encurtar — e em 2026-09-23 a derivação MUDOU DE DONO
//!
//! O `4` por fileira era uma constante; a 1.ª cura mediu quantos cabiam no rótulo mais largo da
//! família (`chips_por_fileira`, uma régua LOCAL ao Tween). ⛔ **Essa régua MORREU** quando toda
//! escolha do Inspector passou pela porta [`ph2d_editor_core::property_row::paint_choice_row`]: o
//! grupo segmentado adaptativo dá a cada chip a largura NATURAL do rótulo e reflui por fileiras,
//! logo *«quantos cabem»* deixou de ser um número da secção. ⇒ a metade que comparava a contagem
//! pintada com a da régua **morreu com ela** (a premissa fica à vista aqui), e a que fica é a que
//! nunca dependeu dela: **o rectângulo PINTADO cabe o rótulo PINTADO**.
//!
//! ⚠️ A escada de larguras que media a régua sozinha também saiu: sem régua local não há o que
//! medir fora da rota, e o corte de um rótulo em qualquer largura do dock é a pergunta da varredura
//! das elisões do app inteiro (`ph2d-panel-registry-init`), que corre sobre a escada dela.

use ph2d_text::TextSystem;
use ph2d_tokens::{Spacing, TypeToken};

/// ⭐⭐⭐ **O CHIP QUE O PAINEL DE FACTO PINTA cabe o rótulo que ele de facto mostra.**
///
/// ⚠️ A largura medida é a do **rectângulo pintado**, que é exactamente o que o botão recebe — pela
/// rota do produto, com a secção pintada inteira.
///
/// ⛔⛔ **A metade da CONTAGEM morreu (2026-09-23), com a premissa à vista:** ela afirmava que o
/// pintor punha na fileira o número que a régua local `chips_por_fileira` mandava. A régua saiu —
/// a escolha passou pela porta da ESCOLHA, que reflui por larguras naturais — e comparar com ela
/// seria medir um número que ninguém lê.
///
/// **Mutação que deve sangrar:** a porta voltar a partes IGUAIS (o `4` de antes, ou qualquer
/// divisão da fileira que não pergunte ao rótulo).
#[test]
fn o_chip_pintado_cabe_o_rotulo_pintado() {
    let rects = seccao_pintada();
    let fonte = ph2d_editor_core::widget::Button::label_font_px();
    let mut ts = TextSystem::new();
    let mut cortados = Vec::new();
    let mut medidos = 0usize;
    for (i, id) in ph2d_panel_inspector::ids::INSP_TWEEN_CANAL
        .iter()
        .enumerate()
    {
        let Some(r) = rects.iter().find(|(n, _)| n == id).map(|(_, r)| *r) else {
            continue;
        };
        medidos += 1;
        let rotulo = ph2d_i18n::tr(ph2d_tween::Canal::ALL[i].label_key());
        let m = ts.prefix_width(rotulo, fonte);
        if m > r.w {
            cortados.push(format!(
                "{rotulo:?} mede {m:.1} px num chip PINTADO de {:.1}",
                r.w
            ));
        }
    }
    // ⛔ Piso de população: sem chips pintados tudo acima é trivialmente verde.
    assert_eq!(
        medidos,
        ph2d_tween::Canal::ALL.len(),
        "a familia dos canais nao foi pintada inteira"
    );
    assert!(
        cortados.is_empty(),
        "chips CORTADOS no painel REAL — e dois que cortem no mesmo prefixo sao indistinguiveis \
         sob o dedo:\n  {}",
        cortados.join("\n  ")
    );
}

/// Pinta a secção Tween pela rota do painel e devolve todos os rectângulos registados.
fn seccao_pintada() -> Vec<(ph2d_a11y::NodeId, ph2d_editor_core::zones::Rect)> {
    use ph2d_editor_core::tween_edits::{InspectorTweenInfo, InspectorTweenRow};
    use ph2d_editor_core::zones::Rect;
    use ph2d_panel_inspector::{InspectorPanel, InspectorState, set_current_inspector_tween};
    use ph2d_ui_testkit::MockPanelHost;

    let mut host = MockPanelHost::with_panel::<InspectorPanel>();
    let mut state = InspectorState::default();
    set_current_inspector_tween(Some(InspectorTweenInfo {
        entity_bits: 0x7CEE_00C1,
        rows: vec![InspectorTweenRow {
            canal: ph2d_tween::Canal::Opacity.tag(),
            de: [1.0, 0.0, 0.0, 0.0],
            para: [0.0, 0.0, 0.0, 0.0],
            familia: 0,
            modo: 0,
            ao_acabar: ph2d_tween::AoAcabar::Hold.tag(),
            ciclo: ph2d_tween::Ciclo::Reinicia.tag(),
            duracao_us: Some(400_000),
            repeat: true,
            autostart: true,
        }],
        tem_sprite: true,
        selected_count: 1,
    }));
    let rects = host.paint::<InspectorPanel>(
        &mut state,
        Rect {
            x: 0.0,
            y: 0.0,
            w: ph2d_tokens::INSPECTOR_W_PX,
            h: 2400.0,
        },
    );
    set_current_inspector_tween(None);
    rects
}

/// ⭐⭐ **O CONTROLO do fenómeno: com o `4` fixo a foto do dono REAPARECE** — senão o gate da
/// rota passaria por vácuo (uma família cujos rótulos cabem sempre não prova que se mede o corte).
///
/// ⚠️ *Um gate sem controlo positivo do próprio fenómeno mede o nada e fica verde* — a lei que o
/// gate da guarda do pincel de pose pagou duas vezes.
#[test]
fn a_regua_dos_chips_acusa_a_configuracao_que_o_dono_fotografou() {
    let fonte = ph2d_editor_core::widget::Button::label_font_px();
    let gap = Spacing::Xs.px();
    let mut ts = TextSystem::new();
    // A largura de uma linha de card no painel de omissão (304 px), onde a foto foi tirada.
    let w = 304.0 - 2.0 * ph2d_tokens::PANEL_HEAD_PAD_PX - 2.0 * Spacing::Sm.px();
    let cw = (w - gap * 3.0) / 4.0; // o `4` fixo de antes da cura
    let canais: Vec<&str> = ph2d_tween::Canal::ALL
        .iter()
        .map(|c| ph2d_i18n::tr(c.label_key()))
        .collect();
    let cortados: Vec<&&str> = canais
        .iter()
        .filter(|t| ts.prefix_width(t, fonte) > cw)
        .collect();
    assert_eq!(
        cortados.len(),
        3,
        "com 4 por fileira a foto do dono mostra TRES cortados, e a regua le^ {cortados:?}"
    );
    // ⛔ E a metade que diz porque isto é pior que feio: dois deles cortam no MESMO prefixo.
    let prefixos: Vec<String> = cortados
        .iter()
        .map(|t| t.chars().take(7).collect())
        .collect();
    assert!(
        prefixos
            .iter()
            .enumerate()
            .any(|(i, a)| prefixos.iter().skip(i + 1).any(|b| a == b)),
        "controlo: a foto mostra DOIS chips a ler «Positio...» — se nao ha' prefixo repetido, a \
         regua deixou de conter o fenomeno"
    );
    // ⚠️ E a `Sm` — a fonte ERRADA — não acusa nenhum: é o que a 1.ª medição leu.
    let sm = TypeToken::Sm.px();
    assert_eq!(
        canais
            .iter()
            .filter(|t| ts.prefix_width(t, sm) > cw)
            .count(),
        0,
        "controlo: a` `Sm` a regua tem de ler ZERO cortados — e' o erro que esta wave curou"
    );
}
