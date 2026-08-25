//! **A METADE DE DESPACHO da janela do Input Map** — irmã de [`super`], cortada por teto de LOC
//! (700) e por responsabilidade: ali *o que a janela É e como se desenha*, aqui *o que os gestos
//! dela fazem*.
//!
//! ⚠️ O corte foi feito na auditoria de 2026-08-24, que engordou os dois lados: o pintor ganhou o
//! recorte de pixels, o fundo que absorve o clique e o indicador da escuta; o despacho ganhou o
//! Enter, a recusa com cara, e a largada do foco ao apagar uma linha.

use super::sync_input_map_rows;
use crate::ids;
use crate::interaction::WidgetEvent;
use crate::screens::hero::HeroScreen;
use ph2d_input::Binding;
use ph2d_tokens::Spacing;

/// **CRIA A ACÇÃO com o nome do campo** — a porta única, e ela existe porque **DOIS** gestos a
/// alcançam: o botão `Add` e o **Enter** no campo.
///
/// ⛔ Auditoria 2026-08-24: o Enter era inerte, porque o `apply` só ouvia `Click`. Escrever um nome
/// e carregar Enter tirava o foco, deixava o texto lá, e nada nascia — o gesto de formulário que
/// toda a gente tenta primeiro.
///
/// ⚠️ **Nome vazio não cria nada, e agora DIZ porquê.** Antes ele era inerte e mudo: o artista
/// carregava em `Add` e nada acontecia, sem uma palavra. Uma acção sem nome é inalcançável por
/// código (a leitura é `pressed("...")`), então criá-la produziria uma linha que nada pode usar —
/// mas *recusar em silêncio lê-se como avariado*.
fn add_action_from_field(hero: &mut HeroScreen) {
    let name = hero
        .store
        .text(ids::INPUT_MAP_NEW_NAME)
        .unwrap_or_default()
        .trim()
        .to_string();
    if name.is_empty() {
        // O campo ganha o foco e o aviso aparece — a recusa passa a ter cara.
        hero.store.set_focus(Some(ids::INPUT_MAP_NEW_NAME));
        return;
    }
    hero.input_map.create(&name);
    let map = hero.input_map.clone();
    sync_input_map_rows(&mut hero.store, &map);
    // ⚠️ Limpar o campo pela MESMA porta que o lê — `InteractiveState::TextInput`. Um `set_text`
    // próprio seria uma segunda escrita do mesmo facto.
    if let Some(crate::interaction::InteractiveState::TextInput { text, caret, .. }) =
        hero.store.get_mut(ids::INPUT_MAP_NEW_NAME)
    {
        text.clear();
        *caret = 0;
    }
}

