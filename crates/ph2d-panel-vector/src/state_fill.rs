//! **Com que TINTA a forma aparece** — a metade do estado do painel que fala de preenchimento,
//! irmã de [`super::state`] pelo teto de 600 LOC (HR-18).
//!
//! O corte é por ASSUNTO, e é o mesmo que a `ph2d-vec-scene` já fez no `lib.rs` desta linha:
//! *com que tinta a forma aparece* × *o que a forma É*. O pai fica com **onde** a forma está e
//! **o que** ela é (a bbox, os vértices, a unidade, o pivô); aqui mora o tipo de preenchimento,
//! o ângulo do gradiente linear, os dois números do ponto de gradiente selecionado e a regra de
//! preenchimento do caminho composto.
//!
//! ⚠️ **Irmão e não filho**, ao contrário do `state_text`: estas seis portas são `pub` para a
//! shell as chamar, então elas re-exportam pela raiz do crate como sempre fizeram — quem as
//! chamava não muda uma linha.

use super::{FillKind, PathFillRule};
use std::cell::{Cell, RefCell};

thread_local! {
    /// Selected path's fill kind (`None` = no path selected / no fill). Drives the
    /// Fill-type selector highlight + whether the gradient controls show.
    static CURRENT_FILL_KIND: Cell<Option<FillKind>> = const { Cell::new(None) };
    /// Selected path's linear-gradient angle in degrees (`None` unless Linear).
    static CURRENT_GRAD_ANGLE: Cell<Option<f64>> = const { Cell::new(None) };
    /// Selected multi-point gradient point's influence (`None` unless a point is
    /// selected) — drives the Influence slider's visibility + value.
    static CURRENT_GRAD_INFLUENCE: Cell<Option<f64>> = const { Cell::new(None) };
    /// Selected multi-point gradient point's jitter (`None` unless a point is
    /// selected) — drives the Jitter slider's visibility + value.
    static CURRENT_GRAD_JITTER: Cell<Option<f64>> = const { Cell::new(None) };
    /// Fill rule of the selected path, `Some` only when it is a COMPOUND path —
    /// the two rules agree on a single contour, so the row would be a no-op there.
    static CURRENT_FILL_RULE: Cell<Option<PathFillRule>> = const { Cell::new(None) };
    /// A LEI do padrão da forma selecionada (`None` = ela não tem padrão) — o que a secção
    /// **Pattern** desenha. Espelho panel-local do `PatternFill` da cena, pela MESMA razão que o
    /// [`FillKind`] o é: o painel não depende da crate do documento.
    /// ⚠️ **Uma por TINTA** (plano 35, wave F): índice `0` = preenchimento, `1` = traço. Uma só
    /// entrada obrigava a secção a ter um ALVO escondido num chip, e o artista mexia num knob e via
    /// o outro sujeito mudar — o report do Enio de 2026-08-28.
    static CURRENT_TEXPAT: RefCell<[Option<TexturePatternRow>; 2]> = const { RefCell::new([None, None]) };
}

/// A lei de um padrão de textura, como o painel a vê (plano 33, W5).
///
/// ⚠️ **`kind` e `mode` são índices**, não os enums da cena: manter o painel independente da
/// `ph2d-vec-scene` é a mesma escolha que o [`FillKind`] já fez. A shell é quem traduz, num sítio só.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct TexturePatternRow {
    /// `0` Grid · `1` Brick (linhas) · `2` Column (colunas) · `3` Hex.
    pub kind: u8,
    /// O desfasamento é `1/n` de uma célula. `1` = nenhum.
    pub offset_denom: f64,
    /// O tamanho de uma cópia — **os DOIS eixos**, em unidades de mundo.
    ///
    /// ⛔ Era **um** número (o lado maior, aspecto sempre preservado) até 2026-08-27: o Enio pediu
    /// para poder achatar a arte de propósito, e a protecção mudou de lei imposta para gesto
    /// escolhido ([`Self::lock_aspect`]).
    pub size: [f64; 2],
    /// ⭐ O **cadeado de proporção** está ligado? Mexer num eixo leva o outro pelo mesmo factor.
    ///
    /// ⚠️ Ele descreve o **gesto**, não o padrão: vive na sessão da shell e **não viaja no
    /// ficheiro**. Um cadeado gravado seria estado que descreve como alguém estava a editar.
    pub lock_aspect: bool,
    /// O vão acrescentado, em unidades de mundo. Negativo = sobreposição.
    pub gap: f64,
    /// A rotação do padrão, em graus.
    pub angle_deg: f64,
    /// **A fase dentro de UMA repetição**, em percentagem, ao longo dos eixos do PADRÃO.
    ///
    /// ⚠️ Substitui as três alças de canvas do plano 33 W6, retiradas por decisão do Enio
    /// (2026-08-27). `100` é o mesmo que `0`: um período inteiro de deslocamento é a identidade.
    pub shift_pct: [f64; 2],
    /// `0` Tile · `1` Mirror · `2` Clamp.
    pub mode: u8,
    /// ⭐⭐⭐ **A junta deste ladrilho consigo próprio VÊ-SE?** (plano 33, W10.)
    ///
    /// Medido no assado por [`ph2d_vec_pattern::wrap_seam`] contra o joelho `SEAM_VISIBLE`, que
    /// saiu de uma varredura na GPU. Acima dele o artista tem **uma aresta dura em cada fronteira**
    /// — não é o filtro, é a arte, e ela repete-se de propósito.
    ///
    /// ⚠️⚠️ **`false` quer dizer «não há nada a dizer», e NUNCA «está provado que fecha».** Um
    /// ladrilho que ainda não assou (a arte a carregar, a forma-fonte apagada) também dá `false` —
    /// *um zero de «não medido» e um de «perfeito» são o mesmo byte*, e aqui os dois têm de levar
    /// à mesma acção: **calar**. Um aviso sobre um ladrilho que não existe é ruído.
    pub wrap_seam_visible: bool,
    /// ⭐⭐⭐ **A FORMA que este padrão usa como arte foi APAGADA.**
    ///
    /// Sem isto, apagar a forma-fonte faz a estampa voltar a **cor chapada** — e a cor chapada é
    /// exactamente o que um preenchimento sólido correcto parece. *O artista perde a autoria e o
    /// app não tem uma palavra a dizer sobre isso.* ⚠️ E o `PatternSource` **não tem variante
    /// vazia**: ao contrário do pincel (cujo `art` é um `Option` e cujo botão já diz *"Pick
    /// Shape…"*), uma estampa não consegue exprimir *"sem arte"* — a secção sobe inteira e normal
    /// por cima de um vínculo morto.
    ///
    /// ⚠️ **Só a fonte-FORMA responde aqui, e a restrição é a decisão.** Uma fonte-IMAGEM que não
    /// resolve pode estar apenas **a carregar** — os pixels dela viajam no ficheiro desde a W8, e a
    /// ausência é transitória por construção. *Um aviso permanente sobre um estado transitório
    /// ensina o artista a ignorar avisos.*
    ///
    /// ⚠️ O estrago é **recuperável enquanto a sessão dura**: o `VecPathId` é um `u64` guardado
    /// dentro da `VecScene`, que o undo repõe **verbatim** — desfazer o apagar devolve a forma com
    /// o MESMO id e o vínculo cura-se sozinho (há gate). O que não se recupera é apagar, **gravar**
    /// e só reparar depois.
    pub art: PatternArt,
}

