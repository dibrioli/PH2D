//! ⭐⭐⭐ **AS TRÊS PERGUNTAS SOBRE UM CONTROLO, no painel que o artista está a usar.**
//!
//! # Porque este ficheiro existe
//!
//! Este repo tem instrumentos afiados para **duas** perguntas sobre um controlo:
//!
//! 1. *está sob o dedo?* — o `HitIndex` recebeu o rectângulo dele (os gates de paint);
//! 2. *o clique chega a um efeito?* — o `apply_event` tem um braço (os `seam_*`).
//!
//! E **nenhum** para a terceira, que é *está no sítio certo?* — foi o report do Enio de
//! 2026-09-02 (*«layout ruim»*), com oito gates verdes sobre a foto do defeito.
//!
//! ⛔⛔ **Há uma quarta, e é a que apanhou o `✕` da faixa de relação:** um id pode estar no
//! `HitIndex` e **não ter `InteractiveState` no store** ⇒ `is_focusable` falso, o `Down` não arma, e
//! o `Click` **nunca nasce**. Os gates de `apply_event` não a vêem, porque um `Click` sintético num
//! teste **não passa pelo store** — ele entra pela porta que o produto só alcança se essa condição
//! já estiver satisfeita.
//!
//! ⇒ este ficheiro corre as quatro **sobre tudo o que o painel de facto pintou**, em vez de sobre
//! uma lista escrita à mão. *A população vem do paint; uma lista aqui envelheceria no primeiro
//! controlo novo.*

use ph2d_editor::interaction::WidgetEvent;
use ph2d_editor::panel::{EventOutcome, PanelHostInternal};
use ph2d_editor::zones::Rect;
use ph2d_panel_asset_browser::state::AssetBrowserState;
use ph2d_panel_asset_browser::{AssetBrowserPanel, PANEL_ID, ids};
use ph2d_ui_testkit::MockPanelHost;

/// ⭐⭐ **O NOME de um id**, para que uma acusação seja accionável.
///
/// ⛔ A 1.ª versão destes gates imprimia `NodeId(2868002753322716995)`. *Um censo que acusa e não
/// nomeia obriga quem o lê a redescobrir a tabela — e a maioria desiste e apaga o gate.* A lista
/// vem dos `const` do painel, e o que ela não conhece sai como hash **com o aviso de que é
/// desconhecido**, que é a única forma honesta de não mentir sobre o que se sabe.
fn name_of(id: ph2d_editor::NodeId) -> String {
    for (n, k) in [
        ("ASSET_PANEL", ids::ASSET_PANEL),
        ("ASSET_SEARCH", ids::ASSET_SEARCH),
        ("ASSET_SIZE", ids::ASSET_SIZE),
        ("ASSET_CLOSE", ids::ASSET_CLOSE),
        ("ASSET_DRAG_HANDLE", ids::ASSET_DRAG_HANDLE),
        ("ASSET_RESIZE_HANDLE_BL", ids::ASSET_RESIZE_HANDLE_BL),
        ("ASSET_CATALOG_TOGGLE", ids::ASSET_CATALOG_TOGGLE),
        ("ASSET_CATALOG_NEW", ids::ASSET_CATALOG_NEW),
        ("ASSET_CATALOG_ALL", ids::ASSET_CATALOG_ALL),
        ("ASSET_CATALOG_UNASSIGNED", ids::ASSET_CATALOG_UNASSIGNED),
        ("ASSET_CATALOG_COL", ids::ASSET_CATALOG_COL),
        ("ASSET_CATALOG_RENAME", ids::ASSET_CATALOG_RENAME),
        ("ASSET_RELATED_CLEAR", ids::ASSET_RELATED_CLEAR),
        ("SCROLLBAR", ph2d_editor::widget::ASSET_BROWSER_SCROLLBAR_ID),
    ] {
        if id == k {
            return n.to_string();
        }
    }
    for (i, k) in ids::ASSET_KIND.iter().enumerate() {
        if id == *k {
            return format!("ASSET_KIND[{i}]");
        }
    }
    for (i, k) in ids::ASSET_SORT.iter().enumerate() {
        if id == *k {
            return format!("ASSET_SORT[{i}]");
        }
    }
    if let Some(i) = (0..ids::MAX_ASSET_CELLS).find(|i| ids::asset_cell_id(*i) == id) {
        return format!("cartao[{i}]");
    }
    if let Some(i) = (0..ids::MAX_CATALOG_ROWS).find(|i| ids::catalog_row_id(*i) == id) {
        return format!("linha de catalogo[{i}]");
    }
    format!("DESCONHECIDO {id:?}")
}

