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
    // ⭐⭐⭐ **A PELE DE AGORA entra na conta, e é ela que faz o desenho não saltar.** Ver
    // [`insere_na_fonte`]. ⛔ `None` (pele que não resolve) cai no corte de repouso de sempre, que
    // é a resposta certa quando não há deformação nenhuma para preservar.
    let pele = crate::skin_live::resolve(sim, &skin, e, &crate::skin_live::bone_index(sim));
    let correcoes = skin.correcoes_resolvidas();
    let (ni, bytes) = insere_na_fonte(&mut fonte, seg, t, pele.as_ref(), &correcoes, PASSAGENS)?;
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
    pele: Option<&ph2d_skeleton::Skin>,
    correcoes: &[ph2d_skeleton::Correccao],
    passagens: usize,
) -> Option<(usize, Vec<u8>)> {
    // ⭐⭐⭐ **O ALVO é o corte do DESENHO, e ele lê-se ANTES de a geometria mudar.**
    let alvo = pele.and_then(|k| alvo_cozido(fonte, k, correcoes, seg, t));
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
    // ⭐⭐⭐ **E O DESENHO NÃO SALTA** — ver [`compensa`]. Sem alvo (pele que não resolve) fica o
    // corte de repouso de sempre.
    if let Some(alvo) = alvo {
        compensa(fonte, pele?, correcoes, ni, &alvo, passagens);
    }
    let bytes = crate::skinned_mesh::grava(fonte)?;
    Some((ni, bytes))
}

/// Os TRÊS pontos de controlo que o corte do DESENHO produz — `(entrada, âncora, saída)` do vértice
/// novo, em coordenadas do desenho deformado.
///
/// ⚠️ **Os dois vizinhos não entram na conta e é por isso que a lei é barata:** o `out` do anterior
/// e o `in` do seguinte já saem certos **ao bit**. A deformação com um peso FIXO é um afim, e os
/// dois são combinações afins de pontos que usam esse mesmo peso — logo o corte comuta com ela ali.
/// *Só o vértice do meio é que mistura os pesos das duas pontas, e é só ele que precisa de conta.*
fn alvo_cozido(
    fonte: &crate::skinned_mesh::SkinnedPath,
    pele: &ph2d_skeleton::Skin,
    correcoes: &[ph2d_skeleton::Correccao],
    seg: usize,
    t: f64,
) -> Option<[[f64; 2]; 3]> {
    let (c, local) = fonte.path.locate_segment(seg)?;
    let (verts, _fechado) = fonte.path.contour(c)?;
    let n = verts.len();
    let (a, b) = (local, (local + 1) % n);
    let (fa, fb) = (fonte.path.flat_vert(c, a)?, fonte.path.flat_vert(c, b)?);
    let wa = pesos_do_no(fonte, pele, correcoes, fa, verts[a].anchor);
    let wb = pesos_do_no(fonte, pele, correcoes, fb, verts[b].anchor);
    let p = [
        pele.blend(verts[a].anchor, &wa),
        pele.blend(verts[a].out_handle, &wa),
        pele.blend(verts[b].in_handle, &wb),
        pele.blend(verts[b].anchor, &wb),
    ];
    let t = t.clamp(0.0, 1.0);
    let q0 = lerp(p[0], p[1], t);
    let q1 = lerp(p[1], p[2], t);
    let q2 = lerp(p[2], p[3], t);
    let r0 = lerp(q0, q1, t);
    let r1 = lerp(q1, q2, t);
    Some([r0, lerp(r0, r1, t), r1])
}

