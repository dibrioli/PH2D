//! ⭐ **O cartão «Collision Layers»** (ficha f1, onda 7): a camada do objecto e, com corpo acima, «Collides with»: a
//! linha da matriz global, vista e editada a partir do objecto. Um quadro por camada visível; clicar é a mesma
//! edição da célula da matriz.

use super::physics_cartao::Tinta;
use super::physics_cartao_forma::cartao_de_fisica;
use super::*;
use crate::camadas_fisica as camadas;
use crate::ids;
use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::interaction::{WidgetEvent, WidgetStore};
use ph2d_editor_core::paint::{
    fill_rounded_rect, folga_do_texto, frame_radius, paint_layer_outline, paint_text_elided,
    resolve,
};
use ph2d_editor_core::panel::{PanelHostInternal, seam_reset_button};
use ph2d_editor_core::screens::hero::InspectorPhysicsInfo;
use ph2d_editor_core::widget::card_ink::tinta;
use ph2d_editor_core::widget::{Dropdown, DropdownOption, paint_dropdown_chip};
use ph2d_editor_core::zones::Rect;
use ph2d_i18n::{tr, tr_with};
use ph2d_tokens::{ColorToken, Radius, Spacing, StrokeToken, TypeToken};

/// Sem corpo acima nada bate na forma: os quadros desmentiriam a frase de cima.
#[must_use]
pub(super) fn solta(info: &InspectorPhysicsInfo) -> bool {
    !info.has_body && info.part_owner.is_empty()
}

/// O cartão **Collision Layers**, de quem tem `Collider`.
#[allow(clippy::too_many_arguments)]
pub(crate) fn secao_das_camadas(
    scene: &mut VectorScene,
    text: &mut TextSystem,
    theme: Theme,
    hit: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    info: &InspectorPhysicsInfo,
) -> f32 {
    let mut t = Tinta {
        scene,
        text,
        theme,
        hit,
        store,
        x,
        w,
    };
    let titulo = tr("card.insp.phys.collision_layers");
    cartao_de_fisica(&mut t, y, ids::CARD_PHYS_LAYERS, titulo, |t, top| {
        camada(t, top, info) + SECTION_BOTTOM_PAD_PX
    })
}

/// A fila não cabe inteira no título dobrado: fica só o nome da camada. O pintor do título é inteiro ou nada, e
/// sem isto o Inspector de fábrica não mostraria resumo nenhum em «Props · Ground, Props, Zones» (medido na foto).
#[must_use]
pub(crate) fn curto_se_a_fila_nao_cabe(
    id: NodeId,
    text: &mut TextSystem,
    largura: f32,
    titulo_w: f32,
) -> Option<String> {
    use ph2d_card_stack::metrics as m;
    if id != ids::CARD_PHYS_LAYERS {
        return None;
    }
    let info = crate::state::current_inspector_physics()?;
    let livre = largura
        - (m::LIST_PAD_X + m::CARD_PAD_X) * 2.0
        - m::LIST_PAD_X
        - m::CARD_CHEVRON
        - m::CARD_CHEVRON_GAP
        - titulo_w
        - m::FIELD_GAP * 2.0;
    let pede = text.prefix_width(&resumo(&info), m::LABEL_PX).ceil();
    (pede > livre).then(|| camadas::rotulo(usize::from(info.layer)))
}

/// O título dobrado: a camada e, com corpo acima, os quadros acesos, os mesmos que a fila mostra.
#[must_use]
pub(super) fn resumo(info: &InspectorPhysicsInfo) -> String {
    resumo_com(info, camadas::rotulo)
}

/// O resumo com os nomes de `nome_de` (cortados no título; inteiros na dica, onda 9e).
fn resumo_com(info: &InspectorPhysicsInfo, nome_de: fn(usize) -> String) -> String {
    let camada = usize::from(info.layer);
    let nome = nome_de(camada);
    if solta(info) {
        return nome;
    }
    let fila: Vec<String> = da_fila(camada)
        .filter(|&j| camadas::colide(camada, j))
        .map(nome_de)
        .collect();
    let alvo = if fila.is_empty() {
        tr("card.insp.phys.none").to_string()
    } else {
        fila.join(", ")
    };
    format!("{nome} \u{b7} {alvo}")
}

