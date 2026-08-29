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
    /// ⭐ **Chapas** — um contorno 2D de **fórmula** puxado em Z. Vazia hoje; é o lote que traz o
    /// catálogo vetorial (estrela, cruz, coração, engrenagem…) para cá.
    Plates,
    /// O que sai de um **desenho** do editor vetorial.
    Drawn,
    /// O que vem de **fora** — uma escultura.
    Imported,
}

impl Family {
    pub(crate) const ALL: [Family; 6] = [
        Family::Blocks,
        Family::Round,
        Family::Rings,
        Family::Plates,
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
    /// O contorno escolhido no editor vetorial, puxado em Z.
    Extrude,
    /// O contorno escolhido, girado em torno de Y.
    Revolve,
    /// Uma escultura de um arquivo (abre diálogo).
    Sculpt,
    /// A escultura **viva** da cena, sem passar pelo disco.
    SculptScene,
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

/// ⭐ **O arredondamento com que uma forma nasce.**
///
/// ⚠️ **Não é zero, e é de propósito:** este é o módulo cujo argumento é o arredondamento, e uma
/// caixa de aresta viva ao nascer esconderia exatamente aquilo que ele faz melhor do que o Blender.
/// É uma **fração do tamanho**, então cabe sempre.
fn round_of(r: f32) -> f32 {
    r * 0.1
}

fn a_box(r: f32) -> Primitive {
    Primitive::Box {
        half: [r; 3],
        round: round_of(r),
    }
}

fn a_sphere(r: f32) -> Primitive {
    Primitive::Sphere { radius: r }
}

fn a_cylinder(r: f32) -> Primitive {
    Primitive::Cylinder {
        radius: r,
        half_height: r * 1.2,
        round: round_of(r),
    }
}

fn a_torus(r: f32) -> Primitive {
    Primitive::Torus {
        major: r,
        minor: r * 0.35,
    }
}

/// ⭐⭐⭐ **A LISTA.** Acrescentar uma forma é acrescentar **uma linha** aqui.
///
/// ⚠️ **A ordem daqui é a ordem DENTRO do grupo** da paleta, e nada mais: nenhum consumidor lê a
/// posição para saber o que a forma é (ver [`Make`]).
pub(crate) const SHAPES: &[Shape] = &[
    Shape {
        key: "panel.model3d.add.box",
        family: Family::Blocks,
        make: Make::Formula(a_box),
    },
    Shape {
        key: "panel.model3d.add.sphere",
        family: Family::Round,
        make: Make::Formula(a_sphere),
    },
    Shape {
        key: "panel.model3d.add.cylinder",
        family: Family::Round,
        make: Make::Formula(a_cylinder),
    },
    Shape {
        key: "panel.model3d.add.torus",
        family: Family::Rings,
        make: Make::Formula(a_torus),
    },
    // ⭐⭐ **AS FORMAS DE PERFIL** (W53) — o desenho do editor vetorial vira peça.
    Shape {
        key: "panel.model3d.add.extrude",
        family: Family::Drawn,
        make: Make::Extrude,
    },
    Shape {
        key: "panel.model3d.add.revolve",
        family: Family::Drawn,
        make: Make::Revolve,
    },
    Shape {
        key: "panel.model3d.add.sculpt",
        family: Family::Imported,
        make: Make::Sculpt,
    },
    Shape {
        key: "panel.model3d.add.sculpt_scene",
        family: Family::Imported,
        make: Make::SculptScene,
    },
];

/// A primitiva que esta posição do catálogo cria, no tamanho do enquadramento.
///
/// ⚠️ `None` para as quatro que não saem de um raio — quem as trata é o braço próprio do
/// `AddShape`, e é o [`Make`] que o diz, nunca um número.
pub(crate) fn shape_at(slot: usize, r: f32) -> Option<Primitive> {
    match SHAPES.get(slot)?.make {
        Make::Formula(f) => Some(f(r)),
        Make::Extrude | Make::Revolve | Make::Sculpt | Make::SculptScene => None,
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
        Make::Formula(_) | Make::Sculpt => true,
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
