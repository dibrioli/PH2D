//! ⛔⛔ **AS EXCEPÇÕES SEM ALVO CHEGAM A PIXEL** (ADR-0164 / F5, critério 3).
//!
//! # Porque este gate mede GLIFOS, e não uma altura
//!
//! O critério pede que a excepção *«apareça na secção»*. Uma altura reservada (`y += line`) não é
//! pintura: o achado §4.2 da auditoria do `source.lsystem` mediu exactamente isso — um gate que
//! prometia *«a queixa chega a pixel»* lia o `content_h`, e **apagar a pintura inteira deixava-o
//! verde**. O arnês ganhou por causa disso um contador de glifos
//! ([`MockPanelHost::paint_and_count_geometry`]), e é ele que este ficheiro usa.
//!
//! ⚠️ **A régua tem de encher o balde nos dois sentidos:** mais órfãos ⇒ mais glifos, e um órfão
//! que NOMEIA a peça ⇒ mais glifos do que um que não a nomeia. *Uma régua que devolve o mesmo
//! número dos dois lados não distingue «não pintou» de «não vejo o que ele pintou».*

use ph2d_editor_core::screens::hero::{InspectorInstanceInfo, InspectorNameInfo, OrphanRow};
use ph2d_editor_core::zones::Rect;
use ph2d_panel_inspector::{
    InspectorPanel, InspectorState, set_current_inspector_instance, set_current_inspector_name,
};
use ph2d_ui_testkit::MockPanelHost;

const ENTITY: u64 = 0x5EED_00B1;
const ROOT: u64 = 0x5EED_00B2;
const VIEWPORT: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 320.0,
    h: 2400.0,
};

fn info(rows: Vec<OrphanRow>) -> InspectorInstanceInfo {
    InspectorInstanceInfo {
        entity_bits: ENTITY,
        master_name: "Ragdoll".into(),
        overridden: Vec::new(),
        orphan_rows: rows,
        root_bits: ROOT,
        is_variant: false,
        apply_levels: Vec::new(),
        apply_levels_beyond: 0,
    }
}

fn orphan(component: &str, piece: &str) -> OrphanRow {
    // ⚠️ A chave é derivada do NOME só para a fixtura ser legível — o que o produto lê dela é a
    // identidade, e o gate do `✕` exige que ela chegue ao barramento inteira.
    OrphanRow {
        component: component.into(),
        piece: piece.into(),
        piece_id: piece.len() as u64 + 1000,
        type_id: component.len() as u64 + 2000,
    }
}

/// Pinta o cartão com estes órfãos e devolve quantos glifos foram encomendados.
fn glyphs(rows: Vec<OrphanRow>) -> u32 {
    let mut host = MockPanelHost::with_panel::<InspectorPanel>();
    let mut state = InspectorState::default();
    set_current_inspector_name(Some(InspectorNameInfo {
        entity_bits: ENTITY,
        name: "Ragdoll".into(),
    }));
    set_current_inspector_instance(Some(info(rows)));
    let (g, _) = host.paint_and_count_geometry::<InspectorPanel>(&mut state, VIEWPORT);
    set_current_inspector_instance(None);
    set_current_inspector_name(None);
    g
}

/// ⭐⭐⭐ **Cada excepção sem alvo é uma LINHA que se vê** — e uma a mais dá mais glifos.
///
/// **Mutação que deve sangrar:** apagar o laço que pinta `info.orphan_rows` no cartão.
#[test]
fn every_unused_override_is_painted_as_its_own_line() {
    let none = glyphs(Vec::new());
    let one = glyphs(vec![orphan("Sprite", "Arm")]);
    let two = glyphs(vec![orphan("Sprite", "Arm"), orphan("Transform", "Leg")]);
    assert!(
        one > none,
        "um orfao nao acrescentou glifo nenhum: {none} -> {one} \u{2014} a linha nao chega a pixel"
    );
    assert!(
        two > one,
        "o segundo orfao nao acrescentou nada: {one} -> {two}"
    );
}