/// Layer em lista com os nomes (os ids e a edição são os de sempre), os quadros de «Collides with» (só com corpo acima), o «Rename…» e o «Open in tab…».
fn camada(t: &mut Tinta<'_>, y: f32, info: &InspectorPhysicsInfo) -> f32 {
    let rotulo = tr("component.field.collider.4");
    let seccao = ph2d_editor_core::property_row::Seccao::medida(t.text, 1, &[rotulo]);
    let linha = ph2d_editor_core::property_row::paint_label_row(
        t.scene,
        t.text,
        t.theme,
        t.x,
        t.w,
        y,
        ph2d_tokens::ROW_H_PX,
        rotulo,
        seccao,
    );
    t.hit.register(ids::INSP_PHYS_LAYER_PICK, linha.control);
    let aberta = matches!(
        t.store.get(ids::INSP_PHYS_LAYER_PICK),
        Some(InteractiveState::Dropdown { open: true, .. })
    );
    let dd = lista_da_camada(info.layer)
        .open(aberta)
        .visual(t.store.dropdown_visual(ids::INSP_PHYS_LAYER_PICK));
    paint_dropdown_chip(&dd, linha.control, t.scene, t.text, t.theme);
    let j = usize::from(info.layer);
    contorno(t, linha.control, j, ids::INSP_PHYS_LAYER_PICK);
    if aberta {
        crate::state_popovers::set_pending_phys_layer_dd(Some((info.layer, linha.control)));
    }
    if let Some(alvo) = ph2d_editor_core::widget::key_mark::paint_decorator_for(
        t.scene,
        t.theme,
        linha.dot,
        &ids::INSP_PHYS_LAYER,
    ) {
        t.hit.register(alvo, linha.dot);
    }
    let y = linha.fim(ph2d_tokens::row_pitch_px());
    let camada = usize::from(info.layer);
    let solta = solta(info);
    if !t.store.is_collapsed(ids::INSP_PHYS_LAYER_RENAMING) {
        let y = rename(t, y, camada);
        let (r, y) = t.botao(
            ids::INSP_PHYS_LAYER_RENAME_DONE,
            tr("card.insp.phys.layer_rename_done"),
            y,
        );
        t.hit.register(ids::INSP_PHYS_LAYER_RENAME_DONE, r);
        return y;
    }
    let y = if solta { y } else { colide_com(t, y, info) };
    if solta {
        let (r, y) = t.botao(
            ids::INSP_PHYS_LAYER_RENAME,
            tr("card.insp.phys.layer_rename"),
            y,
        );
        t.hit.register(ids::INSP_PHYS_LAYER_RENAME, r);
        y
    } else {
        t.dois_botoes(
            y,
            (
                ids::INSP_PHYS_LAYER_RENAME,
                tr("card.insp.phys.layer_rename"),
            ),
            (ids::INSP_PHYS_EDIT_MATRIX, tr("card.insp.phys.edit_matrix")),
        )
    }
}

/// As camadas do modo Rename: as visíveis e a do objecto (a mesma fila das caixinhas).
pub(crate) fn fila_do_rename(camada: usize) -> impl Iterator<Item = usize> {
    da_fila(camada)
}

