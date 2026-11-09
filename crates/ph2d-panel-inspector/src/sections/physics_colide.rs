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

/// ⭐ Quem pergunta pela camada: o collider ou o Ray Sensor (dono 08/11: «os mesmos campos»). A mesma lista, os
/// mesmos quadros da matriz do mundo; só os ids são de cada um.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Quem {
    pub pick: NodeId,
    pub opcoes: &'static [NodeId; ids::INSP_PHYS_LAYERS],
    pub quadros: &'static [NodeId; ids::INSP_PHYS_LAYERS],
    pub mais: NodeId,
    /// Os ids que levam o canal do losango da Layer.
    pub canal: &'static [NodeId],
    /// A frase debaixo da fila: só a dica, com o nome inteiro.
    pub frase: NodeId,
}

pub(crate) static DO_COLLIDER: Quem = Quem {
    pick: ids::INSP_PHYS_LAYER_PICK,
    opcoes: &ids::INSP_PHYS_LAYER,
    quadros: &ids::INSP_PHYS_COLLIDES,
    mais: ids::INSP_PHYS_COLLIDES_ADD,
    canal: &ids::INSP_PHYS_LAYER,
    frase: ids::INSP_PHYS_COLLIDES_APPLIES,
};

pub(crate) static DO_RAIO: Quem = Quem {
    pick: ids::INSP_RAY_LAYER,
    opcoes: &ids::INSP_RAY_LAYER_OPTS,
    quadros: &ids::INSP_RAY_SEES,
    mais: ids::INSP_RAY_SEES_ADD,
    canal: &[ids::INSP_RAY_LAYER],
    frase: ids::INSP_RAY_SEES_APPLIES,
};

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
        .filter(|&j| super::physics_colide_fila::linha(info) & (1 << j) != 0)
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
    let caixas = (linha.control, linha.dot);
    chip(t, caixas, &DO_COLLIDER, info.layer, rotulo.into());
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
    // Dono 08/11: os botões logo abaixo da Layer, antes do «Collides with».
    if solta {
        let (r, y) = t.botao(
            ids::INSP_PHYS_LAYER_RENAME,
            tr("card.insp.phys.layer_rename"),
            y,
        );
        t.hit.register(ids::INSP_PHYS_LAYER_RENAME, r);
        return y;
    }
    let y = t.dois_botoes(
        y,
        (
            ids::INSP_PHYS_LAYER_RENAME,
            tr("card.insp.phys.layer_rename"),
        ),
        (ids::INSP_PHYS_EDIT_MATRIX, tr("card.insp.phys.edit_matrix")),
    );
    super::physics_colide_fila::colide_com(t, y, info)
}

/// As camadas do modo Rename: as visíveis e a do objecto (a mesma fila das caixinhas).
pub(crate) fn fila_do_rename(camada: usize) -> impl Iterator<Item = usize> {
    da_fila(camada)
}

