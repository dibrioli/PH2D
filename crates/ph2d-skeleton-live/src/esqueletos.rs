//! ⭐ **QUANTOS ESQUELETOS HÁ, E QUAL É A RAIZ DE UM OSSO** — a topologia da árvore de ossos, numa
//! porta só.
//!
//! ⚠️ **Ela existe porque a subida à raiz passou a ter DOIS leitores:** o [`skeleton_of`], que já a
//! fazia à mão, e o [`crate::recusa_do_bind`], que precisa de contar esqueletos independentes.
//! *Uma lei escrita em dois sítios ainda não é uma lei — só uma PORTA é*, e este repo já a pagou
//! três vezes (o traço uniforme, a chave da ordem das raízes, a média do anel).
//!
//! ⚠️ **Ela vive aqui e não no `skin_live.rs`**, que estava a `697` linhas de um tecto de `700`:
//! mover a subida para cá **tira** linhas de lá em vez de as somar.

use ph2d_ecs::{ChildOf, Entity, SimWorld};
use ph2d_skeleton_ecs::Bone;

/// **A raiz do esqueleto a que este osso pertence.**
///
/// ⚠️ Sobe enquanto o **pai também for osso** — parar no primeiro pai não-osso é o que permite
/// pendurar um esqueleto inteiro dentro de um grupo sem ele deixar de ser um esqueleto.
#[must_use]
pub fn raiz_do_osso(sim: &SimWorld, osso: Entity) -> Entity {
    let mut raiz = osso;
    while let Some(p) = sim.world().get::<ChildOf>(raiz).map(ChildOf::parent) {
        if sim.world().get::<Bone>(p).is_some() {
            raiz = p;
        } else {
            break;
        }
    }
    raiz
}

/// **As raízes dos esqueletos da cena** — uma por esqueleto independente, ordenadas.
///
/// ⚠️ **Contar RAÍZES e não ossos** é o que distingue *«uma cadeia de três»* de *«três esqueletos»*.
#[must_use]
pub fn bone_roots(sim: &SimWorld) -> Vec<Entity> {
    let mut out: Vec<Entity> = crate::skin_live::ossos_da_cena(sim)
        .into_iter()
        .map(|(e, _)| raiz_do_osso(sim, e))
        .collect();
    out.sort_by_key(|e| e.to_bits());
    out.dedup();
    out
}

#[cfg(test)]
#[path = "esqueletos_tests.rs"]
mod tests;