/// Um painel aberto, com a coluna de catálogos e um filtro de relação ligados — o estado mais
/// POVOADO que o artista alcança, para que a varredura veja o máximo de controlos.
fn painted() -> (
    MockPanelHost,
    AssetBrowserState,
    Vec<(ph2d_editor::NodeId, Rect)>,
) {
    let mut ix = ph2d_asset_index::AssetIndex::new();
    let tex = ph2d_asset_index::AssetRef::Texture { asset: [1; 32] };
    ix.push(ph2d_asset_index::AssetEntry::new(tex, "bark"));
    let mut house = ph2d_asset_index::AssetEntry::new(
        ph2d_asset_index::AssetRef::Component { stable_id: 10 },
        "house",
    );
    house.deps = vec![tex];
    ix.push(house);
    ph2d_panel_asset_browser::set_current_index(ix);

    let mut host = MockPanelHost::with_panel::<AssetBrowserPanel>();
    host.set_panel_visible(PANEL_ID, true);
    let mut st = AssetBrowserState {
        // ⚠️ **A fixtura escolhe o que NÃO é a omissão de propósito.** Com `kind = None` e
        // `sort = Name`, clicar no chip *All* ou no chip *Name* é um no-op **correcto** — e o
        // oráculo *«alguma coisa mudou?»* leria isso como controlo morto. *Uma fixtura no estado de
        // omissão não consegue medir o controlo que devolve ao estado de omissão.*
        kind: Some(ph2d_asset_index::AssetKind::Component),
        sort: ph2d_asset_index::SortBy::Recent,
        show_catalogs: true,
        related: Some((
            ph2d_asset_index::AssetRef::Component { stable_id: 10 },
            ph2d_asset_index::Relation::Uses,
        )),
        ..AssetBrowserState::default()
    };
    let rects = host.paint::<AssetBrowserPanel>(&mut st, Rect::new(0.0, 0.0, 1600.0, 900.0));
    (host, st, rects)
}

/// ⛔⛔ **A pergunta que quase nenhum gate faz: o id pintado TEM ESTADO no store?**
///
/// Sem `InteractiveState`, o `is_focusable` é falso, o `pointer_down` não arma o widget e o `Click`
/// **nunca nasce** — o controlo fica pintado, hit-indexado e morto sob o dedo, com todos os gates de
/// `apply_event` verdes (eles injectam o `Click` a jusante desta condição).
///
/// **Mutação que deve sangrar:** apagar um `store.register(…)` do `populate.rs`.
#[test]
fn every_painted_control_has_state_in_the_store() {
    let (host, _st, rects) = painted();
    assert!(
        rects.len() > 5,
        "o paint devolveu quase nada: {}",
        rects.len()
    );
    let mut mute = Vec::new();
    for (id, _) in &rects {
        if host.store().get(*id).is_none() {
            mute.push(name_of(*id));
        }
    }
    assert!(
        mute.is_empty(),
        "ids PINTADOS e hit-indexados sem `InteractiveState` — o `Down` nao arma e o `Click` \
         nunca nasce:\n  {}",
        mute.join("\n  ")
    );
}

