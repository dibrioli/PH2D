//! Spacing scale tokens. Source: `docs/design/tokens.json` → `spacing.*`.
//!
//! 8 px base scale with sub-base steps for tight UI density. The
//! canonical section gap is 14 px (non-power-of-2 — design choice;
//! see tokens.json).
//!
//! Wave 4 stage A: values now come from `crate::generated::SPACING_*`,
//! `DENSITY_*` and `CHROME_*` const tables (codegen'd by build.rs from
//! `docs/design/tokens.json`). Designer edits the JSON; Rust replicates.

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Spacing {
    /// `xxs` — 2 px (sub-pixel divider gaps).
    Xxs,
    /// `xs` — 4 px (icon ↔ label tight inline).
    Xs,
    /// `sm` — 6 px (compact rows).
    Sm,
    /// `md` — 8 px (default inline padding).
    Md,
    /// `lg` — 12 px (default vertical rhythm, panel padding).
    Lg,
    /// `xl` — 16 px (comfortable padding).
    Xl,
    /// `xl2` (`2xl` in JSON) — 24 px (section separation).
    Xl2,
    /// `xl3` (`3xl` in JSON) — 32 px (major panel margin).
    Xl3,
    /// `xl4` (`4xl` in JSON) — 48 px (hero spacing).
    Xl4,
}

impl Spacing {
    /// O valor de **FÁBRICA** — a tabela gerada do `tokens.json`, sem passar pela camada de
    /// override. `const fn`, e é isso que a mantém legal em contexto `const`.
    ///
    /// ⚠️ Quem quer o número que o app **DESENHA** chama [`Spacing::px`]. Os dois nomes existem
    /// para que o sítio de uso diga qual das duas perguntas está a fazer: um nome só tornaria a
    /// diferença invisível no diff.
    pub const fn factory_px(self) -> f32 {
        match self {
            Self::Xxs => crate::generated::SPACING_XXS,
            Self::Xs => crate::generated::SPACING_XS,
            Self::Sm => crate::generated::SPACING_SM,
            Self::Md => crate::generated::SPACING_MD,
            Self::Lg => crate::generated::SPACING_LG,
            Self::Xl => crate::generated::SPACING_XL,
            Self::Xl2 => crate::generated::SPACING_XL2,
            Self::Xl3 => crate::generated::SPACING_XL3,
            Self::Xl4 => crate::generated::SPACING_XL4,
        }
    }

    /// O valor **VIVO** — o que o artista autorou neste modo, ou a fábrica.
    ///
    /// ⚠️ **Não recebe modo, e isso é a wave inteira numa assinatura:** a pergunta *"qual é o modo
    /// vigente?"* é respondida **uma vez por quadro** pelo [`crate::num_runtime::publish`], que a
    /// lê de onde ela é POSSUÍDA. Enfiá-la aqui obrigaria os ~1200 sítios de leitura a carregá-la,
    /// e a resposta seria a mesma nos 1200.
    ///
    /// ⚠️ Com a escala de fábrica intacta isto é **uma leitura de bool** e o resultado é o
    /// [`Spacing::factory_px`], bit a bit.
    #[must_use]
    pub fn px(self) -> f32 {
        crate::num_runtime::live(crate::num::NumToken::Spacing(self)).unwrap_or(self.factory_px())
    }

    /// Token id (matches JSON key).
    pub const fn id(self) -> &'static str {
        match self {
            Self::Xxs => "xxs",
            Self::Xs => "xs",
            Self::Sm => "sm",
            Self::Md => "md",
            Self::Lg => "lg",
            Self::Xl => "xl",
            Self::Xl2 => "2xl",
            Self::Xl3 => "3xl",
            Self::Xl4 => "4xl",
        }
    }
}

