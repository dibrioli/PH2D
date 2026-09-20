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
//! # ⭐ E a cura é DERIVAR, não encurtar
//!
//! O `4` por fileira era uma constante. Hoje quantos cabem é medido no rótulo mais largo da
//! família ([`ph2d_panel_inspector::chips_por_fileira`]) — ⛔ **e o gate lê essa porta em vez de a
//! reimplementar**: *um gate que reescreve a régua mede o gate, não o painel.*

use ph2d_text::TextSystem;
use ph2d_tokens::{Spacing, TypeToken};

/// A escada de larguras de painel — a mesma disciplina do
/// [`super::every_label_this_panel_paints_fits_its_column`]: *a largura do artista é um ESTADO, e
/// pregá-la é escolher um ponto que envelhece em horas.*
const LARGURAS: &[(f32, &str)] = &[
    (220.0, "o MÍNIMO do dock"),
    (273.3, "amostra datada da largura do dono (14/09)"),
    (304.0, "a largura de OMISSÃO (`inspector-w`)"),
    (348.2, "amostra datada do `layout.txt` do dono (19/09)"),
    (720.0, "o MÁXIMO do dock"),
];

/// A largura de uma linha de card, dada a largura do painel.
fn linha(painel: f32) -> f32 {
    painel - 2.0 * ph2d_tokens::PANEL_HEAD_PAD_PX - 2.0 * Spacing::Sm.px()
}

/// As seis famílias de chip da secção, com os rótulos que o motor declara.
fn familias() -> Vec<(&'static str, Vec<&'static str>)> {
    vec![
        (
            "canal",
            ph2d_tween::Canal::ALL
                .iter()
                .map(|c| ph2d_i18n::tr(c.label_key()))
                .collect(),
        ),
        (
            "curva",
            ph2d_anim::EasingFamily::ALL
                .iter()
                .map(|f| ph2d_i18n::tr(f.label_key()))
                .collect(),
        ),
        (
            "ease",
            ph2d_anim::EasingMode::ALL
                .iter()
                .map(|m| ph2d_i18n::tr(m.label_key()))
                .collect(),
        ),
        (
            "fim",
            ph2d_tween::AoAcabar::ALL
                .iter()
                .map(|a| ph2d_i18n::tr(a.label_key()))
                .collect(),
        ),
        (
            "ciclo",
            ph2d_tween::Ciclo::ALL
                .iter()
                .map(|c| ph2d_i18n::tr(c.label_key()))
                .collect(),
        ),
        (
            "preset",
            ph2d_tween::Preset::ALL
                .iter()
                .map(|p| ph2d_i18n::tr(p.label_key()))
                .collect(),
        ),
    ]
}

/// ⭐⭐⭐ **O CHIP QUE O PAINEL DE FACTO PINTA cabe o rótulo que ele de facto mostra.**
///
/// ⛔⛔ **Esta é a metade que percorre a ROTA, e ela nasceu de uma mutação SOBREVIVENTE:** a 1.ª
/// redacção deste ficheiro chamava a régua ([`ph2d_panel_inspector::chips_por_fileira`])
/// **directamente**, logo repor a constante `4` dentro do pintor deixava-a **verde** — *um gate que
/// chama a função em vez de percorrer a rota afirma que a peça certa existe, nunca que o painel a
/// usa*. A régua sozinha fica no gate irmão, sobre a escada de larguras.
///
/// ⚠️ A largura medida é a do **rectângulo pintado**, que é exactamente o que o botão recebe.
///
/// **Mutação que deve sangrar:** o `por_fileira` do pintor voltar a ser a constante.
#[test]
fn o_chip_pintado_cabe_o_rotulo_pintado() {
    let (pintados, largura_da_fileira, rects) = fileira_de_canais();
    let fonte = ph2d_editor_core::widget::Button::label_font_px();
    let mut ts = TextSystem::new();

    // ⛔⛔⛔ **A METADE QUE PERCORRE A ROTA é a CONTAGEM, e não a largura** — e ela nasceu de uma
    // mutação SOBREVIVENTE, duas vezes.
    //
    // A 1.ª redacção chamava a régua directamente: repor a constante `4` dentro do pintor deixava-a
    // verde. A 2.ª pintava sobre uma escada de larguras de painel — e **o arnês não as honra**: o
    // Inspector tira a largura dele do dock, não do viewport, e os chips saem a `88` px de `220` a
    // `348`. *Variar um número que não chega ao sujeito é medir outro programa.*
    //
    // ⇒ o que se afirma é que **o pintor pôs na fileira o número que a PORTA manda** para a largura
    // que ele de facto usou. Com a constante de volta ele põe `4` onde a porta diz `3`, e isto
    // sangra em qualquer largura.
    let rotulos: Vec<&str> = ph2d_tween::Canal::ALL
        .iter()
        .map(|c| ph2d_i18n::tr(c.label_key()))
        .collect();
    let esperado = ph2d_panel_inspector::chips_por_fileira(&mut ts, largura_da_fileira, &rotulos);
    assert_eq!(
        pintados, esperado,
        "o pintor pos {pintados} chips na fileira e a porta manda {esperado} (fileira de \
         {largura_da_fileira:.1} px) — ele deixou de a consultar"
    );

    // E a lei que a porta serve: o rótulo cabe no rectângulo que o painel lhe deu.
    let mut cortados = Vec::new();
    for (i, id) in ph2d_panel_inspector::ids::INSP_TWEEN_CANAL
        .iter()
        .enumerate()
    {
        let Some(r) = rects.iter().find(|(n, _)| n == id).map(|(_, r)| *r) else {
            continue;
        };
        let rotulo = ph2d_i18n::tr(ph2d_tween::Canal::ALL[i].label_key());
        let m = ts.prefix_width(rotulo, fonte);
        if m > r.w {
            cortados.push(format!(
                "{rotulo:?} mede {m:.1} px num chip PINTADO de {:.1}",
                r.w
            ));
        }
    }
    assert!(
        cortados.is_empty(),
        "chips CORTADOS no painel REAL — e dois que cortem no mesmo prefixo sao indistinguiveis \
         sob o dedo:\n  {}",
        cortados.join("\n  ")
    );
}

