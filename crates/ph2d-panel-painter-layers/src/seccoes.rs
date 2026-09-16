//! ⭐⭐⭐ **AS SECÇÕES DESTE PAINEL — e os rótulos que cada uma pinta.**
//!
//! ⛔⛔ Até 2026-09-16 este painel tinha **três** respostas para *«onde começa o controlo?»*, e as
//! três conviviam dentro do mesmo cartão:
//!
//! | família de linha | o que ela usava | onde o nome saía |
//! |---|---|---|
//! | caixa de marcar | o default (*«não sei que nomes vou pintar»*) | na **metade cega** da faixa |
//! | rótulo + chip · amostra de cor | o literal `LABEL_W = 60,0` | encostado à **esquerda** |
//! | número | um `fn seccao*()` por ficheiro, seis deles | na coluna da secção ✅ |
//!
//! ⇒ *duas colunas de nome alternando linha sim linha não* — o defeito que o §6-quinquies da spec
//! existe para matar, um nível acima: entre FAMÍLIAS de linha em vez de entre linhas.
//!
//! Medido nesse dia com o sistema de texto REAL (`Sm`), sobre os rótulos de linha de propriedade
//! que este painel pinta:
//!
//! | painel | coluna cega | secção declarada |
//! |---|---|---|
//! | `220` (mínimo do dock) | `11` cortados | `7` |
//! | `245` | `6` | **`1`** |
//! | `273,3` (a largura do dono) | `1` | **`0`** |
//! | `304` (omissão) | `0` | `0` |
//!
//! ⚠️ **E a coluna de `60 px` dos chips tinha o defeito dela própria:** o rótulo **`Paint Mode`**
//! mede `65,3 px` e saía cortado **em TODA largura de painel**, porque um literal não cresce com o
//! dock. Com a porta, a coluna mais estreita que ele encontra é `84,0`.
//!
//! ⚠️ **Os `7` que sobram a `220` são o TECTO, não uma falha:** ali a coluna bate em
//! `usable − vão − `[`NUMBER_INPUT_MIN_W_PX`](ph2d_editor_core::widget::NUMBER_INPUT_MIN_W_PX)
//! = `90,0 px`, e o piso da caixa é uma **ordem do dono de 2026-05-24**. *Nessa ponta o nome corta,
//! e é a troca que ele escolheu.*
//!
//! # ⚠️ A unidade é a SECÇÃO, e ela é DERIVADA da chave
//!
//! A tabela de i18n deste painel declara por escrito que as chaves são
//! `<secção que o artista vê>.<nome>` (ver `ph2d_i18n::painter_layers`) — logo a secção de uma
//! linha **lê-se da chave dela**, não de uma lista de opinião. ⛔ Uma coluna por LINHA seria uma
//! coluna por linha: dois nomes de comprimentos diferentes no mesmo cartão sairiam com os controlos
//! em `x` diferentes, que é o que a ordem do dono de 2026-09-14 proíbe (*«as labels alinhadas todas
//! à direita»*).
//!
//! ⛔⛔ **E a medição é sobre os rótulos de LINHA, nunca sobre todas as chaves da secção:** a
//! mesma secção carrega títulos gritados, notas de rodapé e rótulos de botão, e o mais largo deles
//! empurraria a coluna ao tecto sem que nome nenhum precisasse disso.
//!
//! # ⚠️ Esta lista é conferida dos DOIS lados
//!
//! O gate `a_declaracao_das_seccoes_e_o_painel_dizem_o_mesmo` lê o fonte do painel, extrai o rótulo
//! de cada chamada das portas de linha e recusa **uma chave aqui que o painel não pinta** e **um
//! rótulo pintado que não esteja aqui** — com piso de população, para um parse partido não passar
//! por «não há nada a acusar» (`CLAUDE.md` §5.0).

use ph2d_editor_core::widget::Seccao;
use ph2d_text::TextSystem;

/// ⭐⭐⭐ **A DECLARAÇÃO DE UMA SECÇÃO** — quantas componentes a linha mais larga dela traz, e os
/// rótulos que ela pinta.
///
/// ⚠️ **`campos` é da SECÇÃO e não da linha** (spec §6-ter): se cada linha cedesse pelo que ELA
/// precisa, a de um campo não cederia nada e a de dois cederia, e a coluna sairia esfarrapada — que
/// é o que a ordem *«as labels alinhadas todas à direita»* proíbe.
pub struct Declaracao {
    /// O nome da secção — *tudo entre o prefixo do painel e o último componente de uma chave*.
    pub nome: &'static str,
    /// Quantas componentes tem a linha que a secção não quer ver quebrar.
    pub campos: usize,
    /// As chaves dos rótulos que esta secção pinta como LINHA DE PROPRIEDADE.
    pub chaves: &'static [&'static str],
}

