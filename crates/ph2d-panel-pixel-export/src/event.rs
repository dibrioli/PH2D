//! Os gestos das janelas: cada controlo escreve a sua opção, e o verbo volta à família pelo canal.

use ph2d_a11y::NodeId;
use ph2d_editor_core::interaction::{BlenderHitKind, InteractiveState, WidgetEvent, WidgetStore};
use ph2d_editor_core::panel::{EventOutcome, Panel, PanelHostInternal};
use ph2d_editor_core::widget::{ButtonState, TextInputState};
use ph2d_pixel_io::abrir::celulas;
use ph2d_pixel_io::exporta::{Export, ExportAs, ExportFolha};
use ph2d_pixel_io::render::Quadros;
use ph2d_pixel_io::sheet::Limite;

use crate::canal::{self, Pedido, Tela};
use crate::cards_export::DIRECOES;
use crate::cards_folha::{TIPOS, id_do_limite, limite_de, limites_do_tipo};
use crate::comum::{FORMATOS, area_de, camada_de, quadro_de};
use crate::state::{OpcoesAs, OpcoesFo, OpcoesTi, grelha_inicial};
use crate::{PixelExportPanel, PixelExportPanelState, ids};

fn botao(store: &mut WidgetStore, id: NodeId) {
    store.register(
        id,
        InteractiveState::Button {
            state: ButtonState::Normal,
        },
    );
}

pub(crate) fn populate(store: &mut WidgetStore) {
    let fixos = [
        ids::EXPORT_RESET,
        ids::CANCEL,
        ids::OK,
        ids::LAYERS_VISIBLE,
        ids::LAYERS_ALL,
    ]
    .into_iter()
    .chain([ids::FRAMES_ALL, ids::FRAMES_RANGE, ids::AREA_CANVAS])
    .chain([
        ids::JSON_HASH,
        ids::JSON_ARRAY,
        ids::BY_ROWS,
        ids::BY_COLUMNS,
        ids::AS_PIXEL,
        ids::AS_IMAGE,
    ])
    .chain(ids::FORMATS)
    .chain(ids::BOXES)
    .chain(ids::DIRECTIONS)
    .chain(ids::SHEET_TYPES)
    .chain(ids::LIMIT_OPTIONS)
    .chain(ids::CHECKS);
    for id in fixos {
        botao(store, id);
    }
    for (id, min, max) in ids::NUMEROS {
        store.register(
            id,
            InteractiveState::NumberInput {
                state: TextInputState::Normal,
                value: min,
                buffer: min.to_string(),
                caret: 0,
                last_committed: min,
                selection_anchor: None,
            },
        );
        store.set_number_range(id, min, max, 1.0);
    }
    store.register(
        ids::ITEM_NAME,
        InteractiveState::TextInput {
            state: TextInputState::Normal,
            text: String::new(),
            caret: 0,
            selection_anchor: None,
        },
    );
    store.register(
        ids::EXPORT_DRAG,
        InteractiveState::BlenderHit {
            parent: PixelExportPanel::NODE_ID,
            kind: BlenderHitKind::DragHandle,
        },
    );
}

/// Regista as linhas das camadas e tags deste sprite (o sprite decide quantas são).
pub(crate) fn regista_linhas(store: &mut WidgetStore, x: &canal::Contexto) {
    for i in 0..x.camadas.len() {
        botao(store, ids::layer_option(i));
    }
    for i in 0..x.tags.len() {
        botao(store, ids::tag_option(i));
    }
    for i in 0..x.tilesets.len() {
        botao(store, ids::tileset_option(i));
    }
    for i in 0..x.slices.len() {
        botao(store, ids::area_option(i));
    }
}

fn fecha(host: &mut dyn PanelHostInternal) -> EventOutcome {
    host.set_panel_visible(PixelExportPanel::ID, false);
    EventOutcome::Consumed
}

fn numero(host: &dyn PanelHostInternal, id: NodeId) -> Option<u32> {
    let v = host.store().number_value(id)?;
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "faixa da tabela"
    )]
    Some(v.round().max(0.0) as u32)
}

