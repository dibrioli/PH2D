//! **As rows do GESTO DE CRIAÇÃO da §11** — as três rotas que fazem um joint.
//!
//! Irmão do `physics_rows`, e o corte é o que a própria seção já faz: lá mora
//! *o que este CORPO é* (massa, material, camada, dinâmica), aqui *como um joint
//! NASCE*. Separados quando a terceira rota (o rig da W-Rig) levou o arquivo ao
//! teto de 600 LOC dos painéis.
//!
//! As três, e cada uma aparece onde faz sentido:
//!
//! - **Desenhar** (W-J4) — sempre oferecida: aponte um corpo, arraste até outro.
//! - **Join Selected** (W3/W-J4) — 2+ corpos marcados; com 3 ou mais, uma
//!   CORRENTE.
//! - **Rig** (W-Rig) — a subárvore da Hierarquia, e a única que também aparece na
//!   face VAZIA da seção (ela CRIA os corpos, as outras duas precisam deles).

use super::rows::seg_row;
use super::*;

/// Join-kind labels, indexed by `JointKind` tag — the TYPE the next
/// *Join Selected Bodies* (or canvas draw) creates. Same order as §12's
/// `KIND_LABELS`, and it must list every kind that one does: this is the selector
/// that decides what gets CREATED, so a kind missing here is a kind the artist
/// cannot reach at all.
const JOIN_KIND_LABELS: [&str; 8] = [
    "Pin", "Spring", "Rope", "Weld", "Slider", "Rod", "Wheel", "Pulley",
];

/// Paint the joint-creation gesture: the "Join As" kind selector (Pin/Spring/
/// Rope/Weld) and the "Join Selected Bodies" button. Split here for the panel's
/// 600-LOC file cap (the join-kind selector pushed `physics.rs` over); the caller
/// gates on `can_join`. Takes the resolved `join_kind_tag`, not the whole info,
/// like every sibling in this file. The gold standard is creating the joint TYPE
/// you want in one gesture, not making a Pin and converting it in §12.
#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_join_gesture(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    join_kind_tag: u8,
    // Quantos corpos a seleção tem (0 = a rota por seleção não é oferecida).
    // Com 2 o botão liga um par; com N>2 ele faz uma CORRENTE de N−1 joints.
    join_count: u8,
    // O gesto de canvas está armado? (pinta o botão Pressed)
    draw_armed: bool,
    // W-Rig: quantas PARTES um rig da subárvore selecionada tocaria. `0` = não
    // oferecido (nenhuma aresta pai→filho a ligar).
    rig_parts: u8,
) -> f32 {
    // Choose the TYPE first — defaults to Pin, so the common case is still one
    // click on the button below.
    let mut yy = seg_row(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        y,
        "Join As",
        ids::INSP_LIVE_PHYSICS_SECTION,
        &ids::INSP_PHYS_JOIN_KIND,
        &JOIN_KIND_LABELS,
        join_kind_tag,
    );
    // **DUAS rotas de criação, e cada uma aparece onde faz sentido.**
    //
    // *Desenhar* (W-J4) é sempre oferecido: aponte um corpo, arraste até outro, e
    // as âncoras nascem NOS dois pontos. *Join Selected* precisa de uma seleção
    // de 2+ corpos — fato que só a shell vê — e com 3 ou mais faz uma CORRENTE.
    // Nenhuma escola da pesquisa tem as duas; o rótulo do 2º botão diz o que ele
    // vai fazer, em vez de deixar o artista descobrir clicando.
    //
    // A LITERAL `hit_index.register` id, the only form
    // `architecture_panel_wiring_parity` can see.
    let rect = Rect::new(x, yy, w, ROW_H_PX);
    let btn = Button::new(ids::INSP_PHYS_JOIN_DRAW, draw_button_label(draw_armed))
        .kind(ButtonKind::Default)
        .state(if draw_armed {
            ButtonState::Pressed
        } else {
            store
                .button_state(ids::INSP_PHYS_JOIN_DRAW)
                .unwrap_or(ButtonState::Normal)
        });
    paint_button(&btn, rect, scene, text_system, theme);
    hit_index.register(ids::INSP_PHYS_JOIN_DRAW, rect);
    yy += ROW_H_PX + Spacing::Sm.px();

    if join_count >= 2 {
        let label = join_button_label(join_count);
        let rect = Rect::new(x, yy, w, ROW_H_PX);
        let btn = Button::new(ids::INSP_PHYS_JOIN, &label)
            .kind(ButtonKind::Default)
            .state(
                store
                    .button_state(ids::INSP_PHYS_JOIN)
                    .unwrap_or(ButtonState::Normal),
            );
        paint_button(&btn, rect, scene, text_system, theme);
        hit_index.register(ids::INSP_PHYS_JOIN, rect);
        yy += ROW_H_PX + Spacing::Sm.px();
    }
    // **A TERCEIRA rota: o rig sai da HIERARQUIA** (W-Rig). Oferecida só quando há
    // uma aresta pai→filho a ligar — `rig_parts == 0` é a resposta inteira, e um
    // botão que não faria nada é pior que botão nenhum.
    if rig_parts > 0 {
        let label = rig_button_label(rig_parts);
        let rect = Rect::new(x, yy, w, ROW_H_PX);
        let btn = Button::new(ids::INSP_PHYS_RIG, &label)
            .kind(ButtonKind::Default)
            .state(
                store
                    .button_state(ids::INSP_PHYS_RIG)
                    .unwrap_or(ButtonState::Normal),
            );
        paint_button(&btn, rect, scene, text_system, theme);
        hit_index.register(ids::INSP_PHYS_RIG, rect);
        yy += ROW_H_PX + Spacing::Sm.px();
    }
    yy
}