/// O par `X`/`Y` de um *Size* / *Offset* — a linha que uma secção de geometria não quer ver quebrar.
const PAR: usize = crate::number_field::SECTION_FIELDS;

/// ⭐ **TODAS as secções deste painel, e é a ÚNICA declaração delas.**
///
/// ⛔⛔ Antes de 2026-09-16 havia **duas** respostas para a mesma secção: seis ficheiros traziam um
/// `fn seccao*()` com a lista dos números deles, e as caixas de marcar e os chips não tinham
/// nenhuma. *Duas derivações da mesma coluna são duas colunas* — e elas divergiam exactamente onde
/// mais se nota, dentro do mesmo cartão. Hoje os seis delegam aqui.
pub const TODAS: &[Declaracao] = &[
    Declaracao {
        nome: "brush",
        campos: 1,
        chaves: BRUSH,
    },
    Declaracao {
        nome: "clone",
        campos: 1,
        chaves: CLONE,
    },
    Declaracao {
        nome: "composite",
        campos: 1,
        chaves: COMPOSITE,
    },
    Declaracao {
        nome: "deform",
        campos: 1,
        chaves: DEFORM,
    },
    Declaracao {
        nome: "grain",
        campos: PAR,
        chaves: GRAIN,
    },
    Declaracao {
        nome: "impasto",
        campos: 1,
        chaves: IMPASTO,
    },
    Declaracao {
        nome: "line",
        campos: 1,
        chaves: LINE,
    },
    Declaracao {
        nome: "paper",
        campos: PAR,
        chaves: PAPER,
    },
    Declaracao {
        nome: "ramp",
        campos: 1,
        chaves: RAMP,
    },
    Declaracao {
        nome: "sculpt",
        campos: 1,
        chaves: SCULPT,
    },
    Declaracao {
        nome: "selection",
        campos: 1,
        chaves: SELECTION,
    },
    Declaracao {
        nome: "shape",
        campos: PAR,
        chaves: SHAPE,
    },
    Declaracao {
        nome: "stencil",
        campos: PAR,
        chaves: STENCIL,
    },
    Declaracao {
        nome: "stroke",
        campos: 1,
        chaves: STROKE,
    },
    Declaracao {
        nome: "stroke.grid",
        campos: 1,
        chaves: STROKE_GRID,
    },
    Declaracao {
        nome: "stroke.jitter",
        campos: 1,
        chaves: STROKE_JITTER,
    },
    Declaracao {
        nome: "symmetry",
        campos: 1,
        chaves: SYMMETRY,
    },
    Declaracao {
        nome: "taper",
        campos: 1,
        chaves: TAPER,
    },
    Declaracao {
        nome: "watercolor",
        campos: 1,
        chaves: WATERCOLOR,
    },
    Declaracao {
        nome: "wetpaint",
        campos: 1,
        chaves: WETPAINT,
    },
];

