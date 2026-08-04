//! **A CÓPIA SEGUE A ÂNCORA DO MESTRE** — o corolário da âncora viva, do lado do componente.
//!
//! Report do Enio, duas rodadas: *"escalonar o gizmo pela borda direita deixa a esquerda fixa, mas
//! nas instâncias a esquerda não fica fixa — as duas bordas se movem"*, e depois *"faça com que as
//! instâncias se comportem exatamente igual ao mestre: se o canto do mestre está imóvel ao
//! redimensionar, o canto da instância se comporta igual"*.
//!
//! # O mecanismo, e por que ele NÃO é um defeito do produtor
//!
//! O produtor ([`crate::instance_live::place_delta`]) leva a arte do mestre para a cópia por
//! `D(p) = (p − Tm)·I + Ti`, onde `Tm` é a translação de MUNDO do mestre e `I` a pose da cópia.
//! Remover `Tm` é o que faz *mover o mestre não mover ninguém* — a lei do Figma, e a razão de o
//! delta existir.
//!
//! Uma alça de escala ancorada num canto **paga a âncora compensando a translação da entidade**:
//! `Tm` anda para que a quina oposta fique parada. E a translação é exactamente o que a cópia não
//! herda ⇒ a compensação vai para o lixo e o crescimento da cópia sai simétrico em torno de `Ti`.
//!
//! # A cura, e por que ela tem de ser uma MUDANÇA e não uma derivação
//!
//! A imagem do ponto fixo do mestre é `D(A) = (A − Tm)·I_lin + Ti`. Segurá-la é uma equação de uma
//! linha:
//!
//! ```text
//! ΔTi = ΔTm · I_lin
//! ```
//!
//! ⚠️ **Nenhuma derivação pura do documento consegue isto, e é demonstrável:** uma pose com âncora
//! é `p → (p − A)·F + A`, e `A` só é recuperável de `(Tm, Sm)` se você souber o estado ANTERIOR.
//! `A` é um fato da MUDANÇA, não do estado — e uma derivação lê estado. Foi por isso que a
//! tentativa anterior (ancorar na quina MÍNIMA da caixa, o modelo do Figma) acertava a alça direita
//! e **espelhava** a esquerda: uma quina fixa do documento só pode casar com uma das quatro alças.
//!
//! ⚠️ **E é por isso que a cura mora AQUI e não no produtor:** ela é aplicada no instante da
//! mudança, contra um instantâneo do pen-down, exactamente como o [`crate::vec_frame_resize`].
//!
//! # As duas propriedades que caem de graça
//!
//! - **Ctrl (âncora no centro):** o gizmo não compensa translação nenhuma ⇒ `ΔTm = 0` ⇒ a cópia
//!   não anda e cresce simétrica em torno de `Ti`, que é a imagem do pivô do mestre. O mesmo
//!   comportamento, sem um segundo ramo.
//! - **Uma MOLDURA como mestre já estava certa** e não passa por aqui: a alça dela reescreve
//!   GEOMETRIA e deixa a pose em paz, então `D` é um afim fixo e a quina que o
//!   [`crate::vec_frame_resize`] segura em mundo aparece parada na cópia por construção. O mesmo
//!   vale para o `W`/`H` do painel.
//!
//! # O que ela NÃO faz
//!
//! ⚠️ A cópia **anda**; o SUPORTE dela (o retângulo guardado que dá caixa de gizmo e alvo de
//! clique) anda junto porque os dois viajam sob o mesmo `Transform` — e é isso que impede a
//! regressão de seleção que matou a tentativa anterior. O suporte continua a **não crescer**: é o
//! instantâneo que o [`crate::vec_component_edit`] já nomeia, e esta wave não o move nem o piora.

use ph2d_ecs::{Entity, SimWorld, VecInstance};
use ph2d_vec_scene::{VecScene, Xform};

use crate::vec_entities::VecEntityMap;

/// **O que cada cópia era quando o arrasto começou.**
///
/// ⚠️ Absoluto contra o pen-down, e não incremental: o gizmo recomputa uma transformação ABSOLUTA a
/// cada `CursorMoved`, então somar um delta por evento multiplicaria o deslocamento pela taxa de
/// amostragem do rato. A mesma lei do [`crate::vec_frame_resize::FrameResizeStart`], e pela mesma
/// razão.
#[derive(Clone)]
pub(crate) struct InstanceFollow {
    /// A entidade PRIMÁRIA do arrasto que fotografou isto. Ver [`Self::is_for`].
    drag_bits: u64,
    entries: Vec<Entry>,
}

#[derive(Clone, Copy)]
struct Entry {
    /// A cópia que anda.
    instance: u64,
    /// A translação de MUNDO dela no pen-down.
    start_world: [f64; 2],
    /// A parte LINEAR de mundo dela — constante enquanto quem se mexe é o mestre.
    linear: [f64; 4],
    /// O mestre a que ela obedece.
    main: u64,
    /// A translação de MUNDO do mestre no pen-down.
    main_start_world: [f64; 2],
}

impl InstanceFollow {
    /// Este instantâneo é do arrasto de `entity_bits`?
    #[must_use]
    pub(crate) fn is_for(&self, entity_bits: u64) -> bool {
        self.drag_bits == entity_bits
    }