/// ⭐⭐⭐ **O tamanho de um ÍCONE EM LINHA** (14 px). Per tokens.json `chrome.inline-icon`.
///
/// ⛔⛔ **Ele chamava-se `chrome.section-gap` até à wave 19, e o nome MENTIA.** Censo dos seus
/// consumidores, 2026-09-07 — são **quatro**, e nenhum é um vão de secção:
///
/// | onde | o que ele pede |
/// |---|---|
/// | `context_menu_overlay.rs` | o **glifo** de um item de menu |
/// | `topbar/cluster_painter.rs` | o **galo** de um cluster do topo |
/// | `section_header/mod.rs` | o **piso da altura** de um chip de cabeçalho |
/// | `ph2d-panel-hierarchy/row.rs` | a **amostra de cor** de uma linha |
///
/// ⚠️ **Um token cujo nome descreve outra pergunta é pior que um número à solta:** quem quisesse
/// apertar o vão entre secções mexeria neste, e **encolheria quatro ícones** — em painéis
/// diferentes, sem nada a ficar vermelho. E a pergunta que o nome dele reclamava ficou anos sem
/// dono, o que é exactamente como a cauda de um bloco chegou a **quatro** respostas.
///
/// ⇒ o vão de secção tem hoje o nome dele ([`section_gap_px`], derivado), e este passa a chamar-se
/// pelo que os seus quatro leitores pedem. *Renomear não move um pixel — move quem responde.*
pub const INLINE_ICON_PX: f32 = crate::generated::CHROME_INLINE_ICON;

/// Row height by density. Per tokens.json `density.*`.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
pub enum Density {
    /// `compact` row height — max density.
    Compact,
    /// `cozy` row height — balanced.
    Cozy,
    /// `comfortable` row height — comfortable (default for tablet/Pencil).
    #[default]
    Comfortable,
}

impl Density {
    pub const fn row_h_px(self) -> f32 {
        match self {
            Self::Compact => crate::generated::DENSITY_COMPACT,
            Self::Cozy => crate::generated::DENSITY_COZY,
            Self::Comfortable => crate::generated::DENSITY_COMFORTABLE,
        }
    }
}

/// Default icon-button square size. Per tokens.json `chrome.icon-btn-size`.
pub const ICON_BTN_SIZE_PX: f32 = crate::generated::CHROME_ICON_BTN_SIZE;

/// Default body row height. Per tokens.json `chrome.row-h`.
pub const ROW_H_PX: f32 = crate::generated::CHROME_ROW_H;

/// ⭐⭐⭐ **O AVANÇO de uma linha de formulário para a seguinte — e é UMA resposta, não quatro.**
///
/// Enio, 2026-09-06 (com as fotos do Blender e do Godot lado a lado): *«Blender e Godot com
/// aspecto muito mais compacto e profissional. Espaçamento muito regrado e universal.»*
///
/// ⚠️ **A palavra é «universal», e ela nomeia o mecanismo, não o número.** Medido no fonte do
/// Godot (MIT, `editor/themes/theme_modern.cpp`): **nenhuma constante de espaço daquele tema é
/// escolhida** — todas são `base_margin · k` a partir de um `base_spacing = 4`, e a que os
/// contentores lêem tem NOME próprio (`separation_margin`), lido por `BoxContainer`,
/// `HBoxContainer`, `VBoxContainer`, `GridContainer`, `FlowContainer` e `FoldableContainer`.
/// *Um nome é o que impede a segunda resposta.*
///
/// ⛔ **E nós tínhamos QUATRO respostas para a MESMA pergunta**, censadas em 2026-09-06:
/// `Xs` (4 px) em 63 sítios · **`Sm` (6 px) em 20** · `Xxs` (2 px) em 3 · `Md` (8 px) em 1.
/// Os 20 do `Sm` não estavam espalhados: eram o **Inspector** e o **Painter Layers** inteiros —
/// os dois painéis em que o artista vive respiravam 50 % mais que o resto do app, e nenhum
/// teste podia ver isso, porque cada sítio estava certo sozinho.
///
/// **O valor é o do modelo:** `separation_margin` do Godot Modern = `base_spacing` = **4 px**,
/// que é o `Spacing::Xs` desta casa. ⚠️ E ele é o número da PILHA de linhas de formulário, que
/// é a superfície destes painéis — o Godot tem outros dois para outras duas superfícies, e cada
/// um é derivado, nunca escolhido: uma **lista** (`Tree`) tem `pow(round(base·0.175), 3)` = **1**
/// — as linhas encostam sobre um fio, e desde a wave 17 isso é a porta [`list_row_gap_px`]
/// (⚠️ esta linha dizia **0** por truncar `0,7³`: o `EDSCALE_RND` arredonda ANTES do cubo); uma
/// **grelha** tem `widget_margin.y − 2` = **3**. ⇒ quando esta casa precisar de uma dessas, ela
/// nasce **aqui, com nome e derivação**, e não num `+ Spacing::Qualquer` no sítio da pintura.
pub fn row_pitch_px() -> f32 {
    ROW_H_PX + control_gap_px()
}