/// ⭐⭐ **E a linha NOMEIA a peça** — sem a metade que diz *de onde*, duas peças apagadas leem-se
/// `Sprite · Sprite` e o artista não sabe o que o botão *Clear* vai levar.
///
/// **Mutação que deve sangrar:** o `OrphanRow::label` devolver só o componente.
#[test]
fn the_line_names_the_piece_that_died() {
    let named = glyphs(vec![orphan("Sprite", "Arm")]);
    let nameless = glyphs(vec![orphan("Sprite", "")]);
    assert!(
        named > nameless,
        "nomear a peca nao mudou um glifo: {nameless} -> {named}"
    );
    assert_eq!(
        orphan("Sprite", "Arm").label(),
        "Sprite \u{2014} was on \u{201c}Arm\u{201d}"
    );
    assert_eq!(
        orphan("Sprite", "").label(),
        "Sprite",
        "sem peca a frase nao pode dizer que ela se chamava vazio"
    );
}

/// ⛔ **O botão promete o que a lista mostra** — o número dele é DERIVADO das linhas, e não uma
/// segunda contagem que discorda no dia em que uma entrada for saltada.
#[test]
fn the_clear_button_counts_exactly_the_lines_it_will_erase() {
    let i = info(vec![orphan("Sprite", "Arm"), orphan("Transform", "Leg")]);
    assert_eq!(i.orphans(), i.orphan_rows.len());
    assert_eq!(i.summary(), "Follows the component \u{b7} 2 unused");
}

/// ⛔⛔⛔ **O NOME DA PEÇA é do ARTISTA, e a linha dele avança um `line` fixo.**
///
/// A justificação escrita ao lado da conta da altura — *«elas são NOMES do catálogo, curtos por
/// construção»* — é verdadeira para as linhas de componente **overridado** (o mais longo do
/// catálogo tem 20 caracteres) e **falsa** para estas: `Sprite — was on "…"` embrulha um `Name` que
/// o artista escreveu, e um `Name` não tem tecto.
///
/// ⚠️ **O oráculo é onde o BOTÃO aterra.** Se a linha embrulha e a conta não a mede, o
/// *Clear N unused override(s)* fica pintado **por cima** da segunda linha do texto — que é
/// exactamente a foto que o Enio mandou em 2026-08-31 (*«Card com Labels emboladas»*), por outra
/// porta. Um nome mais longo tem de EMPURRAR o botão para baixo.
///
/// **Mutação que deve sangrar:** voltar o avanço das linhas de órfão para um `line` fixo.
#[test]
fn a_long_piece_name_pushes_the_clear_button_down_instead_of_painting_under_it() {
    let short = clear_button_y(vec![orphan("Sprite", "Arm")]);
    let long = clear_button_y(vec![orphan("Sprite", "Left front suspension arm assembly")]);
    assert!(
        long > short + 1.0,
        "o botao nao desceu com o nome longo ({short} -> {long}): a linha embrulhou e a conta da \
         altura nao a mediu, entao o botao esta' pintado por cima do texto"
    );
}

/// Onde o botão dos órfãos aterra, com estas linhas no cartão.
fn clear_button_y(rows: Vec<OrphanRow>) -> f32 {
    let mut host = MockPanelHost::with_panel::<InspectorPanel>();
    let mut state = InspectorState::default();
    set_current_inspector_name(Some(InspectorNameInfo {
        entity_bits: ENTITY,
        name: "Ragdoll".into(),
    }));
    set_current_inspector_instance(Some(info(rows)));
    let rects = host.paint::<InspectorPanel>(&mut state, VIEWPORT);
    set_current_inspector_instance(None);
    set_current_inspector_name(None);
    rects
        .iter()
        .find(|(id, _)| *id == ph2d_editor_core::ids::INSP_INSTANCE_CLEAR_ORPHANS)
        .map(|(_, r)| r.y)
        .expect("o botao dos orfaos tem de estar pintado")
}

