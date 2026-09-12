//! ⭐⭐⭐ **`ph2d-panel-skeleton`** — o painel do ESQUELETO, `Panel<State>` tipado (ADR-0029).
//!
//! O que ele oferece é o que o gesto do modo Osso **não** pode dar: prender formas ao esqueleto,
//! soltá-las, os dois números de um osso, a âncora de cinemática inversa, o limite de ângulo da
//! junta e o **osso inteligente** (girar um osso percorre uma animação inteira).
//!
//! # ⛔⛔ Por que ele saiu do painel de vetor
//!
//! Ele era uma **secção** de lá, e a medição de 2026-09-09 disse o preço: com só um osso escolhido,
//! o cabeçalho dela caía em `y = 1394 px` sobre uma faixa visível de `774` — e acima dela havia
//! **785 px** de secções a falar de coisas que um osso **não tem**:
//!
//! | secção acima | espaço | um osso tem? |
//! |---|---|---|
//! | Traço | 422 px | não |
//! | Encaixe | 246 px | é do desenho |
//! | Mistura | 188 px | não |
//! | Preenchimento | 88 px | não |
//! | Morph | 87 px | não |
//!
//! *Ela nunca cabia na tela por si*, e foi isso que produziu o report do dono
//! (*«selecionar o bone nem sempre abre a seção de skeleton»*) — a rolagem automática tratou o
//! sintoma. **A escolha do painel próprio é dele**, com as três saídas na mão.
//!
//! # ⚠️ Os ids NÃO foram renomeados
//!
//! Um [`ph2d_a11y::NodeId`] é o hash de uma STRING: `vector.bone.*` continua a ser o endereço de
//! cada controlo, e o que mudou é **quem os pinta**. Renomeá-los quebraria tudo o que os referencia
//! por nome — o registo, o encaminhamento e os gates —, e o ganho seria estético.
//!
//! # ⚠️ O corpo vive no vocabulário PARTILHADO
//!
//! As linhas (cabeçalho de secção, botão, botão rotulado, campo rotulado, segmentos) são
//! [`ph2d_editor_core::panel::RowCtx`], a mesma porta que o painel de vetor usa. *O que se
//! duplicaria não é código: é o RITMO* — e duas cópias divergem na primeira vez que alguém mexe
//! num vão.

#![forbid(unsafe_code)]

pub mod ids;
mod paint;
mod seam;
mod section;
pub mod state;

pub use state::{
    SmartBoneView, set_current_bone, set_current_bone_actions, set_current_bone_ik,
    set_current_bone_limit, set_current_bone_smart, set_current_bone_tool, set_current_skin_deform,
    set_current_skinned, set_current_skinned_image,
};

use ph2d_a11y::NodeId;
use ph2d_editor_core::interaction::{WidgetEvent, WidgetStore};
use ph2d_editor_core::panel::{EventOutcome, PaintCtx, Panel, PanelHostInternal};

/// Marcador de tamanho zero que implementa o contrato tipado do painel do esqueleto.
pub struct SkeletonPanel;

impl Panel for SkeletonPanel {
    type State = state::SkeletonPanelState;

    const ID: &'static str = "skeleton";
    const NODE_ID: NodeId = ph2d_editor_core::ids::SKELETON_PANEL;
    /// ⛔ **Nasce FECHADO**, e quem o abre é a shell: ele só tem sujeito numa cena que tem ossos —
    /// *um painel que fala de algo que não existe é ruído*, que é a mesma lei que a secção seguia.
    const DEFAULT_VISIBLE: bool = false;
    const TITLE: &'static str = "Bones";
    /// ⚠️ **Escrito na INTEGRAÇÃO de 2026-09-10, não por esta linha.** O `Panel::ICON` nasceu
    /// obrigatório na `line/UIUX` (fusão 1 da mesma rodada) e este é o único painel novo da rodada
    /// ⇒ o único que não compilava. ⛔ **O glifo é NOVO de propósito** (`docs/design/icons/bone.svg`,
    /// Lucide `bone`, ISC, como os outros 137): reaproveitar um já usado reprova o
    /// `no_two_panels_share_a_glyph`, e reaproveitar um livre mas alheio ao assunto — `Rigid`,
    /// `Pivot`, `Gizmo` — entrega duas abas indistinguíveis, que é exactamente o defeito que a
    /// **ausência de default** naquele trait existe para impedir.
    const ICON: ph2d_editor_core::icons::IconId = ph2d_editor_core::icons::IconId::Bone;
    /// ⚠️ **Uma lista de propriedades não cabe na faixa de baixo** (240 px de altura): ali ela fica
    /// com duas linhas visíveis. ⇒ as duas colunas, e o gesto que o levaria ao fundo não é oferecido.
    const ALLOWED_SLOTS: ph2d_editor_core::screens::slot::SlotSet =
        ph2d_editor_core::screens::slot::SlotSet::SIDES;
    const DEFAULT_SLOT: ph2d_editor_core::screens::slot::Slot =
        ph2d_editor_core::screens::slot::Slot::RightTop;

    fn paint(state: &mut state::SkeletonPanelState, ctx: &mut PaintCtx) {
        paint::paint(state, ctx);
    }

    fn apply_event(
        state: &mut state::SkeletonPanelState,
        host: &mut dyn PanelHostInternal,
        ev: WidgetEvent,
    ) -> EventOutcome {
        seam::apply_event(state, host, ev)
    }

    fn populate(store: &mut WidgetStore) {
        seam::populate(store);
    }
}
