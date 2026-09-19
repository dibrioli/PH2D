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

use ph2d_editor_core::TagsFieldEdit;
use ph2d_editor_core::action_bus::{ComponentEdit, EditorAction};
use ph2d_editor_core::screens::hero::{InspectorTagRow, InspectorTagsInfo};
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

/// A ÁRVORE DO PROJECTO: `Boss` · `Enemy` · `Enemy/Flying` · `Prop`.
///
/// ⚠️ **Ela é a fixtura de OUTRA porta** (`set_current_tag_tree`), e não um campo do instantâneo do
/// objecto — é isso que deixa a caixa de escolha ter o que oferecer a um objecto que ainda não tem
/// `Tags` nenhum, que é o caso normal da secção *Signal Actions*.
fn arvore() -> Vec<InspectorTagRow> {
    vec![
        linha(1, "Boss", "Boss", 0),
        linha(2, "Enemy", "Enemy", 0),
        linha(3, "Enemy/Flying", "Flying", 1),
        linha(4, "Prop", "Prop", 0),
    ]
}

/// O objecto da fixtura: tem as duas tags do meio da árvore.
fn info(full: bool, selected_count: usize) -> InspectorTagsInfo {
    InspectorTagsInfo {
        entity_bits: ENTITY,
        on_object: vec![
            linha(2, "Enemy", "Enemy", 0),
            linha(3, "Enemy/Flying", "Flying", 1),
        ],
        full,
        selected_count,
    }
}

/// ⚠️ **As DUAS portas são semeadas**, e é isso que o desenho novo obriga: a lista que a caixa
/// oferece já não vem do instantâneo do objecto.
fn host(i: InspectorTagsInfo) -> (MockPanelHost, InspectorState) {
    let h = MockPanelHost::with_panel::<InspectorPanel>();
    set_current_inspector_tags(Some(i));
    ph2d_panel_inspector::set_current_tag_tree(arvore());
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
    ph2d_panel_inspector::set_current_tag_tree(Vec::new());
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
            EditorAction::InspectorComponentEdit {
                entity_bits: ENTITY,
                edit: ComponentEdit::Tags(TagsFieldEdit::Remove(3)),
            }
        )),
        "o x do segundo chip tinha de mandar `Remove(3)`; o que chegou foi {acoes:?}"
    );
    set_current_inspector_tags(None);
    ph2d_panel_inspector::set_current_tag_tree(Vec::new());
}

/// ⭐⭐ **A lista não oferece o que o objecto já tem** — oferecê-lo seria um gesto que o
/// `Tags::insert` recusa, e o painel nunca oferece o que vai ser recusado.
///
/// **Mutação que deve sangrar:** apagar o filtro `!info.on_object.iter().any(...)`.
#[test]
fn a_lista_nao_oferece_as_tags_que_o_objecto_ja_tem() {
    let i = info(false, 1);
    let opcoes = ph2d_panel_inspector::probe_tag_options(&arvore(), &i.on_object, "");
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
    let i = info(false, 1);
    // Uma tag acentuada NA ÁRVORE, e o objecto não a tem.
    let mut arv = arvore();
    arv.push(linha(5, "Inimigo/Aéreo", "Aéreo", 1));
    let vals = |f: &str| -> Vec<u64> {
        ph2d_panel_inspector::probe_tag_options(&arv, &i.on_object, &ph2d_label_fold::fold(f))
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
    ph2d_panel_inspector::set_current_tag_tree(Vec::new());
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
    ph2d_panel_inspector::set_current_tag_tree(Vec::new());
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
            EditorAction::InspectorComponentEdit {
                entity_bits: ENTITY,
                edit: ComponentEdit::Tags(TagsFieldEdit::Add(1)),
            }
        )),
        "escolher a 1.a opcao tinha de mandar `Add(1)` (a `Boss`); o que chegou foi {acoes:?}"
    );
    set_current_inspector_tags(None);
    ph2d_panel_inspector::set_current_tag_tree(Vec::new());
}

// ─── O ALVO POR TAG de uma SIGNAL ACTION (W3b) ───────────────────────────────────────────────
//
// ⚠️ **Vivem NESTE ficheiro e não no da secção Actions**, e a razão é o assunto: o que se prova
// aqui é que a ÁRVORE DE TAGS chega a uma segunda superfície e está viva lá. O seam do verbo e das
// linhas continua onde estava.

use ph2d_editor_core::screens::hero::{ActionFieldEdit, InspectorActionInfo, InspectorActionRow};
use ph2d_panel_inspector::set_current_inspector_action;

fn accao(target_tag: Option<u64>, path: &str) -> InspectorActionInfo {
    InspectorActionInfo {
        entity_bits: ENTITY,
        rows: vec![InspectorActionRow {
            on: "alarme".into(),
            target: "Parede".into(),
            verb_tag: 0,
            arg: String::new(),
            uses_arg: false,
            // ⚠️ O modo sai do PAYLOAD nesta fixtura — ela é sobre a tag, e um modo escrito à mão
            // ao lado de uma tag presente daria dois estados para a mesma linha.
            target_mode: if target_tag.is_some() { 1 } else { 0 },
            from_tag: 0,
            target_tag,
            target_tag_path: path.into(),
        }],
        verb_labels: vec!["Show".into(), "Hide".into()],
        selected_count: 1,
    }
}