/// ⭐⭐⭐ **A terceira pergunta: nenhum controlo fica COMPLETAMENTE tapado por um posterior.**
///
/// O `HitIndex` resolve **de trás para a frente** — o último registado ganha. ⇒ um controlo
/// inteiramente coberto por outro registado DEPOIS dele é **inalcançável**, com o paint a continuar
/// a desenhá-lo. É a quarta espécie de controlo morto, e a única forma dela que se mede **sem
/// inventar um limiar**.
///
/// ⛔⛔ **A 1.ª versão desta régua proibia qualquer sobreposição PARCIAL, e era forte demais.** Ela
/// acusou a faixa de arrasto do cabeçalho contra a caixa de busca: eles partilham **10 px**, a busca
/// é registada depois e portanto **ganha**, e a consequência é a faixa de arrasto ser 10 px mais
/// curta do que se desenha — nenhum controlo morto, nenhuma acção errada. Satisfazê-la obrigaria a
/// mexer no chrome de **todos** os painéis. *Uma régua que acusa o benigno é abandonada na primeira
/// semana, e leva o achado verdadeiro com ela.*
///
/// ⏳ **E o que ela NÃO alcança fica NOMEADO:** o report do Enio (*«layout ruim»*) era um **rótulo**
/// por baixo de um botão, e um rótulo não é um rect registado — varredura de `HitIndex` nenhuma o
/// vê. Aquele caso tem o gate específico dele (`the_band_never_covers_the_catalog_column`); a lei
/// geral exigiria o painel publicar a extensão do texto, que hoje não existe.
#[test]
fn no_painted_control_is_buried_by_a_later_one() {
    let (_host, _st, rects) = painted();
    let contains = |a: &Rect, b: &Rect| {
        a.x <= b.x + 0.5
            && a.y <= b.y + 0.5
            && a.x + a.w >= b.x + b.w - 0.5
            && a.y + a.h >= b.y + b.h - 0.5
    };
    let mut buried = Vec::new();
    for (i, (ida, ra)) in rects.iter().enumerate() {
        // ⚠️ Só os registados DEPOIS: quem vem antes perde o desempate, logo não enterra ninguém.
        for (idb, rb) in rects.iter().skip(i + 1) {
            if ida != idb && contains(rb, ra) {
                buried.push(format!(
                    "{} {ra:?} fica INTEIRAMENTE debaixo de {} {rb:?}",
                    name_of(*ida),
                    name_of(*idb)
                ));
            }
        }
    }
    assert!(
        buried.is_empty(),
        "controlos pintados e INALCANCAVEIS (o `HitIndex` resolve de tras para a frente):\n  {}",
        buried.join("\n  ")
    );
}

/// ⚠️ **E nenhum controlo é pintado FORA do painel.** Um rectângulo que sai da moldura é clicável
/// onde não há nada desenhado — o gémeo silencioso do que se pisa.
#[test]
fn no_painted_control_escapes_the_panel() {
    let (host, _st, rects) = painted();
    let panel = host
        .store()
        .panel_rect(ids::ASSET_PANEL)
        .expect("o painel publica o proprio rect");
    let mut outside = Vec::new();
    for (id, r) in &rects {
        if r.x < panel.x - 0.5
            || r.y < panel.y - 0.5
            || r.x + r.w > panel.x + panel.w + 0.5
            || r.y + r.h > panel.y + panel.h + 0.5
        {
            outside.push(format!("{} {r:?} fora de {panel:?}", name_of(*id)));
        }
    }
    assert!(
        outside.is_empty(),
        "controlos pintados FORA da moldura do painel:\n  {}",
        outside.join("\n  ")
    );
}

