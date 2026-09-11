//! **UMA LINHA DE MENU PINTADA TEM DE ESTAR REGISTADA NO STORE.**
//!
//! # O defeito, e por que ele não dá erro nenhum
//!
//! O despachante de ponteiro rejeita um id que não está no [`WidgetStore`]: sem um estado de
//! `Button`, o id não é `is_focusable`, não fica `active` no Down, e **nunca emite `Click`**. A
//! linha pinta, o rato passa por cima, o artista clica — e não acontece coisa nenhuma.
//!
//! ⚠️ **Nada nesta cadeia falha alto.** Não há erro, não há log, não há aviso: o único sintoma é um
//! botão morto, e a conclusão do artista é que o app está partido.
//!
//! # Como isto foi encontrado, que é o motivo de o gate existir
//!
//! Enio, 2026-08-21: *"Export não abre dialog para salvar."* A linha **"Export Image…"** tinha
//! nascido no mesmo dia, e estava correcta em **dois** dos três sítios que uma linha de menu exige:
//!
//! | sítio | estava? |
//! |---|---|
//! | a **tabela** (`menu_rows`) — o que se pinta | ✅ |
//! | o **guarda** + a cadeia (`ph2d-panel-hierarchy/src/event.rs`) — o que se despacha | ✅ |
//! | o **registo** (`pre_populate`) — o que se pode CLICAR | ❌ |
//!
//! ⚠️ E o gate irmão `every_hierarchy_row_menu_entry_dispatches_something` estava **verde**, porque
//! ele chama o handler **directamente** — salta a camada de ponteiro, que é precisamente onde o
//! clique morria. *Dois gates a medir a mesma linha podem estar os dois certos e deixar o buraco no
//! meio: um mede o que o handler faz, o outro tem de medir se o handler chega a ser chamado.*
//!
//! # Por que a fonte é a TABELA
//!
//! ⛔ Uma lista de ids dentro deste teste teria de ser actualizada para cobrir o caso novo — e um
//! gate que precisa de ser actualizado para apanhar o caso novo não apanha caso novo nenhum. Ele lê
//! `menu_rows` para **todos** os `ContextMenuKind` que têm tabela, por isso uma linha nova entra
//! aqui no dia em que é pintada, sem ninguém se lembrar.
//!
//! ⛔⛔ **E durante meses ele NÃO fazia isso — o doc acima prometia a propriedade derivada e o
//! código negava-a.** A lista de tipos era um `vec![]` escrito à mão com **UMA** variante
//! (`HierarchyRow`) de **trinta**: `SaveMenu`, `OpenMenu`, `SettingsMenu`, `ThemeSelector`, os
//! quatro modais e os dez menus da timeline eram invisíveis a um gate que se dizia exaustivo.
//! Apagar `ids::CTX_MENU_SAVE` do `pre_populate` deixava a linha *Save · Cmd+S* pintada e morta,
//! com este gate VERDE. Hoje a lista é [`ContextMenuKind::ALL`], e o gate irmão
//! [`the_sample_list_names_every_variant_of_the_enum`] recusa uma variante que não entre lá.
//!
//! ⚠️ **Os modais deixaram de precisar de exclusão, e não porque a regra afrouxou:** o `menu_rows`
//! devolve `&[]` para `NewImageDialog`, `SheetSizeDialog`, `RenamePaletteDialog` e `SceneList`
//! (eles pintam o próprio corpo, com o próprio registo), então varrê-los custa zero e cobre-os no
//! dia em que algum ganhar uma row de tabela. *Uma exclusão escrita à mão é mais uma lista a
//! apodrecer.*

use std::fs;

use ph2d_editor_core::HeroScreen;
use ph2d_editor_core::NodeId;
use ph2d_editor_core::interaction::ContextMenuKind;
use ph2d_editor_core::screens::hero::menu_rows::menu_rows;

#[test]
fn every_painted_menu_row_is_registered_and_therefore_clickable() {
    // ⚠️ Pelo `HeroScreen::new`, que é quem chama o `pre_populate_store` no produto. Construir um
    // `WidgetStore` à mão e chamar o povoador directamente mediria uma cadeia que o app não usa.
    let hero = HeroScreen::new(NodeId(1));

    let mut dead: Vec<String> = Vec::new();
    let mut rows_seen = 0usize;
    for kind in ContextMenuKind::ALL.iter().copied() {
        for (id, label, _) in menu_rows(kind) {
            rows_seen += 1;
            // ⚠️ **A barra é a do `is_focusable` do despachante**
            // (`interaction/dispatch/focus.rs`), e não uma inventada aqui: o que mata o clique é o
            // ramo `None => false` — um id **ausente** do store não fica `active` no Down e o
            // `Click` nunca nasce.
            //
            // ⚠️ **Não se exige `Button`.** As linhas de menu registam-se como `Plain`, que o
            // `is_focusable` aceita — a primeira versão deste gate exigiu `Button` e acusou as
            // dezassete, incluindo as que funcionam há meses. *Um gate que acusa o legítimo é
            // desligado na primeira semana; a barra tem de ser a que o produto de facto usa.*
            if hero.store.get(*id).is_none() {
                dead.push(format!("{label}   (menu {kind:?})"));
            }
        }
    }
    // ⚠️ **O controle positivo do CORPUS.** Uma lista de amostras que encolhesse — ou um
    // `menu_rows` que passasse a devolver `&[]` — deixaria este gate verde por VÁCUO, que é a
    // forma que ele acabou de curar. A barra é derivada: cada tipo pinta pelo menos uma row, menos
    // os quatro que desenham o próprio corpo.
    assert!(
        rows_seen >= ContextMenuKind::ALL.len(),
        "varri {} tipos de menu e vi só {rows_seen} rows — o corpus está vazio, e um gate que não \
         vê nada passa sempre",
        ContextMenuKind::ALL.len()
    );
    assert!(
        dead.is_empty(),
        "estas linhas de menu sao PINTADAS mas nao estao registadas no `WidgetStore` — o \
         despachante de ponteiro rejeita o id, o clique nunca vira `Click`, e o artista ve' um \
         botao morto:\n  {}\n\n\
         Registe cada uma em `screens/hero/pre_populate.rs` (a lista de `CTX_MENU_*`).\n\n\
         ⚠️ Uma linha de menu vive em TRES sitios: a tabela (`menu_rows`, o que se pinta), o \
         guarda + a cadeia do painel (o que se despacha) e o registo (o que se pode CLICAR). \
         Faltar um dos tres nao da' erro nenhum -- da' um botao que nao faz nada.",
        dead.join("\n  ")
    );
}

