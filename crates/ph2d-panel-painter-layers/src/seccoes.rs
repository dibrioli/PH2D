//! ⭐⭐⭐ **AS SECÇÕES DESTE PAINEL QUE PINTAM LINHAS DE MARCAR — e os rótulos de cada uma.**
//!
//! ⛔⛔ Até 2026-09-16 nenhuma linha de marcar deste painel declarava a secção a que pertence, logo
//! todas caíam no default (*«sou uma linha de formulário e não sei que nomes vou pintar»*), cuja
//! coluna é a **metade cega** da linha. Medido nesse dia com o sistema de texto REAL (`Sm`), sobre
//! os **36** rótulos de marcar deste painel:
//!
//! | painel | cortados ANTES | cortados DEPOIS |
//! |---|---|---|
//! | `220` (mínimo do dock) | `11` | `7` |
//! | `245` | `6` | **`1`** |
//! | `273,3` (a largura do dono) | `1` | **`0`** |
//! | `304` (omissão) | `0` | `0` |
//!
//! ⚠️ **Os `7` que sobram a `220` são o TECTO, não uma falha:** ali a coluna bate em
//! `usable − vão − `[`NUMBER_INPUT_MIN_W_PX`](ph2d_editor_core::widget::NUMBER_INPUT_MIN_W_PX)
//! = `90,0 px`, e o piso da caixa é uma **ordem do dono de 2026-05-24**. *Nessa ponta o nome corta,
//! e é a troca que ele escolheu.*
//!
//! # ⚠️ A unidade é a SECÇÃO, e ela é DERIVADA da chave
//!
//! A tabela de i18n deste painel declara por escrito que as chaves são
//! `<secção que o artista vê>.<nome>` (ver [`ph2d_i18n`], `painter_layers.rs`) — logo a secção de
//! uma linha **lê-se da chave dela**, não de uma lista de opinião. ⛔ Uma coluna por LINHA seria
//! uma coluna por linha: dois nomes de comprimentos diferentes no mesmo cartão sairiam com as
//! caixas em `x` diferentes, que é exactamente o que a ordem do dono de 2026-09-14 proíbe
//! (*«as labels alinhadas todas à direita»*).
//!
//! ⛔⛔ **E a medição é sobre os rótulos de LINHA, nunca sobre todas as chaves da secção:** a
//! mesma secção carrega títulos gritados, notas de rodapé e rótulos de botão, e o mais largo deles
//! empurraria a coluna ao tecto sem que nome nenhum precisasse disso.
//!
//! # ⚠️ Esta lista é conferida dos DOIS lados
//!
//! O gate `cada_nome_de_marcar_cabe_na_coluna_da_seccao` lê o fonte do painel, extrai o rótulo de
//! cada chamada de [`crate::paint_brush_top::paint_checkbox_row`] e recusa **uma chave aqui que o
//! painel não pinta** e **um rótulo pintado que não esteja aqui** — com piso de população, para um
//! parse partido não passar por «não há nada a acusar» (`CLAUDE.md` §5.0).

use ph2d_editor_core::widget::Seccao;
use ph2d_text::TextSystem;

/// *Brush* — o cartão de topo.
pub(crate) const BRUSH: &[&str] = &[
    "panel.painter_layers.brush.sync_tools",
    "panel.painter_layers.brush.accumulate",
];
/// *Clone*.
pub(crate) const CLONE: &[&str] = &["panel.painter_layers.clone.aligned"];
/// *Composite Brush* — vive num cartão com recuo próprio.
pub(crate) const COMPOSITE: &[&str] = &["panel.painter_layers.composite.composite_brush"];
/// *Deform*.
pub(crate) const DEFORM: &[&str] = &["panel.painter_layers.deform.affect_relief"];
/// *Grain* (a textura do papel sob o traço).
pub(crate) const GRAIN: &[&str] = &["panel.painter_layers.grain.rake"];
/// *Impasto* — o relevo, mais o cartão do rig de luz.
pub(crate) const IMPASTO: &[&str] = &[
    "panel.painter_layers.impasto.adjust_last_stroke",
    "panel.painter_layers.impasto.smooth_edges",
    "panel.painter_layers.impasto.show_impasto",
    "panel.painter_layers.impasto.enable",
];
/// *Line* — o cartão do traço procedural, com recuo próprio como o *Composite*.
pub(crate) const LINE: &[&str] = &[
    "panel.painter_layers.line.solid",
    "panel.painter_layers.line.magnetify",
    "panel.painter_layers.line.connection_line",
];
/// *Paper* (o substrato da aquarela).
pub(crate) const PAPER: &[&str] = &["panel.painter_layers.paper.same_as_paper"];
/// *Color Ramp*.
pub(crate) const RAMP: &[&str] = &["panel.painter_layers.ramp.use_color_ramp"];
/// *Sculpt*.
pub(crate) const SCULPT: &[&str] = &["panel.painter_layers.sculpt.rake"];
/// *Selection*.
pub(crate) const SELECTION: &[&str] = &["panel.painter_layers.selection.edit_gizmos"];
/// *Shape* — as camadas de forma, incluindo a caixa por-camada do [`crate::paint_shape_layers`].
///
/// ⚠️ **O `layer_color` é um MOLDE** (`"Layer {n} Color"`, preenchido por `tr_with`), logo o que se
/// mede aqui é o molde e não o texto final. As chavetas medem ~um dígito a mais do que o `1`..`8`
/// que o artista vê ⇒ a coluna erra **para cima**, e uma coluna larga de mais nunca corta um nome.
/// ⛔ E não muda nada nesta secção: o mais largo dela é o *Use Texture Colors*.
pub(crate) const SHAPE: &[&str] = &[
    "panel.painter_layers.shape.automatic",
    "panel.painter_layers.shape.alpha_from_image",
    "panel.painter_layers.shape.per_layer_color",
    "panel.painter_layers.shape.use_texture_colors",
    "panel.painter_layers.shape.layer_color",
];
/// *Stroke* — o cartão de aplicar um traço gravado.
pub(crate) const STROKE: &[&str] = &[
    "panel.painter_layers.stroke.dimensions",
    "panel.painter_layers.stroke.edge_to_edge",
    "panel.painter_layers.stroke.adjust_strength",
    "panel.painter_layers.stroke.tiling_x",
    "panel.painter_layers.stroke.tiling_y",
    "panel.painter_layers.stroke.repeat_image",
    "panel.painter_layers.stroke.trim",
];
/// *Stroke ▸ Grid* — sub-cartão próprio, e por isso coluna própria.
pub(crate) const STROKE_GRID: &[&str] = &["panel.painter_layers.stroke.grid.show_grid"];
/// *Symmetry*.
pub(crate) const SYMMETRY: &[&str] = &[
    "panel.painter_layers.symmetry.use_symmetry",
    "panel.painter_layers.symmetry.circular",
];
/// *Watercolor*.
pub(crate) const WATERCOLOR: &[&str] = &["panel.painter_layers.watercolor.smooth_edges"];
/// *Wet Paint* — inclui a caixa do cartão de inclinação.
pub(crate) const WETPAINT: &[&str] = &[
    "panel.painter_layers.wetpaint.show_wet",
    "panel.painter_layers.wetpaint.paper",
    "panel.painter_layers.wetpaint.tuning",
    "panel.painter_layers.wetpaint.tilt",
];