/// A secção *Signal Actions* pintada, com a árvore do projecto semeada.
fn host_accao(i: InspectorActionInfo) -> (MockPanelHost, InspectorState) {
    let h = MockPanelHost::with_panel::<InspectorPanel>();
    set_current_inspector_action(Some(i));
    ph2d_panel_inspector::set_current_tag_tree(arvore());
    (h, InspectorState::default())
}

/// ⭐⭐⭐ **O segmentado `Name | Tag` vira o alvo da acção** — com o gesto REAL.
///
/// **Mutação que deve sangrar:** tirar o `INSP_ACTION_BY_TAG` do `populate_action` · apagar o braço
/// dos segmentos no `event_action`.
#[test]
fn o_segmentado_vira_o_alvo_da_accao_para_tag() {
    let (mut h, mut st) = host_accao(accao(None, ""));
    let rects = h.paint::<InspectorPanel>(&mut st, VIEWPORT);
    let r = rect_de(
        &rects,
        ph2d_panel_inspector::ids::INSP_ACTION_BY_TAG,
        "o segmento Tag",
    );
    let acoes = carrega(&mut h, &mut st, r, "o segmento Tag");
    assert!(
        acoes.iter().any(|a| matches!(
            a,
            EditorAction::InspectorActionEdit {
                edit: ActionFieldEdit::TargetMode(0, 1),
                ..
            }
        )),
        "o segmento `Tag` tinha de mandar `TargetMode(0, 1)`; o que chegou foi {acoes:?}"
    );
    set_current_inspector_action(None);
    ph2d_panel_inspector::set_current_tag_tree(Vec::new());
}

/// ⛔⛔ **Um modo, UM controlo** — o campo do NOME e a caixa da TAG nunca aparecem os dois.
///
/// ⚠️ *Com os dois à vista, «a quem?» teria duas respostas escritas ao mesmo tempo* — o que o doc do
/// `SignalTarget` recusa no modelo, recusado aqui na tela.
///
/// **Mutação que deve sangrar:** o `if !por_tag { return … }` do `target_rows` apagado.
#[test]
fn um_modo_um_controlo_o_nome_e_a_tag_nunca_aparecem_juntos() {
    let (mut h, mut st) = host_accao(accao(None, ""));
    let por_nome = h.paint::<InspectorPanel>(&mut st, VIEWPORT);
    assert!(
        por_nome
            .iter()
            .any(|(n, _)| *n == ph2d_panel_inspector::ids::INSP_ACTION_TARGET),
        "o modo NOME nao pintou o campo do nome"
    );
    assert!(
        !por_nome
            .iter()
            .any(|(n, _)| *n == ph2d_panel_inspector::ids::INSP_ACTION_TAG_PICK),
        "o modo NOME pintou TAMBEM a caixa da tag"
    );

    let (mut h, mut st) = host_accao(accao(Some(2), "Enemy"));
    let por_tag = h.paint::<InspectorPanel>(&mut st, VIEWPORT);
    assert!(
        por_tag
            .iter()
            .any(|(n, _)| *n == ph2d_panel_inspector::ids::INSP_ACTION_TAG_PICK),
        "o modo TAG nao pintou a caixa da tag"
    );
    assert!(
        !por_tag
            .iter()
            .any(|(n, _)| *n == ph2d_panel_inspector::ids::INSP_ACTION_TARGET),
        "o modo TAG pintou TAMBEM o campo do nome — «a quem?» com duas respostas na tela"
    );
    set_current_inspector_action(None);
    ph2d_panel_inspector::set_current_tag_tree(Vec::new());
}

/// ⭐⭐⭐ **Escolher a tag alvo na lista aberta chega ao barramento** — o gesto que vive no passe
/// diferido, com ids PRÓPRIOS.
///
/// ⛔ **É aqui que se prova que os ids não são os da secção *Tags***: com o mesmo array, o braço
/// daquela secção corre primeiro no router e o clique marcaria o objecto em vez de apontar a acção.
///
/// **Mutação que deve sangrar:** o `INSP_ACTION_TAG_OPT` trocado pelo `INSP_TAGS_OPT` · o braço da
/// tag alvo apagado do passe diferido.
#[test]
fn escolher_a_tag_alvo_aponta_a_accao_e_nao_marca_o_objecto() {
    let (mut h, mut st) = host_accao(accao(Some(0), ""));
    let _ = h.paint::<InspectorPanel>(&mut st, VIEWPORT);
    h.set_dropdown_open(ph2d_panel_inspector::ids::INSP_ACTION_TAG_PICK, true);
    let rects = h.paint::<InspectorPanel>(&mut st, VIEWPORT);
    // A 1.ª opção é a `Boss` — aqui a lista é a árvore INTEIRA, sem tirar nada.
    let r = rect_de(
        &rects,
        ph2d_panel_inspector::ids::INSP_ACTION_TAG_OPT[0],
        "a 1.a opcao da tag alvo",
    );
    let acoes = carrega(&mut h, &mut st, r, "a 1.a opcao da tag alvo");
    assert!(
        acoes.iter().any(|a| matches!(
            a,
            EditorAction::InspectorActionEdit {
                edit: ActionFieldEdit::TargetTag(0, 1),
                ..
            }
        )),
        "escolher a 1.a opcao tinha de apontar a ACCAO a' `Boss`; o que chegou foi {acoes:?}"
    );
    assert!(
        !acoes.iter().any(|a| matches!(
            a,
            EditorAction::InspectorComponentEdit {
                edit: ComponentEdit::Tags(_),
                ..
            }
        )),
        "o clique na tag ALVO marcou o objecto — os dois selectores partilham ids"
    );
    set_current_inspector_action(None);
    ph2d_panel_inspector::set_current_tag_tree(Vec::new());
}

