//! ⭐⭐⭐ **A secção TAGS é PINTADA e está VIVA sob o dedo** (TOP-20 #9, W3a).
//!
//! # Porque este ficheiro existe
//!
//! O `every_painted_id_is_reachable` declara o alcance dele — *«a cena completa do lado SPRITE»* —
//! e deixa cada família nova ao seam dela. Sem este ficheiro, os chips, a caixa de escolha e o
//! `Create` eram pintados, hit-registados… e **mortos sob o dedo** se alguém esquecesse o
//! `populate_tags`: o despachante decide pelo `is_focusable`, e um id sem entrada no store não é
//! focável. *Um controlo nunca pintado e um morto sob o dedo dão o MESMO report.*
//!
//! # ⚠️ Por isso o gesto é REAL, e não um `WidgetEvent::Click` sintético
//!
//! Um `Click` construído à mão **pula a checagem de focabilidade** — ele passaria com o
//! `populate_tags` inteiro apagado. O `Down`+`Up` cai no rectângulo que a pintura registou e
//! atravessa a mesma cadeia que o dedo do artista.

use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::screens::hero::{InspectorTagRow, InspectorTagsInfo, TagsFieldEdit};
use ph2d_editor_core::zones::Rect;
use ph2d_host::{PointerButton, PointerEvent, PointerKind, PointerSource};
use ph2d_panel_inspector::{InspectorPanel, InspectorState, set_current_inspector_tags};
use ph2d_ui_testkit::MockPanelHost;

const VIEWPORT: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 1600.0,
    h: 900.0,
};
const SEC: u128 = 1_000_000_000;
const ENTITY: u64 = 0xBEEF_0009;

fn pointer(kind: PointerKind, x: f32, y: f32, t: u128) -> PointerEvent {
    PointerEvent {
        kind,
        x,
        y,
        button: PointerButton::Primary,
        source: PointerSource::Mouse,
        pressure: 1.0,
        timestamp_ns: t,
    }
}

fn linha(id: u64, path: &str, label: &str, depth: usize) -> InspectorTagRow {
    InspectorTagRow {
        id,
        path: path.into(),
        label: label.into(),
        depth,
    }
}

/// A árvore da fixtura: `Boss` · `Enemy` · `Enemy/Flying` · `Prop`, com o objecto a ter as duas do
/// meio. ⚠️ **As tags do objecto e as do projecto são listas diferentes de propósito** — é isso que
/// deixa a caixa de escolha ter o que oferecer.
fn info(full: bool, selected_count: usize) -> InspectorTagsInfo {
    InspectorTagsInfo {
        entity_bits: ENTITY,
        on_object: vec![
            linha(2, "Enemy", "Enemy", 0),
            linha(3, "Enemy/Flying", "Flying", 1),
        ],
        all: vec![
            linha(1, "Boss", "Boss", 0),
            linha(2, "Enemy", "Enemy", 0),
            linha(3, "Enemy/Flying", "Flying", 1),
            linha(4, "Prop", "Prop", 0),
        ],
        full,
        selected_count,
    }
}

fn host(i: InspectorTagsInfo) -> (MockPanelHost, InspectorState) {
    let h = MockPanelHost::with_panel::<InspectorPanel>();
    set_current_inspector_tags(Some(i));
    (h, InspectorState::default())
}

/// Carrega no rect dado com o ponteiro REAL e devolve o que chegou ao barramento.
fn carrega(
    h: &mut MockPanelHost,
    st: &mut InspectorState,
    r: Rect,
    quem: &str,
) -> Vec<EditorAction> {
    let (cx, cy) = (r.x + r.w * 0.5, r.y + r.h * 0.5);
    let _ = h.drained_actions();
    let mut evs = h.dispatch_pointer_event(pointer(PointerKind::Down, cx, cy, SEC));
    evs.extend(h.dispatch_pointer_event(pointer(PointerKind::Up, cx, cy, SEC + SEC / 100)));
    assert!(
        !evs.is_empty(),
        "o ponteiro sobre `{quem}` nao virou evento nenhum — ele esta' desenhado e nao existe para \
         o dispatcher (falta o registo no populate_tags)"
    );
    for ev in evs {
        h.apply_panel_event::<InspectorPanel>(st, ev);
    }
    h.drained_actions()
}

fn rect_de(rects: &[(ph2d_a11y::NodeId, Rect)], id: ph2d_a11y::NodeId, quem: &str) -> Rect {
    rects
        .iter()
        .find(|(n, _)| *n == id)
        .map(|(_, r)| *r)
        .unwrap_or_else(|| panic!("`{quem}` nao foi PINTADO com area clicavel"))
}

