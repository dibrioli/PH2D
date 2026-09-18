//! ⭐⭐⭐ **O COLISOR QUE UM OBJECTO DA CENA DECLARA** — doc 115 W4, ordem do dono (2026-09-17):
//! *«tirar o collide e deixar tudo pela Shape e pelos outros objetos (como vector, Sprite, Flip)
//! que serão criados com seus próprios colliders (sem usar o grafo)»*.
//!
//! # A lei, numa linha: a caixa do objecto é o QUADRADO UNITÁRIO
//!
//! ⛔⛔⛔ **E a §10.1 do doc 115 escreveu-a errada — `ph2d_collider_box = size / 2` — no sítio
//! exacto que decide o número.** A porta que lê a declaração
//! ([`ph2d_contact::declarado`]) converte de GEOMETRIA para MUNDO multiplicando as meias pelo
//! `size` da própria linha, e a corrente da aparência **já traz** esse `size`:
//!
//! ```text
//!   meia_mundo = meia_declarada · |size|
//! ```
//!
//! ⇒ declarar `size / 2` daria `size² / 2`: certo em `size = 1`, o **dobro** em `size = 2`, e
//! **metade** em `size = 0,5`. *Um erro que a fixtura mais natural do mundo — o objecto de tamanho
//! um — não consegue ver.* Quem o apanhou foi ler a porta em vez de a supor
//! (⚠️ e a §10.1 dizia-se MEDIDA: ela mediu que o `size` viaja, e não o que o leitor lhe faz).
//!
//! ⇒ a declaração certa é [`MEIA_DO_OBJECTO`] = `[0,5, 0,5]`, **constante**, porque a aparência de
//! um objecto é um quadro que ocupa exactamente `size` e está centrado no `P` dele. O `size` da
//! linha faz o resto, e faz mais do que isto sozinho saberia fazer: um `motion.scale` a jusante
//! encolhe a peça **e** o colisor dela, por construção.
//!
//! ⭐ E isto é mais barato do que o plano previa: não são *«três colunas derivadas»*, é **UMA
//! coluna constante**.
//!
//! # As três ausências, e cada uma é uma LEI e não uma omissão
//!
//! - **Sem [`COLLIDER_OFFSET_COLUMN`].** O quadro de um objecto é centrado no `P` (o
//!   `SinkStyle::anchor_for` com o pivô de omissão dá `[0, 0]`), logo o centro do colisor **é** o
//!   `P`. É a lei estrutural que o `collider.rs` da forma já escreve: *a arte centrada declara,
//!   pela ausência, o que toda declaração anterior a esta coluna já queria dizer*.
//! - **Sem [`COLLIDER_COLUMN`]** (o raio). Um objecto é um quadro; um disco seria uma segunda
//!   resposta à mesma pergunta, e a porta da leitura já declara que a caixa ganha.
//! - **Sem material** (`friction`/`bounce`/`rolling`) e **sem** `inv_inertia`. A forma declara-os
//!   porque o cartão dela os autora; um objecto **não tem cartão**, e escrever um default aqui
//!   seria autorar em nome do artista — a ausência é que quer dizer *«não declarei material
//!   nenhum»*.
//!
//! # ⚠️ O que esta declaração NÃO sabe, nomeado
//!
//! - **É a caixa do QUADRO, não da tinta.** Uma sprite com margem transparente declara a margem
//!   junto com a arte. A forma tem uma caixa mais apertada (`collider_fit_half`, a caixa
//!   envolvente da GEOMETRIA); um objecto raster não tem geometria para apertar, e derivar a caixa
//!   da alfa é uma leitura de textura por quadro que ninguém mediu.
//! - **Uma folha vectorial ou Flip de um grupo é assada na orientação de MUNDO dela** e viaja com
//!   `rot = 0` (o limite v1 do médio, já declarado no [`super::leaf_from_object`]) ⇒ a caixa dela é
//!   alinhada aos eixos enquanto a arte pode estar rodada.
//! - **O `pivot` do sink desloca o quadro DESENHADO e não o `P` separado.** Pré-existente e
//!   partilhado com a forma: a separação acontece a montante do lowering, que é onde o pivô entra.

use ph2d_nodegraph::attr::{COLLIDER_BOX_COLUMN, Column, Stream};

/// **As meias extensões do colisor de um objecto, em unidades de GEOMETRIA.**
///
/// ⚠️ `0,5` e não `1`: a aparência de um objecto ocupa `size` (o quadro vai de `−size/2` a
/// `+size/2` à volta do `P`), ao contrário da geometria de uma `source.shape`, que vive em **raio
/// 1** e por isso declara meias da ordem de `1`. As duas passam pela mesma porta e chegam ao mesmo
/// sítio; o que difere é a convenção de cada médio, que é precisamente por isso que a declaração
/// nasce em quem DESENHA.
pub(super) const MEIA_DO_OBJECTO: [f32; 2] = [0.5, 0.5];

/// **A PORTA: a corrente da aparência de um objecto, com a forma dele declarada.**
///
/// ⚠️ **Uma porta e não três `.with` iguais.** Os três construtores da aparência — a sprite/ladrilho
/// ([`super::streams::appearance_tile`]), o vector vivo ([`super::streams::appearance_vector`]) e o
/// grupo ([`super::group_stream`]) — respondem à mesma pergunta para médios diferentes, e a lei
/// escrita três vezes divergiria no dia do quarto médio. *Uma lei escrita em dois sítios ainda não
/// é uma lei; só uma porta é.*
///
/// Uma corrente vazia sai como entrou: uma coluna de zero linhas não é uma declaração, e o leitor
/// exige que todo comprimento case com o da nuvem.
pub(super) fn com_colisor(s: Stream) -> Stream {
    let n = s.count();
    if n == 0 {
        return s;
    }
    s.with(COLLIDER_BOX_COLUMN, Column::Vec2(vec![MEIA_DO_OBJECTO; n]))
}

#[cfg(test)]
#[path = "motion_bridge_objects_collider_tests.rs"]
mod tests;