// ─── A CERCA e O OUTRO LADO da mesma secção (suplente #24, 2026-09-19) ────────────────────────
//
// ⚠️ **Vivem NESTE ficheiro e não num irmão novo**, e a razão é o ARNÊS: o `host_accao`, o
// `carrega` e o `rect_de` acima são exactamente o que estes gates precisam, e uma segunda cópia
// deles divergiria em silêncio — a lei que esta casa escreve sobre funções com dois chamadores.
// O assunto vizinho paga a companhia: os três segmentos do *a quem?* e os dois do *de quem?* são o
// MESMO widget, na MESMA linha da secção.

/// ⭐⭐⭐ **O segmento «Who Hit» vira o alvo para QUEM BATEU** — com o gesto REAL.
///
/// ⚠️ **É o 3.º modo, e a posição É a tag**: um `TargetMode(0, 1)` aqui apontaria a acção a uma TAG
/// em vez de ao outro lado do contacto, e compila.
///
/// **Mutações que devem sangrar:** tirar o `INSP_ACTION_BY_OTHER` do `populate_action` · apagar o
/// braço dele no `event_action` · trocar a ordem do `ActionTargetMode::ALL`.
#[test]
fn o_segmentado_vira_o_alvo_da_accao_para_quem_bateu() {
    let (mut h, mut st) = host_accao(accao(None, ""));
    let rects = h.paint::<InspectorPanel>(&mut st, VIEWPORT);
    let r = rect_de(
        &rects,
        ph2d_panel_inspector::ids::INSP_ACTION_BY_OTHER,
        "o segmento Who Hit",
    );
    let acoes = carrega(&mut h, &mut st, r, "o segmento Who Hit");
    assert!(
        acoes.iter().any(|a| matches!(
            a,
            EditorAction::InspectorActionEdit {
                edit: ActionFieldEdit::TargetMode(0, 2),
                ..
            }
        )),
        "o segmento `Who Hit` tinha de mandar `TargetMode(0, 2)`; o que chegou foi {acoes:?}"
    );
    set_current_inspector_action(None);
    ph2d_panel_inspector::set_current_tag_tree(Vec::new());
}

/// ⭐⭐⭐ **A CERCA chega ao barramento pelos DOIS segmentos** — e é ela que faz um tiro atingir um
/// inimigo e não os dez (medido: `10` efeitos para um sinal, antes desta wave).
///
/// ⚠️ **Os dois no mesmo gate**, porque o defeito de um lado é invisível do outro: um `From Anyone`
/// que mandasse a tag errada devolveria a cerca ao default e lia-se como *«o clique não fez nada»*.
///
/// **Mutações que devem sangrar:** tirar qualquer um dos dois do `populate_action` · trocar as
/// tags no braço do `event_action`.
#[test]
fn a_cerca_chega_ao_barramento_pelos_dois_segmentos() {
    for (id, tag, quem) in [
        (
            ph2d_panel_inspector::ids::INSP_ACTION_FROM_MYSELF,
            1_u8,
            "o segmento From Myself",
        ),
        (
            ph2d_panel_inspector::ids::INSP_ACTION_FROM_ANYONE,
            0_u8,
            "o segmento From Anyone",
        ),
    ] {
        let (mut h, mut st) = host_accao(accao(None, ""));
        let rects = h.paint::<InspectorPanel>(&mut st, VIEWPORT);
        let r = rect_de(&rects, id, quem);
        let acoes = carrega(&mut h, &mut st, r, quem);
        assert!(
            acoes.iter().any(|a| matches!(
                a,
                EditorAction::InspectorActionEdit {
                    edit: ActionFieldEdit::From(0, t),
                    ..
                } if *t == tag
            )),
            "{quem} tinha de mandar `From(0, {tag})`; o que chegou foi {acoes:?}"
        );
        set_current_inspector_action(None);
        ph2d_panel_inspector::set_current_tag_tree(Vec::new());
    }
}