/// A metade de DESPACHO — ligada ao `dispatch_all` pelo `ph2d-chrome-sync`.
pub fn apply(hero: &mut HeroScreen, event: WidgetEvent) -> bool {
    // ⭐ **ENTER NO CAMPO CRIA A ACÇÃO** — auditoria 2026-08-24.
    //
    // O `dispatch_key` emite `Submit` para um `TextInput` de linha única, e o `apply` só ouvia
    // `Click`: escrever um nome e carregar Enter tirava o foco, deixava o texto lá, e **não nascia
    // nada**. É o gesto de formulário que toda a gente tenta primeiro, e ele era inerte.
    if let WidgetEvent::Submit(sid) = event
        && sid == ids::INPUT_MAP_NEW_NAME
    {
        add_action_from_field(hero);
        return true;
    }
    // ⭐ **OS DOIS NÚMEROS DA ZONA** — e eles chegam por `ValueChanged`, não por `Click`.
    if let WidgetEvent::ValueChanged(id) = event {
        let v = hero.store.slider(id).map_or(0.0, |(_, x)| x);
        let ids_of: Vec<_> = hero.input_map.actions().iter().map(|a| a.id).collect();
        for (row, aid) in ids_of.into_iter().enumerate() {
            let is_dz = id == ids::input_map_deadzone_id(row);
            let is_pp = id == ids::input_map_press_point_id(row);
            if !is_dz && !is_pp {
                continue;
            }
            if let Some(a) = hero.input_map.get_mut(aid) {
                // ⚠️ **A porta da acção COAGE** (`press_point >= dead_zone`), então o outro número
                // pode mover-se — e é por isso que a sincronia logo abaixo re-semeia os DOIS
                // sliders: sem ela, o slider mostraria o valor que o dedo pediu em vez do que
                // ficou, e o artista veria a janela discordar do produto.
                let (dz, pp) = if is_dz {
                    (v, a.press_point)
                } else {
                    (a.dead_zone, v)
                };
                a.set_zone(dz, pp);
            }
            let map = hero.input_map.clone();
            sync_input_map_rows(&mut hero.store, &map);
            return true;
        }
        return false;
    }
    let WidgetEvent::Click(id) = event else {
        return false;
    };
    // ⭐ **O ABRIDOR, e ele é o PRIMEIRO ramo de propósito** — antes da guarda de "janela aberta".
    // Um abridor atrás dessa guarda seria a piada de precisar da janela aberta para a abrir, e é a
    // forma exacta de uma feature ficar inalcançável com todos os gates verdes.
    if id == ids::CTX_MENU_SETTINGS_INPUT_MAP {
        // ⚠️ A janela nasce no canto útil da viewport, não sob o cursor: ela é grande e o cursor
        // está no menu, no topo — abrir ali poria metade dela fora do ecrã antes do clamp.
        let v = hero.last_viewport;
        hero.store
            .open_input_map(v.x + Spacing::Xl4.px(), v.y + Spacing::Xl4.px());
        // ⚠️ E as linhas do mapa que veio do FICHEIRO registam-se aqui: elas existiam antes de a
        // janela existir, e nenhum gesto as criou nesta sessão.
        let map = hero.input_map.clone();
        sync_input_map_rows(&mut hero.store, &map);
        return true;
    }
    // ⭐ **A TECLA APANHADA** — o `Click` sintético que o despacho de teclado emitiu. Ele vem antes
    // da guarda da janela pelo mesmo motivo do abridor: se a janela fechasse entre a tecla e este
    // ramo, a captura ficaria pendurada para sempre.
    // O `Esc` durante a escuta: consumido, sem ligar nada. A lei já desarmou; aqui só se
    // reconhece o evento para ele não vazar para outro chrome.
    if id == ids::INPUT_MAP_LISTEN_CANCELLED {
        return true;
    }
    if id == ids::INPUT_MAP_BIND_CAPTURED {
        let key = hero.store.take_captured_key();
        let armed = hero.store.input_map_listening();
        hero.store.stop_listening();
        if let (Some(k), Some(aid)) = (key, armed)
            && let Some(a) = hero.input_map.get_mut(aid)
        {
            let b = Binding::Key(k);
            // ⚠️ **Ligar duas vezes a mesma tecla não a duplica.** A lista é um conjunto de
            // caminhos até a acção, e dois caminhos idênticos não são dois caminhos — seriam duas
            // linhas iguais no painel, uma delas impossível de distinguir da outra ao apagar.
            if !a.bindings.contains(&b) {
                a.bindings.push(b);
            }
        }
        let map = hero.input_map.clone();
        sync_input_map_rows(&mut hero.store, &map);
        return true;
    }
    if hero.store.input_map_pos().is_none() {
        return false;
    }
    if id == ids::INPUT_MAP_CLOSE {
        hero.store.close_input_map();
        return true;
    }
    // Um clique nu na faixa do título (sem arrasto) — consome, para nunca vazar para outro chrome.
    if id == ids::INPUT_MAP_HANDLE {
        return true;
    }
    if id == ids::INPUT_MAP_ADD {
        add_action_from_field(hero);
        return true;
    }
    // As linhas: o índice vem da POSIÇÃO na lista, e o id é derivado dela — então a resolução é
    // percorrer a lista a comparar, que é o que o pintor fez para a registar.
    let ids_of: Vec<_> = hero.input_map.actions().iter().map(|a| a.id).collect();
    for (row, aid) in ids_of.into_iter().enumerate() {
        if id == ids::input_map_delete_action_id(row) {
            hero.input_map.remove(aid);
            // ⛔ **O FOCO tem de largar a linha apagada** — auditoria 2026-08-24. Os ids das linhas
            // são derivados do ÍNDICE, então o foco preso no id da linha `row` passa a apontar,
            // depois da remoção, para a acção que **subiu** para esse índice: um Enter apagava a
            // acção seguinte. *Um id derivado da posição obriga quem guarda posição a largá-la.*
            hero.store.set_focus(None);
            let map = hero.input_map.clone();
            sync_input_map_rows(&mut hero.store, &map);
            // ⚠️ Apagar a acção à escuta **desarma** a escuta: senão a próxima tecla iria para um
            // id que já não existe, e sumiria sem que nada na tela dissesse porquê.
            if hero.store.input_map_listening() == Some(aid) {
                hero.store.stop_listening();
            }
            return true;
        }
        if id == ids::input_map_listen_id(row) {
            // Carregar de novo no botão já armado **desarma** — o gesto tem de ter volta.
            if hero.store.input_map_listening() == Some(aid) {
                hero.store.stop_listening();
            } else {
                hero.store.listen_for_binding(aid);
            }
            return true;
        }
        let n = hero.input_map.get(aid).map_or(0, |a| a.bindings.len());
        for bi in 0..n {
            if id == ids::input_map_delete_binding_id(row, bi) {
                if let Some(a) = hero.input_map.get_mut(aid) {
                    a.bindings.remove(bi);
                }
                let map = hero.input_map.clone();
                sync_input_map_rows(&mut hero.store, &map);
                return true;
            }
        }
    }
    false
}
