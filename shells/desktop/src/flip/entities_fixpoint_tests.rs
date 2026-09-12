//! ⭐⭐⭐ **O PONTO FIXO DA MÍDIA FLIP** — o gate de comportamento que faltava à cura de 2026-09-08.
//!
//! # Porque ele nasce depois da cura, e não antes
//!
//! O censo `every_document_to_tree_bridge_is_in_the_net` achou que o
//! [`crate::flip::entities::sync`] corria no passe do desenho e **não** na rede do fim do quadro, e
//! a cura entrou no mesmo dia. Mas um censo é **textual**: ele afirma que a chamada está lá, nunca
//! que ela FECHA o que promete fechar. ⚠️ *Um gate de texto verde sobre uma semântica errada lê-se
//! exactamente como um produto correcto* — e é essa metade que este ficheiro mede.
//!
//! # A lei, a mesma dos irmãos vectoriais
//!
//! > **A captura do undo tem de ser PONTO FIXO dos sistemas.** Fotografar, deixar o quadro seguinte
//! > correr sem entrada nenhuma, e fotografar outra vez tem de dar a MESMA foto.
//!
//! O `sync` do Flip é **bidireccional** como o do vector: apagar a entidade pela Hierarquia leva o
//! objecto do documento — e, sem a rede, leva-o **no quadro seguinte**, sozinho. O
//! `post_frame_undo` lê essa convergência como acção do artista e nasce o passo **fantasma** que o
//! dono reportou em 2026-09-07, noutra mídia.
//!
//! ⚠️ **Este arnês modela o quadro do FLIP** — a ponte e o assentamento do pivô —, e a rede que
//! corre depois deles. *Um arnês mede exactamente os escritores que alguém se lembrou de lhe pôr
//! dentro*, e foi essa lacuna que deixou passar o report de 2026-09-07.

use super::*;
use crate::flip::transform::settle_origins;
use crate::undo::ProjectState;
use ph2d_ecs::scene::{ComponentRegistry, register_ecs_components};
use ph2d_flip::{FlipStroke, Hold, KeyKind};

/// ⭐⭐⭐ **Um objecto de animação COM ARTE, longe da origem** — e a fixtura é a metade que decide.
///
/// # Porque ela existe, e o que a mutação disse
///
/// A 1.ª redacção destes gates criava objectos **vazios** (`push_object` e mais nada), e a prova de
/// mutação foi clara: apagar o assentamento do pivô da rede **SOBREVIVEU**. Não era um gate a
/// menos nem uma linha redundante — era a **terceira leitura** de uma mutação sobrevivente:
/// *a fixtura não produzia o fenómeno*. O [`crate::flip::transform::settle_origins`] só age sobre um
/// objecto que tenha `geometry_bbox()` **e** cujo centro não seja `(0, 0)`; sem arte, ele não tem o
/// que assentar e a linha é invisível.
///
/// ⚠️ **O traço vai de `(4, 4)` a `(6, 6)`** — centro em `(5, 5)`, bem longe da origem, que é
/// exactamente a condição que o passe exige para mover o pivô.
fn object_with_art(doc: &mut FlipDoc, nome: &str, x: f32) -> FlipObjectId {
    let oid = doc.push_object(nome);
    let obj = doc.object_mut(oid).expect("o objecto acabou de nascer");
    let l = obj.add_layer("Traco");
    let d = obj
        .insert_frame(l, 0, Hold::Implicit, KeyKind::Keyframe)
        .expect("o desenho acabou de nascer");
    let mut s = FlipStroke::new();
    s.push_default(ph2d_core::Vec2::new(x, 4.0));
    s.push_default(ph2d_core::Vec2::new(x + 2.0, 6.0));
    obj.drawing_mut(d)
        .expect("o desenho existe")
        .strokes
        .push(s);
    oid
}

/// O pedaço do quadro do Flip que muta o estado que o undo fotografa.
struct FlipFrame {
    reg: ComponentRegistry,
    undo_cache: ph2d_ecs::scene::incremental::CaptureCache,
}

impl FlipFrame {
    fn new() -> Self {
        let mut reg = ComponentRegistry::new();
        register_ecs_components(&mut reg);
        ph2d_render::register_render_components(&mut reg);
        Self {
            reg,
            undo_cache: ph2d_ecs::scene::incremental::CaptureCache::new(),
        }
    }

    /// O passe do DESENHO: a ponte e o assentamento do pivô, na ordem do `render_loop`.
    fn run(&mut self, sim: &mut SimWorld, doc: &mut FlipDoc, map: &mut FlipEntityMap) {
        sync(sim, doc, map);
        // ⚠️ Sem objecto em gesto: a mão não está a desenhar neste arnês.
        settle_origins(sim, doc, map, None);
    }

    /// ⭐ **A REDE do fim do quadro** — o espelho de
    /// [`crate::vec_tree_settle::App::settle_tree_before_capture`] na parte que é do Flip.
    ///
    /// **Mutação que deve sangrar:** apagar a chamada ao `sync` daqui — o gate abaixo volta a
    /// vermelho com a mensagem do passo fantasma.
    fn settle(&mut self, sim: &mut SimWorld, doc: &mut FlipDoc, map: &mut FlipEntityMap) {
        sync(sim, doc, map);
        settle_origins(sim, doc, map, None);
    }

