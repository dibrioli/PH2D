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

/// **E quanto do PAINEL a coluna tem de ser.** A segunda metade da pergunta, e sem ela a primeira
/// fecha um CICLO.
///
/// ⛔⛔⛔ **Report do dono (2026-09-20): *«pisca do lado direito quando escondemos o inspector e
/// aumentamos muito a área do grafo de nós»*.** Com a coluna da direita vazia a área de desenho
/// cresce para dentro dela (que é o que este módulo existe para permitir) — e o painel que vive na
/// área passa a **publicar um rect que cobre a coluna**. No quadro seguinte a régua de cima lia
/// isso como *«a coluna está ocupada»*, a área encolhia, o painel encolhia com ela, e no quadro a
/// seguir a coluna voltava a ler-se livre. **Período dois, a 60 Hz** — uma faixa da largura do
/// Inspector a aparecer e a desaparecer.
///
/// ⚠️ **A régua fechava um ciclo consigo mesma:** *a área cresce porque a coluna está livre; a
/// coluna lê-se ocupada porque a área cresceu.*
///
/// ⭐⭐ **O discriminador é de FORMA e continua sem lista nenhuma:** um inquilino de coluna publica
/// **o rect da coluna** (são `18` painéis a publicar o `ctx.slot`, que é o slot do encaixe), e um
/// painel da ÁREA que transbordou para cima dela é muito mais largo. Medido na reprodução do
/// report (janela `1930×1012`, grafo no máximo): a coluna é **`0,185`** da área do grafo e
/// **`1,000`** da área de um inquilino a sério — o vale tem `5,4×`, e a barra fica no meio, no
/// mesmo número da metade de cima: *para tomar uma coluna, o painel e a coluna têm de ser,
/// maioritariamente, a mesma coisa*.
///
/// ⚠️ **Ela só APERTA:** todo rect que deixa de tomar uma coluna já não a tomava por outra via
/// nenhuma — logo os casos que a metade de cima recusa (o popover que roça) continuam recusados,
/// e o inquilino que nenhuma lista nomeia continua a entrar.
const PANEL_IS_THE_COLUMN_FRAC: f32 = 0.5; // LITERAL-PX-OK: fracção de área, não um token de desenho

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

/// O rect `r` **toma** a coluna `col`?
///
/// As duas metades, e cada uma recusa uma coisa diferente: a sobreposição cobre ao menos
/// [`COLUMN_TAKEN_FRAC`] da **coluna** (senão é um popover a roçar) **e** ao menos
/// [`PANEL_IS_THE_COLUMN_FRAC`] do **painel** (senão é um painel da área que transbordou para cima
/// dela — ver a doc daquela constante, que é o report do piscar).
fn takes(r: Rect, col: Rect) -> bool {
    let area = col.w * col.h;
    let area_r = r.w * r.h;
    if area <= 0.0 || area_r <= 0.0 {
        return false;
    }
    let w = (r.x + r.w).min(col.x + col.w) - r.x.max(col.x);
    let h = (r.y + r.h).min(col.y + col.h) - r.y.max(col.y);
    if w <= 0.0 || h <= 0.0 {
        return false;
    }
    let sobreposta = w * h;
    sobreposta / area >= COLUMN_TAKEN_FRAC && sobreposta / area_r >= PANEL_IS_THE_COLUMN_FRAC
}
