#![forbid(unsafe_code)]
//! `ph2d-panel-model3d` — **o painel do módulo de modelagem 3D** ([ADR-0161], W4).
//!
//! Uma linha por nó do documento que tenha **raio**, com o raio editável ao vivo.
//!
//! # ⭐ Por que este painel é a demonstração do módulo inteiro
//!
//! A promessa do modelador implícito é *o raio do filete fica editável para sempre*, porque ele é
//! parâmetro da operação e não geometria assada — coisa que nem o Blender nem o MoI dão. Um painel
//! que mostra esses raios e os deixa mexer **é** essa promessa, e é a única forma de a provar sem
//! pedir a ninguém que acredite.
//!
//! # ⚠️ Não confundir com `ph2d-panel-sculpt3d`
//!
//! Aquele é o painel do módulo de **escultura**. São dois módulos 3D, duas linhas paralelas, e dois
//! prefixos de id que nunca se cruzam.
//!
//! # O limite mostrado é o limite APLICADO
//!
//! Nenhuma faixa é recalculada aqui: o teto de cada raio vem de [`ph2d_field::FieldDoc::radius_bound`],
//! a mesma função que a validação usa. Dois lados a calcular a mesma regra é como um controle passa
//! a oferecer valores que o documento recusa — e o artista vê o número parar sem explicação.
//!
//! [ADR-0161]: ../../../docs/architecture/decisions/0161-3d-modeling-is-an-implicit-field-tree-and-what-the-artist-sees-is-the-traced-field.md

pub mod state;

mod event;
mod paint;
/// As três formas de uma linha do painel — ver [`paint_rows`].
mod paint_rows;
mod populate;

pub use populate::{CHIP_FAMILY_COUNT, MAX_MODES, MAX_ROWS};

/// O identificador do painel — a **chave de visibilidade** que o shell usa para o abrir.
///
/// ⚠️ Exposto aqui de propósito: sem isto quem abre o painel escreve o literal `"model3d"`,
/// e uma segunda cópia dessa chave é exatamente como se alterna a visibilidade de um painel
/// que ninguém pinta — o modo de falha que o comentário do `panel-physics` no `Cargo.toml`
/// do shell já regista como pago.
pub const PANEL_ID: &str = "model3d";
pub use state::{
    ModeChip, ModelIntent, ModelSnapshot, ParamRow, drain_intents, last_content_h, publish,
};

use ph2d_a11y::NodeId;
use ph2d_editor_core::interaction::{WidgetEvent, WidgetStore};
use ph2d_editor_core::panel::{EventOutcome, PaintCtx, Panel, PanelHostInternal};

/// Marcador de tamanho zero que implementa o contrato tipado do painel.
///
/// ⚠️ O nome é load-bearing: o `ph2d-panel-sync` extrai `pub struct <Nome>Panel` deste arquivo e
/// entra em pânico se ele não existir.
pub struct Model3dPanel;

impl Panel for Model3dPanel {
    type State = state::Model3dPanelState;

    const ID: &'static str = PANEL_ID;
    const NODE_ID: NodeId = ph2d_editor_core::ids::MODEL3D_PANEL;
    /// Fechado até alguém o abrir. O módulo ainda entra por variável de ambiente, e um painel que
    /// nascesse aberto ocuparia o encaixe da direita em toda sessão que não é de modelagem.
    const DEFAULT_VISIBLE: bool = false;

    fn paint(state: &mut Self::State, ctx: &mut PaintCtx) {
        paint::paint(state, ctx);
    }

    fn apply_event(
        state: &mut Self::State,
        host: &mut dyn PanelHostInternal,
        ev: WidgetEvent,
    ) -> EventOutcome {
        event::apply_event(state, host, ev)
    }

    fn populate(store: &mut WidgetStore) {
        populate::populate(store);
    }
}