/// **A margem vertical de um WIDGET no modelo** — `increased_margin + 1`, com o `increased_margin`
/// a ser o `base_spacing` desta casa (`theme_modern.cpp:286`). Existe só para o
/// [`control_gap_px`] poder ser escrito como o Godot o escreve.
fn widget_margin_y_px() -> f32 {
    Spacing::Xs.px() + 1.0
}

/// ⭐⭐⭐ **O VÃO ENTRE DOIS CONTROLOS de uma secção — e é UM número, para tudo.**
///
/// Enio, 2026-09-07, com a foto do painel *3D Model*: *«entre grupos de botões temos um
/// espaçamento, entre sliders outro espaçamento. Para ambos vamos colocar o padrão de espaçamento
/// de 3 px».*
///
/// ⭐ **O 3 dele é EXACTAMENTE o número do modelo, e ele chegou lá pelo olho:**
/// `GridContainer.v_separation = round(widget_margin.y − 2)` = `(4 + 1) − 2` = **3**
/// (`theme_modern.cpp:983`). O corpo de um painel desta casa **é** uma grelha de controlos, e era
/// a constante da grelha que faltava.
///
/// ⛔⛔ **Ela FUNDE duas portas que a wave 19 tinha separado — um dia antes — e o motivo não é o
/// veredito do dono, é a PREMISSA da 19 ter dissolvido.** Aquela wave defendeu um `block_gap` (6)
/// maior que o `row_gap` (4) com este argumento: *«a fronteira de um bloco tem de se ler mais que
/// a fronteira entre duas linhas DELE»*. Isso só vale enquanto as linhas de um bloco distam o vão
/// de linha — e a wave 20 **junta os botões em grupos**, onde as peças distam um **fio de 1 px**.
/// ⇒ com o interior a `1`, uma fronteira a `3` já se lê com folga, e o degrau do meio deixa de
/// pagar-se. *Uma recusa medida responde uma pergunta; quando alguém muda o substrato, ela tem de
/// ser reconferida* (§0.0).
///
/// ⇒ a escada fica com **três** degraus, cada um uma constante do Godot Modern:
///
/// | pergunta | porta | valor | derivação |
/// |---|---|---|---|
/// | duas linhas de uma LISTA | [`list_row_gap_px`] | 1 | `Tree.v_separation` |
/// | dois CONTROLOS de uma secção | **esta** | **3** | `GridContainer.v_separation` |
/// | dois CARTÕES de secção | [`section_gap_px`] | 8 | `Separator separation` |
///
/// ⚠️ **O nome é `control_gap`, e não `row_gap`, de propósito.** O antigo descrevia uma população
/// mais estreita do que a que o chamava — e é exactamente assim que um `block_gap` nasce ao lado
/// dele. *Um nome que só cobre metade dos leitores convida o segundo número.*
pub fn control_gap_px() -> f32 {
    widget_margin_y_px() - 2.0
}

/// ⭐⭐ **O que fica depois de um CARTÃO DE SECÇÃO** — e este número já era shipado, sem nome.
///
/// O modelo de cartão (wave 9) fecha cada secção com `y + card_pad()·2`, onde o `card_pad` é o
/// `Spacing::Xs`: **8 px**, contados do fim do conteúdo, com o recuo de baixo do cartão a ser
/// metade deles. ⭐ **É exactamente o `Separator separation = base_margin · 2` do Godot Modern**
/// (`theme_modern.cpp:885`) — duas derivações independentes no mesmo número, e nenhuma delas
/// escolhida.
///
/// ⚠️ **Dar-lhe nome não muda um pixel: muda quem responde.** Enquanto ele vivia como `pad * 2.0`
/// dentro do `close_at`, a pergunta *«quanto separa dois blocos?»* não tinha onde ser comparada
/// com *«quanto separa duas secções?»* — e foi essa ausência que deixou a cauda de um bloco crescer
/// para quatro respostas, uma delas (`Md` = 8) **igual à da secção**, que é a que faz um bloco
/// interior ler-se como uma secção.
pub fn section_gap_px() -> f32 {
    Spacing::Xs.px() * 2.0
}