/// *Brush* — o cartão de topo (a caixa de sincronizar, a de acumular, e os chips de Blend / Paint
/// Mode / Preset, mais a amostra de cor).
pub(crate) const BRUSH: &[&str] = &[
    "panel.painter_layers.brush.sync_tools",
    "panel.painter_layers.brush.accumulate",
    "panel.painter_layers.brush.blend",
    "panel.painter_layers.brush.color",
    "panel.painter_layers.brush.paint_mode",
    "panel.painter_layers.brush.preset",
];
/// *Clone*.
pub(crate) const CLONE: &[&str] = &["panel.painter_layers.clone.aligned"];
/// *Composite Brush* — vive num cartão com recuo próprio.
pub(crate) const COMPOSITE: &[&str] = &["panel.painter_layers.composite.composite_brush"];
/// *Deform*.
pub(crate) const DEFORM: &[&str] = &["panel.painter_layers.deform.affect_relief"];
/// *Grain* (a textura do papel sob o traço) — os números do ladrilho e o chip do tipo.
pub(crate) const GRAIN: &[&str] = &[
    "panel.painter_layers.grain.rake",
    "panel.painter_layers.grain.grain",
    "panel.painter_layers.grain.mapping",
    "panel.painter_layers.grain.angle",
    "panel.painter_layers.grain.offset",
    "panel.painter_layers.grain.size",
    "panel.painter_layers.grain.depth",
];
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
    "panel.painter_layers.line.type",
    "panel.painter_layers.line.line_width",
    "panel.painter_layers.line.opacity",
    "panel.painter_layers.line.density",
    "panel.painter_layers.line.reach",
    "panel.painter_layers.line.history",
    "panel.painter_layers.line.rungs",
    "panel.painter_layers.line.weight",
    "panel.painter_layers.line.gravity",
    "panel.painter_layers.line.friction",
    "panel.painter_layers.line.roughness",
    "panel.painter_layers.line.bowing",
    "panel.painter_layers.line.passes",
];
/// *Paper* (o substrato da aquarela) — o dente, o relevo, a cor e os números do ladrilho.
pub(crate) const PAPER: &[&str] = &[
    "panel.painter_layers.paper.same_as_paper",
    "panel.painter_layers.paper.paper",
    "panel.painter_layers.paper.mapping",
    "panel.painter_layers.paper.color",
    "panel.painter_layers.paper.amount",
    "panel.painter_layers.paper.angle",
    "panel.painter_layers.paper.offset",
    "panel.painter_layers.paper.size",
    "panel.painter_layers.paper.tooth",
    "panel.painter_layers.paper.relief",
    "panel.painter_layers.paper.roughness",
];
/// *Color Ramp*.
pub(crate) const RAMP: &[&str] = &["panel.painter_layers.ramp.use_color_ramp"];
/// *Sculpt*.
pub(crate) const SCULPT: &[&str] = &["panel.painter_layers.sculpt.rake"];
/// *Selection*.
pub(crate) const SELECTION: &[&str] = &["panel.painter_layers.selection.edit_gizmos"];
/// *Shape* — as camadas de forma, o depósito e os números da estampa.
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
    "panel.painter_layers.shape.falloff",
    "panel.painter_layers.shape.follow",
    "panel.painter_layers.shape.texture",
    "panel.painter_layers.shape.relief",
    "panel.painter_layers.shape.shine",
    "panel.painter_layers.shape.angle",
    "panel.painter_layers.shape.offset",
    "panel.painter_layers.shape.size",
];
/// *Stencil*.
pub(crate) const STENCIL: &[&str] = &[
    "panel.painter_layers.stencil.size",
    "panel.painter_layers.stencil.offset",
    "panel.painter_layers.stencil.rotation",
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
    "panel.painter_layers.stroke.method",
];
/// *Stroke ▸ Grid* — sub-cartão próprio, e por isso coluna própria.
pub(crate) const STROKE_GRID: &[&str] = &["panel.painter_layers.stroke.grid.show_grid"];
/// *Stroke ▸ Jitter* — sub-cartão próprio.
pub(crate) const STROKE_JITTER: &[&str] = &["panel.painter_layers.stroke.jitter.unit"];
/// *Symmetry*.
pub(crate) const SYMMETRY: &[&str] = &[
    "panel.painter_layers.symmetry.use_symmetry",
    "panel.painter_layers.symmetry.circular",
];
/// *Taper*.
pub(crate) const TAPER: &[&str] = &[
    "panel.painter_layers.taper.tip",
    "panel.painter_layers.taper.opacity",
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

/// ⭐⭐ **A [`Seccao`] de uma declaração** — medida sobre os rótulos dela, no sistema de texto real.
pub(crate) fn seccao(text_system: &mut TextSystem, d: &Declaracao) -> Seccao {
    let rotulos: Vec<&'static str> = d.chaves.iter().map(|k| ph2d_i18n::tr(k)).collect();
    Seccao::medida(text_system, d.campos, &rotulos)
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

/// A declaração a que uma chave pertence.
#[must_use]
pub fn declaracao_da_chave(chave: &str) -> Option<&'static Declaracao> {
    let nome = nome_da_seccao(chave)?;
    TODAS.iter().find(|d| d.nome == nome)
}

/// ⭐⭐⭐ **A [`Seccao`] a que uma CHAVE pertence** — a porta que todo pintor de linha deste painel
/// usa, e a razão de eles receberem a chave em vez do texto.
///
/// ⛔⛔ *Uma linha que recebe o texto já traduzido não sabe a que secção pertence*, e a alternativa
/// — cada sítio de pintura escolher a sua — é a lista à mão que este repo já viu acusar, quatro
/// vezes, exactamente quem fez a coisa certa (`§34.6` do handoff). Aqui a secção **lê-se da chave**
/// e não há sítio onde a escolher errado.
///
/// ⚠️ **Uma chave de secção não declarada cai no rótulo dela sozinho** — nunca pior do que o
/// default de antes — e o `debug_assert` acusa-a na suíte, que é onde ela tem de aparecer.
pub(crate) fn seccao_da_chave(text_system: &mut TextSystem, chave: &str) -> Seccao {
    let d = declaracao_da_chave(chave);
    debug_assert!(
        d.is_some(),
        "a chave `{chave}` pinta uma linha de propriedade e a secção dela não está em `TODAS` — \
         a coluna dela ficaria sozinha, desalinhada das irmãs do mesmo cartão"
    );
    match d {
        Some(d) => seccao(text_system, d),
        None => Seccao::medida(text_system, 1, &[ph2d_i18n::tr(chave)]),
    }
}
