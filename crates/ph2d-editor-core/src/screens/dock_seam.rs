//! **A COSTURA DE LARGURA de uma coluna docada** — as medidas do chrome, o vocabulário dos lados,
//! e a geometria do agarre.
//!
//! Cortado do `layout.rs` em 2026-08-30 pelo tecto de LOC (777/700), e o corte é por
//! RESPONSABILIDADE: aquele ficheiro responde *«onde fica cada rect?»* e este responde *«como uma
//! coluna se redimensiona?»* — uma pergunta sobre um GESTO, e o único sítio deste módulo que
//! precisa de saber que existe um ponteiro.

use super::layout::{HIERARCHY_W, HeroLayout, INSPECTOR_W, TIMELINE_DOCK_H, TOPBAR_H};
use crate::zones::Rect;

/// **As MEDIDAS do chrome deste quadro** — larguras e alturas, nunca modos.
///
/// ⭐ *«Sem chrome legado»* é `rail_w = 0` e `top_bar_h = 0`; *«a coluna foi arrastada»* é um
/// `left_dock_w` diferente. O [`HeroLayout`] não conhece nenhuma dessas frases — ele recebe
/// números, e a aritmética dele é a mesma sempre.
///
/// ⚠️ **Uma struct e não seis argumentos:** o construtor já tinha cinco, e cada medida nova a mais
/// é uma posição a mais para trocar em silêncio — `for_viewport_bands(v, m, 57.0, 64.0, 308.0,
/// 304.0, …)` é uma linha que ninguém revê.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct ChromeBands {
    /// Largura do trilho de ferramentas. `0` = fora.
    pub rail_w: f32,
    /// Altura da barra de topo. `0` = fora.
    pub top_bar_h: f32,
    /// Largura da coluna da ESQUERDA.
    pub left_dock_w: f32,
    /// Largura da coluna da DIREITA.
    pub right_dock_w: f32,
    /// ⭐ Altura da **fila de ferramentas** por cima da área de desenho. `0` = fora.
    ///
    /// ⚠️ **Ela sai da ÁREA, não da janela** — ao contrário das outras quatro, que cortam a
    /// viewport. É a spec §4: a toolbar é uma REGIÃO da área, irmã da régua, e por isso vive entre
    /// as colunas em vez de atravessar o ecrã.
    pub tool_bar_h: f32,
    /// ⭐⭐ Altura do **CABEÇALHO DA ÁREA** (D2, metade 2) — a faixa do editor, por cima da fila de
    /// ferramentas. `0` = fora.
    ///
    /// ⚠️ Como a fila, ela sai da **ÁREA** e não da janela: é uma região, não uma barra.
    pub area_header_h: f32,
    /// ⭐ Altura da **faixa do fundo** (o timeline, ou a tira do Flip por baixo dele).
    ///
    /// ⚠️ Ela é AUTORADA como as duas colunas (`WidgetStore::dock_bottom_h`) — o topo dela é uma
    /// costura, e quem partilha a banda segue. ⛔ Não é um interruptor: a faixa só é **desenhada**
    /// se o painel dela estiver visível, e essa pergunta é do `hero/paint.rs`.
    pub bottom_dock_h: f32,
}

impl ChromeBands {
    /// As medidas de fábrica — o trilho e a barra presentes, as colunas nas larguras dos tokens.
    pub const DEFAULT: Self = Self {
        rail_w: 57.0, // LITERAL-PX-OK: espelha `tool_rail_width_px()` no preset Small
        top_bar_h: TOPBAR_H,
        left_dock_w: HIERARCHY_W,
        right_dock_w: INSPECTOR_W,
        // ⚠️ **ZERO, e é o mockup que o pede**: o `DEFAULT` descreve a referência de desenho, que
        // tem o trilho VERTICAL. A fila horizontal é o chrome de produção, e quem a mede é o
        // `hero/paint.rs` — ela depende do preset de tamanho do chip, que é autorado.
        tool_bar_h: 0.0,
        // ⚠️ **ZERO no mockup**, pela razão da fila: o `DEFAULT` descreve a referência de desenho,
        // e o cabeçalho é chrome de produção — quem o mede é o `hero/frame_layout.rs`.
        area_header_h: 0.0,
        bottom_dock_h: TIMELINE_DOCK_H,
    };
}

