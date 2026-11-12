//! Gates da paleta em LISTA — pela porta da busca ([`scoped`]), do store e do pintor.

use super::super::{PaletteSelect, WidgetStore, item_ids, look, paint as paint_palette, scoped};
use super::*;

const VP: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 1187.0,
    h: 953.0,
};

fn id(n: &str) -> NodeId {
    ph2d_tool_registry::hash_node_id_runtime(n)
}

fn item(n: &str) -> PaletteItem {
    PaletteItem {
        badge: None,
        label: n.into(),
        id: id(n),
        keywords: "",
    }
}

/// Tools (Select, Move) e File (Save com atalho; a fatia Export com dois comandos).
fn model(list: bool, recent: &[&str]) -> PaletteModel {
    let group = |title: &str, subs: Vec<PaletteSub>| PaletteGroup {
        title: title.into(),
        color: ColorToken::NodeCatSource,
        subs,
    };
    let sub = |title: Option<&str>, items: &[&str]| PaletteSub {
        title: title.map(str::to_owned),
        items: items.iter().map(|n| item(n)).collect(),
    };
    PaletteModel {
        title: "Commands".into(),
        groups: vec![
            group("Tools", vec![sub(None, &["Select", "Move"])]),
            group(
                "File",
                vec![
                    sub(None, &["Save\tCtrl+S"]),
                    sub(
                        Some("Export"),
                        &["Export Sheet\u{2026}", "Export Video\u{2026}"],
                    ),
                ],
            ),
        ],
        toggle: None,
        tips: Vec::new(),
        select: list.then(|| PaletteSelect {
            initial: id("Select"),
            teach: Vec::new(),
        }),
        layout: if list {
            PaletteLayout::List {
                recent: recent.iter().map(|n| id(n)).collect(),
            }
        } else {
            PaletteLayout::Cloud
        },
    }
}

/// `(rótulo, caminho)` de cada linha de item, na ordem em que se pinta.
fn lines(m: &PaletteModel, query: &str) -> Vec<(String, String)> {
    let view = scoped(m, query, None);
    rows(&view.model)
        .into_iter()
        .filter_map(|r| match r {
            Row::Item(it, path) => Some((it.label.clone(), path.unwrap_or("").to_owned())),
            Row::Head(_) => None,
        })
        .collect()
}

fn painted(store: &WidgetStore) -> Vec<(NodeId, Rect)> {
    let mut scene = VectorScene::new();
    let mut ts = TextSystem::new();
    let mut hits = HitIndex::new();
    let motion = crate::motion::UiMotion::default();
    let m = store.command_palette_model().expect("aberta").clone();
    let query = store.command_palette_query().to_owned();
    let _ = paint_palette(
        &mut scene,
        &mut ts,
        Theme::default(),
        &mut hits,
        &m,
        &query,
        VP,
        &motion,
        store,
    );
    let ids = item_ids(&m);
    hits.iter_registrations()
        .filter(|(id, _)| ids.contains(id))
        .collect()
}

/// ⭐ **«Recent» em cima: o mais novo primeiro, sem repetir, e o que o modelo não tem não aparece.**
///
/// (Mutações: não tirar os recentes dos grupos ⇒ RED no «sem repetir»; não filtrar pelo modelo ⇒
/// RED no «Gone».)
#[test]
fn os_recentes_ficam_em_cima_sem_repetir_e_somem_se_o_item_nao_existe() {
    let m = model(true, &["Export Video\u{2026}", "Gone", "Move"]);
    let view = scoped(&m, "", None);
    assert_eq!(view.model.groups[0].title, "Recent");
    let got = lines(&m, "");
    let names: Vec<&str> = got.iter().map(|(n, _)| n.as_str()).collect();
    assert_eq!(
        names,
        [
            "Export Video\u{2026}",
            "Move",
            "Select",
            "Save\tCtrl+S",
            "Export Sheet\u{2026}"
        ]
    );
    // Toda linha diz o caminho INTEIRO, no Recent e dentro do grupo (dono 12/11).
    assert_eq!(got[0].1, "File \u{00b7} Export");
    assert_eq!(got[1].1, "Tools");
    assert_eq!(got[4].1, "File \u{00b7} Export");
    // Sem recentes, nenhuma secção «Recent».
    let plain = model(true, &[]);
    assert_eq!(scoped(&plain, "", None).model.groups[0].title, "Tools");
    // No máximo RECENT_SHOWN.
    let mut many = model(true, &[]);
    many.groups[0].subs[0].items = (0..9).map(|i| item(&format!("T{i}"))).collect();
    many.layout = PaletteLayout::List {
        recent: (0..9).map(|i| id(&format!("T{i}"))).collect(),
    };
    assert_eq!(
        scoped(&many, "", None).model.groups[0].subs.len(),
        RECENT_SHOWN
    );
}

