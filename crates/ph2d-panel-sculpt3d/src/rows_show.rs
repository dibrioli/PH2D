//! **AS PERGUNTAS QUE UMA FILEIRA FAZ** — os predicados de `show` que mais de
//! uma tabela partilha.
//!
//! ⚠️ **O corte é por RESPONSABILIDADE e foi forçado pelo tecto de LOC**
//! (2026-09-15, `rows.rs` a `606` contra `600`): o pai declara **que fileiras
//! existem e o que cada uma escreve**; aqui mora **quando cada uma aparece**. ⛔
//! Uma entrada no `FILE_OVERAGE_OK` não era saída — o `CLAUDE.md` §5.0 declara
//! que a cura de um tecto é o corte.
//!
//! ⚠️ Eles continuam `pub(super)` e os módulos-irmãos continuam a escrever
//! `use super::always;`, porque o pai os **re-exporta**: mover a lei não pode
//! mudar o endereço por onde as tabelas lhe chamam.

use crate::state::Sculpt3dUi;

/// Sempre visível. ⚠️ `pub(super)` porque a tabela do sombreamento é um
/// módulo FILHO e as duas a partilham — duas cópias divergiriam no dia em que
/// *sempre* ganhasse uma exceção.
pub(crate) fn always(_: &Sculpt3dUi) -> bool {
    true
}

/// **O RAIO CHEGA AO BARRO COM ESTE PINCEL EM MÃOS?** — a porta da pista do
/// raio.
///
/// ⚠️ **A resposta vem do MOTOR** ([`ph2d_sculpt3d::Verb::o_raio_chega_ao_barro`]),
/// nunca de um `match` aqui: uma segunda cópia da condição divergiria na
/// primeira wave que mexesse numa delas, e a que o artista vê é a que envelhece.
pub(crate) fn tem_raio(u: &Sculpt3dUi) -> bool {
    u.brush.verb.o_raio_chega_ao_barro()
}

/// **ESTE PINCEL LÊ O CAMINHO DA MÃO?** — a porta da pista de suavização do
/// traço, que só o LAÇO do Box Trim usa.
///
/// ⛔ **DUAS metades, e nenhuma basta:** o verbo tem de ser o corte (com um
/// pincel na mão não há traço de ecrã nenhum) **e** a forma tem de ser a que
/// guarda um caminho — a caixa e o círculo saem de dois pontos, e suavizar dois
/// pontos é o controlo morto que esta casa varre a cada wave.
pub(crate) fn suaviza_o_traco(u: &Sculpt3dUi) -> bool {
    u.brush.verb == ph2d_sculpt3d::Verb::BoxTrim && u.brush.trim_forma.le_o_caminho()
}

/// **O DAB LÊ A DISTÂNCIA COM ESTE PINCEL EM MÃOS?** — a porta da row de
/// [`Dureza`](BRUSH), e a MESMA que a largura do campo já segue
/// (`RefMode::field`).
///
/// ⚠️ **A dureza é a etapa da referência que remapeia a distância que a curva
/// do dab lê.** Com um campo elástico armado (`RefMode::L` + um verbo que
/// declare `elastic_field`) não existe curva do dab — o suporte é o
/// `kelvinlet::rim_landing`, que é uma indicadora com aterrissagem, e o
/// `shaped_distance` **não é chamado**. Medido pela porta do produto em
/// `ph2d-sculpt3d/tests/it/measure_where_the_curve_knobs_reach.rs`
/// (`neither_curve_knob_reaches_an_elastic_field`): dois valores de dureza dão o
/// mesmo barro **ao bit**, e o MESMO verbo no `s-mode` os separa.
///
/// ⚠️ **A pergunta é feita à porta do MOTOR, nunca a uma lista de verbos aqui** —
/// o `stroke_dab_core` escolhe o regime exatamente onde essa porta devolve
/// `Some`, e uma segunda cópia da pergunta é como a próxima nasce desalinhada.
/// É a mesma frase que o `paint_elastic_scales_row` já carrega.
///
/// ⚠️ **E ela vale só para a DUREZA, não para o Falloff ao lado.** O seletor de
/// curva é inerte nos mesmos regimes (e num terceiro, o do `Verb::Mask`) e é
/// pintado assim mesmo, por uma cerca com motivo escrito:
/// `the_basic_level_never_hides_the_curve_that_shapes_the_dab` porta a decisão do
/// Blender — *o painel de queda é dobrado, nunca ausente*. Ver o bloco no
/// `paint/brush.rs`, que traz as duas recusas medidas.
pub(crate) fn shapes_the_distance(u: &Sculpt3dUi) -> bool {
    // ⚠️⚠️ **DUAS metades, e a segunda chegou pelo CENSO DOS KNOBS** (2026-09-15).
    // A primeira pergunta ao MODO (sob um campo elástico a curva inteira é o
    // perfil do campo, e o dab não lê distância nenhuma); a segunda pergunta ao
    // VERBO, porque há três cuja LEI não tem distância para remapear —
    // [`ph2d_sculpt3d::Verb::a_lei_le_a_distancia_ao_cursor`], com a fonte de
    // cada um lá dentro.
    //
    // ⛔ **A lente do painel era mais larga que a do consumidor**, que é a forma
    // que o `CLAUDE.md` §5.0 nomeia — e, como ali, *a regra certa já estava
    // escrita no mesmo ficheiro, para o mesmo controlo*: faltava-lhe o segundo
    // lado. O censo mediu o `Verb::Pose` a arrastar a dureza de `0,00` a `0,95`
    // com desvio `0,000e0` no barro.
    u.brush.verb.a_lei_le_a_distancia_ao_cursor() && u.brush.mode.field(u.brush.verb).is_none()
}
