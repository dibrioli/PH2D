//! Os gestos: cada controlo vira um pedido à família; os números da janela ficam no estado.

use ph2d_a11y::NodeId;
use ph2d_editor_core::interaction::{BlenderHitKind, InteractiveState, WidgetEvent, WidgetStore};
use ph2d_editor_core::panel::cards::user_data;
use ph2d_editor_core::panel::{EventOutcome, Panel, PanelHostInternal};
use ph2d_editor_core::widget::{ButtonState, TextInputState};
use ph2d_pixel_core::Forma;
use ph2d_pixel_tiles::desenho::Modo;

use crate::canal::{self, Alvo, Eixo, Pedido, Tela};
use crate::{PixelTilesPanel, PixelTilesPanelState, ids};

fn botao(store: &mut WidgetStore, id: NodeId) {
    store.register(
        id,
        InteractiveState::Button {
            state: ButtonState::Normal,
        },
    );
}

pub(crate) fn populate(store: &mut WidgetStore) {
    for id in ids::BOTOES.into_iter().chain([ids::MATCH_FLIPS]) {
        botao(store, id);
    }
    for id in [ids::TILE_W, ids::TILE_H] {
        store.register(
            id,
            InteractiveState::NumberInput {
                state: TextInputState::Normal,
                value: 1.0,
                buffer: 1.0_f64.to_string(),
                caret: 0,
                last_committed: 1.0,
                selection_anchor: None,
            },
        );
        store.set_number_range(id, 1.0, ids::TILE_SIDE_MAX, 1.0);
    }
    store.register(
        ids::TILES_DRAG,
        InteractiveState::BlenderHit {
            parent: PixelTilesPanel::NODE_ID,
            kind: BlenderHitKind::DragHandle,
        },
    );
    ph2d_editor_core::panel::cards::borda::regista(
        store,
        ids::TILES_EDGE,
        PixelTilesPanel::NODE_ID,
    );
}

/// As células do tileset: o sprite decide quantas são.
pub(crate) fn regista_tiles(store: &mut WidgetStore, n: usize) {
    for i in 0..n {
        botao(store, ids::tile(i));
    }
}

/// As linhas do cartão Terrain: o tileset decide quantos terrenos há.
pub(crate) fn regista_terrenos(store: &mut WidgetStore, n: usize) {
    for i in 0..n {
        botao(store, ids::terreno(i));
    }
}

/// As linhas da janela: o sprite decide quantos tilesets há.
pub(crate) fn regista_conjuntos(store: &mut WidgetStore, n: usize) {
    for i in 0..n {
        botao(store, ids::conjunto(i));
    }
}

/// O pedido de um clique fixo.
fn pedido_de(id: NodeId) -> Option<Pedido> {
    Some(match id {
        i if i == ids::TARGET_PIXELS => Pedido::Alvo(Alvo::Pixels),
        i if i == ids::TARGET_TILES => Pedido::Alvo(Alvo::Tiles),
        i if i == ids::TARGET_TERRAIN => Pedido::Alvo(Alvo::Terreno),
        i if i == ids::NEW_TERRAIN => Pedido::NovoTerreno,
        i if i == ids::DELETE_TERRAIN => Pedido::ApagaTerreno,
        i if i == ids::TERRAIN_TEMPLATE => Pedido::Gabarito,
        i if i == ids::MODE_MANUAL => Pedido::Modo(Modo::Manual),
        i if i == ids::MODE_AUTO => Pedido::Modo(Modo::Auto),
        i if i == ids::MODE_STACK => Pedido::Modo(Modo::Stack),
        i if i == ids::FLIP_X => Pedido::Flip(Eixo::X),
        i if i == ids::FLIP_Y => Pedido::Flip(Eixo::Y),
        i if i == ids::FLIP_D => Pedido::Flip(Eixo::D),
        i if i == ids::ADD_TILE => Pedido::NovoTile,
        i if i == ids::DELETE_TILE => Pedido::ApagaTile,
        _ => return None,
    })
}

