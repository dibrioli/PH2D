//! **O CODEGEN DE PAINEL** — a árvore autorada descreve um painel, e esta crate escreve o código
//! dele (plano UI/UX §4 W8b, o *degrau 3* do §2).
//!
//! # A decisão que o §2 do plano já tinha tomado, e que esta crate honra
//!
//! O artista **não desenha um slider**: ele desenha a *aparência* de um slider, e o comportamento
//! continua sendo o dos 44 widgets tipados do `ph2d-editor-core`. As duas alternativas foram
//! recusadas com motivo — *o desenho VIRAR o widget* reimplementaria drag/foco/teclado/a11y no
//! canvas (duas respostas a *"o que é um slider?"*, e a do canvas nasce pior), e *o widget virar
//! desenho* trocaria 44 widgets testados por um interpretador.
//!
//! ⇒ O que sai daqui é **código-fonte que USA o catálogo**, não um interpretador que o substitui.
//!
//! # ⚠️ Esta crate NÃO conhece o catálogo de widgets, e isso é estrutural
//!
//! Ela não depende do `ph2d-editor-core`. Sem alcance ao `WidgetKind` ela **não CONSEGUE** ter
//! opinião sobre o que um `Slider` é: o **identificador** do tipo chega pronto no [`RowSpec`],
//! resolvido por quem é dono dele. Uma tabela `código → nome` aqui dentro seria a **segunda
//! resposta** a *"quais são os tipos vestíveis?"*, e ela driftaria do enum no dia em que alguém
//! acrescentasse um — em silêncio, porque um número desconhecido não falha, ele só não casa.
//!
//! É a mesma contenção que o `ph2d-paint-gpu` usa contra o `ph2d-painter-brush`, e ela é
//! **verificada por arch-gate sobre este `Cargo.toml`**, não por disciplina.
//!
//! # O que ele emite, e por que é uma TABELA
//!
//! Uma **tabela de rows**, no formato que os painéis deste repo já usam — o `SECTIONS` do
//! `ph2d-panel-physics`, cuja lei é *uma tabela, quatro consumidores* (`paint` · `populate` ·
//! `apply_event` · o gate de seam iteram a MESMA lista). Emitir o *chrome* — a moldura, o
//! title-bar, as alças de arraste, o gate de visibilidade — seria copiar para dentro do gerado
//! umas cem linhas que **são idênticas em todo painel** e que já têm dono; e um dia em que o
//! chrome mudasse, todo painel gerado ficaria para trás.
//!
//! ⚠️ **Daí a régua desta wave:** *o gerado é o que VARIA entre dois painéis, e nada mais.*
//!
//! ⚠️ **A row é uma STRUCT (`RowConst`) definida por quem COMPILA o gerado**, não por esta crate.
//!
//! Ela nasceu TUPLA, com o argumento de que *"uma struct teria de ser definida em algum lugar, e o
//! único lugar honesto seria junto do `WidgetKind`"* — superfície foundational nova. A premissa
//! estava errada: o lugar honesto é a **crate do painel**, que já hospeda o `include!`. O emissor
//! escreve o nome do tipo sem o conhecer, exactamente como já escreve `WidgetKind::Slider`.
//!
//! ⚠️ **Quem disse que a tupla tinha crescido foi o `clippy::type_complexity`**, à quinta posição.
//! E o preço real não era a legibilidade: uma tupla é **posicional**, então todo campo novo
//! re-escreve cada consumidor que a desestrutura — o que aconteceu duas vezes no mesmo dia (a cor,
//! e depois o glifo). Um campo novo numa struct não toca em quem não o lê.
//!
//! ⚠️ **E a prova de compilação fica igual**: `WidgetKind::Slider` continua conferido pelo
//! compilador, e agora o nome dos CAMPOS também — renomear um variante do catálogo, ou um campo do
//! `RowConst`, **quebra o build do gerado**, que é exatamente o que se quer de código gerado.
//!
//! # O ID de cada row é DERIVADO da chave, nunca cunhado aqui
//!
//! Um `NodeId` literal num arquivo gerado teria de entrar no gate de colisão do repo — e um
//! gerador que cunha ids é um gerador que pode cunhar o mesmo duas vezes. A row carrega a
//! **chave** e o id sai de `hash(chave)` em runtime, que é o idioma que o `command_palette` e o
//! painel de Wet Tuning já usam.

