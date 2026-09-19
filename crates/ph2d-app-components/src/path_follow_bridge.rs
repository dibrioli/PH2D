//! **A ponte do SEGUIDOR DE CAMINHO** (suplente #23) — onde o nome vira uma curva, e a curva vira
//! uma pose de MUNDO.
//!
//! # Porque ela mora aqui, e não na fundação
//!
//! Medido (§5.0 do plano 20): o `ph2d-ecs` **não** declara o `ph2d-vec-scene` nas
//! `[dependencies]`, e esta crate declara **as duas** — mais o `ph2d-vec-entities`. É a doutrina do
//! `VecPathRef` (*«não põe geometria no ECS»*) a decidir o endereço: o componente guarda o **NOME**
//! da forma, e quem sabe o que é uma cúbica é a família.
//!
//! # ⛔⛔ Ela NÃO escreve no mundo
//!
//! Ela **lê** e devolve o que os caminhos pedem. Quem escreve é o passe do TWEEN, e a razão é o
//! ledger: a chave dele é `(entidade, driver)` e a [`ph2d_preview_drive::PreviewDrive::driven`]
//! troca o autorado quando *«o que o motor encontrou não é o que ele deixou»* ⇒ **um segundo passe
//! que fotografasse a pose depois do tween leria a pré-visualização dele como documento**. Esse é o
//! defeito que o cabeçalho do [`crate::tween_bridge`] já descreve, e a cura é UM censo por entidade.
//!
//! # ⚠️ O que ela NÃO resolve, declarado
//!
//! Um caminho **composto** (com `subpaths`) é percorrido pelo **primeiro contorno**. A `ArcPath` é
//! a porta de *«onde fica o arco `s` neste CONTORNO»*, e um caminho com furos não tem um percurso
//! único — escolher um seria inventar a lei em vez de a autorar.

use std::collections::BTreeMap;

use ph2d_ecs::{Entity, SimWorld, StableId};
use ph2d_vec_scene::arc_path::ArcPath;
use ph2d_vec_scene::{VecPathId, VecScene, Xform};

/// **O que um seguidor pede, já em MUNDO** — a saída desta ponte.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PoseDeCaminho {
    /// Quem anda.
    pub entity: Entity,
    /// Onde, em coordenadas de MUNDO.
    pub mundo: [f32; 2],
    /// Para onde aponta, em radianos de MUNDO. `None` = o seguidor não alinha, e a rotação é de
    /// quem a autorou.
    pub angulo: Option<f32>,
}

/// A curva de um caminho, pronta a ser percorrida, **mais o afim que a leva ao mundo**.
struct Pista {
    arco: ArcPath,
    afim: Xform,
}

/// ⭐⭐⭐ **O que os seguidores da cena pedem neste quadro.**
///
/// ⚠️ **Zero custo quando não há nenhum:** a lista da fundação vem vazia e a função devolve antes
/// de tocar na cena vectorial — que é o caso de toda cena que já existe.
///
/// ⚠️ **Uma pista é construída UMA vez por quadro, mesmo com N seguidores nela.** Construir um
/// `ArcPath` custa um `arclen` por segmento (Gauss-Legendre de 16 nós), e o doc dele avisa por
/// escrito: *«um `ArcPath` por amostra transformaria um efeito linear em quadrático»*.
pub fn a_escrever(sim: &mut SimWorld, cena: &VecScene) -> Vec<PoseDeCaminho> {
    let pedidos = ph2d_ecs::path_follow::a_seguir(sim.world_mut());
    if pedidos.is_empty() {
        return Vec::new();
    }
    let mut pistas: BTreeMap<VecPathId, Option<Pista>> = BTreeMap::new();
    let mut fora = Vec::with_capacity(pedidos.len());
    for p in &pedidos {
        let Some((dono, id)) = caminho_chamado(sim, &p.caminho) else {
            continue;
        };
        let pista = pistas
            .entry(id)
            .or_insert_with(|| monta_pista(sim, cena, dono, id));
        let Some(pista) = pista.as_ref() else {
            continue;
        };
        let (ponto, tangente) = pista.arco.frame_at(p.fraccao * pista.arco.total());
        let ponto = pista.afim.apply(ponto);
        // ⚠️ **A tangente é um VECTOR**, não um ponto: sob uma pose com translação, `apply` daria
        // uma direcção deslocada — e o objecto apontaria para um sítio que não existe.
        let tangente = pista.afim.apply_vec(tangente);
        let n = tangente[0].hypot(tangente[1]);
        // ⚠️ **Numa cúspide a tangente é NULA** (o `frame_at` declara-o), e aqui isso vale duas
        // coisas: não há lado para onde deslocar, e não há para onde apontar.
        let (lado, angulo) = if n > 0.0 {
            let t = [tangente[0] / n, tangente[1] / n];
            (
                [-t[1] * f64::from(p.lado), t[0] * f64::from(p.lado)],
                p.alinha
                    .then(|| (t[1].atan2(t[0]) as f32) + p.angulo.to_radians()),
            )
        } else {
            ([0.0, 0.0], None)
        };
        fora.push(PoseDeCaminho {
            entity: p.entity,
            mundo: [(ponto[0] + lado[0]) as f32, (ponto[1] + lado[1]) as f32],
            angulo,
        });
    }
    fora
}

/// **A entidade e o `VecPathId` da forma que se CHAMA assim** — pela porta do nome, que é a
/// referência durável desta casa (a mesma que o `follow_target` da câmera percorre).
///
/// ⚠️ **Devolve as DUAS coisas**: o id endereça a geometria e a entidade carrega a **pose**, e
/// procurar a segunda outra vez (varrendo o mundo por quem tem aquele `VecPathRef`) seria uma
/// segunda resposta à pergunta que esta já respondeu.
fn caminho_chamado(sim: &mut SimWorld, nome: &str) -> Option<(Entity, VecPathId)> {
    let nome = nome.trim();
    if nome.is_empty() {
        return None;
    }
    let world = sim.world_mut();
    let id = ph2d_ecs::stable_id_for_name(world, nome);
    let e = ph2d_ecs::entity_of_stable_id(world, StableId(id))?;
    Some((e, world.get::<ph2d_ecs::VecPathRef>(e)?.0))
}

/// Prepara a curva **COZIDA** e o afim local→mundo da entidade que a carrega.
///
/// ⚠️ **COZIDA, e não autorada:** o artista vê a saída dos Live Path Effects (a quina viva, o
/// offset, o zig-zag), e um seguidor que percorresse a fonte andaria ao lado do que está na tela.
fn monta_pista(sim: &SimWorld, cena: &VecScene, dono: Entity, id: VecPathId) -> Option<Pista> {
    let cozido = cena.path(id)?.cooked();
    let arco = ArcPath::from_contour(&cozido.verts, cozido.closed)?;
    // ⚠️ **Um contorno degenerado tem comprimento ZERO**, e quem divide por ele lê `NaN` — o doc do
    // `ArcPath::total` manda testá-lo, em vez de o presumir positivo.
    if arco.total() <= 0.0 {
        return None;
    }
    let afim = ph2d_vec_entities::transform::xform_of_transform(
        ph2d_vec_entities::transform::world_transform(sim, dono),
    );
    Some(Pista { arco, afim })
}

#[cfg(test)]
#[path = "path_follow_bridge_tests.rs"]
mod tests;