/// **A faixa de agarre que redimensiona uma coluna** — a borda INTERIOR dela.
///
/// ⭐ Enio, 2026-08-30: *«os painéis devem ser redimensionáveis para esquerda e para direita e com
/// setas bidirecionais no cursor; os pontinhos de redimensionamento podem ser retirados. A borda
/// inteira serve para redimensionar»*.
///
/// ⚠️ **Ela vive DENTRO da coluna**, nos últimos [`DOCK_SEAM_PX`] px antes da área de desenho — e
/// não a cavalo da fronteira, como uma costura de divisória costuma ser. A razão é medida: desde
/// que a régua ficou **colada** à coluna, a faixa dela começa exactamente onde a coluna acaba, e
/// uma costura centrada roubaria metade do agarre à régua. *A borda é do painel.*
pub const DOCK_SEAM_PX: f32 = 6.0; // LITERAL-PX-OK: largura de agarre, irmã do GRAB_HALF_PX do 3D

/// Qual coluna, quando o ponteiro está sobre uma costura de largura.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum DockSide {
    /// A coluna da esquerda.
    Left,
    /// A coluna da direita.
    Right,
}

impl HeroLayout {
    /// **As duas colunas laterais, ORDENADAS POR `x`** — `(esquerda, direita)`.
    ///
    /// ⚠️ **Existe para o `mirrored` não ser uma inversão escrita à mão em cada chamador.** Sob
    /// espelho a Hierarchy vai para a direita e o dock de *takeover* para a esquerda; pedir
    /// *«o rect da Hierarchy»* e chamar-lhe *«a coluna da esquerda»* é a forma exacta do erro
    /// que o compilador não vê. Aqui a resposta vem da **posição**, que é o que a pergunta
    /// significa.
    #[must_use]
    pub fn side_columns(&self) -> (Rect, Rect) {
        if self.hierarchy.x <= self.inspector.x {
            (self.hierarchy, self.inspector)
        } else {
            (self.inspector, self.hierarchy)
        }
    }

    /// **A faixa de agarre que redimensiona esta coluna** — os últimos [`DOCK_SEAM_PX`] px dela,
    /// do lado da área de desenho.
    ///
    /// ⚠️ Devolve um rect de largura zero quando a coluna está vazia (`w == 0`): sem painel não há
    /// borda para agarrar, e um agarre sobre o nada seria chrome vivo e invisível.
    #[must_use]
    pub fn dock_seam(&self, side: DockSide) -> Rect {
        let (left_col, right_col) = self.side_columns();
        let col = match side {
            DockSide::Left => left_col,
            DockSide::Right => right_col,
        };
        let occupied = match side {
            DockSide::Left => self.docks.left,
            DockSide::Right => self.docks.right,
        };
        if !occupied || col.w <= 0.0 || col.h <= 0.0 {
            return Rect::new(col.x, col.y, 0.0, 0.0);
        }
        let w = DOCK_SEAM_PX.min(col.w);
        match side {
            // A borda INTERIOR: à direita na coluna da esquerda, à esquerda na da direita.
            DockSide::Left => Rect::new(col.x + col.w - w, col.y, w, col.h),
            DockSide::Right => Rect::new(col.x, col.y, w, col.h),
        }
    }

    /// **Sobre qual costura de largura está o ponteiro, se alguma** — a porta única do gesto e do
    /// cursor.
    ///
    /// ⚠️ Ela existe para que *«onde a seta bidirecional aparece»* e *«onde o arrasto pega»* sejam
    /// a **mesma** pergunta: a seta a aparecer um pixel ao lado de onde o gesto agarra lê-se como
    /// «às vezes não pega», que é o defeito que o irmão dela no 3D já pagou.
    #[must_use]
    pub fn dock_seam_at(&self, p: (f32, f32)) -> Option<DockSide> {
        for side in [DockSide::Left, DockSide::Right] {
            let r = self.dock_seam(side);
            if r.w > 0.0 && p.0 >= r.x && p.0 < r.x + r.w && p.1 >= r.y && p.1 < r.y + r.h {
                return Some(side);
            }
        }
        None
    }

    /// A largura que a coluna passa a ter se a costura for solta em `x`.
    ///
    /// ⚠️ **A conta é do LADO**: à esquerda a largura cresce com o `x`, à direita ela decresce —
    /// é a inversão que se escreve ao contrário sem o compilador reclamar.
    #[must_use]
    pub fn dock_width_for(&self, side: DockSide, x: f32) -> f32 {
        let (left_col, right_col) = self.side_columns();
        match side {
            DockSide::Left => x - left_col.x,
            DockSide::Right => right_col.x + right_col.w - x,
        }
    }
}