#![forbid(unsafe_code)]

/// Uma linha do painel, já resolvida: o **identificador** do tipo de widget (nunca o código
/// numérico — ver o doc da crate) e o que ela mostra.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RowSpec {
    /// O identificador do tipo no catálogo — `"Slider"`, `"Toggle"`, … Quem o resolve é o dono do
    /// enum; aqui ele é texto.
    pub kind: String,
    /// O rótulo que a row mostra. Sai do `Name` da entidade autorada.
    pub label: String,
    /// A chave estável da row. O `NodeId` é `hash(chave)` em runtime — ver o doc da crate.
    pub key: String,
    /// **A cor que esta row mostra**, quando o tipo dela É uma cor (a `ColorSwatch`).
    ///
    /// ⚠️ É o único parâmetro que o retângulo, o rótulo e os tokens **não conseguem** responder:
    /// *que cor?* é conteúdo, e ele vem do preenchimento da forma que veste o widget. Todo outro
    /// tipo emite `None`, e o `None` da swatch **é** o xadrez de transparência — ou seja, *nenhuma
    /// cor escolhida*, que é o que ele significa em toda UI de cor que existe.
    ///
    /// ⚠️ E ele é um SNAPSHOT, como o `label` ao lado: o painel gerado mostra a cor que a forma
    /// tinha quando o código foi escrito. Quem quer a cor VIVA olha a moldura no canvas, que lê o
    /// documento a cada quadro.
    pub rgba: Option<[u8; 4]>,
    /// **O glifo que esta row desenha**, quando o tipo dela É um botão de ícone — o desenho da
    /// forma que veste o widget, em **texto SVG**.
    ///
    /// ⚠️ **Texto, e não geometria, porque o destino é um `const`:** um `BezPath` não é
    /// construível em tempo de compilação. O painel gerado carrega a string e reconstitui a curva
    /// uma vez, no `OnceLock` que já resolve os ids — o mesmo lugar, o mesmo custo amortizado.
    ///
    /// ⚠️ **E esta crate segue sem opinião sobre geometria**: ela não conhece `BezPath`, não sabe
    /// normalizar nada e não vai aprender. Quem produz a string é o dono do documento, exactamente
    /// como o `kind` chega aqui já resolvido em texto.
    pub icon: Option<String>,
    /// **O ícone ESCOLHIDO do catálogo do editor**, pelo slug — a outra rota do glifo.
    ///
    /// ⚠️ **O SLUG, nunca o número:** o discriminante de `IconId` é a posição alfabética do
    /// arquivo SVG, então acrescentar um ícone empurra todos os que vêm depois dele — e um número
    /// no código gerado passaria a nomear outro glifo, em silêncio, em todo painel já commitado.
    ///
    /// ⚠️ Ele e o [`Self::icon`] são **mutuamente exclusivos por construção**: quem os preenche
    /// pergunta a precedência à porta única do catálogo, e ela devolve UM dos dois.
    pub icon_slug: Option<String>,
    /// Os rótulos das opções, para a família de LISTA — os filhos que o controle possui.
    pub options: Vec<String>,
}

/// **O painel que uma árvore autorada descreve.**
///
/// ⚠️ Ele é *dado simples de propósito*: quem o constrói lê o ECS e o documento (a shell), quem o
/// consome escreve texto (esta crate). A fronteira entre os dois é esta struct, e é ela que torna
/// as duas metades testáveis de cabeça para baixo — sem janela, sem GPU e sem ECS.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct PanelSpec {
    /// O slug do painel — sai do `Name` da moldura.
    pub id: String,
    /// O título que a barra mostra.
    pub title: String,
    pub rows: Vec<RowSpec>,
}