/// ⭐ **Todas as secções, para o gate** — a mesma lista que o painel pinta, lida do outro lado.
pub const TODAS: &[(&str, &[&str])] = &[
    ("brush", BRUSH),
    ("clone", CLONE),
    ("composite", COMPOSITE),
    ("deform", DEFORM),
    ("grain", GRAIN),
    ("impasto", IMPASTO),
    ("line", LINE),
    ("paper", PAPER),
    ("ramp", RAMP),
    ("sculpt", SCULPT),
    ("selection", SELECTION),
    ("shape", SHAPE),
    ("stroke", STROKE),
    ("stroke.grid", STROKE_GRID),
    ("symmetry", SYMMETRY),
    ("watercolor", WATERCOLOR),
    ("wetpaint", WETPAINT),
];

/// ⭐⭐ **A [`Seccao`] de uma secção deste painel** — medida sobre os rótulos dela, no sistema de
/// texto real, como o Inspector faz.
///
/// ⚠️ **`campos = 1`**: uma linha de marcar tem um controlo só, logo não há cedência de par a medir.
pub(crate) fn seccao(text_system: &mut TextSystem, chaves: &[&str]) -> Seccao {
    let rotulos: Vec<&'static str> = chaves.iter().map(|k| ph2d_i18n::tr(k)).collect();
    Seccao::medida(text_system, 1, &rotulos)
}

/// O nome da secção de uma chave — *tudo entre o prefixo do painel e o último componente*.
///
/// ⚠️ **É a regra que a própria tabela de i18n declara** (`<secção>.<nome>`), e por isso
/// `panel.painter_layers.stroke.grid.show_grid` responde `stroke.grid`: um sub-cartão é uma secção.
#[must_use]
pub fn nome_da_seccao(chave: &str) -> Option<&str> {
    chave
        .strip_prefix("panel.painter_layers.")
        .and_then(|resto| resto.rsplit_once('.'))
        .map(|(sec, _)| sec)
}

/// ⭐⭐⭐ **A [`Seccao`] a que uma CHAVE pertence** — a porta que o pintor de uma linha de marcar
/// usa, e a razão de ele receber a chave em vez do texto.
///
/// ⛔⛔ *Uma linha que recebe o texto já traduzido não sabe a que secção pertence*, e a alternativa
/// — cada sítio de pintura escolher a sua — é a lista à mão que este repo já viu acusar, quatro
/// vezes, exactamente quem fez a coisa certa (`§34.6` do handoff). Aqui a secção **lê-se da chave**
/// e não há sítio onde a escolher errado.
///
/// ⚠️ **Uma chave de secção não declarada cai no rótulo dela sozinho** — nunca pior do que o
/// default de antes — e o `debug_assert` acusa-a na suíte, que é onde ela tem de aparecer.
pub(crate) fn seccao_da_chave(text_system: &mut TextSystem, chave: &str) -> Seccao {
    let chaves = nome_da_seccao(chave)
        .and_then(|n| TODAS.iter().find(|(s, _)| *s == n))
        .map(|(_, c)| *c);
    debug_assert!(
        chaves.is_some(),
        "a chave `{chave}` pinta uma linha de marcar e a secção dela não está em `TODAS` — \
         a coluna dela ficaria sozinha, desalinhada das irmãs do mesmo cartão"
    );
    seccao(text_system, chaves.unwrap_or(std::slice::from_ref(&chave)))
}
