//! ⭐⭐⭐ **O CATÁLOGO DE FORMAS** (W100) — tudo o que se pode acrescentar a uma peça, numa lista só:
//! o **rótulo**, a **família** e **como se constrói**.
//!
//! # Por que ele saiu do painel
//!
//! Até aqui a lista era um `[&'static str; 8]` no [`super::panel`], e as quatro entradas que não são
//! primitivas (`Extrude`, `Revolve`, e as duas esculturas) eram alcançadas por **constantes de
//! posição derivadas do FIM** (`SHAPES.len() - 4`, `- 3`, `- 2`, `- 1`). Isso funcionou enquanto a
//! lista era fixa, e é uma armadilha no dia em que ela cresce: o comentário delas diz, com todas as
//! letras, que uma forma nova entra *«antes das esculturas»* — ⛔ **acrescentar no fim faz o botão
//! *Extrude* passar a abrir o diálogo de escultura, sem erro nenhum.**
//!
//! ⭐ A cura é a entrada **trazer o próprio construtor** ([`Make`]): quem cria uma forma pergunta à
//! entrada como ela se faz, e nunca à posição dela. *Uma forma nova passa a ser UMA LINHA desta
//! tabela* — e é isso que a fila do Enio (2026-08-28: *«ao final quero todas»*) precisa, com 47
//! formas do catálogo vetorial e 15 sólidas por vir ([doc 08](../../../docs/3DModeling/08_formas_por_formula.md)).
//!
//! # ⚠️ A FAMÍLIA não é enfeite — é o que torna 60 formas navegáveis
//!
//! A fileira de chips do painel corta em **8** (`MAX_MODES`) e já tinha 8. Ela não escala, e a
//! resposta desta casa para *«um catálogo grande com categorias»* já existe e já shipou **três
//! vezes**: a paleta do `ph2d-editor-core` (a biblioteca de nós do Motion, o `Ctrl+K` global, e o
//! `+` do Inspector). Ver [`crate::field3d_shape_palette`].

use ph2d_field::Primitive;

/// ⭐ **A que grupo da paleta esta forma pertence.**
///
/// ⚠️ **A ordem das variantes é a ordem dos grupos na paleta** — [`Family::ALL`] é a fonte, como o
/// `Mode::ALL` e o `UnaryKind::ALL`. Um grupo sem nenhuma forma **não é pintado** (a lei que a
/// paleta de componentes já aplica), então uma família pode nascer vazia à espera do lote dela.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Family {
    /// Caixas e blocos — o que tem faces planas e cantos.
    Blocks,
    /// O que é redondo de origem — esfera, cilindro, cone, cápsula.
    Round,
    /// Anéis e tubos — o que tem um furo no meio por construção.
    Rings,
    /// ⭐ **Chapas** — um contorno 2D de **fórmula** puxado em Z. ⚠️ **Nasceu vazia na W100 e a
    /// estrela abriu-a na W103**; o que falta dela é o que a composição não faz (a auditoria do
    /// [doc 08 §4](../../../docs/3DModeling/08_formas_por_formula.md) mediu que cruz, lua, gota e
    /// engrenagem já se fazem com o que existe — a engrenagem é *um dente + `Radial`*).
    Plates,
    /// ⭐⭐ **SINAIS** — setas, balões e símbolos: o que se põe numa cena para ela DIZER alguma
    /// coisa, em vez de para ela ter uma peça.
    ///
    /// ⚠️ **Ela nasceu porque a `Plates` chegou a 25 itens** (W120), e a paleta existe exactamente
    /// para isso não acontecer: a fileira de chips cortava em `MAX_MODES = 8`, e um grupo que a
    /// substitui e volta a ficar longo devolve o problema pelo outro lado.
    Signs,
    /// O que sai de um **desenho** do editor vetorial.
    Drawn,
    /// O que vem de **fora** — uma escultura.
    Imported,
}

impl Family {
    pub(crate) const ALL: [Family; 7] = [
        Family::Blocks,
        Family::Round,
        Family::Rings,
        Family::Plates,
        Family::Signs,
        Family::Drawn,
        Family::Imported,
    ];