/// ⭐⭐⭐ **O `✕` DE UMA LINHA larga AQUELA excepção** — e o que viaja é a CHAVE, nunca o índice.
///
/// ⚠️ A metade que importa é a segunda: a linha `1` de três tem de mandar a chave da linha `1`. Um
/// braço que mandasse o índice ficaria verde num cartão de uma linha só e escolheria a errada assim
/// que a lista crescesse — e o cartão é reconstruído a cada quadro.
///
/// **Mutação que deve sangrar:** o `drop_orphan_click` a mandar `orphan_rows[0]`, ou a sair do
/// `SINGLE_ID_CLICKS`.
#[test]
fn the_x_of_a_row_drops_that_rows_exception_by_key() {
    use ph2d_editor_core::action_bus::EditorAction;
    use ph2d_editor_core::interaction::WidgetEvent;

    let rows = vec![
        orphan("Sprite", "Arm"),
        orphan("Transform", "Wheel"),
        orphan("Collider", "Hub"),
    ];
    let want = rows[1].clone();
    let mut host = MockPanelHost::with_panel::<InspectorPanel>();
    let mut state = InspectorState::default();
    set_current_inspector_name(Some(InspectorNameInfo {
        entity_bits: ENTITY,
        name: "Ragdoll".into(),
    }));
    set_current_inspector_instance(Some(info(rows)));
    let rects = host.paint::<InspectorPanel>(&mut state, VIEWPORT);
    let id = ph2d_editor_core::ids::INSP_INSTANCE_DROP_ORPHAN[1];
    assert!(
        rects.iter().any(|(r, _)| *r == id),
        "o `x` da 2.a linha nao foi pintado nem hit-indexado"
    );
    let out = host.apply_panel_event::<InspectorPanel>(&mut state, WidgetEvent::Click(id));
    let drained = host.drained_actions();
    set_current_inspector_instance(None);
    set_current_inspector_name(None);

    assert_eq!(
        out,
        ph2d_editor_core::panel::EventOutcome::Consumed,
        "o clique nao foi consumido"
    );
    assert_eq!(
        drained,
        vec![EditorAction::InspectorDropUnusedOverride {
            root_bits: ROOT,
            piece: want.piece_id,
            type_id: want.type_id,
        }],
        "o `x` da linha 1 nao mandou a chave DELA"
    );
}

/// ⛔ **Acima do tecto da tabela de ids a linha continua a ver-se, e a AUSÊNCIA do botão é DITA.**
///
/// ⚠️ A lista não tem tecto de propósito (esconder linhas com um botão que apaga tudo seria
/// esconder exactamente o que o gesto destrói). O que tem tecto é o `✕`. ⇒ o cartão tem de dizer
/// quantas ficaram sem ele — *uma linha que perde o botão em silêncio lê-se como um botão morto.*
///
/// ⛔⛔ **A 1.ª redacção deste gate contava GLIFOS, e a mutação que apagava o aviso SOBREVIVEU:**
/// três linhas a mais acrescentam glifos por serem três linhas, tenham ou não o aviso por baixo.
/// *Uma régua que mede a população errada dá o mesmo número com e sem a cura.* ⇒ o oráculo passa a
/// ser o DESLOCAMENTO do botão: o aviso é uma linha, logo três órfãos a mais têm de empurrar o
/// botão **mais do que três linhas**.
///
/// **Mutação que deve sangrar:** não pintar a linha do `dropless`.
#[test]
fn the_rows_beyond_the_id_table_say_they_have_no_button() {
    let cap = ph2d_editor_core::ids::MAX_INSTANCE_ORPHAN_ROWS;
    let rows = |n: usize| -> Vec<OrphanRow> {
        (0..n).map(|i| orphan("Sprite", &format!("P{i}"))).collect()
    };
    // A altura de UMA linha, medida no próprio cartão — nenhum número escrito à mão.
    let one_row = clear_button_y(rows(2)) - clear_button_y(rows(1));
    assert!(one_row > 0.0, "duas linhas nao empurraram o botao");

    let moved = clear_button_y(rows(cap + 3)) - clear_button_y(rows(cap));
    assert!(
        moved > one_row * 3.5,
        "tres orfaos acima do tecto empurraram o botao {moved} (uma linha = {one_row}): sao as \
         tres linhas e mais NADA — o aviso de que elas ficaram sem `x` nao foi pintado"
    );
}