/// ⭐ O modo Rename: um campo por camada, com o nome INTEIRO; a do objecto na pílula (a mesma da aba).
fn rename(t: &mut Tinta<'_>, mut y: f32, camada: usize) -> f32 {
    let fila: Vec<usize> = da_fila(camada).collect();
    let rotulos: Vec<String> = fila.iter().map(|j| j.to_string()).collect();
    let refs: Vec<&str> = rotulos.iter().map(String::as_str).collect();
    let seccao = ph2d_editor_core::property_row::Seccao::medida(t.text, 1, &refs);
    for (p, &j) in fila.iter().enumerate() {
        if j == camada {
            let raio = frame_radius(t.theme, Radius::Sm.px());
            let r = Rect::new(t.x, y, t.w, ph2d_tokens::ROW_H_PX);
            fill_rounded_rect(t.scene, r, raio, resolve(ColorToken::AccentSoft, t.theme));
        }
        let id = ids::INSP_PHYS_LAYER_NAMES[j];
        let dica = camadas::rotulo_padrao(j);
        y = ph2d_editor_core::property_row::paint_text_row(
            t.scene,
            t.text,
            t.theme,
            t.hit,
            t.store,
            t.x,
            t.w,
            y,
            refs[p],
            id,
            TextInput::new(id, "").placeholder(&dica),
            seccao,
        );
        if let Some(campo) = t.hit.rect_for(id) {
            contorno(t, campo, j, id);
        }
    }
    y
}

/// O contorno da cor da camada `j` na moldura do controlo (o seletor Layer, o campo do Rename; onda 9f).
fn contorno(t: &mut Tinta<'_>, caixa: Rect, j: usize, id: ph2d_a11y::NodeId) {
    let raio = frame_radius(t.theme, Radius::Sm.px());
    paint_layer_outline(t.scene, t.theme, id, j, caixa, raio, true);
}

/// As opções da lista: as camadas visíveis e a do objecto, pelo nome ou pelo número.
fn opcoes(camada: u8) -> Vec<DropdownOption<u8>> {
    ids::INSP_PHYS_LAYER
        .iter()
        .enumerate()
        // A face clássica guarda o alcance dos seus 8 chips de antes (0–7), em uso ou não.
        .filter(|&(i, _)| {
            camadas::visivel(i) || usize::from(camada) == i || (!crate::cards::active() && i < 8)
        })
        // A lista do seletor mostra o nome INTEIRO (a largura dela cresce com a opção mais longa; onda 9e).
        .map(|(i, &id)| DropdownOption::new(id, i as u8, camadas::rotulo_inteiro(i)))
        .collect()
}

/// A lista da camada, com a camada do objecto escolhida: o que o chip mostra.
#[must_use]
pub(crate) fn lista_da_camada(camada: u8) -> Dropdown<u8> {
    let mut dd = Dropdown::new(ids::INSP_PHYS_LAYER_PICK, "", opcoes(camada));
    dd.select(camada);
    dd
}

/// As camadas da fila: as visíveis e, mesmo fora delas, a do objecto.
fn da_fila(camada: usize) -> impl Iterator<Item = usize> {
    (0..ids::INSP_PHYS_LAYERS).filter(move |&j| camadas::visivel(j) || j == camada)
}

/// A caixa de um quadro em `x`; a largura sai do nome.
fn largura(t: &mut Tinta<'_>, texto: &str, com_caixa: bool) -> f32 {
    let (fonte, folga) = (TypeToken::Sm.px(), Spacing::Xs.px());
    let caixa = if com_caixa { fonte + folga } else { 0.0 };
    let medida = t.text.prefix_width(texto, fonte).ceil();
    folga + caixa + medida + folga
}