    /// O título do grupo na paleta. ⚠️ Em inglês e literal — a paleta é do `ph2d-editor-core` e
    /// recebe `String`, não chave (HR-15: quem traduz é quem pinta, e aqui quem pinta é o widget
    /// genérico, que recebe o rótulo já resolvido; é o que a paleta de componentes faz).
    pub(crate) fn title(self) -> &'static str {
        match self {
            Family::Blocks => "Blocks",
            Family::Round => "Round",
            Family::Rings => "Rings & tubes",
            Family::Plates => "Plates",
            Family::Signs => "Signs & symbols",
            Family::Drawn => "From a drawing",
            Family::Imported => "Imported",
        }
    }

    /// A tinta do grupo. ⚠️ **Um token por família** — há 7 `NodeCat*` e 6 famílias, então nenhuma
    /// partilha tinta (ao contrário da paleta de componentes, que mapeia 12 em 7). Escolher cor é
    /// decisão de design (§7): estes são os tokens que existem, não hex novo.
    pub(crate) fn color(self) -> ph2d_tokens::ColorToken {
        use ph2d_tokens::ColorToken as T;
        match self {
            Family::Blocks => T::NodeCatSource,
            Family::Round => T::NodeCatTransform,
            Family::Rings => T::NodeCatDistribute,
            Family::Plates => T::NodeCatFx,
            // ⚠️ **O SÉTIMO token, e o último** — há exactamente sete `NodeCat*`, então uma família
            // nova depois desta tem de partilhar tinta ou trazer um token, que é decisão de design
            // (§7) e não desta linha.
            Family::Signs => T::NodeCatOutput,
            Family::Drawn => T::NodeCatFocus,
            Family::Imported => T::NodeCatUtility,
        }
    }
}

/// ⭐⭐⭐ **COMO esta forma se constrói** — e é isto que substitui as constantes de posição.
///
/// ⚠️ **`Formula` carrega o construtor**, não um índice: um `match slot` posicional sobrevive a
/// acrescentar no fim e parte-se ao inserir no meio, **em silêncio** (o slot seguinte passa a
/// construir a forma do vizinho). Com o ponteiro na linha, a posição deixa de significar coisa
/// nenhuma — que é a propriedade que uma lista de 60 precisa.
///
/// ⚠️ As outras quatro **não são construíveis a partir de um raio**: duas precisam do contorno
/// desenhado e duas de um arquivo, e as duas coisas vivem fora do mundo. Quem as trata é o braço
/// próprio do `AddShape` ([`super::intents`]).
#[derive(Clone, Copy)]
pub(crate) enum Make {
    /// Uma primitiva de fórmula, no tamanho do enquadramento.
    Formula(fn(f32) -> Primitive),
    /// ⭐⭐⭐ **UMA RECEITA** (W138) — duas ou mais formas e o verbo com que se juntam.
    ///
    /// # ⛔ Por que estas duas NÃO são primitivas
    ///
    /// O `CLAUDE.md` §5.0 manda medir se a composição já exprime o item **antes** de o construir,
    /// e a medição (`probe_composed_shapes`) diz que sim, com o campo a ser **exactamente
    /// 1-Lipschitz** nas duas:
    ///
    /// | peça | `‖∇f‖` | pior campo/verdade | passos de 1,0 | de 4,0 |
    /// |---|---:|---:|---:|---:|
    /// | esfera (controlo) | 1,000 | 1,0000 | 6 | 6 |
    /// | esfera com cratera (A − B) | 1,000 | 0,5495 | 5 | 9 |
    /// | lente (A ∩ B) | 1,000 | 0,9674 | 10 | 8 |
    ///
    /// ⚠️ **A premissa do plano estava meia errada:** ele dizia que *«a nossa subtracção não dá a
    /// distância exacta na cratera»* — verdade para a cratera (`0,55`) e **falsa** para a lente
    /// (`0,97`, que é exacta a menos da amostragem do oráculo). E o módulo nunca precisou da
    /// distância exacta: precisa de um MINORANTE (doc 06 §124). O que a fórmula fechada compraria
    /// está medido e são **`9` passos contra `6`** numa região pequena.
    ///
    /// ⭐⭐ **E a composição entrega o que uma primitiva não pode:** a cratera continua a ser uma
    /// esfera na Hierarquia — move-se, redimensiona-se, duplica-se. Uma `DeathStar { ra, rb, d }`
    /// congelaria a peça em três números e daria **uma** cratera para sempre. *No idioma deste
    /// módulo (ADR-0161) a peça é uma árvore editável, e uma receita é mais essa árvore do que uma
    /// variante nova seria.*
    ///
    /// ⇒ zero variantes novas no [`Primitive`], zero degraus de `FIELD_DOC_VERSION`, zero linhas na
    /// tabela de dimensões.
    Composed(fn(f32) -> Recipe),
    /// O contorno escolhido no editor vetorial, puxado em Z.
    Extrude,
    /// O contorno escolhido, girado em torno de Y.
    Revolve,
    /// Uma escultura de um arquivo (abre diálogo).
    Sculpt,
    /// A escultura **viva** da cena, sem passar pelo disco.
    SculptScene,
}