/// ⭐⭐⭐ **MOVE OS TRÊS PONTOS DE REPOUSO do vértice novo para que o DESENHO fique onde estava.**
///
/// ⛔⛔⛔ **Report do dono, 2026-09-19: *«o ponto criado na malha já conectada aos ossos deforma a
/// malha»*.** O corte de de Casteljau na geometria de REPOUSO preserva a curva de repouso e **não** a
/// desenhada: os quatro pontos de controlo de um segmento desenhado saem de **duas** linhas de pesos
/// diferentes, e o corte só comuta com a deformação quando as duas são iguais. ⚠️ *Eu tinha medido
/// isso, chamado-lhe refinamento e dito ao dono que era normal — ele recusou, e a régua dele é a que
/// manda: o desenho é o que o artista vê.*
///
/// ⭐⭐ **A conta é uma INVERSÃO, e ela existe porque a deformação com um peso FIXO é um AFIM.** O
/// `recook` lê **uma** linha de pesos — a da âncora — e aplica-a às três metades do vértice; com essa
/// linha fixa, `x ↦ blend(x, w)` é `L·x + c`. ⇒ o ponto de repouso que desenha em `X` é `L⁻¹(X − c)`,
/// e `L` e `c` lêem-se com **três** avaliações da própria porta, sem uma segunda cópia da lei.
///
/// ⭐ **Em REPOUSO ela é a IDENTIDADE ao bit** — ali toda pose é a identidade, logo `L = I`, `c = 0`,
/// e o resultado é exactamente o corte de repouso de sempre. *A compensação só existe onde há
/// deformação para preservar.*
///
/// ⚠️ **DUAS passagens, e a segunda não é zelo:** a linha de pesos do vértice depende da posição dele
/// (o `quota` de um osso que dobra e as manchas pintadas à mão), e mover a âncora muda-a. A primeira
/// passagem resolve com a âncora provisória; a segunda repete com a âncora já movida. ⛔ Sem a
/// segunda, uma forma com manchas fica com um resíduo que ninguém explica.
///
/// ⛔ **Uma pose SINGULAR (um osso achatado a zero) não se inverte**, e ali a compensação é saltada —
/// fica o corte de repouso. *Um `NaN` na arte é pior do que um ponto que se mexe.*
/// ⭐⭐ **QUANTAS PASSAGENS a compensação faz**, e o número saiu da escada MEDIDA.
///
/// ⛔⛔ **O número não é escolhido: a escada foi MEDIDA** sobre uma forma com mancha pintada, onde o
/// peso depende da posição e a convergência é visível (`ph2d_skeleton_live::ponto_novo`, o gate
/// `a_segunda_passagem_e_exigida_por_uma_mancha`):
///
/// | passagens | quanto o desenho ainda salta |
/// |---:|---:|
/// | 1 | `0,0772` |
/// | 2 | `0,0014` |
/// | 3 | `0,0000259` |
/// | 4 | `0,00000047` |
/// | 6 | `0,000000000` |
///
/// ⇒ cada passagem divide o resíduo por **~55**, e **`6` é onde a medição chega ao zero da
/// máquina**. ⚠️ **Não há recurso nenhum a limitar aqui** — isto corre uma vez por CLIQUE, e uma
/// passagem são cinco avaliações da mistura mais uma inversão `2×2`. *O número não é um tecto de
/// custo: é o ponto em que a escada acaba.*
///
/// ⛔ **Ele era `2`, e ficou observável por uma MUTAÇÃO que sobreviveu** — antes da fixtura da
/// mancha nada no corpus fazia o peso depender da posição, e ali **uma** passagem já acerta. *Uma
/// constante que nenhuma fixtura distingue é um palpite com cara de lei.*
pub const PASSAGENS: usize = 6;

fn compensa(
    fonte: &mut crate::skinned_mesh::SkinnedPath,
    pele: &ph2d_skeleton::Skin,
    correcoes: &[ph2d_skeleton::Correccao],
    ni: usize,
    alvo: &[[f64; 2]; 3],
    passagens: usize,
) {
    for _ in 0..passagens {
        let Some(ancora) = fonte.path.verts_all().nth(ni).map(|v| v.anchor) else {
            return;
        };
        let w = pesos_do_no(fonte, pele, correcoes, ni, ancora);
        let c = pele.blend([0.0, 0.0], &w);
        let u = sub(pele.blend([1.0, 0.0], &w), c);
        let v = sub(pele.blend([0.0, 1.0], &w), c);
        let det = u[0].mul_add(v[1], -(v[0] * u[1]));
        if det.abs() < 1e-12 {
            return;
        }
        let inverso = |x: [f64; 2]| {
            let d = sub(x, c);
            [
                d[0].mul_add(v[1], -(d[1] * v[0])) / det,
                d[1].mul_add(u[0], -(d[0] * u[1])) / det,
            ]
        };
        let Some((cc, local)) = fonte.path.locate_vert(ni) else {
            return;
        };
        let Some((verts, _)) = fonte.path.contour_mut(cc) else {
            return;
        };
        let Some(vert) = verts.get_mut(local) else {
            return;
        };
        vert.in_handle = inverso(alvo[0]);
        vert.anchor = inverso(alvo[1]);
        vert.out_handle = inverso(alvo[2]);
        // ⛔⛔ **Aqui esteve uma saída antecipada por convergência, e ela SAIU por uma mutação que
        // sobreviveu:** apagá-la não muda um bit do resultado — ela só poupava passagens de custo
        // nulo. *Uma linha que a mutação não consegue matar não é lei, é comentário com sintaxe de
        // código*, e é a segunda desta wave a cair pela mesma régua.
    }
}

/// A linha de pesos que o `recook` vai usar para o nó `k`, com as manchas já somadas.
///
/// ⚠️ **É a MESMA porta que o desenho chama** ([`ph2d_skeleton::Skin::weights_corrected`]) — uma
/// segunda cópia da conta divergiria dela no primeiro ajuste, e o ponto compensado passaria a
/// desenhar noutro sítio.
fn pesos_do_no(
    fonte: &crate::skinned_mesh::SkinnedPath,
    pele: &ph2d_skeleton::Skin,
    correcoes: &[ph2d_skeleton::Correccao],
    k: usize,
    onde: [f64; 2],
) -> Vec<f64> {
    let mut w = pele.scratch();
    pele.weights_corrected(onde, fonte.linha_do_no(k), &mut w, correcoes);
    w
}

fn lerp(a: [f64; 2], b: [f64; 2], t: f64) -> [f64; 2] {
    [
        (b[0] - a[0]).mul_add(t, a[0]),
        (b[1] - a[1]).mul_add(t, a[1]),
    ]
}

fn sub(a: [f64; 2], b: [f64; 2]) -> [f64; 2] {
    [a[0] - b[0], a[1] - b[1]]
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
