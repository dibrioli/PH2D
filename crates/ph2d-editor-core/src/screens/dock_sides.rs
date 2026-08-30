//! **Quais colunas laterais estão ocupadas** — o insumo que o [`super::layout::HeroLayout`] não
//! consegue derivar de si mesmo, e a peça que já custou três tentativas.
//!
//! Cortado do `layout.rs` em 2026-08-30 pelo tecto de LOC (711/700), e o corte é por
//! RESPONSABILIDADE: o `layout.rs` responde *«onde fica cada rect?»* e este módulo responde
//! *«qual coluna tem alguém lá dentro?»* — a segunda pergunta é sobre o mundo, não sobre
//! geometria, e é a única do ficheiro que precisa de um facto de runtime.

use crate::zones::Rect;

/// Quais colunas laterais estão **abertas** neste quadro — a única coisa que o layout não
/// consegue derivar de si mesmo.
///
/// ⚠️ **Um painel fechado não ocupa coluna**, e a [`HeroLayout::draw_area`] tem de crescer para
/// dentro dela: reservar a faixa de um painel que não está lá poria a régua da esquerda a
/// flutuar no meio do desenho, e não a reservar quando ele ESTÁ lá devolve o defeito que a
/// área existe para curar. O sítio que sabe a resposta é o mesmo que constrói o layout
/// (`screens/hero/paint.rs`), e é lá que a pergunta é feita.
///
/// ⛔⛔ **SÓ HÁ UM CAMPO, e a ausência do segundo é a correcção de uma REGRESSÃO
/// (auditoria de 2026-08-30).** A 1.ª versão tinha um `right: bool` alimentado por uma lista de
/// cinco chaves (`["inspector", "bgremoval", "padding", "painter_sidebar", "painter_layers"]`).
/// A lista estava **errada**, e errada exactamente no modo que importa:
///
/// - ao pegar na ferramenta Vector, o *bridge* dela põe `panel_visible("inspector") = false`
///   (`shells/desktop/src/render_loop/vector_bridge.rs`) e o **painel Vector** passa a desenhar
///   no rect do dock direito (`ph2d-panel-vector/src/paint.rs`, `ctx.layout.inspector`);
/// - `"vector"` não estava na lista ⇒ `right` dava `false` ⇒ a área crescia **para dentro do
///   painel** e a régua de cima ficava **31,2 % tapada** — *pior* que os 29,4 % que esta wave
///   dizia ter curado —, com o gesto da guia a roubar os 20 px de cima do cabeçalho dele.
///
/// ⭐⭐ **A cura não é uma lista maior: são DEZASSETE as crates de painel que desenham no rect
/// do dock direito** (`ctx.layout.inspector` / `ctx.layout.padding`) — é um slot de *takeover*
/// com inquilinos mutuamente exclusivos, não um painel. Uma lista de dezassete nomes mantida à
/// mão numa crate que não os conhece apodrece no primeiro painel novo.
///
/// ⛔⛔ **A 2.ª tentativa foi um TEOREMA, e ele durou UM DIA.** Ele dizia: *a única coisa que lê
/// a `draw_area` é a régua, e `rulers_live()` exige `panel_visible("vector")` ⇒ régua viva ⇒
/// coluna da direita ocupada ⇒ reservá-la sempre custa zero*. No mesmo 2026-08-30 o Enio pediu
/// **as réguas em todos os modos e layouts**, e a primeira implicação evaporou-se. *Quem move o
/// número que tornava algo inalcançável tem de reconferir a nota* (`CLAUDE.md` §0.0) — e quem o
/// moveu fui eu, horas depois de o escrever.
///
/// ⭐⭐⭐ **A 3.ª é a que a auditoria já recomendava, e não é uma lista nem uma dedução:
/// PERGUNTA-SE AO QUE ACONTECEU.** Todo painel deste app publica o próprio rect por quadro
/// (`WidgetStore::set_panel_rect`) e **limpa-o quando deixa de ser pintado** — são 20 crates a
/// fazê-lo. ⇒ *«esta coluna está ocupada?»* responde-se cruzando os rects publicados com o rect
/// da coluna, e a resposta é imune a inquilino novo, a *bridge* novo e a lista esquecida —
/// **porque não há lista**.
///
/// ⚠️ **Preço, nomeado: UM QUADRO de atraso** ao abrir ou fechar um painel; a área ajusta-se no
/// quadro seguinte. Invisível a 60 fps, e é o preço de perguntar por um facto em vez de o prever.
///
/// ⚠️ **E os dois campos voltam a ser LADOS**, não painéis: com a resposta a vir da geometria, o
/// `mirrored` já está embutido em *qual coluna é qual*, e quem constrói o layout entrega os dois
/// rects. A 2.ª tentativa chamou-lhe `hierarchy_open` porque só um lado podia ficar vazio; com a
/// derivação, os dois podem.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct DockSides {
    /// A coluna da ESQUERDA está ocupada por algum painel?
    pub left: bool,
    /// E a da DIREITA?
    pub right: bool,
}

/// **Quanto de uma coluna um painel tem de cobrir para a ocupar.**
///
/// ⚠️ Não é `> 0`: painéis flutuantes (o Grid Snap, a galeria de widgets, um popover) roçam a
/// coluna sem a tomar, e um roçar a reservá-la faria a régua saltar enquanto o artista arrasta
/// outra coisa. Meia coluna é o que separa *«está lá»* de *«passou por cima»* — os inquilinos a
/// sério publicam o rect da coluna INTEIRO.
const COLUMN_TAKEN_FRAC: f32 = 0.5; // LITERAL-PX-OK: fracção de área, não um token de desenho

impl DockSides {
    /// As duas colunas ocupadas — o estado do mockup de referência, e o que os construtores que
    /// **não perguntam** assumem (`for_viewport` e irmãos, usados por fixtures e testes de
    /// geometria de chrome, que pintam os dois painéis).
    pub const BOTH: Self = Self {
        left: true,
        right: true,
    };
    /// Nenhuma coluna ocupada — a área de desenho vai do trilho à borda direita.
    pub const NONE: Self = Self {
        left: false,
        right: false,
    };

    /// **Pergunta aos rects que os painéis PUBLICARAM** quais colunas estão ocupadas — a porta
    /// única, e a razão de não haver lista nenhuma.
    ///
    /// `left_col` e `right_col` são os rects das duas colunas — tire-os de
    /// [`HeroLayout::side_columns`], que os devolve **ordenados por `x`** e é por isso imune ao
    /// `mirrored`. `published` é o que o `WidgetStore` guardou no quadro anterior.
    #[must_use]
    pub fn from_published(
        left_col: Rect,
        right_col: Rect,
        published: impl IntoIterator<Item = Rect>,
    ) -> Self {
        let mut out = Self::NONE;
        for r in published {
            if takes(r, left_col) {
                out.left = true;
            }
            if takes(r, right_col) {
                out.right = true;
            }
        }
        out
    }
}

/// O rect `r` **toma** a coluna `col`? (cobre ao menos [`COLUMN_TAKEN_FRAC`] da área dela)
fn takes(r: Rect, col: Rect) -> bool {
    let area = col.w * col.h;
    if area <= 0.0 {
        return false;
    }
    let w = (r.x + r.w).min(col.x + col.w) - r.x.max(col.x);
    let h = (r.y + r.h).min(col.y + col.h) - r.y.max(col.y);
    if w <= 0.0 || h <= 0.0 {
        return false;
    }
    (w * h) / area >= COLUMN_TAKEN_FRAC
}