    fn capture(&mut self, sim: &mut SimWorld, doc: &FlipDoc) -> ProjectState {
        ProjectState::capture(
            &ph2d_preview_drive::PreviewDrive::default(),
            sim,
            &ph2d_vec_scene::VecScene::new(),
            doc,
            &ph2d_guides::GuideSet::default(),
            &ph2d_ui_state::StateSets::default(),
            &crate::project_library::LibraryDoc::default(),
            &self.reg,
            &mut self.undo_cache,
            None,
        )
    }
}

/// ⭐⭐⭐ **APAGAR um objecto de ANIMAÇÃO tarde deixa a captura no ponto fixo.**
///
/// O gesto é o do menu de contexto da Hierarquia (*Delete*), que corre no
/// `render_loop::hierarchy::dispatch` — ~2 300 linhas **depois** de a ponte já ter corrido.
#[test]
fn a_late_flip_delete_leaves_the_capture_a_fixed_point() {
    let mut sim = SimWorld::default();
    let mut doc = FlipDoc::new();
    let mut map = FlipEntityMap::new();
    let mut frame = FlipFrame::new();

    let a = object_with_art(&mut doc, "Cena", 4.0);
    let b = object_with_art(&mut doc, "Personagem", 10.0);
    frame.run(&mut sim, &mut doc, &mut map);
    let vitima = Entity::from_bits(map[&b]);

    // O gesto TARDIO: a Hierarquia despawna depois de a ponte já ter corrido neste quadro.
    let _ = sim.world_mut().despawn(vitima);
    frame.settle(&mut sim, &mut doc, &mut map);
    let depois_do_gesto = frame.capture(&mut sim, &doc);

    // ⚠️ **CONTROLO — as DUAS metades**: sem elas o ponto fixo abaixo mediria um quadro parado.
    assert!(
        sim.world().get_entity(vitima).is_err(),
        "a fixtura nao apagou nada — o ponto fixo abaixo mediria um quadro parado"
    );
    assert!(
        !doc.objects().iter().any(|o| o.id == b),
        "a rede nao levou o OBJECTO junto com a entidade — e' a metade do `sync` que faz a \
         fotografia ser coerente, e sem ela o ponto fixo passa por acidente noutro sitio"
    );
    assert!(
        doc.objects().iter().any(|o| o.id == a),
        "a fixtura apagou o objecto errado"
    );

    // E agora o quadro seguinte, **sem entrada nenhuma**.
    frame.run(&mut sim, &mut doc, &mut map);
    let quadro_seguinte = frame.capture(&mut sim, &doc);

    assert!(
        depois_do_gesto == quadro_seguinte,
        "apagar um objecto de ANIMACAO tarde deixa a captura fora do ponto fixo: o quadro seguinte \
         muda o documento sozinho, e o `post_frame_undo` le isso como uma accao do artista — e' o \
         passo fantasma do report de 2026-09-07, noutra midia"
    );
}

/// ⭐⭐⭐ **E o ESPELHO: um objecto NOVO no documento tarde.**
///
/// Apagar tira do MUNDO; criar põe no DOCUMENTO, e quem cunha a entidade é a ponte. As duas metades
/// da mesma latência — *um gate só deixaria metade da classe por medir*, que foi exactamente o que
/// os irmãos vectoriais ensinaram.
#[test]
fn a_late_flip_insert_leaves_the_capture_a_fixed_point() {
    let mut sim = SimWorld::default();
    let mut doc = FlipDoc::new();
    let mut map = FlipEntityMap::new();
    let mut frame = FlipFrame::new();

    object_with_art(&mut doc, "Cena", 4.0);
    frame.run(&mut sim, &mut doc, &mut map);
    let antes = doc.objects().len();

    // O gesto TARDIO: o objecto entra no documento depois de a ponte já ter corrido.
    let novo = object_with_art(&mut doc, "Recem-chegado", 10.0);
    frame.settle(&mut sim, &mut doc, &mut map);
    let depois_do_gesto = frame.capture(&mut sim, &doc);

    // ⚠️ **CONTROLO — as DUAS metades**: o documento cresceu, e a entidade dele já foi cunhada.
    assert_eq!(doc.objects().len(), antes + 1, "a fixtura nao criou nada");
    let bits = *map.get(&novo).expect(
        "a rede nao cunhou a entidade do objecto novo — sem ela a fotografia guarda um documento \
         e um mundo que discordam",
    );
    assert!(
        sim.world()
            .get::<ph2d_ecs::StableId>(Entity::from_bits(bits))
            .is_some(),
        "a entidade nasceu SEM identidade duravel — ela entraria no snapshot sem `StableId` e o \
         primeiro Ctrl+Z nao teria o que repor"
    );

    frame.run(&mut sim, &mut doc, &mut map);
    let quadro_seguinte = frame.capture(&mut sim, &doc);

    assert!(
        depois_do_gesto == quadro_seguinte,
        "criar um objecto de ANIMACAO tarde deixa a captura fora do ponto fixo: o mundo ganha a \
         entidade sozinho no quadro seguinte, e o `post_frame_undo` le isso como uma accao do \
         artista"
    );
}
