//! ⭐ **A cena 38** (`PH2D_OBJECT_MODE_SMOKE=38`, F29 — `docs/UI_New_and_Simple/F29_plano.md`): o
//! composto (elipse + duas linhas soldadas + três tintas do balde) arrastado sobre um quadrado verde
//! mais antigo — as tintas são filhas da rede e desenham inteiras por cima dele (Bloco A); um
//! Subtrair com o cortador transparente dá um crescente VERMELHO (Bloco B); a lâmina do Corte
//! seleccionada põe [Cortar] [Descartar] na ④ (Bloco C). Imprime `[object-mode-smoke] 38: …`.

use crate::state::VecState;
use ph2d_ecs::{Entity, SimWorld};
use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::screens::hero::HeroScreen;
use ph2d_editor_core::tool::PanelEvent;
use ph2d_tool_vector::DrawMode;
use ph2d_vec_scene::{Paint, Rgba8, ShapeKind, VecPathId, VecScene};
use std::sync::Mutex;
use std::sync::atomic::{AtomicU8, Ordering};

const VERDE: Rgba8 = Rgba8::new(40, 170, 70, 255);
const VERMELHO: Rgba8 = Rgba8::new(220, 40, 40, 255);
const AZUL: Rgba8 = Rgba8::new(40, 90, 220, 255);
const TINTAS: [Rgba8; 3] = [
    Rgba8::new(250, 210, 0, 255),
    Rgba8::new(0, 200, 220, 255),
    Rgba8::new(220, 0, 200, 255),
];
/// O centro da elipse do composto e o meio-lado dela.
const ELIPSE: [f64; 2] = [1.2, 0.6];
const RAIO: f64 = 0.8;
/// Três das quatro cunhas que o X faz na elipse: em cima, à direita, em baixo.
const SEMENTES: [[f64; 2]; 3] = [
    [ELIPSE[0], ELIPSE[1] + 0.56],
    [ELIPSE[0] + 0.56, ELIPSE[1]],
    [ELIPSE[0], ELIPSE[1] - 0.56],
];
/// O arrasto do composto: metade dele fica sobre o quadrado verde (`[-2.0, 0.6]`, meio-lado 0.7).
const ARRASTO: [f64; 2] = [-2.8, 0.0];

/// O estágio do `smoke_step` — UM só, no buraco entre a cena 11 (`40..=41`) e a 12 (`50..`): os
/// passos contam-se em [`PASSO`].
pub(crate) const FIRST: u8 = 42;
pub(crate) const LAST: u8 = FIRST;

const QUADRADO: u8 = 0;
const ELIPSE_S: u8 = 1;
const LINHA_A: u8 = 2;
const LINHA_B: u8 = 3;
const SOLDA: u8 = 4;
const BALDE: u8 = 5;
const DEPOSITA: u8 = 6;
const ARRASTA: u8 = 7;
const CIRCULO: u8 = 8;
const CORTADOR: u8 = 9;
const SUBTRAI: u8 = 10;
const QUADRADO_AZUL: u8 = 11;
const CORTE: u8 = 12;
const LAMINA: u8 = 13;
const FIM: u8 = 14;

/// O passo dentro da cena.
static PASSO: AtomicU8 = AtomicU8::new(QUADRADO);
/// Quadros de espera do passo corrente.
static ESPERA: AtomicU8 = AtomicU8::new(0);

/// O que a cena lembra entre quadros, para a linha do fim.
struct Memoria {
    quadrado: VecPathId,
    rede: VecPathId,
    tintas: Vec<VecPathId>,
    antes: Vec<f64>,
    depois: Vec<f64>,
    z_depois: String,
    pais: String,
    booleana: String,
}

static MEM: Mutex<Memoria> = Mutex::new(Memoria {
    quadrado: 0,
    rede: 0,
    tintas: Vec::new(),
    antes: Vec::new(),
    depois: Vec::new(),
    z_depois: String::new(),
    pais: String::new(),
    booleana: String::new(),
});

/// `true` depois de `n` chamadas seguidas (a espera é zerada a cada troca de estágio).
fn esperou(n: u8) -> bool {
    let k = ESPERA.fetch_add(1, Ordering::Relaxed);
    k >= n
}

fn clique(hero: &mut HeroScreen, id: ph2d_editor_core::NodeId) {
    hero.bus
        .push(EditorAction::ToolPanelEvent(PanelEvent::Click(id)));
}

/// O Edit já passou à forma mais nova: só então a seleção da caneta não é reduzida por ele.
fn assentou(vec: &VecState) -> bool {
    vec.edit.objects.first().copied() == vec.entities.values().last().copied()
}