/// ⭐⭐⭐ **ESTE BIND CAIU NA LEI DERIVADA?** — a pergunta de que o envelope depende, e a única que
/// o separa do padrão-ouro.
///
/// ⚠️ **Ela é sobre o BIND e nunca sobre a mídia.** Uma [`crate::skinned_mesh::SkinnedMesh`] (uma
/// imagem) e um [`crate::skinned_mesh::SkinnedPath`] (uma forma) guardam a MESMA coisa: a tabela
/// dos *Bounded Biharmonic Weights* resolvida ao prender, ou **nada**. Com tabela, o quadro lê os
/// pesos guardados e o `strength` **não entra na conta**; sem tabela, o quadro cai na lei euclidiana
/// (`bump` sobre a distância ao eixo), que é onde o alcance manda.
///
/// ⛔ **Os bytes são opacos de propósito** e quem os sabe ler é quem sabe o que a coisa É — a lei
/// que o [`crate::skin_image::skinned_mesh_of`] já escreve. ⇒ a mídia entra aqui só para ESCOLHER o
/// descodificador, nunca para decidir a resposta.
///
/// ⚠️ **Vazia é uma resposta, e é a que interessa:** um caminho ABERTO não tem interior, logo não
/// tem domínio, logo não tem pesos; e uma imagem cujo solver não convergiu escreve o mesmo vazio
/// (com a queixa no log). *Nos dois casos o envelope volta a mandar.*
#[must_use]
fn caiu_na_lei_derivada(mundo: &ph2d_ecs::World, e: ph2d_ecs::Entity) -> bool {
    let Some(skin) = mundo.get::<ph2d_skeleton_ecs::SkinBind>(e) else {
        return false;
    };
    // ⭐⭐⭐ **A ESCOLHA DO ARTISTA decide PRIMEIRO, e pela MESMA porta que o quadro lê**
    // (`SkinBind::pesos_do_quadro`, a wave de 2026-09-19). ⚠️ Sem esta linha, escolher *«por
    // alcance»* numa forma preenchida deformaria pelo envelope **sem mancha e sem alça** — o
    // artista ganharia o efeito e perderia o controlo dele, que é pior do que não ter a escolha.
    //
    // ⛔ Ela pergunta à porta com uma tabela NÃO-VAZIA de propósito: o que se mede aqui é a
    // ESCOLHA, e uma tabela vazia responderia «derivada» por outro motivo, escondendo-a.
    if skin.pesos_do_quadro(&[1.0]).is_empty() {
        return true;
    }
    if crate::skin_image::is_skinned_image(mundo, e) {
        return match postcard::from_bytes::<crate::skinned_mesh::SkinnedMesh>(&skin.source) {
            // ⛔ `ossos() == 0` e não `pesos.is_empty()`: a tabela que **não fecha** com a malha é
            // recusada a jusante e cai na mesma lei derivada — as duas leituras têm de concordar,
            // senão o gizmo some exactamente no caso em que o alcance volta a mandar.
            Ok(m) => !m.valida() || m.ossos() == 0,
            // Uma fonte que nem descodifica é pulada pelo quadro: não há deformação nenhuma para o
            // envelope governar, e acender a mancha ali seria prometer um efeito que não existe.
            Err(_) => false,
        };
    }
    match postcard::from_bytes::<crate::skinned_mesh::SkinnedPath>(&skin.source) {
        Ok(m) => !m.valida() || m.ossos() == 0,
        Err(_) => false,
    }
}

/// ⭐⭐⭐ **O ENVELOPE AINDA MANDA NESTE OSSO?** (report do dono, 2026-09-18:
/// *«Por que o envelope já não influencia na deformação?»* e, no dia seguinte, *«não vi em nenhum
/// dos casos o envelope fazer diferença»*).
///
/// ⛔⛔⛔ **A REDACÇÃO ANTERIOR ESTAVA ERRADA, e a medição que a derruba está na
/// [`crate::sonda_do_envelope_no_vector_tests`].** Ela perguntava *«há aqui uma forma VECTORIAL
/// presa?»*, com o argumento escrito de que *«o padrão-ouro precisa de uma malha do domínio, e uma
/// Bézier não tem uma»*. ⚠️ **Essa premissa expirou em 2026-09-15**, quando o
/// `ph2d_vec_skin::pesos::pesos_do_caminho` passou a construir a malha do INTERIOR de um contorno
/// fechado e a resolver os mesmos BBW. Medido, pela porta do produto, sobre o mesmo esqueleto de
/// três ossos, variando o `strength` do do meio de `0,1` a `8,0`:
///
/// | forma | fechada? | amplitude da deformação |
/// |---|---|---:|
/// | `Rectangle` · `Ellipse` · `Star` · `Polygon` | fechada | **`0,000000`** |
/// | `Line` | ABERTA | `2,03` |
/// | `Arc` | ABERTA | `4,25` |
/// | `Spiral` | ABERTA | `2,05` |
///
/// ⇒ *o dono tinha mais razão do que a minha resposta lhe deu*: o envelope é inerte em **toda**
/// forma preenchida e em **toda** imagem que resolve — a mídia nunca foi a pergunta certa.
///
/// ⭐⭐ **A pergunta certa é a do BIND:** ele guarda a tabela do padrão-ouro, ou não guarda? Ver
/// [`caiu_na_lei_derivada`]. Um osso cujo alcance ainda governa alguma coisa é um osso que alguma
/// pele consulta pela lei euclidiana.
///
/// ⚠️⚠️ **LIMITE MEDIDO, necessário-mas-não-suficiente:** o envelope também é **inerte** numa pele
/// presa a **UM** osso — os pesos renormalizam e o único osso leva sempre a fatia inteira (medido
/// na [`crate::sonda_do_envelope_tests`]: `[1.0]` a `strength = 0,3` **e** a `4,0`; com **três**
/// ossos vai de `[1, 0, 0]` a `[0,42, 0,58, 0]`). *O alcance só decide quando DOIS ossos disputam o
/// mesmo ponto.* ⛔ Contar essa disputa por ponto seria caro e frágil ⇒ esta porta esconde o caso
/// **claro** e mostra o resto, que é o lado conservador — *esconder um controlo vivo é pior do que
/// mostrar um inerte.*
///
/// ⚠️ **Sem `StableId` a resposta é SIM**, porque nenhum tendão pode nomear um osso acabado de
/// nascer: ali a ausência de prova não é prova de ausência.
#[must_use]
pub fn o_envelope_deste_osso_manda(sim: &SimWorld, osso: ph2d_ecs::Entity) -> bool {
    let mundo = sim.world();
    let Some(id) = mundo.get::<ph2d_ecs::StableId>(osso).copied() else {
        return true;
    };
    let Some(mut q) = mundo.try_query::<(ph2d_ecs::Entity, &ph2d_skeleton_ecs::SkinBind)>() else {
        return false;
    };
    q.iter(mundo)
        .any(|(e, b)| b.tendons.iter().any(|t| t.bone == id) && caiu_na_lei_derivada(mundo, e))
}