/// O fundo com a caixinha acesa ou apagada (ou sem ela, no «+») e o nome.
fn pinta(t: &mut Tinta<'_>, r: Rect, texto: &str, id: ph2d_a11y::NodeId, aceso: Option<bool>) {
    let (fonte, folga) = (TypeToken::Sm.px(), Spacing::Xs.px());
    let raio = frame_radius(t.theme, Radius::Sm.px());
    let fundo = if t.store.hover_live(id) > 0.0 {
        ColorToken::CardFieldHover
    } else {
        ColorToken::CardField
    };
    fill_rounded_rect(t.scene, r, raio, tinta(fundo, t.theme));
    let mut x = r.x + folga;
    if let Some(on) = aceso {
        let caixa = Rect::new(x, r.y + (r.h - fonte) * 0.5, fonte, fonte);
        if on {
            fill_rounded_rect(t.scene, caixa, raio, resolve(ColorToken::Accent, t.theme));
        } else {
            // FRAME-RAW-OK: o quadro apagado é só o contorno; a forma é o que distingue os dois estados
            ph2d_editor_core::paint::stroke_rounded_rect(
                t.scene,
                caixa,
                raio,
                StrokeToken::Default.px(),
                tinta(ColorToken::CardLabel, t.theme),
            );
        }
        x += fonte + folga;
    }
    let cor = if aceso == Some(false) {
        ColorToken::CardLabel
    } else {
        ColorToken::KanbanCardFg
    };
    let (topo, cabe) = (
        r.y + folga_do_texto(r.h, fonte),
        (r.x + r.w - folga - x).max(0.0),
    );
    paint_text_elided(
        t.text,
        t.scene,
        texto,
        x,
        topo,
        fonte,
        cabe,
        tinta(cor, t.theme),
    );
    // O contorno da cor da camada na moldura do quadro; desligado, mais apagado (onda 9f; o «+» não tem camada).
    if let Some(j) = ids::INSP_PHYS_COLLIDES.iter().position(|&o| o == id) {
        paint_layer_outline(t.scene, t.theme, id, j, r, raio, aceso != Some(false));
    }
    ph2d_editor_core::panel::cards::censo::controlo_maior_que_a_pega(id, r);
    t.hit.register(id, r);
}

/// O rótulo com a marca, os quadros quebrando pelo tamanho do nome, a frase e o «Open in tab…».
pub(super) fn colide_com(t: &mut Tinta<'_>, y: f32, info: &InspectorPhysicsInfo) -> f32 {
    let camada = usize::from(info.layer);
    let rotulo = tr("card.insp.phys.collides_with");
    let seccao = ph2d_editor_core::property_row::Seccao::medida(t.text, 1, &[rotulo]);
    let linha = ph2d_editor_core::property_row::paint_label_row(
        t.scene,
        t.text,
        t.theme,
        t.x,
        t.w,
        y,
        ph2d_tokens::ROW_H_PX,
        rotulo,
        seccao,
    );
    if let Some(alvo) = ph2d_editor_core::widget::key_mark::paint_decorator_for(
        t.scene,
        t.theme,
        linha.dot,
        &ids::INSP_PHYS_COLLIDES,
    ) {
        t.hit.register(alvo, linha.dot);
    }
    let c = linha.control;
    let gap = Spacing::Xs.px();
    let (mut cx, mut cy) = (c.x, c.y);
    let mut assenta = |cw: f32| {
        if cx > c.x && cx + cw > c.x + c.w {
            cx = c.x;
            cy += c.h + gap;
        }
        let r = Rect::new(cx, cy, cw.min(c.w), c.h);
        cx += r.w + gap;
        (r, cy)
    };
    let mut fundo = c.y;
    for j in da_fila(camada) {
        let nome = camadas::rotulo(j);
        let cw = largura(t, &nome, true);
        let (r, ultima) = assenta(cw);
        fundo = ultima;
        pinta(
            t,
            r,
            &nome,
            ids::INSP_PHYS_COLLIDES[j],
            Some(camadas::colide(camada, j)),
        );
    }
    // Nas 32 não há mais camada para mostrar: o «+» sai.
    if camadas::proxima_livre().is_some() {
        let mais = tr("card.insp.phys.collides_add");
        let cw = largura(t, mais, false);
        let (r, ultima) = assenta(cw);
        fundo = ultima;
        pinta(t, r, mais, ids::INSP_PHYS_COLLIDES_ADD, None);
    }
    ph2d_editor_core::widget::composto::grupo(ids::INSP_PHYS_COLLIDES.iter().copied());
    let y = linha.fim(ph2d_tokens::row_pitch_px()) + (fundo - c.y);
    let frase = tr_with(
        "card.insp.phys.collides_applies",
        &[("layer", &camadas::rotulo(camada))],
    );
    t.frase(y, &frase, ColorToken::Text3)
}