impl Make {
    /// ⭐⭐⭐ **QUE PRIMITIVA esta porta produz** — `None` quando ela não produz nenhuma.
    ///
    /// # ⚠️ Por que ela existe, e o que substituiu
    ///
    /// A pergunta *«toda primitiva que o motor sabe fazer tem uma porta?»* era respondida por
    /// `key.ends_with(kind.key())` — uma **convenção de nome**. A W101 partiu-a com uma linha
    /// honesta: o `add.cone_truncated` produz um [`ph2d_field::PrimitiveKind::Cone`] e não acaba em
    /// «cone». Pior do que reprovar sobre um catálogo correto, uma régua de string **aprova** uma
    /// chave que calhe de acabar bem sem construir nada daquilo.
    ///
    /// ⚠️ **E o `shape_at` sozinho também não chega:** ele constrói de um raio, e o `Extrude` /
    /// `Revolve` não saem de um raio — nem por isso deixam de produzir uma primitiva. É o `Make`
    /// que sabe, porque é ele que escolhe a porta.
    ///
    /// ⚠️ Uma **escultura não é uma primitiva** (`NodeShape::Sampled`), e por isso é `None` — não
    /// é uma lacuna a preencher.
    ///
    /// ⚠️ **`cfg(test)` pela razão do [`slot_of`]**: em produção ninguém pergunta *«que família é
    /// esta porta?»* — quem cria já tem a forma na mão. Quem pergunta é o censo de alcance, e ele
    /// não pode perguntá-lo a uma convenção de nome.
    #[cfg(test)]
    pub(crate) fn builds(self) -> Option<ph2d_field::PrimitiveKind> {
        match self {
            // ⚠️ O raio é arbitrário: o que se pergunta é a FAMÍLIA, e ela não depende do tamanho.
            Make::Formula(f) => Some(f(1.0).kind()),
            Make::Extrude => Some(ph2d_field::PrimitiveKind::Extrude),
            Make::Revolve => Some(ph2d_field::PrimitiveKind::Revolve),
            // ⚠️ **Uma receita não «constrói uma primitiva», e o `None` aqui é a resposta certa e
            // não uma lacuna:** o censo que consome isto pergunta *«toda primitiva do motor tem uma
            // porta?»*, e as peças de uma receita são primitivas que já têm porta própria. Dizer
            // `Some(Sphere)` faria uma receita **cobrir** a esfera e um catálogo sem o botão dela
            // passaria no censo.
            Make::Composed(_) | Make::Sculpt | Make::SculptScene => None,
        }
    }
}

/// ⭐⭐ **A RECEITA de uma forma composta** — as peças, em ordem, e o verbo que as junta.
///
/// ⚠️ **A ORDEM é o significado** na subtracção (`children[0]` menos as seguintes), e é por isso
/// que ela é um `Vec` e não um conjunto: ordenar por qualquer outra coisa — os bits da entidade, a
/// ordem da consulta — faria o gesto tirar a peça errada.
///
/// ⚠️ **O deslocamento é RELATIVO ao alvo da câmera**, na mesma unidade em que a primitiva foi
/// construída: quem chama passa `r` uma vez e a receita inteira sai no tamanho do enquadramento.
pub(crate) struct Recipe {
    pub op: ph2d_field::Op,
    pub parts: Vec<(Primitive, [f32; 3])>,
}

/// Uma linha do catálogo.
pub(crate) struct Shape {
    /// A chave i18n do rótulo — **e a identidade da forma na paleta** (o item é o hash dela).
    ///
    /// ⚠️ A chave e não a posição: um rótulo muda quando o produto quiser, e a posição muda quando
    /// alguém insere uma linha. A chave é o que sobrevive aos dois.
    pub key: &'static str,
    pub family: Family,
    pub make: Make,
}

/// ⭐ Os construtores de cada forma — ver [`make`].
#[path = "field3d_shapes_make.rs"]
mod make;
pub(crate) use make::*;