/// O caminho mais novo (os ids crescem).
fn mais_novo(vec: &VecState) -> Option<VecPathId> {
    vec.entities.keys().last().copied()
}

/// A área do contorno primário (sobre as âncoras), no espaço do caminho.
fn area(scene: &VecScene, id: VecPathId) -> f64 {
    let Some(p) = scene.path(id) else {
        return f64::NAN;
    };
    let v = &p.verts;
    let s: f64 = (0..v.len())
        .map(|i| {
            let (a, b) = (v[i].anchor, v[(i + 1) % v.len()].anchor);
            a[0] * b[1] - b[0] * a[1]
        })
        .sum();
    (s / 2.0).abs()
}

fn pai_de(sim: &SimWorld, vec: &VecState, id: VecPathId) -> Option<VecPathId> {
    let bits = *vec.entities.get(&id)?;
    let pai = sim
        .world()
        .get::<ph2d_ecs::ChildOf>(Entity::from_bits(bits))?
        .parent()
        .to_bits();
    vec.entities
        .iter()
        .find(|(_, b)| **b == pai)
        .map(|(k, _)| *k)
}

/// A pilha de desenho do Vetor, pela porta do produto.
fn pilha(sim: &mut SimWorld) -> Vec<VecPathId> {
    use ph2d_ecs::scene::{HierarchySnapshot, HierarchyWalkState, build_hierarchy_snapshot};
    let mut state = HierarchyWalkState::new(sim.world_mut());
    let mut snap = HierarchySnapshot::default();
    build_hierarchy_snapshot(sim.world(), &mut state, &mut Vec::new(), &mut snap);
    ph2d_vec_entities::entities::z_order(sim.world(), &snap)
}

fn fill_de(scene: &VecScene, id: VecPathId) -> String {
    scene
        .path(id)
        .map_or_else(|| "?".to_owned(), |p| format!("{:?}", p.fill))
}