/// O rotulo do botao de DESENHAR, que e um toggle (W-J4b).
///
/// ⚠️ **"Cancel Joint Drawing", nao "Cancel Joint"**: nao existe joint nenhum
/// para cancelar — o gesto ainda nao criou nada, e nomear uma coisa que nao esta
/// la faria o artista procurar o que ele desfez. O que sai do ar e o MODO.
const fn draw_button_label(armed: bool) -> &'static str {
    if armed {
        "Cancel Joint Drawing"
    } else {
        "Draw Joint on Canvas"
    }
}

/// **O rótulo do botão da rota por seleção diz quantos corpos ele vai ligar.**
///
/// Dois = *Join Selected Bodies*; três ou mais = *Chain N Selected Bodies*. Um
/// rótulo fixo sobre cinco corpos é como um artista descobre a CORRENTE por
/// acidente — e este texto é a única coisa na tela que sabe a diferença.
///
/// ⚠️ Este doc estava ANCORADO no `draw_button_label` acima, que descreve outra
/// coisa; reancorado aqui de passagem — um comentário que descreve a função de
/// baixo enquanto mora na de cima é pior que comentário nenhum.
fn join_button_label(join_count: u8) -> String {
    if join_count > 2 {
        format!("Chain {join_count} Selected Bodies")
    } else {
        "Join Selected Bodies".to_string()
    }
}

/// **E o do RIG diz quantas partes ele vai tocar** (W-Rig) — a mesma lei do
/// `Bake 5.0s to Timeline` e do `Paste to 3 Joints`: um clique que muda N objetos
/// diz N antes de ser clicado.
///
/// O número é a **divulgação** do que a expansão de subárvore fez: se você
/// selecionou um tronco esperando três partes e o botão diz seis, você vê o que
/// vai acontecer sem ter de desfazer para descobrir.
#[must_use]
pub fn rig_button_label(rig_parts: u8) -> String {
    format!("Rig {rig_parts} Parts from Hierarchy")
}

#[cfg(test)]
mod join_label_tests {
    /// **Um rótulo por id, no seletor que decide o que é CRIADO** (W-J5b).
    ///
    /// ⚠️ **A lista de tipos de joint existe DUAS vezes**, de propósito: aqui o
    /// tipo que o próximo gesto CRIA (§11 *Join As*), no `sections::joint` o tipo
    /// que a joint selecionada É (§12 *Kind*). O Slider chegou só na segunda
    /// (W-J5), e o preço foi um tipo que a simulação tinha e o artista **não
    /// conseguia escolher** — o `seg_row` faz `option_ids.zip(labels)` e um `zip`
    /// TRUNCA, então o 5º rótulo foi descartado em silêncio, sem erro e sem
    /// warning (Enio: *"Slider não aparece no painel de joints"*).
    ///
    /// O gate irmão em `sections::joint` afirma o mesmo do OUTRO par. Dois pares,
    /// duas asserções — foi escrever só uma que deixou este passar.
    #[test]
    fn every_join_kind_label_has_an_id_to_be_clicked_by() {
        assert_eq!(
            super::JOIN_KIND_LABELS.len(),
            ph2d_editor_core::ids::INSP_PHYS_JOIN_KIND.len(),
            "um rotulo sem id e um chip que o seg_row DESCARTA no zip"
        );
    }

    /// **As DUAS listas de tipo têm de ter o mesmo tamanho** — a que CRIA (§11)
    /// e a que CONVERTE (§12).
    ///
    /// ⚠️ É o gate que faltava, e a ausência dele tem nome: o Slider entrou só
    /// na lista de conversão (W-J5) e o resultado foi um tipo que a simulação
    /// tinha e o artista **não conseguia criar** (Enio: *"Slider não aparece no
    /// painel de joints"*). Os dois gates de comprimento existentes conferem
    /// cada par contra os PRÓPRIOS rótulos, então os dois ficavam verdes com as
    /// listas divergindo uma da outra — que é exatamente o que aconteceu.
    #[test]
    fn the_create_list_and_the_convert_list_know_the_same_kinds() {
        assert_eq!(
            ph2d_editor_core::ids::INSP_PHYS_JOIN_KIND.len(),
            ph2d_editor_core::ids::INSP_JOINT_KIND.len(),
            "um tipo que só a §12 conhece é um tipo que ninguém consegue CRIAR; \
             um que só a §11 conhece é um que ninguém consegue AFINAR"
        );
    }

    use super::{draw_button_label, join_button_label};

    /// Mutação-testada: devolver sempre `"Join Selected Bodies"` faz o caso de 4
    /// ficar RED.
    #[test]
    fn the_label_names_the_chain_when_there_is_one() {
        assert_eq!(join_button_label(2), "Join Selected Bodies");
        assert_eq!(join_button_label(4), "Chain 4 Selected Bodies");
        // O botão não é oferecido abaixo de 2, mas o rótulo não deve inventar
        // uma corrente se algum dia for.
        assert_eq!(join_button_label(0), "Join Selected Bodies");
    }

    /// **O toggle DIZ que e um toggle** (W-J4b). Sem isto o botao armado se
    /// parecia com o desarmado e o unico sinal era o Pressed — e um artista que
    /// nao ve saida conclui que nao ha.
    ///
    /// ⚠️ Pina tambem que o desarmado NAO diz "Cancel": o rotulo tem de nomear a
    /// acao que o clique vai fazer, e nao o estado em que o botao esta.
    #[test]
    fn the_draw_button_names_the_action_the_click_will_take() {
        assert_eq!(draw_button_label(false), "Draw Joint on Canvas");
        assert_eq!(draw_button_label(true), "Cancel Joint Drawing");
    }
}