/// ⭐⭐⭐ **O vão entre duas linhas de uma LISTA — e ele NÃO é o de um formulário.**
///
/// Um formulário empilha controlos INDEPENDENTES (`label | control`), e o vão diz *estes são
/// dois assuntos*. Uma lista empilha os ITENS DE UMA COISA SÓ — as camadas, os objectos da cena,
/// as variações de um som —, e ali o vão diz o contrário: *isto é um corpo*. É a mesma lei do
/// grupo de botões que a wave 10 portou, virada na vertical.
///
/// **O número é derivado, não escolhido** (Godot Modern, MIT, `theme_modern.cpp:650`):
///
/// ```text
/// tree_v_sep = enable_touch_optimizations
///     ? separation_margin * 0.9                     // 4 * 0.9 = 3.6 -> int 3
///     : pow(EDSCALE_RND(base_margin * 0.175), 3);   // round(0.7) = 1 -> 1^3 = 1
/// ```
///
/// ⇒ com o `base_spacing` desta casa (`Spacing::Xs` = 4) dá **1 px**: as linhas encostam sobre um
/// fio, exactamente como as peças de um grupo (`SEGMENT_HAIRLINE`). ⛔ **Não é zero** — dois itens
/// seleccionados em seguida têm de continuar a ler-se como dois.
///
/// ⚠️⚠️ **E o `0` que este repo escreveu duas vezes era MEU, não do modelo.** A wave 8 registou
/// *«`Tree.v_separation = pow(base·0.175,3) = 0`»* por truncar `0,7³ = 0,343`; o `EDSCALE_RND`
/// **arredonda primeiro** e o cubo é de `1`. *Uma derivação copiada sem se avaliar a expressão
/// inteira é um número escolhido com cara de lei.*
///
/// ⏳ **O ramo de TOQUE fica NOMEADO e por construir** (`separation_margin · 0,9` = **3 px**): o
/// alvo desta casa é tablet, e o próprio modelo dá à lista mais ar quando o dedo é o ponteiro.
/// Ligá-lo é decisão do dono, e exige o interruptor que ainda não existe — ⛔ não o adivinhe.
pub fn list_row_gap_px() -> f32 {
    (Spacing::Xs.px() * 0.175).round().powi(3)
}

/// ⭐⭐⭐ **A COLUNA DO CHEVRON de uma linha de árvore** — e ela era escrita em DOIS ficheiros.
///
/// A hierarquia declarava-a em `row.rs` (`chev_w`) e outra vez em `paint.rs` (`chev_col_w`), esta
/// segunda com o comentário `// sync with row.rs chev_w` ao lado. ⚠️ **Um comentário que manda
/// sincronizar é a confissão de que falta uma porta** — e as duas cópias não estavam a ser
/// vigiadas por nada: cada uma estava certa sozinha, e a que derivasse desalinhava a linha de
/// parentesco por baixo da seta, que é literalmente um report que este painel já pagou
/// (Enio, 2026-05-26: *«a linha que mostra parentesco deveria sair exactamente abaixo da
/// setinha»*).
///
/// ⚠️ **Ela é também o PISO do recuo** ([`list_indent_px`]): um passo mais estreito que a coluna
/// da seta punha a seta de um filho por cima da do pai.
pub fn tree_chevron_col_px() -> f32 {
    Spacing::Lg.px()
}

/// ⭐⭐⭐ **O RECUO de um filho numa árvore — e é UM número, não quatro.**
///
/// **O número é derivado, não escolhido** (Godot Modern, MIT, `theme_modern.cpp:653`):
///
/// ```text
/// item_margin = EDSCALE_RND(MAX(3 * increased_margin, 12))
/// ```
///
/// ⇒ com o `increased_margin` desta casa (`Spacing::Xs` = 4): `MAX(12, 12)` = **12 px**.
///
/// ⛔⛔ **E nós tínhamos QUATRO respostas, uma por superfície** — censadas em 2026-09-07:
///
/// | superfície | escrevia | passo |
/// |---|---|---|
/// | Hierarquia | `Spacing::Xl` | **16** |
/// | `variant_editor` | `INDENT_PX = 16.0` à mão | **16** |
/// | Painter Layers | `LAYER_INDENT_STEP = 14.0` à mão | **14** |
/// | Catálogo do Asset Browser | `Spacing::Md` | **8** |
/// | `tree_view` (a GALERIA) | `Spacing::Lg` | **12** ✅ |
///
/// ⭐⭐ **A galeria de widgets já tinha a resposta do modelo, e nenhuma superfície do produto a
/// copiou** — o `tree_view` é a peça de referência do cromo e vive só na bancada. *Uma referência
/// que ninguém chama não ensina nada: ela só regista que a resposta certa já era conhecida.*
///
/// ⚠️ **O PISO tem recurso, e o recurso é a coluna da seta** ([`tree_chevron_col_px`], 12 px):
/// dois níveis consecutivos põem as suas setas a `passo` de distância, logo um passo abaixo da
/// largura da seta faz a do filho entrar por cima da do pai. É por isso que ele é escrito como um
/// `max` contra ESSA largura, e não contra o `12` cru que o Godot escreve — *um limite legítimo
/// diz de que recurso ele é* (§0.0).
///
/// ⚠️ **Hoje os dois lados do `max` valem 12, logo o piso não é observável no produto** — e é
/// exactamente por isso que a derivação vive em [`indent_from`], que os testes chamam com um
/// `increased_margin` menor. *Uma cerca que só se lê no valor de hoje é uma cerca que a próxima
/// mutação atravessa sem acordar ninguém.*
pub fn list_indent_px() -> f32 {
    indent_from(Spacing::Xs.px(), tree_chevron_col_px())
}

