#![forbid(unsafe_code)]
//! **O VETOR VESTE A PELE** — o 1.º cliente da [`ph2d_skeleton`] (estudo 42 item 5,
//! [doc 47](../../../docs/Vector%20Module/47_o_desenho_ganha_ossos.md)).
//!
//! A lei — *para onde vai um PONTO* — vive no módulo e não sabe o que é um caminho. O que vive
//! aqui é a única coisa que é do vetor: **o que é um ponto aqui**.
//!
//! # ⭐⭐⭐ O PESO É DO NÓ, e as três metades do vértice movem-se por ele
//!
//! Âncora, alça de entrada e alça de saída são **três pontos** e **um só peso**: o da ÂNCORA. O
//! artista pesa *um nó do desenho* — é o que ele vê, é o que ele agarra, e é o único sítio onde o
//! pincel de peso pode pousar uma mancha.
//!
//! ## ⛔⛔ Isto REVOGA o que este cabeçalho dizia até 2026-09-19
//!
//! Ele dizia *«cada um com os pesos da posição DELE … pesar o vértice inteiro pela âncora faria uma
//! alça que atravessa uma junta rodar com o osso errado»* — o `CubicWeight` do Rive. **Ordem do
//! dono, repetida duas vezes:** *«o algoritmo continua considerando pesos em alças e não apenas nos
//! pontos»*. A objecção acima fica **registada e não vencida**, e o preço dela está medido no gate
//! [`tests::uma_alca_move_se_pelo_peso_da_ancora_dela`].
//!
//! ⚠️ **E a medição deu-lhe razão por um motivo que a objecção não via:** com um peso por metade, a
//! mancha do pincel é um BUMP no espaço — ela vale `1` no centro (a âncora) e menos nas alças, que
//! estão ao lado. Medido na arte do próprio dono (a barra de `PH2D_VEC_BONE_SMOKE=1`, um dab de
//! `amount = 1` num nó): a âncora fica em `0,5000`, a alça de saída em `0,5000` e a **alça de
//! entrada em `0,2150`** — `43 %` do que o artista acabou de dar ao nó. ⭐⭐ **E a assimetria é o
//! mais duro:** das duas alças do MESMO nó uma seguia e a outra não, o que parte a tangente
//! exactamente no ponto pintado. *Um peso que o artista não consegue entregar ao nó inteiro num
//! gesto não é um peso que ele controla.* O gate que o mede é o
//! `ph2d_skeleton_live::peso_a_mao_indicador_tests::um_dab_chega_inteiro_as_duas_alcas_do_no`.
//!
//! ⭐ E há uma propriedade que se ganha: a curva deixa de poder **cisalhar no nó**. Com três pesos
//! distintos as duas alças de um mesmo nó podem seguir ossos diferentes, e a tangente parte-se ali
//! — exactamente onde o desenho tem de ser liso.
//!
//! ⚠️ **Por que o LBS e não a `ph2d-vec-envelope`:** a mistura de afins ainda é um afim, então uma
//! Bézier deformada continua a ser uma Bézier — exacta e editável, sem reamostrar. O envelope paga
//! `sample + fit` porque o mapa dele não é afim.

/// ⭐⭐⭐ **Os pesos do PADRÃO-OURO para uma forma vectorial** — a 2.ª mídia a deixar a lei
/// euclidiana derivada.
pub mod pesos;

use ph2d_skeleton::Skin;
use ph2d_vec_scene::VecPath;

/// **Deforma o caminho inteiro, em lugar** — âncora e as duas alças de todo vértice de todo
/// contorno.
///
/// ⚠️ **O `corner_radius` viaja INTACTO.** Ele é fonte (o raio que o cozimento resolve), e a
/// deformação de uma pele é localmente quase-rígida — escalá-lo pediria um factor por VÉRTICE,
/// que é a mesma conta da caneta do bug #27 (`√|det|`) mas com um afim diferente por ponto.
/// Fica **nomeado**, não esquecido.
pub fn apply(skin: &Skin, path: &mut VecPath) {
    aplica_com(skin, path, &[]);
}