/// ⭐⭐⭐ **Um chip por tag, e a caixa de escolha com o campo** — a metade que o report do `Timers`
/// cobrou.
///
/// **Mutação que deve sangrar:** tirar o `chips(...)` do corpo da secção.
#[test]
fn os_chips_e_a_caixa_de_escolha_sao_pintados() {
    let (mut h, mut st) = host(info(false, 1));
    let rects = h.paint::<InspectorPanel>(&mut st, VIEWPORT);
    for (id, quem) in [
        (ph2d_panel_inspector::ids::INSP_TAGS_CHIP[0], "chip 0"),
        (ph2d_panel_inspector::ids::INSP_TAGS_CHIP[1], "chip 1"),
        (ph2d_panel_inspector::ids::INSP_TAGS_PICK, "a caixa"),
        (ph2d_panel_inspector::ids::INSP_TAGS_NEW, "o campo"),
    ] {
        let r = rect_de(&rects, id, quem);
        assert!(r.w > 0.0 && r.h > 0.0, "`{quem}` sem area: {r:?}");
    }
    // ⛔ O terceiro chip NÃO é pintado: o objecto tem duas tags. Um chip por id do array, em vez de
    // um por tag, encheria a secção de pílulas vazias.
    assert!(
        !rects
            .iter()
            .any(|(n, _)| *n == ph2d_panel_inspector::ids::INSP_TAGS_CHIP[2]),
        "foi pintado um chip para uma tag que o objecto nao tem"
    );
    set_current_inspector_tags(None);
}

/// ⭐⭐⭐ **O `×` de um chip chega ao barramento como `Remove`** — com o gesto REAL.
///
/// ⚠️ **É o `×` e não a pílula**: o corpo do chip não é hit-registado, e é isso que impede um
/// clique distraído de desmarcar o objecto.
///
/// **Mutação que deve sangrar:** tirar o `INSP_TAGS_CHIP` do `populate_tags` · registar a pílula
/// inteira em vez do `close_rect`.
#[test]
fn o_x_de_um_chip_tira_a_tag_deste_objecto() {
    let (mut h, mut st) = host(info(false, 1));
    let rects = h.paint::<InspectorPanel>(&mut st, VIEWPORT);
    let r = rect_de(
        &rects,
        ph2d_panel_inspector::ids::INSP_TAGS_CHIP[1],
        "o x do chip 1",
    );
    // ⛔⛔ **O rect registado é o `×`, não a pílula** — e esta asserção é o que faltava quando a
    // mutação sobreviveu: carregar no CENTRO do rect acerta na tag tanto num caso como no outro,
    // logo o gesto sozinho não distingue os dois. O discriminador é a GEOMETRIA: o `×` é o quadrado
    // que o `Tag::close_rect` calcula, e a pílula é muitas vezes mais larga.
    let lado = (ph2d_tokens::ROW_H_PX * 0.7).clamp(10.0, 16.0);
    assert!(
        (r.w - lado).abs() < 0.01 && (r.h - lado).abs() < 0.01,
        "o rect registado do chip nao e' o `x` ({lado:.1} px quadrado) — e' {r:?}: a pilula inteira \
         apanha o clique, e um toque distraido no rotulo desmarcaria o objecto"
    );
    let acoes = carrega(&mut h, &mut st, r, "o x do chip 1");
    assert!(
        acoes.iter().any(|a| matches!(
            a,
            EditorAction::InspectorTagsEdit {
                entity_bits: ENTITY,
                edit: TagsFieldEdit::Remove(3),
            }
        )),
        "o x do segundo chip tinha de mandar `Remove(3)`; o que chegou foi {acoes:?}"
    );
    set_current_inspector_tags(None);
}

/// ⭐⭐ **A lista não oferece o que o objecto já tem** — oferecê-lo seria um gesto que o
/// `Tags::insert` recusa, e o painel nunca oferece o que vai ser recusado.
///
/// **Mutação que deve sangrar:** apagar o filtro `!info.on_object.iter().any(...)`.
#[test]
fn a_lista_nao_oferece_as_tags_que_o_objecto_ja_tem() {
    let i = info(false, 1);
    let opcoes = ph2d_panel_inspector::probe_tag_options(&i, "");
    let ids: Vec<u64> = opcoes.iter().map(|o| o.value).collect();
    assert_eq!(
        ids,
        vec![1, 4],
        "a lista devia oferecer so' `Boss` e `Prop` — o objecto ja' tem as outras duas"
    );
}

/// ⭐⭐⭐ **A busca DOBRA, e é a decisão do dono** — *«maiúscula não importa, letra acentuada não
/// importa»* —, e ela procura no CAMINHO inteiro.
///
/// ⚠️ **`BOSS` tem de achar `Boss`**, e procurar por uma família tem de trazer os membros dela.
///
/// **Mutação que deve sangrar:** trocar o `ph2d_label_fold::fold` por um `to_lowercase` (cai no
/// caso acentuado) · comparar contra `row.label` em vez de `row.path`.
#[test]
fn a_busca_dobra_e_procura_no_caminho_inteiro() {
    let mut i = info(false, 1);
    // Uma tag acentuada, e o objecto não a tem.
    i.all.push(linha(5, "Inimigo/Aéreo", "Aéreo", 1));
    let vals = |f: &str| -> Vec<u64> {
        ph2d_panel_inspector::probe_tag_options(&i, &ph2d_label_fold::fold(f))
            .iter()
            .map(|o| o.value)
            .collect()
    };
    assert_eq!(vals("BOSS"), vec![1], "a busca nao dobrou a maiuscula");
    assert_eq!(vals("aereo"), vec![5], "a busca nao dobrou o acento");
    // ⚠️ Procurar pela FAMÍLIA traz o membro dela, que é o que a comparação por caminho compra.
    assert_eq!(
        vals("inimigo"),
        vec![5],
        "procurar pela familia nao trouxe o membro — a busca esta' a olhar so' para o rotulo"
    );
}