/// **Em que pé está a ARTE de um padrão** — e são TRÊS estados, não dois.
///
/// ⛔⛔ Isto era um `bool` (`art_missing`), e o report do Enio de 2026-08-30 mostrou que ele juntava
/// duas coisas que pedem frases OPOSTAS: *"nunca foi escolhida"* e *"foi apagada"*. É a mesma
/// família de *"«pausado» e «terminado» leem-se igual no `playing == false`"*: um bit que responde a
/// duas perguntas responde mal a uma delas.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum PatternArt {
    /// Há arte e ela resolve — o caminho comum, e o único em que a secção não avisa nada.
    Ready,
    /// ⭐ **O padrão acabou de nascer e o artista ainda não disse qual é a arte** — o estado que faz
    /// o painel ser o ESCOLHEDOR, com *Source…* e *Use Shape…* lado a lado.
    NotChosen,
    /// A forma que servia de arte foi apagada (W11). Recuperável por undo enquanto a sessão durar.
    Deleted,
}

/// Publica a lei do padrão da tinta `slot` (`None` esconde a secção dela).
pub fn set_current_texture_pattern(slot: usize, row: Option<TexturePatternRow>) {
    CURRENT_TEXPAT.with(|c| {
        if let Some(v) = c.borrow_mut().get_mut(slot) {
            *v = row;
        }
    });
}

/// A lei do padrão da tinta `slot` neste quadro (`None` ⇒ a secção dela nem sobe).
pub(crate) fn current_texture_pattern(slot: usize) -> Option<TexturePatternRow> {
    CURRENT_TEXPAT.with(|c| c.borrow().get(slot).copied().flatten())
}

/// Publish the selected path's fill kind + linear angle (both `None` when no path
/// is selected or it has no fill / isn't linear).
pub fn set_current_fill(kind: Option<FillKind>, angle_deg: Option<f64>) {
    CURRENT_FILL_KIND.with(|c| c.set(kind));
    CURRENT_GRAD_ANGLE.with(|c| c.set(angle_deg));
}

/// The selected path's fill kind this frame (`None` ⇒ hide the Fill-type selector).
pub(crate) fn current_fill_kind() -> Option<FillKind> {
    CURRENT_FILL_KIND.with(Cell::get)
}

/// The selected path's linear-gradient angle this frame (`None` unless Linear).
pub(crate) fn current_grad_angle() -> Option<f64> {
    CURRENT_GRAD_ANGLE.with(Cell::get)
}

/// Publish the selected multi-point gradient point's influence (`None` = no point).
pub fn set_current_grad_influence(v: Option<f64>) {
    CURRENT_GRAD_INFLUENCE.with(|c| c.set(v));
}

/// The selected multi-point point's influence this frame (drives the slider).
pub(crate) fn current_grad_influence() -> Option<f64> {
    CURRENT_GRAD_INFLUENCE.with(Cell::get)
}

/// Publish the selected multi-point gradient point's jitter (`None` = no point).
pub fn set_current_grad_jitter(v: Option<f64>) {
    CURRENT_GRAD_JITTER.with(|c| c.set(v));
}

/// The selected multi-point point's jitter this frame (drives the slider).
pub(crate) fn current_grad_jitter() -> Option<f64> {
    CURRENT_GRAD_JITTER.with(Cell::get)
}

/// Publish the selected path's fill rule — `None` unless it is a compound path
/// (the Fill Rule row hides otherwise, since both rules would paint the same).
pub fn set_current_fill_rule(rule: Option<PathFillRule>) {
    CURRENT_FILL_RULE.with(|c| c.set(rule));
}

/// The selected compound path's fill rule this frame (`None` = not compound).
pub(crate) fn current_fill_rule() -> Option<PathFillRule> {
    CURRENT_FILL_RULE.with(Cell::get)
}
