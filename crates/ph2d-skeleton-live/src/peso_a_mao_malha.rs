//! ⭐⭐⭐ **O LATTICE À VISTA** — a malha do domínio que o bind guardou, desenhável.
//!
//! # ⛔⛔⛔ Porque este ficheiro existe
//!
//! Report do dono (2026-09-20), sobre a wave que pôs o campo do domínio a sobreviver ao bind:
//! *«não deveria aparecer o lattice na hora de pintar os pesos? Parece sem mudanças para mim»*.
//!
//! As duas metades da frase são verdade, e a medição dá-lhe razão nas duas:
//!
//! | o que | medido |
//! |---|---|
//! | o campo muda o desenho do braço da cena | **`0,04 %`–`0,13 %`** da barra |
//! | o mesmo braço **sem** a subdivisão do bind | `5,5 %`–`10,4 %` |
//! | a malha do domínio daquele braço | `498` vértices, **nunca desenhados** |
//! | o que o pincel de peso mostra hoje | `34` **pontos** |
//!
//! ⇒ *a subdivisão do bind já entrega nós que chegam, logo o campo quase não move aquela peça* — e
//! o que o artista via eram trinta e quatro pontos soltos onde existem quinhentos vértices com a
//! resposta do padrão-ouro. **A malha estava a ser calculada, guardada, consultada e invisível.**
//!
//! ⚠️ **Ela é uma LEITURA e não um segundo motor:** cada vértice é passado pela MESMA porta do
//! produto ([`ph2d_skeleton::Skin::weights_corrected`] seguida de `blend`) que a arte atravessa —
//! ⛔ uma segunda lei aqui mostraria ao artista um retículo que a forma não obedece, que é pior do
//! que não mostrar nenhum.

use crate::peso_a_mao::{mundo_e_escala, tendao_de};
use ph2d_ecs::{Entity, SimWorld};
use ph2d_skeleton_ecs::SkinBind;

/// **A malha do domínio de uma pele, já deformada e pronta a desenhar.**
///
/// ⚠️ **Os vértices vêm em MUNDO e já POSADOS** — é o retículo a fazer o trabalho dele, não o de
/// repouso por cima de uma arte dobrada. Com o esqueleto em repouso as duas coincidem, e é por isso
/// que a diferença só se vê com um osso torcido: *um indicador de repouso mentiria exactamente no
/// instante em que o artista precisa dele.*
#[derive(Clone, Debug, PartialEq)]
pub struct MalhaDoPeso {
    /// Vértice a vértice, em coordenadas de MUNDO.
    pub verts: Vec<[f64; 2]>,
    /// O peso do osso em foco em cada vértice — a mesma grandeza que os pontos já pintam.
    pub pesos: Vec<f64>,
    /// Os triângulos, por índice em [`Self::verts`].
    pub tris: Vec<[u32; 3]>,
}

/// ⭐⭐⭐ **A MALHA DE UMA PELE, com o peso deste osso em cada vértice.**
///
/// `None` quando não há o que mostrar, e cada `None` é uma resposta diferente que o desenho lê
/// igual — por isso ficam todas aqui e não espalhadas por quem chama:
///
/// - a coisa não tem pele, ou a pele não resolve (osso apagado);
/// - **este osso não governa esta pele** (o mesmo `None` que o [`crate::peso_a_mao::pinta`] devolve
///   como `OssoDeFora`) — ⛔ desenhar-lhe um retículo azul prometeria um pincel que a porta ao lado
///   recusa;
/// - a fonte guardada não é um CAMINHO (uma imagem já **é** a própria malha, e os pontos dela são
///   todos os vértices — o retículo seria a segunda cópia do que já se vê);
/// - o bind é anterior ao campo, ou a lei é `Envelope` ⇒ não há domínio guardado;
/// - ⚠️ **a tabela do campo não fecha com a pele** (`campo.ossos() != pele.len()`), que é o que
///   acontece com um osso *bendy*: ali cada osso autorado dá N sub-ossos e as colunas deixam de
///   corresponder. *Desenhar mesmo assim daria pesos plausíveis sobre os ossos errados.*
#[must_use]
pub fn malha_do_peso(sim: &SimWorld, alvo: Entity, osso: Entity) -> Option<MalhaDoPeso> {
    let skin = sim.world().get::<SkinBind>(alvo)?.clone();
    let pele = crate::skin_live::skin_of(sim, alvo)?;
    let tendao = tendao_de(sim, &skin, osso)?;
    let g = crate::skinned_mesh::le(&skin.source)?;
    let campo = g.campo.as_ref()?;
    if !campo.valida() || campo.ossos() != pele.len() {
        return None;
    }
    // ⚠️ **A LEI passa por aqui** — com `Envelope` o quadro ignora a tabela guardada, logo o
    // retículo do padrão-ouro descreveria uma deformação que a arte não faz.
    if skin.pesos_do_quadro(&campo.pesos).is_empty() {
        return None;
    }
    let correcoes = skin.correcoes_resolvidas();
    let (x, _) = mundo_e_escala(sim, alvo);
    let n = campo.malha.rest.len();
    let mut w = pele.scratch();
    let (mut verts, mut pesos) = (Vec::with_capacity(n), Vec::with_capacity(n));
    for i in 0..n {
        let p = campo.local_do_vertice(i)?;
        let linha = campo.linha_do_vertice(i)?;
        pele.weights_corrected(p, Some(linha), &mut w, &correcoes);
        verts.push(x.apply(pele.blend(p, &w)));
        pesos.push(w.get(tendao).copied().unwrap_or(0.0));
    }
    Some(MalhaDoPeso {
        verts,
        pesos,
        tris: campo.malha.tris.clone(),
    })
}

/// ⭐⭐ **O QUE O INDICADOR MOSTRA** — o retículo de TODA a arte que este osso governa.
///
/// ⚠️ **Irmã exacta da [`crate::peso_a_mao::pontos_do_indicador`], e de propósito:** as duas
/// respondem *«o que este OSSO governa»* e nenhuma pergunta onde o rato está — foi esse o report de
/// 2026-09-19 (*«as cores só aparecem se o mouse estiver sobre a forma»*), e uma segunda lei aqui
/// faria o retículo piscar enquanto os pontos ficavam.
#[must_use]
pub fn malhas_do_indicador(sim: &SimWorld, osso: Option<Entity>) -> Vec<MalhaDoPeso> {
    let Some(osso) = osso else {
        return Vec::new();
    };
    let peles: Vec<Entity> = sim
        .world()
        .iter_entities()
        .filter(|er| er.get::<SkinBind>().is_some())
        .map(|er| er.id())
        .collect();
    peles
        .into_iter()
        .filter_map(|alvo| malha_do_peso(sim, alvo, osso))
        .collect()
}

#[cfg(test)]
#[path = "peso_a_mao_malha_tests.rs"]
mod tests;