/// ⛔⛔ **Um objecto cheio não mostra a caixa** — deixá-la ali a recusar cada escolha ensinaria que
/// o painel está avariado. O porquê fica no lugar dela.
///
/// **Mutação que deve sangrar:** apagar o `return` do ramo `info.full`.
#[test]
fn um_objecto_cheio_esconde_a_caixa_e_diz_porque() {
    let (mut h, mut st) = host(info(true, 1));
    let rects = h.paint::<InspectorPanel>(&mut st, VIEWPORT);
    for (id, quem) in [
        (ph2d_panel_inspector::ids::INSP_TAGS_PICK, "a caixa"),
        (ph2d_panel_inspector::ids::INSP_TAGS_NEW, "o campo"),
        (ph2d_panel_inspector::ids::INSP_TAGS_CREATE, "o Create"),
    ] {
        assert!(
            !rects.iter().any(|(n, _)| *n == id),
            "`{quem}` foi pintado para um objecto que ja' nao aceita tags"
        );
    }
    // ⚠️ E os chips FICAM — é por eles que o artista tira uma para caber outra.
    assert!(
        rects
            .iter()
            .any(|(n, _)| *n == ph2d_panel_inspector::ids::INSP_TAGS_CHIP[0]),
        "os chips sumiram com o objecto cheio — nao haveria por onde tirar uma"
    );
    set_current_inspector_tags(None);
}

/// ⛔ **O `Create` só existe quando há um nome que ainda não é de ninguém.**
///
/// ⚠️ Com o campo VAZIO ele não é pintado — um botão que consome o clique sem fazer nada lê-se como
/// avaria.
///
/// **Mutação que deve sangrar:** pintar o `Create` incondicionalmente.
#[test]
fn o_create_nao_existe_sem_um_nome_novo() {
    let (mut h, mut st) = host(info(false, 1));
    let rects = h.paint::<InspectorPanel>(&mut st, VIEWPORT);
    assert!(
        !rects
            .iter()
            .any(|(n, _)| *n == ph2d_panel_inspector::ids::INSP_TAGS_CREATE),
        "o `Create` foi pintado com o campo vazio"
    );
    set_current_inspector_tags(None);
}

/// ⭐⭐⭐ **Escolher uma tag da LISTA ABERTA chega ao barramento** — com o gesto REAL, sobre o
/// popover que o passe diferido pinta.
///
/// # Porque este gate existe
///
/// ⛔ Era o ÚNICO gesto desta secção sem gate de gesto: os outros dois (o `×` e o `Create`) pintam
/// no corpo da secção, e este vive num passe que corre DEPOIS de todas elas, com o rect a viajar
/// por um slot. *Uma opção pintada no sítio certo e ausente do `populate` morre sob o dedo em
/// silêncio*, e nenhum gate sobre `pick_options` (que é uma função pura) o veria.
///
/// **Mutações que devem sangrar:** tirar o `INSP_TAGS_OPT` do `populate_tags` · apagar o braço das
/// TAGS do passe diferido em `popovers.rs` (a lista deixa de ser pintada, logo não há rect).
#[test]
fn escolher_uma_tag_da_lista_aberta_chega_ao_barramento() {
    let (mut h, mut st) = host(info(false, 1));
    let _ = h.paint::<InspectorPanel>(&mut st, VIEWPORT);
    h.set_dropdown_open(ph2d_panel_inspector::ids::INSP_TAGS_PICK, true);
    let rects = h.paint::<InspectorPanel>(&mut st, VIEWPORT);

    // A 1.ª opção é a `Boss` — a fixtura oferece `Boss` e `Prop` (as outras duas já estão no
    // objecto), pela ordem da árvore.
    let r = rect_de(
        &rects,
        ph2d_panel_inspector::ids::INSP_TAGS_OPT[0],
        "a 1.a opcao da lista",
    );
    let acoes = carrega(&mut h, &mut st, r, "a 1.a opcao da lista");
    assert!(
        acoes.iter().any(|a| matches!(
            a,
            EditorAction::InspectorTagsEdit {
                entity_bits: ENTITY,
                edit: TagsFieldEdit::Add(1),
            }
        )),
        "escolher a 1.a opcao tinha de mandar `Add(1)` (a `Boss`); o que chegou foi {acoes:?}"
    );
    set_current_inspector_tags(None);
}