/// ⭐ **Digitando, a lista é PLANA, com o caminho inteiro; os recentes que casam vêm primeiro; a
/// contagem é «n of m».**
///
/// (Mutações: ordenar sem a idade ⇒ RED; manter os cabeçalhos ⇒ RED no título vazio.)
#[test]
fn a_busca_e_uma_lista_plana_com_o_caminho_e_a_contagem() {
    let m = model(true, &["Export Video\u{2026}"]);
    let view = scoped(&m, "exp", None);
    assert_eq!(view.model.groups.len(), 1);
    assert!(
        view.model.groups[0].title.is_empty(),
        "lista plana, sem cabeçalho"
    );
    assert_eq!(
        lines(&m, "exp"),
        [
            (
                "Export Video\u{2026}".to_owned(),
                "File \u{00b7} Export".to_owned()
            ),
            (
                "Export Sheet\u{2026}".to_owned(),
                "File \u{00b7} Export".to_owned()
            ),
        ]
    );
    assert_eq!(
        view.top,
        Some(id("Export Video\u{2026}")),
        "o Enter é a 1.ª linha"
    );
    assert_eq!((view.model.item_count(), m.item_count()), (2, 5));
    assert_eq!(
        ph2d_i18n::tr_with("chrome.palette.n_of_m", &[("n", &2usize), ("m", &5usize)]),
        "2 of 5"
    );
    // O caminho também acha: «file» traz tudo o que mora em File.
    assert_eq!(lines(&m, "file").len(), 3);
    // O nome ganha do caminho: «save» é a primeira linha.
    assert_eq!(lines(&m, "sav")[0].0, "Save\tCtrl+S");
    assert!(scoped(&m, "zzzz", None).model.groups.is_empty());
}

/// ⭐ **↑ ↓ andam linha a linha pela ordem pintada, o Enter corre a destacada, e a busca leva o
/// destaque à primeira linha.**
#[test]
fn as_setas_andam_linha_a_linha_e_o_enter_corre_a_destacada() {
    let mut store = WidgetStore::with_capacity(4);
    store.open_command_palette(model(true, &["Move"]));
    let order = item_ids(&scoped(store.command_palette_model().unwrap(), "", None).model);
    assert_eq!(order[0], id("Move"), "o recente é a 1.ª linha");
    store.highlight_command_palette(order[0]);
    for want in &order[1..] {
        store.step_command_palette_highlight(1);
        assert_eq!(store.command_palette_highlight(), Some(*want));
    }
    store.step_command_palette_highlight(1);
    assert_eq!(
        store.command_palette_highlight(),
        order.last().copied(),
        "preso ao fim"
    );
    assert_eq!(store.command_palette_enter_target(), order.last().copied());
    store.step_command_palette_highlight(-99);
    assert_eq!(store.command_palette_highlight(), Some(order[0]));
    for c in "video".chars() {
        store.command_palette_push_char(c);
    }
    assert_eq!(
        store.command_palette_enter_target(),
        Some(id("Export Video\u{2026}"))
    );
}

/// ⭐ **A linha selecionada rola para ficar à vista** — e a que já está à vista não mexe na roda.
#[test]
fn a_linha_destacada_rola_para_ficar_a_vista() {
    let m = model(true, &[]);
    let view = scoped(&m, "", None);
    let (y0, h0) = row_span(&view.model, id("Select")).unwrap();
    assert_eq!(y0, 0.0, "a 1.ª linha leva o cabeçalho dela");
    assert_eq!(h0, head_h() + ROW_H_PX);
    let last = row_span(&view.model, id("Export Video\u{2026}")).unwrap();
    assert_eq!(last.0 + last.1, content_h(&view.model));
    assert_eq!(last.1, ROW_H_PX);
    let view_h = ROW_H_PX * 2.0;
    let down = follow_scroll(last, 0.0, view_h);
    assert_eq!(
        down,
        content_h(&view.model) - view_h,
        "desceu até a linha caber"
    );
    assert_eq!(follow_scroll(last, down, view_h), down, "à vista: não mexe");
    assert_eq!(follow_scroll((y0, h0), down, view_h), 0.0, "subiu ao topo");
    assert_eq!(row_span(&view.model, id("Gone")), None);
}

