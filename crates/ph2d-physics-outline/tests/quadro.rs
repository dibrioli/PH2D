//! ⭐ **O retraço de seguir a imagem cabe no passo do gesto que mudou a imagem** (GEF02 09/11, (d)).
//!
//! O modelo do quadro é o de `shells/desktop/src/undo_drag_tests.rs`: no fim de cada quadro com
//! entrada, o documento que mudou desde a base vira UM passo; sem entrada, a mudança espera e FUNDE-SE
//! no passo seguinte. O documento aqui é o que o Apply e o retraço tocam: a imagem do corpo
//! (`SpritePixels`), os anéis da forma e o `Transform` de um vizinho (a acção seguinte, sem relação).
//!
//! As fases do quadro da shell, pela ordem de `fase_hero_frame.rs`: o assar SILENCIOSO do Painter
//! (`fase_painter_persist`) corre ANTES de `inspector_commits`; o Apply (`drain_painter` e toda
//! ferramenta de imagem, em `image_edit::dispatch`) corre DEPOIS, em `fase_image_edit_apply`.

use ph2d_ecs::scene::{
    ComponentRegistry, EditorCommandQueue, apply_editor_commands, register_ecs_components,
};
use ph2d_ecs::{ChildOf, SimWorld, SpritePixels, Transform};
use ph2d_physics_ecs::{
    BodyKind, Collider, ColliderOutline, ColliderShape, RigidBody, register_physics_components,
};
use ph2d_render::Sprite;
use std::collections::BTreeMap;

const PPM: f32 = 8.0;

fn img(on: impl Fn(u32, u32) -> bool) -> Vec<u8> {
    (0..32u32)
        .flat_map(|y| (0..32u32).map(move |x| (x, y)))
        .flat_map(|(x, y)| [255, 255, 255, if on(x, y) { 255 } else { 0 }])
        .collect()
}

/// Onde o retraço de seguir corre no quadro.
#[derive(Clone, Copy, Debug)]
enum Ordem {
    /// Antes do Apply (o `inspector_commits` de 09/11).
    Antes,
    /// Depois do Apply (`fase_image_edit_apply`).
    Depois,
}

/// Que assar muda a imagem.
#[derive(Clone, Copy, Debug)]
enum Assar {
    /// O Apply (`drain_painter`, as ferramentas de imagem): depois de `inspector_commits`.
    Apply,
    /// O assar silencioso do Painter (`auto_commit_painter`): antes de `inspector_commits`.
    Silencioso,
}

type Doc = (Option<SpritePixels>, Option<ColliderOutline>, Transform);

struct Cena {
    reg: ComponentRegistry,
    sim: SimWorld,
    db: ph2d_asset::AssetDb,
    q: EditorCommandQueue,
    corpo: ph2d_ecs::Entity,
    forma: ph2d_ecs::Entity,
    vizinho: ph2d_ecs::Entity,
}

impl Cena {
    /// Argola 32×32 px num Static de 4×4 m, forma Outline FILHA a seguir a imagem, já traçada e vista.
    fn nova() -> Self {
        let mut reg = ComponentRegistry::new();
        register_ecs_components(&mut reg);
        register_physics_components(&mut reg);
        let db = ph2d_asset::AssetDb::new();
        let argola = img(|x, y| !((8..24).contains(&x) && (8..24).contains(&y)));
        let argola = db.insert_image_rgba8(32, 32, argola);
        let mut sim = SimWorld::new();
        let w = sim.world_mut();
        let corpo = w
            .spawn((
                RigidBody {
                    kind: BodyKind::Static,
                },
                Transform::default(),
                Sprite::atlas(0, [4.0, 4.0], [1.0; 4]),
                SpritePixels(argola),
            ))
            .id();
        let outline = ColliderShape::Outline {
            half_x: 2.0,
            half_y: 2.0,
        };
        let c = Collider {
            shape: outline,
            ..Collider::default()
        };
        let forma = w.spawn((c, Transform::default(), ChildOf(corpo))).id();
        let vizinho = w.spawn(Transform::default()).id();
        let mut cena = Self {
            reg,
            sim,
            db,
            q: EditorCommandQueue::new(),
            corpo,
            forma,
            vizinho,
        };
        assert_eq!(cena.segue(), 1, "a 1.ª vista traça a argola");
        assert_eq!(cena.aneis(), 2);
        cena
    }

    fn segue(&mut self) -> usize {
        let atlas = BTreeMap::new();
        let (db, q, reg) = (&self.db, &self.q, &self.reg);
        ph2d_physics_outline::follow_phase(self.sim.world_mut(), db, &atlas, PPM, q, reg)
            .expect("aplica")
    }

    fn aneis(&self) -> usize {
        self.sim
            .world()
            .get::<ColliderOutline>(self.forma)
            .map_or(0, |o| o.rings.len())
    }

    fn doc(&self) -> Doc {
        let w = self.sim.world();
        (
            w.get::<SpritePixels>(self.corpo).copied(),
            w.get::<ColliderOutline>(self.forma).cloned(),
            *w.get::<Transform>(self.vizinho).expect("vizinho"),
        )
    }