/// A derivação do [`list_indent_px`], com os dois termos ABERTOS.
///
/// ⚠️ **Existe para o piso ser matável por mutação.** Com os números de hoje `3 · 4` e a coluna da
/// seta valem os dois `12`, então apagar o `.max(..)` devolve o mesmo valor e a mutação
/// **sobrevive** — o defeito que esta linha já pagou quatro vezes (*escrevo a guarda certa e não a
/// gateio*). Chamada com um `increased_margin` menor, a cerca volta a ser observável.
fn indent_from(increased_margin: f32, chevron_col: f32) -> f32 {
    (increased_margin * 3.0).max(chevron_col)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scale_is_strictly_increasing() {
        let scale = [
            Spacing::Xxs,
            Spacing::Xs,
            Spacing::Sm,
            Spacing::Md,
            Spacing::Lg,
            Spacing::Xl,
            Spacing::Xl2,
            Spacing::Xl3,
            Spacing::Xl4,
        ];
        for w in scale.windows(2) {
            assert!(
                w[0].px() < w[1].px(),
                "scale broken at {:?} → {:?}",
                w[0],
                w[1]
            );
        }
    }

    #[test]
    fn ids_match_tokens_json() {
        assert_eq!(Spacing::Xxs.id(), "xxs");
        assert_eq!(Spacing::Xl2.id(), "2xl");
        assert_eq!(Spacing::Xl4.id(), "4xl");
    }

    #[test]
    fn density_row_height_strictly_increasing() {
        assert!(Density::Compact.row_h_px() < Density::Cozy.row_h_px());
        assert!(Density::Cozy.row_h_px() < Density::Comfortable.row_h_px());
    }

    #[test]
    fn comfortable_is_default() {
        assert_eq!(Density::default(), Density::Comfortable);
    }

    /// ⭐ **O recuo e' o `item_margin` do modelo** — `MAX(3 * increased_margin, 12)`.
    #[test]
    fn the_indent_is_the_models_item_margin() {
        assert_eq!(list_indent_px(), Spacing::Lg.px());
        assert_eq!(list_indent_px(), 12.0);
    }

    /// ⛔⛔ **O PISO existe, e o recurso dele e' a coluna da seta.**
    ///
    /// Com os numeros de hoje os dois lados do `max` valem 12, logo apagar o piso nao muda o
    /// produto e a mutacao SOBREVIVE. Este teste chama a derivacao com um `increased_margin`
    /// menor, que e' onde a cerca se le^.
    #[test]
    fn a_narrower_scale_never_pushes_a_child_arrow_over_its_parents() {
        let chev = tree_chevron_col_px();
        // Uma escala apertada (`increased_margin` = 2) pediria um passo de 6 px — metade da
        // coluna da seta. O piso segura-o na largura dela.
        assert_eq!(indent_from(2.0, chev), chev);
        // E acima do piso a derivacao manda: uma escala larga passa a 3x a margem.
        assert_eq!(indent_from(8.0, chev), 24.0);
    }

    /// ⚠️ **A coluna da seta e' UM numero** — ela era declarada em dois ficheiros da hierarquia.
    #[test]
    fn the_chevron_column_is_the_floor_of_the_indent() {
        assert!(
            list_indent_px() >= tree_chevron_col_px(),
            "o passo do recuo ficou menor que a coluna da seta: a seta de um filho entra por \
             cima da do pai"
        );
    }
}