/// ⭐⭐⭐ **[`apply`] com os pesos do PADRÃO-OURO guardados no bind** — `pesos` vazio ⇒ a lei
/// derivada, que é o caminho de sempre **ao bit**.
///
/// A tabela é achatada e a ordem é a de [`VecPath::for_each_vert_mut`]: para o vértice `k`, as três
/// casas `3k`, `3k+1`, `3k+2` são **âncora**, **alça de entrada** e **alça de saída**.
///
/// ⚠️⚠️ **As três metades partilham o peso da ÂNCORA** (ordem do dono, 2026-09-19 — ver o
/// cabeçalho do módulo, que dizia o contrário até esse dia). ⇒ das `3k` linhas da tabela só as de
/// índice `3k` são LIDAS; as duas das alças continuam a ser gravadas pelo bind e ficam **dívida
/// nomeada** — apagá-las mudava o formato guardado em bytes opacos por uma economia que ninguém
/// mediu.
///
/// ⛔ **Uma tabela que não fecha com o caminho é IGNORADA** (cai na lei derivada) em vez de ser lida
/// deslocada: uma tabela deslocada por um ponto entrega pesos plausíveis e arte errada.
pub fn aplica_com(skin: &Skin, path: &mut VecPath, pesos: &[f64]) {
    aplica_corrigido(skin, path, pesos, &[]);
}

/// ⭐⭐⭐ **O MESMO, COM AS CORRECÇÕES À MÃO DO ARTISTA** — ver [`ph2d_skeleton::Correccao`].
///
/// ⛔ Com `correcoes` vazio ela é **byte-idêntica** ao [`aplica_com`], e é por isso que aquele
/// delega aqui em vez de duplicar o laço: *duas cópias do mesmo percurso divergem no primeiro
/// ajuste*, e este já se partiu uma vez quando o padrão-ouro chegou.
pub fn aplica_corrigido(
    skin: &Skin,
    path: &mut VecPath,
    pesos: &[f64],
    correcoes: &[ph2d_skeleton::Correccao],
) {
    let mut w = skin.scratch();
    let pontos = path.verts_all().count() * 3;
    let n = pesos.len().checked_div(pontos).unwrap_or(0);
    let usa = n > 0 && pesos.len() == pontos * n;
    let mut k = 0usize;
    path.for_each_vert_mut(|v| {
        // ⭐⭐⭐ **UMA conta de peso por VÉRTICE, feita na ÂNCORA** — e as três metades movem-se por
        // ela. É a lei do cabeçalho escrita na forma que este laço tem: *um vértice, um peso*.
        //
        // ⚠️ **A escolha da lei (derivada ou padrão-ouro) é um `Option`, e a porta é UMA** — ver
        // [`ph2d_skeleton::Skin::weights_corrected`]. Ela colapsa o `if` que cada consumidor tinha
        // de escrever, e é o que faz a correcção valer nas duas sem ser escrita duas vezes.
        //
        // ⚠️ **`k` é o índice ACHATADO da âncora** (`3 ×` o número do vértice), que é a linha da
        // tabela do bind que se lê — a mesma que o [`dono_do_peso`] devolve para as três metades.
        let guardados = usa.then(|| &pesos[k * n..(k + 1) * n]);
        skin.weights_corrected(v.anchor, guardados, &mut w, correcoes);
        for p in [&mut v.anchor, &mut v.in_handle, &mut v.out_handle] {
            *p = skin.blend(*p, &w);
        }
        k += 3;
    });
}

/// ⭐⭐⭐ **DE QUEM É O PESO DESTE PONTO** — a lei do módulo, para quem percorre a lista ACHATADA.
///
/// Na ordem de [`ph2d_vec_scene::VecPath::for_each_vert_mut`] os índices `3k`, `3k+1` e `3k+2` são
/// a âncora e as duas alças do vértice `k`; o peso das três é o da **âncora**, logo `3k`.
///
/// ⚠️ **Ela existe para a lei não ser escrita duas vezes.** O [`aplica_corrigido`] percorre por
/// VÉRTICE e exprime-a na forma dele (uma conta, três misturas); quem percorre ponto a ponto — o
/// indicador do pincel de peso e o instantâneo posado que o hit-test usa — precisa da mesma
/// resposta a partir de um índice qualquer. *Duas cópias divergiam no primeiro ajuste, e o sintoma
/// seria o indicador a mostrar um peso que a arte não tem.*
#[must_use]
pub const fn dono_do_peso(k: usize) -> usize {
    k - k % 3
}

/// **Este ponto da lista achatada é um NÓ?** — ver [`dono_do_peso`].
///
/// ⚠️ **Derivada dela e não uma segunda conta:** *«é um nó»* e *«de quem é o peso»* são a mesma
/// pergunta, e escrever `k % 3 == 0` ao lado seria a segunda resposta que envelhece sozinha.
#[must_use]
pub const fn e_no(k: usize) -> bool {
    dono_do_peso(k) == k
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;

#[cfg(test)]
#[path = "sonda_da_pele_como_warp_tests.rs"]
mod sonda_da_pele_como_warp_tests;
