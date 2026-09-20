//! ⭐⭐⭐ **O RETRATO DO CANAL QUE UM VERBO ESCREVE** — a porta que os censos
//! desta crate usam para perguntar *«este dab mudou alguma coisa?»*.
//!
//! ⛔⛔ **Ela existe porque a pergunta estava escrita em TRÊS harnesses, cada um
//! com a sua cópia de `if paints_mask() { máscara } else { posições }`** — e o
//! doc do [`Verb::paints_mask`] já tinha previsto por escrito o dia em que isso
//! partisse: *«duas listas divergiriam no dia em que entrar o segundo verbo de
//! canal (Paint, na W7)»*. O dia foi 2026-09-19, e os três censos acusaram o
//! [`Verb::Paint`] com a frase **certa sobre a régua deles e errada sobre o
//! produto**: *«o dab não fez nada em canal nenhum»*. Ele fazia — na COR, que
//! nenhum deles olhava.
//!
//! ⚠️ **O retrato é um `Vec<f32>` achatado de propósito:** os três censos
//! comparam duas corridas entre si, e o que eles precisam é de *alguma coisa
//! diferiu*, não do tipo do canal. Devolver três tipos obrigaria cada chamador
//! a voltar a ramificar — que é exactamente o defeito que esta porta apaga.

#[cfg(test)]
use crate::{Brush, Verb};
use ph2d_mesh::Mesh;

/// **O canal que este verbo escreve, achatado.**
///
/// ⚠️ **A ausência de um canal `Option` vem como o DEFAULT dele e não vazia:**
/// uma malha que ninguém pintou tem cor (branca) tanto quanto uma esfera tem
/// posições, e devolver vazio faria a comparação ser entre dois nadas —
/// verdadeira por construção, que é a tautologia que esta casa já pagou.
#[must_use]
#[cfg(test)]
pub(crate) fn retrato_do_canal(mesh: &Mesh, verb: Verb) -> Vec<f32> {
    if verb.paints_mask() {
        return mesh.masks().map_or_else(
            || vec![ph2d_mesh::DEFAULT_MASK; mesh.vert_count()],
            <[f32]>::to_vec,
        );
    }
    let fonte: Vec<[f32; 3]> = if verb.paints_color() {
        mesh.colors().map_or_else(
            || vec![ph2d_mesh::DEFAULT_COLOR; mesh.vert_count()],
            <[[f32; 3]]>::to_vec,
        )
    } else {
        mesh.positions().to_vec()
    };
    fonte.into_iter().flatten().collect()
}

/// **O maior desvio entre dois retratos** — `0` quer dizer *nada mudou*.
///
/// ⚠️ **Comprimentos diferentes contam como MUDANÇA e não como erro:** um passe
/// de topologia a meio de um traço muda a contagem de vértices, e ali a
/// resposta honesta é *«mudou»*.
#[must_use]
#[cfg(test)]
pub(crate) fn desvio(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() {
        return f32::INFINITY;
    }
    a.iter()
        .zip(b)
        .fold(0.0f32, |m, (x, y)| m.max((x - y).abs()))
}

/// **O pincel deste verbo com o canal dele em condições de mudar** — o que um
/// censo precisa para o dab não ser inerte por configuração.
///
/// ⚠️ **A COR do pincel tem de ser DIFERENTE da cor de repouso da malha**, e
/// esta é a armadilha que o [`Verb::Paint`] traz de novo: o `DEFAULT_COLOR` é
/// **branco** e um pincel branco sobre barro branco escreve um no-op perfeito.
/// Um censo que não troque a cor lê *«o dab não fez nada»* sobre um pincel
/// impecável — a forma de que este ficheiro é a cura, repetida um nível abaixo.
#[must_use]
#[cfg(test)]
pub(crate) fn pincel_com_canal_vivo(mut brush: Brush) -> Brush {
    if brush.verb.paints_color() {
        brush.color = [0.0, 0.0, 0.0];
    }
    brush
}

/// **A PEÇA com cor a VARIAR** — o que os verbos que leem o ANEL precisam para
/// não serem inertes por construção.
///
/// ⛔⛔ **O pincel preto não chega para eles, e a diferença é o que esta função
/// existe para dizer:** a pintura deposita uma cor e basta que ela difira do
/// barro; o [`Verb::Blur`] e o [`Verb::SmearColor`] leem a VIZINHANÇA, e numa
/// peça toda branca a média da vizinhança **é branca** — eles devolvem o que já
/// lá estava, ao bit. Um censo corrido assim acusa-os de mortos com toda a
/// razão sobre a fixtura e nenhuma sobre o produto.
///
/// ⚠️ **O padrão é uma faixa pelo `x`**, e não ruído: uma fronteira nítida é o
/// que uma média do anel tem de esbater e o que um transporte tem de mover, e
/// ela existe em toda peça deste repo (todas atravessam a origem).
pub fn semeia_cor(mesh: &mut Mesh) {
    let faixa: Vec<[f32; 3]> = mesh
        .positions()
        .iter()
        .map(|p| {
            if p[0] < 0.0 {
                [1.0, 0.0, 0.0]
            } else {
                [0.0, 0.0, 1.0]
            }
        })
        .collect();
    mesh.colors_mut().copy_from_slice(&faixa);
}
