//! ⭐⭐⭐ **UM PONTO NOVO NUMA FORMA PRESA** — a caneta acrescenta controlo onde o artista quer, e
//! ele SOBREVIVE ao quadro.
//!
//! # ⛔⛔⛔ Por que isto não era «zero de arquitectura»
//!
//! A F26 deixou ao dono duas saídas para *«pintar peso entre os vértices de uma forma vectorial»* e
//! escreveu sobre a primeira que o gesto já existia. ⚠️ **MEDIDO em 2026-09-19 e falso:** a caneta
//! escreve no documento VIVO, e o [`crate::skin_live::recook`] reconstrói esse documento a partir da
//! geometria **autorada** que o bind guardou — uma vez por quadro. *O ponto aparece sob o dedo e
//! desaparece sozinho*, sem erro, sem aviso e sem recusa.
//!
//! ⛔ **E o contorno óbvio — «acrescente o ponto e carregue em *Bind* outra vez» — custa o trabalho
//! do artista:** o [`ph2d_skeleton_ecs::SkinBind::new`] nasce com `correcoes: vazio` e
//! `law: Auto`, logo um re-bind deita fora **todas as correcções pintadas à mão** e a escolha de lei
//! daquele desenho.
//!
//! # ⭐⭐ A lei: a fonte é que ganha o ponto, e a tabela cresce com ele
//!
//! O ponto entra na **geometria autorada** (o `SkinBind::source`), no mesmo segmento e no mesmo
//! parâmetro em que a mão o pediu, pelo mesmo [`ph2d_vec_scene::split_segment`] de sempre. O quadro
//! seguinte re-deriva o desenho dali — *não é preciso escrever no documento vivo, e escrever seria a
//! segunda resposta à mesma pergunta*.
//!
//! ⚠️ **A linha de pesos do nó novo é a MISTURA das dos dois vizinhos, no mesmo `t`** — e não a lei
//! automática. ⛔ A tabela guardada vem do padrão-ouro (uma resolução global sobre a malha do
//! domínio); pedir a lei derivada só para este nó poria **um ponto a obedecer a outra lei** no meio
//! de uma forma, que é um vinco onde o artista pediu controlo. E re-resolver o global mudaria o peso
//! de **todos** os outros nós, apagando a linha de base que ele corrigiu.
//!
//! ⭐ A mistura é uma combinação **convexa** de duas partições da unidade, logo ela é uma partição da
//! unidade — não há normalização a fazer, e há gate a afirmá-lo.
//!
//! # ⚠️ A forma desenhada move-se um pouco — e o salto é REFINAMENTO, medido
//!
//! O desenho cozido é a Bézier dos pontos de controlo **deformados**, e não a imagem verdadeira da
//! curva de repouso pela pele (que é `t ↦ blend(repouso(t), peso(t))`, com o peso a variar ao longo
//! do segmento). ⇒ *ele já é uma aproximação, e cada pedaço a mais refina-a.*
//!
//! ⭐⭐⭐ **A escada da subdivisão prova-o** (`a_escada_da_subdivisao_diz_se_o_salto_e_refinamento`):
//! cortando o mesmo segmento `1 → 2 → 4 → 8` vezes, o desvio entre degraus cai
//! **`18,89 % → 3,13 % → 1,00 %`** da peça. Uma sequência que converge geometricamente não está a
//! corromper nada — está a aproximar-se do limite.
//!
//! | fixtura | salto ao acrescentar um ponto |
//! |---|---|
//! | esqueleto em **REPOUSO** | **`0` ao bit** — a pose é a identidade, e o corte é o de sempre |
//! | aresta **CRUA** (um segmento a atravessar os dois ossos) | **`18,89 %`** da peça |
//! | aresta **DESENHADA** em 8, pior segmento (o da junta) | **`0,91 %`** da peça |
//!
//! ⚠️ **Os `18,89 %` não são o custo de acrescentar um ponto — são o tamanho do erro que aquele
//! único segmento já tinha, e o corte mostra-o.** Numa forma desenhada com pontos, que é o que um
//! artista faz para ter controlo, o pior salto é `0,91 %`. ⭐ E quando as duas pontas de um segmento
//! têm o mesmo peso — o caso comum longe das juntas — a forma é preservada **ao bit**.

use ph2d_ecs::{Entity, SimWorld};
use ph2d_skeleton_ecs::SkinBind;
use ph2d_vec_entities::entities::VecEntityMap;
use ph2d_vec_scene::VecPathId;

/// ⭐⭐⭐ **Acrescenta um ponto ao caminho PRESO `id`**, no segmento `seg` e no parâmetro `t`.
///
/// Devolve o índice plano do vértice novo, ou `None` quando esta forma **não está presa** — e aí
/// quem chama faz o que sempre fez, que é inserir no documento vivo.
///
/// ⚠️ **Ela não toca na cena**: quem re-deriva o desenho é o `recook` do quadro. *Escrever aqui
/// também seria a segunda resposta à mesma pergunta, e a que o artista vê é a que envelhece.*
pub fn insere_ponto(
    sim: &mut SimWorld,
    map: &VecEntityMap,
    id: VecPathId,
    seg: usize,
    t: f64,
) -> Option<usize> {
    let e = Entity::from_bits(*map.get(&id)?);
    let skin = sim.world().get::<SkinBind>(e)?.clone();
    let mut fonte = crate::skinned_mesh::le(&skin.source)?;
    let (ni, bytes) = insere_na_fonte(&mut fonte, seg, t)?;
    sim.world_mut().get_mut::<SkinBind>(e)?.source = bytes;
    Some(ni)
}