/// **A lista de amostras NOMEIA todas as variantes do enum** — a metade que impede
/// [`ContextMenuKind::ALL`] de apodrecer como apodreceu o `vec![]` que ela substituiu.
///
/// ⚠️ **Os dois lados são DERIVADOS, e de fontes diferentes:** as variantes que EXISTEM saem do
/// fonte do `enum` (`interaction/types_menu.rs`), e as que a lista COBRE saem do `Debug` das
/// amostras — não de um segundo texto escrito à mão, que seria a mesma doença um nível acima.
///
/// ⛔ **Sem ele, «derive a lista» resolvia-se movendo a lista de sítio.** Uma variante nova entra
/// no `enum` e este gate fica vermelho a nomeá-la; é a única coisa que faz o
/// `every_painted_menu_row_is_registered_and_therefore_clickable` valer a promessa do doc dele.
#[test]
fn the_sample_list_names_every_variant_of_the_enum() {
    let src = fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/interaction/types_menu.rs"
    ))
    .expect("interaction/types_menu.rs");

    let declared = enum_variants(&src, "ContextMenuKind");
    let covered: Vec<String> = ContextMenuKind::ALL
        .iter()
        .map(|k| {
            format!("{k:?}")
                .chars()
                .take_while(|c| c.is_alphanumeric() || *c == '_')
                .collect()
        })
        .collect();

    let missing: Vec<&String> = declared.iter().filter(|v| !covered.contains(v)).collect();
    assert!(
        missing.is_empty(),
        "estas variantes de `ContextMenuKind` nao tem amostra em `ContextMenuKind::ALL`, entao \
         TODO gate que varre os menus e' cego a elas:\n  {missing:?}\n\n\
         Acrescente uma amostra em `interaction/types_menu.rs` (payload inerte basta -- os gates \
         perguntam pela FORMA do menu, nao pelo alvo do clique).",
    );

    // …e o outro sentido, que é AO MESMO TEMPO o controle positivo do parser: se o
    // `enum_variants` deixasse de ler o `enum` (o arquivo muda de nome, o `enum` muda de forma) ele
    // devolveria `[]`, o `missing` acima ficaria vazio — verde por vácuo, a doença que este gate
    // veio curar — e é ESTA asserção que dispara, porque `ContextMenuKind::ALL` nunca é vazia.
    // *Os dois sentidos juntos são igualdade de conjuntos, e é por isso que nenhum piso escrito à
    // mão faz falta aqui.*
    let stray: Vec<&String> = covered.iter().filter(|c| !declared.contains(c)).collect();
    assert!(
        stray.is_empty(),
        "estas amostras nao correspondem a variante nenhuma do `enum` — ou o `Debug` mudou, ou o \
         parser deixou de achar o `enum` (leu {} variantes): {stray:?}",
        declared.len()
    );
}

/// Os nomes das variantes de `enum <name>`, lidos do fonte.
///
/// ⚠️ **A profundidade é lida ANTES de a linha ser contada**, e é isso que separa uma variante
/// (`SectionOutline { section: NodeId },`, no nível de topo do corpo) de um CAMPO dela
/// (`panel: NodeId,`, um nível abaixo). Contar por indentação seria um proxy que o `rustfmt`
/// expira; contar chaves é a estrutura.
fn enum_variants(src: &str, name: &str) -> Vec<String> {
    let header = format!("pub enum {name} {{");
    let at = src
        .find(&header)
        .unwrap_or_else(|| panic!("não achei `{header}` — o `enum` mudou de forma ou de arquivo"));
    let body = &src[at + header.len()..];

    let mut depth = 0i32;
    let mut out = Vec::new();
    for line in body.lines() {
        let trimmed = line.trim_start();
        if depth == 0
            && !trimmed.starts_with("//")
            && !trimmed.starts_with('#')
            && trimmed.starts_with(|c: char| c.is_ascii_uppercase())
        {
            out.push(
                trimmed
                    .chars()
                    .take_while(|c| c.is_alphanumeric() || *c == '_')
                    .collect(),
            );
        }
        for c in line.chars() {
            match c {
                '{' => depth += 1,
                '}' => depth -= 1,
                _ => {}
            }
        }
        if depth < 0 {
            break; // a chave que fecha o `enum`
        }
    }
    out
}