#[must_use]
pub fn influence_radius(sim: &SimWorld, bits: u64) -> Option<f64> {
    let e = Entity::from_bits(bits);
    let forca = sim.world().get::<Bone>(e)?.strength;
    let (_, a, b) = crate::skin_live::bone_segments(sim)
        .into_iter()
        .find(|(x, _, _)| *x == bits)?;
    Some((b[0] - a[0]).hypot(b[1] - a[1]) * forca.max(0.0))
}

/// **A região de influência de um osso** — `(raio, origem, ponta)` em MUNDO, para o overlay.
pub fn influence_region(sim: &SimWorld, bits: u64) -> Option<(f64, [f64; 2], [f64; 2])> {
    // ⭐⭐⭐ **A MANCHA SÓ EXISTE ONDE O ENVELOPE MANDA** (report do dono, 2026-09-18: *«o gizmo do
    // envelope fica sempre visível mesmo quando não é usado?»* — sim, ficava).
    //
    // ⛔⛔ Com os pesos do **padrão-ouro** uma imagem deforma igual a `1` e a `2` (medido, coluna a
    // coluna), logo num rig só de imagens esta região desenhava *«até onde este osso alcança»* sobre
    // uma lei que **não usa alcance nenhum**. ⚠️ E o envelope não morreu — **mudou de dono**: uma
    // forma vectorial presa continua na lei euclidiana, e ali a mancha diz a verdade.
    //
    // ⭐⭐ **A lei entra AQUI e não em quem desenha, porque esta porta tem DOIS consumidores** — o
    // desenho da mancha e o **hit-test da alça**. Curar só o pintor deixaria o artista a arrastar
    // uma alça invisível, que é pior do que a mancha a mais.
    let e = Entity::try_from_bits(bits)?;
    if !crate::esqueletos::o_envelope_deste_osso_manda(sim, e) {
        return None;
    }
    let r = influence_radius(sim, bits)?;
    let (_, a, b) = crate::skin_live::bone_segments(sim)
        .into_iter()
        .find(|(x, _, _)| *x == bits)?;
    Some((r, a, b))
}