/// ⭐⭐ **A lista pinta UMA linha por item; as outras paletas continuam em nuvem.**
///
/// (Mutação: o `paint` a pintar sempre em lista ⇒ RED na metade da nuvem; `look` sempre o da lista
/// ⇒ RED na largura.)
#[test]
fn a_lista_e_uma_linha_por_item_e_as_outras_paletas_continuam_em_nuvem() {
    let mut store = WidgetStore::with_capacity(4);
    store.open_command_palette(model(true, &[]));
    let rects = painted(&store);
    assert_eq!(rects.len(), 5);
    for pair in rects.windows(2) {
        let (a, b) = (pair[0].1, pair[1].1);
        assert_eq!((a.x, a.w, a.h), (b.x, b.w, ROW_H_PX));
        assert!(b.y >= a.y + a.h, "duas linhas na mesma altura");
    }
    // A mesma paleta em nuvem: pílulas lado a lado, e o acabamento de sempre.
    store.open_command_palette(model(false, &[]));
    let cloud = painted(&store);
    assert_eq!(cloud.len(), 5);
    assert!(
        cloud.windows(2).any(|p| p[0].1.y == p[1].1.y),
        "a nuvem deixou de pôr pílulas na mesma linha"
    );
    let cloud_model = model(false, &[]);
    assert_eq!(look::look(&cloud_model), look::classic());
    assert_eq!(
        item_ids(&scoped(&cloud_model, "", None).model),
        item_ids(&cloud_model),
        "a nuvem não se rearruma"
    );
    assert!(!cloud_model.two_step() && !model(true, &[]).two_step());
    assert_eq!(look::look(&model(true, &[])).card_max_w, LIST_MAX_W);
}

/// ⭐ **Sem busca, cada linha leva o caminho dela e o atalho na coluna** — igual à lista filtrada
/// (dono 12/11: os títulos ficam, o caminho na linha é obrigatório).
///
/// (Mutação: devolver a `sub.title` crua no ramo vazio ⇒ RED nos «Tools» sem caminho.)
#[test]
fn sem_busca_cada_linha_leva_o_caminho_e_o_atalho() {
    let m = model(true, &[]);
    let all = lines(&m, "");
    assert_eq!(
        all,
        [
            ("Select".to_owned(), "Tools".to_owned()),
            ("Move".to_owned(), "Tools".to_owned()),
            ("Save\tCtrl+S".to_owned(), "File".to_owned()),
            (
                "Export Sheet\u{2026}".to_owned(),
                "File \u{00b7} Export".to_owned()
            ),
            (
                "Export Video\u{2026}".to_owned(),
                "File \u{00b7} Export".to_owned()
            ),
        ]
    );
    assert_eq!(split_shortcut(&all[2].0).1, Some("Ctrl+S"));
}

/// ⭐ **O rodapé separa as dicas por «·».**
#[test]
fn o_rodape_separa_as_dicas_por_ponto() {
    assert_eq!(
        footer_text(),
        "Enter to run \u{00b7} Alt+Enter to show where it lives \u{00b7} Up and Down to move \u{00b7} Esc to close"
    );
}

/// ⭐⭐ **A busca não salta de sítio**: o topo do cartão é o mesmo com a busca vazia e com texto,
/// colado por baixo da faixa; só a altura muda, para baixo. A nuvem continua centrada.
///
/// (Mutação: voltar a centrar a lista ⇒ RED no topo que mexe.)
#[test]
fn o_topo_da_lista_fica_preso_por_baixo_da_faixa() {
    let mut ts = TextSystem::new();
    let m = model(true, &[]);
    let top = |query: &str, ts: &mut TextSystem| {
        let view = scoped(&m, query, None);
        let none = view.model.groups.is_empty();
        let mt = super::super::scroll::metrics(ts, &view.model, VP, none, query);
        mt.card
    };
    let (empty, typed) = (top("", &mut ts), top("exp", &mut ts));
    assert_eq!(empty.y, VP.y + super::super::scroll::list_top());
    assert_eq!(typed.y, empty.y, "o topo mexeu ao digitar");
    assert!(typed.h < empty.h, "a altura encolhe para baixo");
    assert_eq!(
        empty.x + empty.w * 0.5,
        VP.x + VP.w * 0.5,
        "centrada no campo"
    );
    let cloud = model(false, &[]);
    let mc = super::super::scroll::metrics(&mut ts, &cloud, VP, false, "");
    assert_eq!(
        mc.card.y,
        VP.y + (VP.h - mc.card.h) * 0.5,
        "a nuvem fica centrada"
    );
}

/// ⭐ **A última linha não sai cortada a meio**: só se pinta (e se clica) a linha inteira dentro
/// do corpo.
#[test]
fn a_linha_que_nao_cabe_inteira_nao_se_pinta() {
    let m = model(true, &[]);
    let view = scoped(&m, "", None);
    let mut scene = VectorScene::new();
    let mut ts = TextSystem::new();
    let mut hit = HitIndex::new();
    let body = Rect::new(0.0, 0.0, 400.0, head_h() + ROW_H_PX * 1.5);
    paint_rows(
        &mut scene,
        &mut ts,
        Theme::default(),
        &mut hit,
        &view.model,
        (0.0, 0.0, 400.0),
        body,
        [None, None],
    );
    assert!(hit.rect_for(id("Select")).is_some());
    assert!(hit.rect_for(id("Move")).is_none(), "meia linha pintada");
}