/// ⭐⭐ **A metade PURA** — a fonte guardada com um ponto a mais e a tabela a fechar.
///
/// ⚠️ **Ela é separada de propósito:** a lei mede-se sem um mundo ECS, e é aqui que a aritmética da
/// tabela é gateada. ⛔ `None` deixa a fonte **intocada** — meia fonte é pior do que a antiga.
#[must_use]
pub fn insere_na_fonte(
    fonte: &mut crate::skinned_mesh::SkinnedPath,
    seg: usize,
    t: f64,
) -> Option<(usize, Vec<u8>)> {
    let ossos = fonte.ossos();
    // ⭐ Uma tabela VAZIA é uma resposta: um caminho aberto não tem domínio, logo o desenho corre na
    // lei derivada e não há linha nenhuma para crescer. Ali o ponto novo entra e mais nada.
    let tabela = (ossos > 0 && fonte.valida()).then(|| fonte.pesos.clone());
    let ni = ph2d_vec_scene::split_segment(&mut fonte.path, seg, t)?;
    if let Some(antes) = tabela {
        let nova = linha_do_ponto_novo(&fonte.path, &antes, ossos, ni, t)?;
        // ⚠️ **TRÊS linhas e não uma**: a tabela é por ponto de CONTROLO (âncora · alça de entrada ·
        // alça de saída), e só a da âncora é lida desde 2026-09-19. As duas das alças são gravadas
        // com o mesmo conteúdo — *elas são amostras, nunca incógnitas* — porque a forma da tabela
        // viaja em bytes opacos e encolhê-la mudaria o que já está guardado.
        let em = ni * 3 * ossos;
        let mut tres = Vec::with_capacity(3 * ossos);
        for _ in 0..3 {
            tres.extend_from_slice(&nova);
        }
        fonte.pesos.splice(em..em, tres);
        // ⛔⛔ **Aqui esteve um `if !fonte.valida() { return None }`, e ele SAIU por uma mutação que
        // sobreviveu.** Depois de um splice correcto a tabela fecha **por construção**, logo aquele
        // ramo era inalcançável: *uma linha que a mutação não consegue matar não é lei, é comentário
        // com sintaxe de código*. E a cerca que importa já existe a jusante — o `recook` recusa uma
        // tabela que não feche e cai na lei derivada, que é onde ela tem de estar (lá ela defende-se
        // de uma fonte GRAVADA por outra versão, que é o caso real).
    }
    let bytes = crate::skinned_mesh::grava(fonte)?;
    Some((ni, bytes))
}

/// A linha de pesos do nó `ni` do caminho JÁ partido, misturada das dos dois vizinhos dele.
///
/// ⚠️ **Os vizinhos leem-se do caminho NOVO e as linhas da tabela VELHA**, e a tradução entre os dois
/// índices é uma subtracção: inserir em `ni` empurra para a frente tudo o que estava em `ni` ou
/// depois. *Ler a tabela velha com um índice novo desloca a forma inteira em silêncio.*
fn linha_do_ponto_novo(
    path: &ph2d_vec_scene::VecPath,
    antes: &[f64],
    ossos: usize,
    ni: usize,
    t: f64,
) -> Option<Vec<f64>> {
    let (c, local) = path.locate_vert(ni)?;
    let (verts, fechado) = path.contour(c)?;
    let n = verts.len();
    // ⛔ Num contorno ABERTO um vértice de ponta não tem os dois vizinhos, e o `split_segment` nunca
    // o produz — ele insere sempre ENTRE duas âncoras. A guarda existe para a terceira chamada.
    let anterior = if local > 0 {
        local - 1
    } else if fechado {
        n - 1
    } else {
        return None;
    };
    let seguinte = if local + 1 < n {
        local + 1
    } else if fechado {
        0
    } else {
        return None;
    };
    let velho = |f: usize| if f < ni { f } else { f - 1 };
    let a = velho(path.flat_vert(c, anterior)?);
    let b = velho(path.flat_vert(c, seguinte)?);
    let ra = antes.get(a * 3 * ossos..a * 3 * ossos + ossos)?;
    let rb = antes.get(b * 3 * ossos..b * 3 * ossos + ossos)?;
    let t = t.clamp(0.0, 1.0);
    Some(
        ra.iter()
            .zip(rb)
            .map(|(x, y)| x + (y - x) * t)
            .collect::<Vec<f64>>(),
    )
}

#[cfg(test)]
#[path = "ponto_novo_tests.rs"]
mod tests;