/// O prefixo de toda linha gerada que não é dado — o cabeçalho que diz *não edite isto*.
const HEADER: &str = "\
// @generated by ph2d-ui-codegen — NÃO EDITE À MÃO.
//
// ⚠️ Editar a SAÍDA de um gerador é o que deixa um gate de staleness vermelho, e é uma cicatriz
// que este repo já carrega (o `pub mod command_palette;` escrito à mão dentro do bloco que o
// `ph2d-widget-sync` gera). O que se edita é o DESENHO; isto é a sombra dele.
//
// O gate que apanha isto é o `the_generated_panel_is_what_the_emitter_emits`, na
// `ph2d-panel-authored`: ele re-emite este arquivo a partir das constantes que ele próprio declara
// e compara byte a byte.
";

/// **Escreve o código da tabela de rows deste painel.**
///
/// O texto sai pronto para `rustfmt` e é **determinista**: a mesma `PanelSpec` produz os mesmos
/// bytes, porque é isso que torna o gate de staleness possível.
#[must_use]
pub fn emit(spec: &PanelSpec) -> String {
    let mut out = String::with_capacity(512 + spec.rows.len() * 96);
    out.push_str(HEADER);
    out.push_str("\npub const PANEL_ID: &str = ");
    push_str_literal(&mut out, &spec.id);
    out.push_str(";\npub const PANEL_TITLE: &str = ");
    push_str_literal(&mut out, &spec.title);
    out.push_str(
        ";\n\n/// A tabela deste painel. A chave vira `NodeId` por hash; a cor só é `Some` onde o tipo\n/// É uma cor, e o ícone (SVG) onde ele É um botão de ícone.\npub const ROWS: &[RowConst] = &[\n",
    );
    for r in &spec.rows {
        out.push_str("    RowConst {\n        kind: WidgetKind::");
        out.push_str(&r.kind);
        out.push_str(",\n        label: ");
        push_str_literal(&mut out, &r.label);
        out.push_str(",\n        key: ");
        push_str_literal(&mut out, &r.key);
        out.push_str(",\n        rgba: ");
        match r.rgba {
            None => out.push_str("None"),
            Some([r8, g8, b8, a8]) => {
                out.push_str(&format!("Some([{r8}, {g8}, {b8}, {a8}])"));
            }
        }
        out.push_str(",\n        icon: ");
        push_opt_str(&mut out, r.icon.as_deref());
        out.push_str(",\n        icon_slug: ");
        push_opt_str(&mut out, r.icon_slug.as_deref());
        // ⚠️ Uma FATIA e não um `Option`: *sem opções* e *lista vazia* são a mesma coisa para um
        // tipo que não é de lista, e um `Option<&[..]>` daria dois jeitos de dizer o mesmo nada.
        out.push_str(",\n        options: &[");
        for (i, o) in r.options.iter().enumerate() {
            if i > 0 {
                out.push_str(", ");
            }
            push_str_lit(&mut out, o);
        }
        out.push(']');
        out.push_str(",\n    },\n");
    }
    out.push_str("];\n");
    out
}

/// Um literal de texto escapado — a metade que o [`push_opt_str`] embrulha em `Some`.
fn push_str_lit(out: &mut String, v: &str) {
    out.push('"');
    for c in v.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            _ => out.push(c),
        }
    }
    out.push('"');
}

/// `Some("…")` ou `None` — o campo opcional de texto, escapado.
fn push_opt_str(out: &mut String, v: Option<&str>) {
    match v {
        None => out.push_str("None"),
        Some(d) => {
            out.push_str("Some(");
            push_str_literal(out, d);
            out.push(')');
        }
    }
}

/// Um literal de string Rust, com o que precisa de escape **escapado**.
///
/// ⚠️ Ela existe porque o rótulo vem do `Name` da entidade, que o artista digita: uma aspa num
/// nome de camada não pode produzir um arquivo que não compila. O conjunto é o mínimo que o
/// léxico do Rust exige para uma string comum — `\` e `"` — mais os dois controles que um
/// copiar-e-colar traz (`\n`, `\r`) e que quebrariam a linha no meio.
fn push_str_literal(out: &mut String, s: &str) {
    out.push('"');
    for c in s.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            _ => out.push(c),
        }
    }
    out.push('"');
}

#[cfg(test)]
#[path = "lib_tests.rs"]
mod tests;
