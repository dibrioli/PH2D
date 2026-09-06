//! ⭐⭐⭐ **OS ARTEFACTOS DERIVADOS DE UMA FORMA NESTE QUADRO** — irmão de [`super`] pelo tecto de
//! 700 LOC, e o corte é por RESPONSABILIDADE: ali mora o MOTOR de desenho; aqui, o vocabulário de
//! *"o que esta forma recebeu de fora neste quadro"*.
//!
//! Todos eles têm a mesma forma e a mesma razão: são **cozidos e memoizados na SHELL** (esta crate
//! não alcança a cena), chegam por quadro, e o `None`/vazio é sempre **desenho certo** — a cor de
//! recurso de um padrão, a silhueta da forma para uma camada não dilatada —, nunca uma desistência.

use ph2d_vec_scene::{VecPath, VecPathId};

use super::PatternTile;

/// ⭐⭐⭐ **A GEOMETRIA DILATADA de cada camada da pilha** (v22) — o *offset de CAD* de UM atributo.
///
/// A chave é `(forma, índice da camada NO DOCUMENTO)`, e o valor é o **caminho** dilatado.
///
/// ⭐ **O caminho, e não a tesselação, e a divisão é o ponto:** o que é caro é o OFFSET (85–440 µs
/// a encolher) e fica memoizado na shell; tesselar é `~0,13 µs` e corre aqui por quadro, como o de
/// qualquer forma. Entregar `BezPath`s prontos obrigaria a duplicar o que a [`PathTess`] já sabe
/// fazer (o desenho de preenchimento ≠ o de traço, o tracejado AJUSTADO ao comprimento) — e a
/// cópia envelheceria na primeira regra nova.
///
/// ⚠️ **Cozido e memoizado na SHELL**, como os ladrilhos e a arte de pincel ao lado, e pela mesma
/// razão medida: encolher uma silhueta custa `85–440 µs` (o sweep booleano) contra `~0,13 µs` de
/// tesselar a forma — **~1 000×** —, e o renderer não tem estado entre quadros onde guardar isso.
/// Crescer é barato (`~0,5 µs`, a dilatação de Minkowski), mas passa pelo mesmo memo: **uma porta**.
///
/// ⚠️ O índice é o do DOCUMENTO e não o da iteração filtrada — desarmar a camada `1` não pode fazer
/// a `3` receber a geometria da `2`.
pub type DilatedPaints = std::collections::BTreeMap<(VecPathId, usize), VecPath>;

/// ⭐⭐⭐ **OS ARTEFACTOS DERIVADOS DESTA FORMA NESTE QUADRO** — o ladrilho, a arte do pincel e a
/// geometria dilatada de cada camada.
///
/// ⚠️ **Um pacote e não quatro argumentos, e a razão é medida:** eles chegam SEMPRE juntos, são
/// todos resolvidos pelo id da FONTE, e a alternativa é uma assinatura de oito parâmetros — que é
/// onde o `clippy` desenha a linha e onde um chamador troca dois `None` de posição sem o
/// compilador reparar. ⛔ Silenciar o aviso seria armengo; o corte é por assunto.
///
/// [`Derived::NONE`] é o neutro, e é o que as rotas SEM arte passam — as mesmas que o censo
/// `the_artless_draw_routes_are_declared` enumera uma a uma.
#[derive(Clone, Copy, Default)]
pub(crate) struct Derived<'a> {
    pub tile: Option<&'a PatternTile>,
    pub stroke_tile: Option<&'a PatternTile>,
    pub brush_art: Option<&'a [ph2d_vec_scene::VecPath]>,
    pub dilated: Option<&'a DilatedPaints>,
}

impl Derived<'_> {
    /// Nenhuma arte — o neutro que as rotas declaradas passam.
    pub(crate) const NONE: Self = Self {
        tile: None,
        stroke_tile: None,
        brush_art: None,
        dilated: None,
    };
}