/// Pinta a secção e devolve **(quantos chips de canal partilham a 1.ª fileira, a largura dessa
/// fileira, todos os rectângulos)**.
///
/// ⚠️ A largura sai dos próprios rectângulos (do bordo esquerdo do primeiro ao direito do último),
/// e não de um token: *o que se mede é o que o pintor usou.*
fn fileira_de_canais() -> (
    usize,
    f32,
    Vec<(ph2d_a11y::NodeId, ph2d_editor_core::zones::Rect)>,
) {
    use ph2d_editor_core::tween_edits::{InspectorTweenInfo, InspectorTweenRow};
    use ph2d_editor_core::zones::Rect;
    use ph2d_panel_inspector::{InspectorPanel, InspectorState, ids, set_current_inspector_tween};
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

    let mut da_familia: Vec<Rect> = ids::INSP_TWEEN_CANAL
        .iter()
        .filter_map(|id| rects.iter().find(|(n, _)| n == id).map(|(_, r)| *r))
        .collect();
    // ⛔ Piso de população: sem chips pintados tudo abaixo é trivialmente verde.
    assert_eq!(
        da_familia.len(),
        ph2d_tween::Canal::ALL.len(),
        "a familia dos canais nao foi pintada inteira"
    );
    let topo = da_familia[0].y;
    da_familia.retain(|r| (r.y - topo).abs() < 0.5);
    let esquerda = da_familia.iter().map(|r| r.x).fold(f32::INFINITY, f32::min);
    let direita = da_familia
        .iter()
        .map(|r| r.x + r.w)
        .fold(f32::NEG_INFINITY, f32::max);
    (da_familia.len(), direita - esquerda, rects)
}

/// ⭐⭐ **A RÉGUA, sobre a escada de larguras do dock** — a outra metade.
///
/// ⚠️ Esta chama a porta de propósito: ela mede a *régua*, e quem mede a *rota* é o gate acima.
///
/// **Mutação que deve sangrar:** um rótulo novo mais comprido do que a fileira aguenta.
#[test]
fn nenhum_chip_da_seccao_tween_sai_cortado() {
    let fonte = ph2d_editor_core::widget::Button::label_font_px();
    let gap = Spacing::Xs.px();
    let mut ts = TextSystem::new();
    let mut cortados = Vec::new();
    for (painel, porque) in LARGURAS {
        let w = linha(*painel);
        for (nome, rotulos) in familias() {
            // ⛔ Piso de população: uma família vazia satisfaz o laço em silêncio.
            assert!(!rotulos.is_empty(), "a familia «{nome}» esta' vazia");
            let n = ph2d_panel_inspector::chips_por_fileira(&mut ts, w, &rotulos);
            assert!(n >= 1, "a regua devolveu ZERO chips para «{nome}»");
            #[allow(clippy::cast_precision_loss)]
            let nf = n as f32;
            let cw = (w - gap * (nf - 1.0)) / nf;
            for t in &rotulos {
                let m = ts.prefix_width(t, fonte);
                if m > cw {
                    cortados.push(format!(
                        "painel {painel} ({porque}): «{nome}» / {t:?} mede {m:.1} px num chip de \
                         {cw:.1} ({n} por fileira)"
                    ));
                }
            }
        }
    }
    assert!(
        cortados.is_empty(),
        "chips CORTADOS — e dois que cortem no mesmo prefixo sao indistinguiveis sob o dedo:\n  {}",
        cortados.join("\n  ")
    );
}

/// ⭐⭐ **O CONTROLO da régua: com o `4` fixo ela ACUSA** — senão o gate acima passaria por vácuo.
///
/// ⚠️ *Um gate sem controlo positivo do próprio fenómeno mede o nada e fica verde* — a lei que o
/// gate da guarda do pincel de pose pagou duas vezes.
#[test]
fn a_regua_dos_chips_acusa_a_configuracao_que_o_dono_fotografou() {
    let fonte = ph2d_editor_core::widget::Button::label_font_px();
    let gap = Spacing::Xs.px();
    let mut ts = TextSystem::new();
    let w = linha(304.0); // a largura de omissão, onde a foto foi tirada
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