    /// Um quadro: as fases pela ordem da shell; `pixeis` é o que o assar escreve neste quadro.
    fn quadro(&mut self, ordem: Ordem, assar: Assar, pixeis: Option<ph2d_asset::AssetId>) {
        let escreve = |c: &mut Self| {
            if let Some(id) = pixeis {
                c.sim
                    .world_mut()
                    .entity_mut(c.corpo)
                    .insert(SpritePixels(id));
            }
        };
        if matches!(assar, Assar::Silencioso) {
            escreve(self);
        }
        if matches!(ordem, Ordem::Antes) {
            self.segue();
        }
        if matches!(assar, Assar::Apply) {
            escreve(self);
        }
        if matches!(ordem, Ordem::Depois) {
            self.segue();
        }
        apply_editor_commands(self.sim.world_mut(), &self.q, &self.reg).expect("fila");
    }
}

/// Fim do quadro (`post_frame_undo_steps`): com entrada e documento mudado, a base vira um passo.
fn fim(passos: &mut Vec<(Doc, Doc)>, base: &mut Doc, agora: Doc, entrada: bool) {
    if entrada && *base != agora {
        passos.push((std::mem::replace(base, agora.clone()), agora));
    }
}

/// Mede: o Apply (quadro com entrada), um quadro sem entrada, e a acção seguinte sem relação (o
/// vizinho anda). Devolve os passos (antes, depois) e o número de anéis antes do Apply.
fn mede(ordem: Ordem, assar: Assar) -> (Vec<(Doc, Doc)>, Doc) {
    let mut c = Cena::nova();
    let disco = c.db.insert_image_rgba8(32, 32, img(|_, _| true));
    let inicio = c.doc();
    let mut base = inicio.clone();
    let mut passos = Vec::new();
    c.quadro(ordem, assar, Some(disco));
    fim(&mut passos, &mut base, c.doc(), true);
    c.quadro(ordem, assar, None);
    fim(&mut passos, &mut base, c.doc(), false);
    c.sim
        .world_mut()
        .get_mut::<Transform>(c.vizinho)
        .expect("vizinho")
        .translation
        .x = 3.0;
    c.quadro(ordem, assar, None);
    fim(&mut passos, &mut base, c.doc(), true);
    assert_eq!(
        c.aneis(),
        1,
        "no fim o disco tem um anel ({ordem:?}, {assar:?})"
    );
    (passos, inicio)
}

/// ⭐ **O Apply é UM passo que leva a imagem E o contorno; a acção seguinte não leva o retraço** —
/// com o retraço de seguir DEPOIS do Apply, e com o assar silencioso em qualquer ordem. O Ctrl+Z do
/// Apply devolve a argola E os dois anéis.
///
/// Medido (09/11) com o retraço ANTES do Apply (a ordem que a shell tinha): o passo do Apply leva a
/// imagem com os anéis VELHOS (2 anéis sobre o disco) e o retraço dobra-se no passo da acção seguinte
/// (o vizinho a andar desfaz também o contorno).
#[test]
fn o_apply_e_um_passo_com_imagem_e_contorno_e_a_acao_seguinte_nao_leva_o_retraco() {
    for (ordem, assar, certo) in [
        (Ordem::Depois, Assar::Apply, true),
        (Ordem::Depois, Assar::Silencioso, true),
        (Ordem::Antes, Assar::Silencioso, true),
        (Ordem::Antes, Assar::Apply, false),
    ] {
        let (passos, inicio) = mede(ordem, assar);
        let caso = format!("{ordem:?} × {assar:?}");
        assert_eq!(passos.len(), 2, "{caso}: o Apply e a acção seguinte");
        let (antes_apply, depois_apply) = &passos[0];
        let (antes_acao, depois_acao) = &passos[1];
        assert_eq!(
            antes_apply, &inicio,
            "{caso}: o Ctrl+Z do Apply volta ao início"
        );
        let aneis = |d: &Doc| d.1.as_ref().map_or(0, |o| o.rings.len());
        let apply_leva_o_contorno = aneis(depois_apply) == 1;
        let acao_so_o_vizinho = antes_acao.1 == depois_acao.1;
        assert_eq!(
            (apply_leva_o_contorno, acao_so_o_vizinho),
            (certo, certo),
            "{caso}: anéis depois do Apply {}, a acção seguinte mexe no contorno: {}",
            aneis(depois_apply),
            !acao_so_o_vizinho
        );
    }
}

/// ⭐ **A shell chama a fase de seguir DEPOIS do Apply das ferramentas de imagem, e só aí** (a ordem que
/// o teste acima mede). *Mutação: a chamada de volta a `inspector_commits` ⇒ vermelho.*
#[test]
fn a_shell_segue_a_imagem_depois_do_apply() {
    let raiz = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../shells/desktop/src/render_loop/"
    );
    let ler = |f: &str| std::fs::read_to_string(format!("{raiz}{f}")).expect(f);
    let apply = ler("fase_image_edit_apply.rs");
    let dreno = apply
        .find("image_edit::dispatch(")
        .expect("o dreno do Apply");
    let segue = apply.find("follow_phase(");
    assert!(
        segue.is_some_and(|s| s > dreno),
        "a fase de seguir não corre depois do Apply (fase_image_edit_apply.rs)"
    );
    assert_eq!(
        apply.matches("follow_phase(").count(),
        1,
        "uma vez por quadro"
    );
    for f in ["inspector_commits.rs", "fase_inspector_commits.rs"] {
        let s = ler(f);
        assert!(
            !s.contains("retrace_followed(") && !s.contains("follow_phase("),
            "{f} ainda segue a imagem ANTES do Apply"
        );
    }
}