/// Um quadro da cena 38; `Some(9)` = acabou.
#[allow(clippy::too_many_lines)]
pub fn step(
    vec: &mut VecState,
    scene: &mut VecScene,
    sim: &mut SimWorld,
    hero: &mut HeroScreen,
) -> Option<u8> {
    let n = vec.entities.len();
    let mut mem = MEM.lock().ok()?;
    let next = match PASSO.load(Ordering::Relaxed) {
        QUADRADO if n == 0 => {
            crate::vector_mode::draw_shape(vec, scene, ShapeKind::Rectangle, [-2.0, 0.6], 0.7);
            ELIPSE_S
        }
        ELIPSE_S if n == 1 => {
            mem.quadrado = mais_novo(vec)?;
            scene.path_mut(mem.quadrado)?.fill = Some(Paint::solid(VERDE));
            crate::vector_mode::draw_shape(vec, scene, ShapeKind::Ellipse, ELIPSE, RAIO);
            LINHA_A
        }
        LINHA_A if n == 2 => {
            let id = mais_novo(vec)?;
            scene.path_mut(id)?.fill = None;
            let d = 1.0;
            let (a, b) = (
                [ELIPSE[0] - d, ELIPSE[1] - d],
                [ELIPSE[0] + d, ELIPSE[1] + d],
            );
            crate::vector_mode::draw_line(vec, scene, a, b);
            LINHA_B
        }
        LINHA_B if n == 3 => {
            let d = 1.0;
            let (a, b) = (
                [ELIPSE[0] - d, ELIPSE[1] + d],
                [ELIPSE[0] + d, ELIPSE[1] - d],
            );
            crate::vector_mode::draw_line(vec, scene, a, b);
            SOLDA
        }
        SOLDA if n == 4 && assentou(vec) => {
            let q = mem.quadrado;
            let tres: Vec<VecPathId> = vec.entities.keys().copied().filter(|k| *k != q).collect();
            vec.pen.select_many(&tres);
            clique(hero, ph2d_panel_vector::ids::VECTOR_PATH_WELD);
            BALDE
        }
        BALDE if n == 2 => {
            let q = mem.quadrado;
            mem.rede = vec.entities.keys().copied().find(|k| *k != q)?;
            clique(hero, ph2d_tool_vector::ids::VECTOR_MODE_BUCKET);
            DEPOSITA
        }
        DEPOSITA if vec.draw_config.mode == DrawMode::Bucket => {
            let cache = vec.bucket_cache.as_ref()?;
            let hits: Vec<_> = SEMENTES
                .iter()
                .map(|s| crate::bucket::hover(cache, *s))
                .collect::<Option<_>>()?;
            for (hit, cor) in hits.iter().zip(TINTAS) {
                let id = crate::bucket::deposit(sim, scene, &vec.entities, hit, cor);
                #[allow(clippy::cast_possible_truncation)]
                let seed = [hit.seed[0] as f32, hit.seed[1] as f32];
                vec.bucket_new.push((id, seed, hit.ancoras.clone()));
                mem.tintas.push(id);
            }
            vec.bucket_cache = None;
            ARRASTA
        }
        // ~4 s parado antes do arrasto: a rajada da foto apanha o «antes», já sem os avisos do arranque.
        ARRASTA if n == 5 && esperou(240) => {
            mem.antes = mem.tintas.iter().map(|t| area(scene, *t)).collect();
            let bits = *vec.entities.get(&mem.rede)?;
            ph2d_vec_entities::transform::translate_shape_world(
                sim,
                Entity::from_bits(bits),
                ARRASTO,
            );
            CIRCULO
        }
        CIRCULO if esperou(4) => {
            mem.depois = mem.tintas.iter().map(|t| area(scene, *t)).collect();
            let z = pilha(sim);
            let (q, r) = (mem.quadrado, mem.rede);
            let nome = |id: &VecPathId| match id {
                id if *id == q => "quadrado".to_owned(),
                id if *id == r => format!("rede#{id}"),
                id => format!("tinta#{id}"),
            };
            mem.z_depois = z.iter().map(nome).collect::<Vec<_>>().join(" < ");
            mem.pais = mem
                .tintas
                .iter()
                .map(|t| format!("{t}→{:?}", pai_de(sim, vec, *t)))
                .collect::<Vec<_>>()
                .join(" ");
            crate::vector_mode::draw_shape(vec, scene, ShapeKind::Ellipse, [2.4, 0.6], 0.6);
            CORTADOR
        }
        CORTADOR if n == 6 => {
            let id = mais_novo(vec)?;
            scene.path_mut(id)?.fill = Some(Paint::solid(VERMELHO));
            crate::vector_mode::draw_shape(vec, scene, ShapeKind::Rectangle, [3.0, 0.6], 0.7);
            SUBTRAI
        }
        SUBTRAI if n == 7 && assentou(vec) => {
            let ids: Vec<VecPathId> = vec.entities.keys().rev().take(2).copied().collect();
            scene.path_mut(ids[0])?.fill = None;
            vec.pen.select_many(&[ids[1], ids[0]]);
            clique(hero, ph2d_panel_vector::ids::VECTOR_BOOL_SUBTRACT);
            QUADRADO_AZUL
        }
        QUADRADO_AZUL if esperou(if n == 7 { 60 } else { 3 }) => {
            let id = mais_novo(vec)?;
            mem.booleana = format!("#{id} n={n} fill={}", fill_de(scene, id));
            crate::vector_mode::draw_shape(vec, scene, ShapeKind::Rectangle, [0.0, -1.2], 0.5);
            CORTE
        }
        CORTE if esperou(2) => {
            let id = mais_novo(vec)?;
            scene.path_mut(id)?.fill = Some(Paint::solid(AZUL));
            clique(hero, ph2d_tool_vector::ids::VECTOR_MODE_CUT);
            LAMINA
        }
        LAMINA if vec.draw_config.mode == DrawMode::Cut => {
            vec.pen
                .on_press_blade(scene, [-1.0, -1.2], 0.01, &mut |q| q);
            let lamina = vec.pen.selected();
            vec.pen.on_release();
            vec.pen.on_press_blade(scene, [1.0, -1.2], 0.01, &mut |q| q);
            vec.pen.on_release();
            vec.pen.finish();
            vec.cut_pending = lamina;
            vec.pen.select(lamina);
            FIM
        }
        FIM => {
            let barra = hero
                .store
                .tool_bars()
                .map(|b| b.actions.iter().flatten().map(|x| x.id).collect::<Vec<_>>());
            let ids = barra.clone().unwrap_or_default();
            let cortar = ids.contains(&ph2d_panel_vector::ids::VECTOR_CUT_APPLY);
            let descartar = ids.contains(&ph2d_panel_vector::ids::VECTOR_CUT_DISCARD);
            if !(cortar && descartar) && !esperou(90) {
                return None;
            }
            eprintln!(
                "[object-mode-smoke] 38: pais das tintas (rede #{}): {} · z: {} · área antes {:?} \
                 depois {:?} · booleana {} · ④ do Corte: {} botões, Cortar={cortar} \
                 Descartar={descartar}",
                mem.rede,
                mem.pais,
                mem.z_depois,
                mem.antes,
                mem.depois,
                mem.booleana,
                ids.len(),
            );
            return Some(9);
        }
        _ => return None,
    };
    ESPERA.store(0, Ordering::Relaxed);
    PASSO.store(next, Ordering::Relaxed);
    None
}