fn fecha(st: &mut PixelTilesPanelState, p: Pedido) -> EventOutcome {
    st.tela = Tela::Tileset;
    canal::pede(p);
    EventOutcome::Consumed
}

fn numero(host: &dyn PanelHostInternal, id: NodeId) -> Option<u16> {
    let v = host.store().number_value(id)?;
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "a faixa do campo cabe em 16 bits"
    )]
    Some(ph2d_editor_core::math::safe_clamp_f64(v.round(), 1.0, ids::TILE_SIDE_MAX) as u16)
}

pub(crate) fn apply_event(
    st: &mut PixelTilesPanelState,
    host: &mut dyn PanelHostInternal,
    ev: WidgetEvent,
) -> EventOutcome {
    // Aqui nada distingue o duplo clique: dois cliques depressa são dois gestos.
    let ev = match ev {
        WidgetEvent::DoubleClick(id) => WidgetEvent::Click(id),
        outro => outro,
    };
    if let Some(v) = canal::contexto().user
        && let Some(e) = user_data::on_event(host.store(), &ids::user_data(), &v, &ev)
    {
        if let Some(e) = e {
            canal::pede(Pedido::UserData(e));
        }
        return EventOutcome::Consumed;
    }
    match ev {
        WidgetEvent::Click(id) if id == ids::TILES_CANCEL => fecha(st, Pedido::Fechou),
        WidgetEvent::Click(id) if id == ids::TILES_OK => {
            let escolhido = st.escolhido(&canal::contexto());
            let conjunto = escolhido.map(|c| c.id);
            let (w, h) = escolhido.map_or((st.tile_w, st.tile_h), |c| (c.tile_w, c.tile_h));
            let p = match st.tela {
                Tela::Converter => Pedido::Converte {
                    conjunto,
                    w,
                    h,
                    flips: st.flips && w == h,
                },
                _ => Pedido::NovaCamada {
                    conjunto,
                    w,
                    h,
                    forma: st.forma,
                },
            };
            fecha(st, p)
        }
        WidgetEvent::Click(id) if id == ids::NEW_SET => {
            st.conjunto = None;
            EventOutcome::Consumed
        }
        WidgetEvent::Click(id) if id == ids::MATCH_FLIPS => {
            st.flips = !st.flips;
            EventOutcome::Consumed
        }
        WidgetEvent::Click(id) if ids::SHAPES.contains(&id) => {
            let i = ids::SHAPES.iter().position(|s| *s == id).unwrap_or(0);
            let forma = Forma::TODAS[i];
            if st.tela == Tela::Tileset {
                canal::pede(Pedido::Forma(forma));
            } else {
                st.forma = forma;
            }
            EventOutcome::Consumed
        }
        WidgetEvent::Click(id) => {
            let x = canal::contexto();
            if let Some(c) = (0..x.conjuntos.len())
                .find(|&i| ids::conjunto(i) == id)
                .map(|i| x.conjuntos[i])
            {
                st.conjunto = Some(c.id);
                return EventOutcome::Consumed;
            }
            let n = x.imagens.len();
            let p = pedido_de(id)
                .or_else(|| {
                    (0..n)
                        .find(|&i| ids::tile(i) == id)
                        .map(|i| Pedido::Tile(i as u32))
                })
                .or_else(|| {
                    (0..x.terrenos.len())
                        .find(|&i| ids::terreno(i) == id)
                        .map(|i| Pedido::Terreno(i as u32))
                });
            match p {
                Some(p) => {
                    canal::pede(p);
                    EventOutcome::Consumed
                }
                None => EventOutcome::Ignored,
            }
        }
        WidgetEvent::ValueChanged(id) if id == ids::TILE_W || id == ids::TILE_H => {
            if let Some(n) = numero(host, id) {
                if id == ids::TILE_W {
                    st.tile_w = n;
                } else {
                    st.tile_h = n;
                }
            }
            EventOutcome::Consumed
        }
        _ => EventOutcome::Ignored,
    }
}