/// ⭐ O modo Rename: um campo por camada, com o nome INTEIRO; a do objecto na pílula (a mesma da aba).
pub(super) fn rename(t: &mut Tinta<'_>, mut y: f32, camada: usize) -> f32 {
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

/// O chip da lista da camada de `quem` em `caixa`: a escolhida, o contorno da cor dela, aberta, o popover diferido, e
/// o losango em `dot` (o canal de `quem.canal`: a tabela de canais ou o campo do catálogo do `rotulo`).
pub(super) fn chip(
    t: &mut Tinta<'_>,
    (caixa, dot): (Rect, Rect),
    quem: &'static Quem,
    camada: u8,
    rotulo: ph2d_editor_core::widget::key_mark::Rotulo<'_>,
) {
    t.hit.register(quem.pick, caixa);
    let aberta = matches!(
        t.store.get(quem.pick),
        Some(InteractiveState::Dropdown { open: true, .. })
    );
    let dd = lista(quem, camada)
        .open(aberta)
        .visual(t.store.dropdown_visual(quem.pick));
    paint_dropdown_chip(&dd, caixa, t.scene, t.text, t.theme);
    contorno(t, caixa, usize::from(camada), quem.pick);
    if aberta {
        crate::state_popovers::set_pending_phys_layer_dd(Some((quem, camada, caixa)));
    }
    if let Some(alvo) = ph2d_editor_core::widget::key_mark::paint_decorator_campo(
        t.scene, t.theme, dot, quem.canal, rotulo,
    ) {
        t.hit.register(alvo, dot);
    }
}

/// As opções da lista: as camadas visíveis e a do objecto, pelo nome ou pelo número.
fn opcoes(quem: &Quem, camada: u8) -> Vec<DropdownOption<u8>> {
    quem.opcoes
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

/// A lista da camada de `quem`, com `camada` escolhida.
#[must_use]
pub(crate) fn lista(quem: &Quem, camada: u8) -> Dropdown<u8> {
    let mut dd = Dropdown::new(quem.pick, "", opcoes(quem, camada));
    dd.select(camada);
    dd
}

/// As camadas da fila: as visíveis e, mesmo fora delas, a do objecto.
pub(super) fn da_fila(camada: usize) -> impl Iterator<Item = usize> {
    (0..ids::INSP_PHYS_LAYERS).filter(move |&j| camadas::visivel(j) || j == camada)
}

/// As dicas dos quadros: dizem, com os nomes de hoje, o que o próximo clique faz.
pub(crate) fn prepara_as_dicas(store: &mut WidgetStore, camada: usize, linha: u32) {
    let chaves = [
        "card.insp.phys.collides_self_on",
        "card.insp.phys.collides_self_off",
        "card.insp.phys.collides_on",
        "card.insp.phys.collides_off",
    ];
    dicas_dos_quadros(store, &DO_COLLIDER, (camada, linha), chaves);
}

/// As dicas dos quadros de `quem` na linha `camada`: `chaves` = (a própria acesa, apagada, outra acesa, apagada).
pub(crate) fn dicas_dos_quadros(
    store: &mut WidgetStore,
    quem: &Quem,
    (camada, linha): (usize, u32),
    chaves: [&str; 4],
) {
    let este = camadas::rotulo_inteiro(camada);
    for j in da_fila(camada) {
        let aceso = linha & (1 << j) != 0;
        let chave = chaves[usize::from(j != camada) * 2 + usize::from(!aceso)];
        let outra = camadas::rotulo_inteiro(j);
        let dica = tr_with(chave, &[("this", &este), ("other", &outra)]);
        poe_a_dica(store, quem.quadros[j], dica);
    }
    // As opções da lista mostram o nome inteiro (a lista cresce com ele): balão nenhum a repetir o que se lê.
    for &id in quem.opcoes {
        poe_a_dica(store, id, String::new());
    }
    // A frase de baixo só ganha balão quando o «…» cortou o nome.
    let frase = tr_with("card.insp.phys.collides_applies", &[("layer", &este)]);
    let cortou = camadas::rotulo(camada) != este;
    poe_a_dica(
        store,
        quem.frase,
        if cortou { frase } else { String::new() },
    );
}

/// O título dobrado de Collision Layers: o resumo inteiro no balão só quando o que se lê encurtou — um nome com «…» ou
/// a fila trocada pela camada só (`curto_se_a_fila_nao_cabe`, medido em `presentes`).
pub(crate) fn dica_do_titulo(store: &mut WidgetStore, ps: &[crate::cards_componentes::Presente]) {
    let p = ps.iter().find(|p| p.id == ids::CARD_PHYS_LAYERS);
    let (Some(p), Some(info)) = (p, crate::state::current_inspector_physics()) else {
        return;
    };
    let inteiro = resumo_com(&info, camadas::rotulo_inteiro);
    let encurtou = p.resumo_curto.is_some() || inteiro != resumo(&info);
    poe_a_dica(
        store,
        ids::CARD_PHYS_LAYERS,
        if encurtou { inteiro } else { String::new() },
    );
}

fn poe_a_dica(store: &mut WidgetStore, id: NodeId, dica: String) {
    if store.tooltip_for(id) != Some(dica.as_str()) {
        store.set_tooltip(id, dica);
    }
}

/// Um quadro de `quem` liga/desliga a célula `(a, b)` da matriz do mundo; o «+» mostra mais uma camada. `edita` =
/// `false` consome o clique sem mexer. `true` = era da fila.
pub(crate) fn clique_da_fila(
    host: &mut dyn PanelHostInternal,
    id: NodeId,
    quem: &Quem,
    a: u8,
    edita: bool,
) -> bool {
    if let Some(b) = quem.quadros.iter().position(|&o| o == id) {
        seam_reset_button(host, id);
        if edita {
            host.bus_mut()
                .push(EditorAction::LayerPairToggle { a, b: b as u8 });
        }
        return true;
    }
    if id == quem.mais {
        seam_reset_button(host, id);
        if let Some(layer) = camadas::proxima_livre() {
            host.bus_mut().push(EditorAction::LayerShow { layer });
        }
        return true;
    }
    false
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
    if super::physics_colide_fila::clique_da_forma(host, id, &info) {
        return true;
    }
    if clique_da_fila(host, id, &DO_COLLIDER, a, !solta(&info)) {
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
            let dd = lista(&DO_COLLIDER, i);
            // Sem nome: «Default» (a 0) ou «Layer n» (onda 9a, dono 07/11).
            let esperado = if i == 2 {
                "Enemy".to_owned()
            } else {
                camadas::rotulo_padrao(usize::from(i))
            };
            assert_eq!(dd.selected_label(), Some(esperado.as_str()), "camada {i}");
        }
        camadas::set_current(Default::default(), [u32::MAX; 32], 0xFF);
    }

    /// A lista só traz as visíveis; a camada do objecto, mesmo fora delas, nunca falta.
    #[test]
    fn a_lista_da_camada_so_traz_as_visiveis_e_a_do_objecto() {
        camadas::set_current(Default::default(), [u32::MAX; 32], 1 | 1 << 20);
        crate::set_cards_on(true);
        assert_eq!(opcoes(&DO_COLLIDER, 0).len(), 2);
        assert_eq!(lista(&DO_COLLIDER, 20).selected_label(), Some("Layer 20"));
        assert_eq!(opcoes(&DO_COLLIDER, 25).len(), 3, "a do objecto entra");
        assert_eq!(lista(&DO_COLLIDER, 25).selected_label(), Some("Layer 25"));
        // A face clássica guarda o alcance dos 8 chips de antes: 0–7, mais a 20 visível.
        crate::set_cards_on(false);
        assert_eq!(opcoes(&DO_COLLIDER, 0).len(), 9);
        camadas::set_current(Default::default(), [u32::MAX; 32], 0xFF);
    }
}