    /// Quantas cópias ele carrega.
    ///
    /// ⚠️ **Só os gates chamam** — o produto nunca pergunta o tamanho do instantâneo, ele
    /// itera. `#[cfg(test)]` em vez de um `#[allow(dead_code)]` porque um acessor `pub(crate)`
    /// sem chamador não é código morto silencioso: é uma **segunda resposta** esperando que
    /// alguém a chame, e o dia em que alguém chamar é o dia em que a contagem passa a ser
    /// consultada num sítio que não a re-derivou.
    #[cfg(test)]
    #[must_use]
    pub(crate) fn len(&self) -> usize {
        self.entries.len()
    }
}

/// A translação de mundo de uma entidade, em `f64` (a unidade do afim do documento).
fn world_translation(sim: &SimWorld, e: Entity) -> [f64; 2] {
    let x = crate::vec_transform::xform_of_transform(crate::vec_transform::world_transform(sim, e));
    [x.0[4], x.0[5]]
}

/// **Fotografa as cópias que obedecem a alguém deste arrasto.** `None` quando não há nenhuma — e
/// aí o caminho comum não paga nem uma escrita.
///
/// `dragged` são as entidades que o gesto vai mexer (a primária mais os extras da multi-seleção).
///
/// ⚠️ **Uma cópia que está ELA PRÓPRIA no arrasto fica de fora.** O artista está a movê-la com a
/// mão; somar-lhe o seguimento do mestre daria dois autores para a mesma translação, e o segundo
/// venceria em silêncio.
#[must_use]
pub(crate) fn begin(
    sim: &SimWorld,
    scene: &VecScene,
    map: &VecEntityMap,
    dragged: &[u64],
    drag_bits: u64,
) -> Option<InstanceFollow> {
    let mut entries: Vec<Entry> = Vec::new();
    // A varredura é sobre a CENA (ordem de z) e não sobre o mundo ECS — a mesma razão do produtor:
    // é ela que dá a ordem estável que o HR-5 exige de qualquer lista que o gesto percorre.
    for path in scene.paths() {
        let Some(&bits) = map.get(&path.id) else {
            continue;
        };
        if dragged.contains(&bits) {
            continue;
        }
        let e = Entity::from_bits(bits);
        let Some(inst) = sim.world().get::<VecInstance>(e) else {
            continue;
        };
        let Some(&main_bits) = map.get(&inst.main) else {
            continue;
        };
        if !dragged.contains(&main_bits) {
            continue;
        }
        let x =
            crate::vec_transform::xform_of_transform(crate::vec_transform::world_transform(sim, e));
        entries.push(Entry {
            instance: bits,
            start_world: [x.0[4], x.0[5]],
            linear: [x.0[0], x.0[1], x.0[2], x.0[3]],
            main: main_bits,
            main_start_world: world_translation(sim, Entity::from_bits(main_bits)),
        });
    }
    (!entries.is_empty()).then_some(InstanceFollow { drag_bits, entries })
}

/// **Põe cada cópia onde a âncora do mestre dela manda.** Idempotente: o alvo é absoluto contra o
/// instantâneo, então re-aplicar sobre o resultado dá o mesmo número.
pub(crate) fn apply(sim: &mut SimWorld, follow: &InstanceFollow) {
    for entry in &follow.entries {
        let main = Entity::from_bits(entry.main);
        let now = world_translation(sim, main);
        let delta = [
            now[0] - entry.main_start_world[0],
            now[1] - entry.main_start_world[1],
        ];
        // ⚠️ `apply_vec` e não `apply`: isto é um DELTA, e transladá-lo tornaria-o um ponto. É a
        // parte linear da pose da cópia que o converte — uma cópia escalada 2× tem de andar 2×,
        // porque o conteúdo dela é 2× maior.
        let moved = Xform([
            entry.linear[0],
            entry.linear[1],
            entry.linear[2],
            entry.linear[3],
            0.0,
            0.0,
        ])
        .apply_vec(delta);
        let target = [
            (entry.start_world[0] + moved[0]) as f32,
            (entry.start_world[1] + moved[1]) as f32,
        ];
        let e = Entity::from_bits(entry.instance);
        let parent = ph2d_ecs::parent_world_transform(sim.world(), e);
        let parent = ph2d_editor::TransformSnapshot {
            translation: [parent.translation.x, parent.translation.y],
            rotation: parent.rotation,
            scale: [parent.scale.x, parent.scale.y],
        };
        let local = ph2d_editor::world_translation_to_local(parent, target);
        if let Some(mut t) = sim.world_mut().get_mut::<ph2d_ecs::Transform>(e) {
            t.translation = ph2d_core::Vec2::new(local[0], local[1]);
        }
    }
}

impl crate::App {
    /// Fotografa as cópias para o arrasto de `entity_bits` — a primária MAIS os extras da
    /// multi-seleção, porque um mestre pode estar em qualquer um dos dois lados.
    #[must_use]
    pub(crate) fn begin_instance_follow(&self, entity_bits: u64) -> Option<InstanceFollow> {
        let gfx = self.gfx.as_ref()?;
        let mut dragged: Vec<u64> = Vec::with_capacity(self.group_drag_starts.len() + 1);
        dragged.push(entity_bits);
        dragged.extend(self.group_drag_starts.iter().map(|s| s.entity_bits));
        begin(
            &gfx.sim,
            &gfx.vec_scene,
            &self.vec_entities,
            &dragged,
            entity_bits,
        )
    }
}

#[cfg(test)]
#[path = "vec_instance_follow_tests.rs"]
mod tests;