/// ⭐ E os dos SINAIS — ver [`make_signs`].
#[path = "field3d_shapes_make_signs.rs"]
mod make_signs;
pub(crate) use make_signs::*;

/// ⭐⭐ E as RECEITAS das formas compostas — ver [`make_composed`].
#[path = "field3d_shapes_make_composed.rs"]
mod make_composed;
pub(crate) use make_composed::*;

/// ⭐ **A LISTA do catálogo** — ver [`table`].
///
/// ⚠️ **Ela saiu deste ficheiro na W136, e o corte é por responsabilidade:** aqui vive o
/// VOCABULÁRIO (o que é uma família, o que é uma linha, como uma forma nasce) e ali vive a
/// LISTA. O ficheiro estava nas `600` linhas do gate de LOC do shell e as três portas novas não
/// cabiam — ⛔ **partir para irmão, nunca uma entrada na allowlist.**
#[path = "field3d_shapes_table.rs"]
mod table;
pub(crate) use table::SHAPES;

/// A primitiva que esta posição do catálogo cria, no tamanho do enquadramento.
///
/// ⚠️ `None` para as quatro que não saem de um raio — quem as trata é o braço próprio do
/// `AddShape`, e é o [`Make`] que o diz, nunca um número.
pub(crate) fn shape_at(slot: usize, r: f32) -> Option<Primitive> {
    match SHAPES.get(slot)?.make {
        Make::Formula(f) => Some(f(r)),
        // ⚠️ **Uma receita não é UMA primitiva**, e devolver a primeira peça dela seria pior do que
        // devolver nada: o chamador criaria meia forma sem erro nenhum. Quem a trata é o braço
        // próprio do `AddShape`, pelo [`recipe_at`].
        Make::Composed(_) | Make::Extrude | Make::Revolve | Make::Sculpt | Make::SculptScene => {
            None
        }
    }
}

/// ⭐ **A receita que esta posição do catálogo cria**, no tamanho do enquadramento — irmã do
/// [`shape_at`], e `None` para tudo o que não é composto.
pub(crate) fn recipe_at(slot: usize, r: f32) -> Option<Recipe> {
    match SHAPES.get(slot)?.make {
        Make::Composed(f) => Some(f(r)),
        Make::Formula(_) | Make::Extrude | Make::Revolve | Make::Sculpt | Make::SculptScene => None,
    }
}

/// ⭐ **Esta forma pode ser criada AGORA?** — a lei da W34 (*o painel oferece exatamente o que o
/// gesto faz*) aplicada às três que dependem do que está escolhido.
///
/// ⚠️ *Um botão «Extrude» sem contorno para extrudar é a affordance que mente*, e o gesto teria de
/// falhar em silêncio ou com um aviso — os dois piores do que não estar lá. As de fórmula são
/// sempre possíveis: uma caixa não depende de nada.
pub(crate) fn available(shape: &Shape, live_sculpt: bool, profile: bool) -> bool {
    match shape.make {
        // ⚠️ Uma receita de primitivas de fórmula não depende de nada, pela mesma razão que uma
        // caixa não depende: as peças dela saem de um raio.
        Make::Formula(_) | Make::Composed(_) | Make::Sculpt => true,
        Make::Extrude | Make::Revolve => profile,
        Make::SculptScene => live_sculpt,
    }
}

/// A posição de uma forma pela **chave** — o que substitui as constantes derivadas do fim da lista.
///
/// ⚠️ Devolve `Option` de propósito: uma chave que não existe é um erro de programação que um gate
/// apanha, e não um `0` silencioso a criar uma caixa.
///
/// ⚠️ **`cfg(test)`, e a ausência em produção é a notícia:** o produto deixou de precisar de
/// procurar uma forma pelo nome — quem cria pergunta ao [`Make`] da linha que a paleta escolheu, e
/// mais ninguém pede *«onde está a escultura?»*. Quem ainda pergunta são os gates, que **não podem**
/// ler a constante que testam (*um teste que lê a constante que testa não testa a constante* — uma
/// prova de mutação passou verde por isso, ver `field3d_import_seam_tests`).
#[cfg(test)]
pub(crate) fn slot_of(key: &str) -> Option<usize> {
    SHAPES.iter().position(|s| s.key == key)
}

#[cfg(test)]
#[path = "field3d_shapes_tests.rs"]
mod tests;