/// O intervalo depois de um dos dois números mudar; os campos contam de 1.
fn intervalo(q: Quadros, de: Option<u32>, ate: Option<u32>) -> Quadros {
    let Quadros::Intervalo { de: d, ate: a } = q else {
        return q;
    };
    Quadros::Intervalo {
        de: de.map_or(d, |n| n.saturating_sub(1) as usize),
        ate: ate.map_or(a, |n| n.saturating_sub(1) as usize),
    }
}

fn verbo(st: &PixelExportPanelState) -> Option<Pedido> {
    match st.tela {
        Tela::ExportAs => Some(Pedido::Exportar(Export::Ficheiro(ExportAs {
            pedido: st.as_.pedido(),
            formato: st.as_.formato,
            repeat: st.as_.lacos,
        }))),
        Tela::Folha => Some(Pedido::Exportar(Export::Folha(ExportFolha {
            opcoes: st.fo.folha.clone(),
            json: st.fo.json_ligado.then_some(st.fo.json),
        }))),
        Tela::Tileset => st
            .tiles
            .pedido(&canal::contexto())
            .map(|t| Pedido::Exportar(Export::Tiles(t))),
        Tela::ImportComo => Some(Pedido::ImportarComo {
            pixel: st.como_pixel,
        }),
        Tela::Import => {
            let i = canal::imagem()?;
            let (c, r) = celulas(&st.grelha, i.w, i.h);
            (c > 0 && r > 0).then_some(Pedido::Importar(st.grelha))
        }
    }
}

fn clique(st: &mut PixelExportPanelState, id: NodeId) -> bool {
    let x = canal::contexto();
    if let Some(i) = (0..x.tilesets.len()).find(|&i| ids::tileset_option(i) == id) {
        st.tiles.conjunto = Some(x.tilesets[i].0);
        return true;
    }
    if let Some(i) = ids::FORMATS.iter().position(|f| *f == id) {
        st.as_.formato = FORMATOS[i];
        return true;
    }
    if let Some(i) = ids::DIRECTIONS.iter().position(|f| *f == id) {
        st.as_.direcao = DIRECOES[i];
        return true;
    }
    if let Some(i) = ids::SHEET_TYPES.iter().position(|f| *f == id) {
        let t = TIPOS[i];
        st.fo.folha.tipo = t;
        // O limite que o tipo novo não aceita cai.
        let l = id_do_limite(st.fo.folha.limite);
        if !limites_do_tipo(t).iter().any(|x| id_do_limite(*x) == l) {
            st.fo.folha.limite = Limite::Nenhum;
        }
        return true;
    }
    if let Some(l) = limite_de(id, st.fo.limite_n) {
        st.fo.folha.limite = l;
        return true;
    }
    if let Some(a) = area_de(&x, id) {
        st.as_.area = a;
        return true;
    }
    if let Some(c) = camada_de(&x, id) {
        match st.tela {
            Tela::Folha => st.fo.folha.camadas = c,
            _ => st.as_.camadas = c,
        }
        return true;
    }
    if let Some(q) = quadro_de(&x, id) {
        match st.tela {
            Tela::Folha => st.fo.folha.quadros = q,
            _ => st.as_.quadros = q,
        }
        return true;
    }
    if id == ids::JSON_HASH || id == ids::JSON_ARRAY {
        st.fo.json.array = id == ids::JSON_ARRAY;
        return true;
    }
    if id == ids::AS_PIXEL || id == ids::AS_IMAGE {
        st.como_pixel = id == ids::AS_PIXEL;
        return true;
    }
    if id == ids::BY_ROWS || id == ids::BY_COLUMNS {
        st.grelha.por_colunas = id == ids::BY_COLUMNS;
        return true;
    }
    let f = &mut st.fo.folha;
    let j = &mut st.fo.json;
    let alvo = match id {
        i if i == ids::MERGE => &mut f.merge_duplicates,
        i if i == ids::IGNORE_EMPTY => &mut f.ignore_empty,
        i if i == ids::POW2 => &mut f.potencia_de_2,
        i if i == ids::SPLIT_LAYERS => &mut f.split_layers,
        i if i == ids::SPLIT_TAGS => &mut f.split_tags,
        i if i == ids::SPLIT_GRID => &mut f.split_grid,
        i if i == ids::SPLIT_SLICES => &mut f.split_slices,
        i if i == ids::TRIM_BY_GRID => &mut f.trim_by_grid,
        i if i == ids::TRIM_CELS => &mut f.trim_cels,
        i if i == ids::TRIM_SPRITE => &mut f.trim_sprite,
        i if i == ids::JSON => &mut st.fo.json_ligado,
        i if i == ids::NORMAL_SHEET => &mut f.normal,
        i if i == ids::GODOT => &mut f.godot,
        i if i == ids::JSON_TAGS => &mut j.tags,
        i if i == ids::JSON_LAYERS => &mut j.layers,
        i if i == ids::JSON_SLICES => &mut j.slices,
        _ => return false,
    };
    *alvo = !*alvo;
    true
}