/// ⭐ **E o clique de cada um chega a algum lado** — a 2.ª pergunta, sobre a mesma população.
///
/// ⚠️ Os ids que NÃO nascem de um `Click` declaram-se: a faixa de arrasto e a alça de
/// redimensionar são `BlenderHit` (o despacho lê o estado no `pointer_down`), e as células da grade
/// respondem a `DoubleClick` e ao arrasto. *Uma excepção nomeada é uma decisão; uma varredura sem
/// excepções seria abandonada na primeira semana.*
#[test]
fn every_painted_control_that_takes_a_click_is_routed() {
    let (_probe, _st0, rects) = painted();
    // ⚠️ **As excepções são NOMEADAS uma a uma, com o gesto que cada uma de facto tem** — uma
    // varredura sem excepções seria abandonada, e uma com uma excepção genérica não mede nada.
    let not_a_click = |id: ph2d_editor::NodeId| {
        // A faixa de arrasto e a alça: `BlenderHit`, lidos no `pointer_down` (nunca `Click`).
        id == ids::ASSET_DRAG_HANDLE
            || id == ids::ASSET_RESIZE_HANDLE_BL
            || id == ph2d_editor::widget::ASSET_BROWSER_SCROLLBAR_ID
            // Um campo de texto responde ao FOCO e às teclas; um `Click` nele é a entrada, não o verbo.
            || id == ids::ASSET_SEARCH
            // Um slider responde a `ValueChanged`; `Click` não é o gesto dele.
            || id == ids::ASSET_SIZE
            // As células: `DoubleClick` (instanciar) e o arrasto. O `Click` só as selecciona no
            // sentido de arrancar um arrasto, e isso vive no despacho do ponteiro.
            || (0..ids::MAX_ASSET_CELLS).any(|i| ids::asset_cell_id(i) == id)
    };
    // ⛔⛔ **O membro JÁ ESCOLHIDO de uma fileira de rádio sai — e a lista é DERIVADA do estado.**
    //
    // Clicar no que já está escolhido é um no-op **correcto**, e nenhum oráculo de *«alguma coisa
    // mudou?»* o distingue de morto. ⚠️ **Derivado, e não uma lista de índices:** um chip novo, ou
    // uma omissão diferente, e uma lista escrita à mão passaria a dispensar o controlo errado — em
    // silêncio, que é como um censo deixa de medir. Só o escolhido sai; os outros membros de cada
    // fileira continuam medidos, logo uma fileira inteiramente morta ainda é apanhada.
    let already_chosen = |id: ph2d_editor::NodeId, st: &AssetBrowserState| {
        if let Some(i) = ids::ASSET_KIND.iter().position(|k| *k == id) {
            return AssetBrowserState::kind_for_chip(i) == st.kind;
        }
        if let Some(i) = ids::ASSET_SORT.iter().position(|k| *k == id) {
            return ph2d_asset_index::SortBy::ALL.get(i) == Some(&st.sort);
        }
        ph2d_panel_asset_browser::catalog_row_pick(id).is_some_and(|p| p == st.pick)
    };
    let mut dead = Vec::new();
    for (id, _) in &rects {
        if not_a_click(*id) {
            continue;
        }
        // ⛔⛔ **UM PAINEL FRESCO POR ID, e a 1.ª versão deste gate não o fazia.** Ela reutilizava
        // o mesmo host: o clique no `ASSET_CLOSE` **fecha o painel**, e daí em diante todos os
        // outros batiam na guarda *«só existe com o painel aberto»* e saíam `Ignored`. ⇒ o censo
        // acusava **catorze controlos de mortos** e todos estavam vivos.
        //
        // *Um censo que partilha estado entre casos mede o efeito colateral do caso anterior* — e
        // este acusava com tanta confiança que eu quase fui procurar o defeito no painel.
        let (mut host, mut st, _) = painted();
        if already_chosen(*id, &st) {
            continue;
        }
        let before = format!("{st:?}");
        // ⛔ **TRÊS canais, e o terceiro só apareceu quando o censo acusou o `ASSET_CLOSE`.** Ele
        // fecha o painel: nem toca no barramento nem no `AssetBrowserState` — o efeito dele vive no
        // HOST. *Um oráculo que enumera os sítios onde um efeito pode aterrar tem de os enumerar
        // todos, e o que falta é sempre o que se descobre por uma acusação falsa.*
        let visible_before = host.panel_visible(PANEL_ID);
        let out = host.apply_panel_event::<AssetBrowserPanel>(&mut st, WidgetEvent::Click(*id));
        let touched = !host.bus().is_empty()
            || format!("{st:?}") != before
            || host.panel_visible(PANEL_ID) != visible_before;
        if out != EventOutcome::Consumed || !touched {
            dead.push(name_of(*id));
        }
    }
    assert!(
        dead.is_empty(),
        "controlos PINTADOS cujo clique nao chega a efeito nenhum:\n  {}",
        dead.join("\n  ")
    );
}

/// O controlo da fixtura: se o `InteractiveState` fosse universal, o 1.º gate mediria nada.
#[test]
fn the_store_can_say_no() {
    let (host, _st, _rects) = painted();
    let stranger = ph2d_editor::NodeId(0xDEAD_BEEF);
    assert!(
        host.store().get(stranger).is_none(),
        "o store devolve estado para um id que ninguem registou — o 1.º gate mede nada"
    );
}