/// As dicas dos quadros: dizem, com os nomes de hoje, o que o próximo clique faz.
pub(crate) fn prepara_as_dicas(store: &mut WidgetStore, camada: usize) {
    // O título dobrado corta os nomes: a dica dele diz o resumo com os nomes inteiros (onda 9e).
    if let Some(info) = crate::state::current_inspector_physics() {
        let inteiro = resumo_com(&info, camadas::rotulo_inteiro);
        let dica = if inteiro == resumo(&info) {
            String::new()
        } else {
            inteiro
        };
        if store.tooltip_for(ids::CARD_PHYS_LAYERS).unwrap_or_default() != dica {
            store.set_tooltip(ids::CARD_PHYS_LAYERS, dica);
        }
    }
    let este = camadas::rotulo_inteiro(camada);
    for j in da_fila(camada) {
        let aceso = camadas::colide(camada, j);
        let chave = match (j == camada, aceso) {
            (true, true) => "card.insp.phys.collides_self_on",
            (true, false) => "card.insp.phys.collides_self_off",
            (false, true) => "card.insp.phys.collides_on",
            (false, false) => "card.insp.phys.collides_off",
        };
        let outra = camadas::rotulo_inteiro(j);
        let dica = tr_with(chave, &[("this", &este), ("other", &outra)]);
        let id = ids::INSP_PHYS_COLLIDES[j];
        if store.tooltip_for(id) != Some(dica.as_str()) {
            store.set_tooltip(id, dica);
        }
    }
}

/// Os cliques da fila. `true` = era dela. Uma forma solta não pinta quadros, e o clique não chega aqui.
pub(crate) fn clique(host: &mut dyn PanelHostInternal, ev: WidgetEvent) -> bool {
    let WidgetEvent::Click(id) = ev else {
        return false;
    };
    let Some(info) = crate::state::current_inspector_physics() else {
        return false;
    };
    let a = info.layer;
    if let Some(b) = ids::INSP_PHYS_COLLIDES.iter().position(|&o| o == id) {
        seam_reset_button(host, id);
        if !solta(&info) {
            host.bus_mut()
                .push(EditorAction::LayerPairToggle { a, b: b as u8 });
        }
        return true;
    }
    if id == ids::INSP_PHYS_COLLIDES_ADD {
        seam_reset_button(host, id);
        if let Some(layer) = camadas::proxima_livre() {
            host.bus_mut().push(EditorAction::LayerShow { layer });
        }
        return true;
    }
    if id == ids::INSP_PHYS_EDIT_MATRIX {
        seam_reset_button(host, id);
        host.bus_mut()
            .push(EditorAction::LayerMatrixOpen { layer: a });
        return true;
    }
    false
}

#[cfg(test)]
mod lista_tests {
    use super::*;

    /// O chip mostra a camada do objecto, com número ou com nome: cada opção carrega o seu próprio valor.
    #[test]
    fn a_lista_da_camada_mostra_a_camada_do_objecto_como_escolhida() {
        let mut nomes: [String; 32] = Default::default();
        nomes[2] = "Enemy".into();
        camadas::set_current(nomes, [u32::MAX; 32], 0xFF);
        for i in 0..8u8 {
            let dd = lista_da_camada(i);
            let esperado = if i == 2 {
                "Enemy".to_owned()
            } else {
                i.to_string()
            };
            assert_eq!(dd.selected_label(), Some(esperado.as_str()), "camada {i}");
        }
        camadas::set_current(Default::default(), [u32::MAX; 32], 0xFF);
    }

    /// A lista só traz as visíveis; a camada do objecto, mesmo fora delas, nunca falta.
    #[test]
    fn a_lista_da_camada_so_traz_as_visiveis_e_a_do_objecto() {
        camadas::set_current(Default::default(), [u32::MAX; 32], 1 | 1 << 20);
        assert_eq!(opcoes(0).len(), 2);
        assert_eq!(lista_da_camada(20).selected_label(), Some("20"));
        assert_eq!(opcoes(25).len(), 3, "a do objecto entra");
        assert_eq!(lista_da_camada(25).selected_label(), Some("25"));
        camadas::set_current(Default::default(), [u32::MAX; 32], 0xFF);
    }
}