fn valor(st: &mut PixelExportPanelState, host: &dyn PanelHostInternal, id: NodeId) -> bool {
    let Some(n) = numero(host, id) else {
        return false;
    };
    let (de, ate) = ((id == ids::FROM).then_some(n), (id == ids::TO).then_some(n));
    let f = &mut st.fo.folha;
    let g = &mut st.grelha;
    match id {
        i if i == ids::FROM || i == ids::TO => match st.tela {
            Tela::Folha => f.quadros = intervalo(f.quadros, de, ate),
            _ => st.as_.quadros = intervalo(st.as_.quadros, de, ate),
        },
        i if i == ids::SCALE => st.as_.escala = n,
        i if i == ids::LOOPS => st.as_.lacos = u16::try_from(n).unwrap_or(u16::MAX),
        i if i == ids::LIMIT_N => st.fo.acerta_limite(n),
        i if i == ids::COLUMNS => st.tiles.colunas = n.max(1),
        i if i == ids::BORDER => f.border = n,
        i if i == ids::SHAPE => f.shape = n,
        i if i == ids::INNER => f.inner = n,
        i if i == ids::EXTRUDE => f.extrude = n,
        i if i == ids::GRID_X => g.x = n,
        i if i == ids::GRID_Y => g.y = n,
        i if i == ids::GRID_W => g.w = n,
        i if i == ids::GRID_H => g.h = n,
        i if i == ids::GAP_X => g.gap_x = n,
        i if i == ids::GAP_Y => g.gap_y = n,
        _ => return false,
    }
    true
}

pub(crate) fn apply_event(
    st: &mut PixelExportPanelState,
    host: &mut dyn PanelHostInternal,
    ev: WidgetEvent,
) -> EventOutcome {
    match ev {
        WidgetEvent::Click(id) if id == ids::CANCEL => fecha(host),
        WidgetEvent::Click(id) if id == ids::EXPORT_RESET => {
            match st.tela {
                Tela::ExportAs => st.as_ = OpcoesAs::default(),
                Tela::Folha => st.fo = OpcoesFo::default(),
                Tela::Tileset => st.tiles = OpcoesTi::default(),
                Tela::ImportComo => {}
                Tela::Import => {
                    st.grelha = canal::imagem()
                        .and_then(|i| i.sugestao)
                        .unwrap_or_else(grelha_inicial);
                }
            }
            EventOutcome::Consumed
        }
        WidgetEvent::Click(id) if id == ids::OK => {
            if let Some(p) = verbo(st) {
                canal::pede(p);
            }
            EventOutcome::Consumed
        }
        WidgetEvent::Click(id) if clique(st, id) => EventOutcome::Consumed,
        WidgetEvent::ValueChanged(id) if valor(st, host, id) => EventOutcome::Consumed,
        WidgetEvent::TextChanged(id) if id == ids::ITEM_NAME => {
            let t = host.store().text(id).unwrap_or("").trim();
            st.fo.folha.nome = (!t.is_empty()).then(|| t.to_owned());
            EventOutcome::Consumed
        }
        _ => EventOutcome::Ignored,
    }
}
