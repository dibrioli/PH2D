//! ⭐ **As cenas 45 e 46** (`PH2D_OBJECT_MODE_SMOKE=45|46`, F44 → F44b — Fill e Stroke no layout do
//! Figma, todas as linhas iguais). Um rectângulo com degradê linear (azul → roxo) de base, traço
//! escuro de base, e três camadas extra: uma SOMBRA (preenchimento deslocado), um BRILHO radial
//! (branco → transparente) e um contorno amarelo TRACEJADO por fora — selecionado, com o Inspector
//! rolado até à secção Fill. A 45 abre o ⚙ do BRILHO (uma camada extra com tipo e degradê, como a
//! base); a 46 esconde o traço de base pelo olho e abre o ⚙ do contorno amarelo (Position, pontas,
//! junções, tracejado — o «Stroke settings» de uma camada). F44c: a 45 acaba na seta branca, com
//! as alças do degradê do brilho no canvas.
//!
//! ⭐ **A cena 47** (F45 — Fill & Stroke numa só pilha, na ordem livre): o mesmo rectângulo com o
//! traço de base GROSSO, e a pilha fora da ordem antiga — a SOMBRA por baixo de tudo (o que antes
//! era impossível), o PREENCHIMENTO de base por CIMA do traço de base (cobre a metade de dentro
//! dele) e o contorno amarelo no topo (um traço por cima do preenchimento). Nenhum painel aberto: o
//! dono arrasta os ⚙.
//!
//! ⭐ **A cena 48** (os lados CAD curvos): a 47 com um canto puxado e outro preso por uma regra CAD — ver
//! [`com_regra_e_um_ponto_puxado`].
//!
//! ⭐ **A cena 49** (F47, «Above Effects»): sombra e desfoque nos Filtros; o traço de base marcado por
//! cima deles e o amarelo não — ver [`por_cima_dos_filtros`].

use crate::state::VecState;
use ph2d_editor_core::screens::hero::HeroScreen;
use ph2d_vec_scene::{
    DashPattern, GradientStop, Paint, PaintEntry, Rgba8, ShapeKind, StrokeAlign, StrokeSpec,
    VecPathId, VecScene,
};
use std::sync::atomic::{AtomicU8, Ordering};

/// O estágio — UM só, a seguir às cenas 43/44 e antes da 12 (`50..`).
pub(crate) const FIRST: u8 = crate::vector_mode_listas::LAST + 1;
pub(crate) const LAST: u8 = FIRST;
const _: () = assert!(LAST < 50);

const AZUL: Rgba8 = Rgba8::new(70, 130, 230, 255);
const ROXO: Rgba8 = Rgba8::new(150, 80, 210, 255);
const ESCURO: Rgba8 = Rgba8::new(30, 30, 40, 255);
const SOMBRA: Rgba8 = Rgba8::new(20, 20, 30, 140);
const AMARELO: Rgba8 = Rgba8::new(245, 200, 60, 255);
const BRANCO: Rgba8 = Rgba8::new(255, 255, 255, 200);
const TRANSPARENTE: Rgba8 = Rgba8::new(255, 255, 255, 0);
/// A camada que cada cena abre: o brilho (45) e o contorno amarelo (46).
const BRILHO: usize = 1;
const FORA: usize = 2;
/// A cena 48: quanto o canto de baixo à direita sai para fora (unidades do mundo; o rectângulo tem 1,2).
const PUXAO: f64 = 0.3;

static PASSO: AtomicU8 = AtomicU8::new(0);
static ROLAGENS: AtomicU8 = AtomicU8::new(0);
const MAX_ROLAGENS: u8 = 40;

/// Um quadro da cena; `Some(9)` = acabou. `qual`: `0` = a 45 · `1` = a 46 (o «Stroke settings» em
/// vez do Fill) · `2` = a 47 (a ordem livre da pilha) · `3` = a 48 (a 47 com os lados CAD).
pub fn step(
    vec: &mut VecState,
    scene: &mut VecScene,
    sim: &mut ph2d_ecs::SimWorld,
    hero: &mut HeroScreen,
    qual: u8,
) -> Option<u8> {
    let filtros = qual == 4;
    let traco = qual == 1;
    let ordem = (2..=3).contains(&qual);
    let lados_cad = qual == 3;
    let passo = PASSO.load(Ordering::Relaxed);
    let ids: Vec<VecPathId> = vec.entities.keys().copied().collect();
    match passo {
        0 => {
            crate::vector_mode::draw_shape(vec, scene, ShapeKind::Rectangle, [0.0, 0.0], 1.2);
            PASSO.store(1, Ordering::Relaxed);
        }
        1 if ids.len() == 1 => {
            let p = scene.path_mut(ids[0])?;
            p.fill = Some(Paint::Linear {
                stops: vec![GradientStop::new(0.0, AZUL), GradientStop::new(1.0, ROXO)],
                start: [-0.6, 0.0],
                end: [0.6, 0.0],
            });
            let mut base = StrokeSpec::new(ESCURO, 0.04);
            base.align = StrokeAlign::Centre;
            p.stroke = Some(base);
            let mut sombra = PaintEntry::fill(Paint::solid(SOMBRA));
            sombra.offset = [0.15, -0.15];
            let brilho = PaintEntry::fill(Paint::Radial {
                stops: vec![
                    GradientStop::new(0.0, BRANCO),
                    GradientStop::new(1.0, TRANSPARENTE),
                ],
                center: [-0.25, 0.25],
                radius: 0.6,
            });
            let mut fora = StrokeSpec::new(AMARELO, 0.05);
            fora.align = StrokeAlign::Outer;
            fora.dash = Some(DashPattern::single(0.12, 0.06));
            // A sombra por BAIXO de tudo não existe na pilha (ela pinta por cima do chão): fica
            // por cima, deslocada — o «offset print» que motivou a camada.
            p.paints = vec![sombra, brilho, PaintEntry::stroke(fora)];
            if traco {
                p.set_stroke_hidden(true);
            }
            if filtros {
                por_cima_dos_filtros(vec, scene, sim, ids[0]);
                return None;
            }
            if ordem {
                ordem_livre(p);
                if lados_cad {
                    com_regra_e_um_ponto_puxado(scene, ids[0]);
                }
                return None;
            }
            eprintln!(
                "[cena {}] rectângulo: degradê azul→roxo de base, traço escuro de base{}, sombra \
                 deslocada, brilho radial e contorno amarelo tracejado por fora",
                if traco { 46 } else { 45 },
                if traco { " ESCONDIDO pelo olho" } else { "" }
            );
            PASSO.store(2, Ordering::Relaxed);
        }
        // A forma entra em Edit um quadro depois do desenho; só então a selecção assenta nela.
        2 if vec.edit.objects.first().copied() == vec.entities.get(ids.first()?).copied() => {
            hero.gizmo
                .replace_selection(vec.entities.get(ids.first()?).copied());
            PASSO.store(3, Ordering::Relaxed);
        }
        3 if filtros => {
            return abre(hero, false, true)
                .inspect(|_| ph2d_panel_vector::open_stroke_settings(&mut hero.store));
        }
        3 => {
            let fim = abre(hero, traco, ordem);
            // F44c: a 45 acaba na seta branca, com a forma na mão — as alças do degradê do BRILHO
            // (a camada aberta) aparecem no canvas, e são elas que o arrasto edita.
            if fim.is_some() && qual == 0 {
                hero.bus
                    .push(ph2d_editor_core::action_bus::EditorAction::ToolPanelEvent(
                        ph2d_editor_core::tool::PanelEvent::Click(
                            ph2d_tool_vector::ids::VECTOR_MODE_NODE,
                        ),
                    ));
                vec.pen.select(ids.first().copied());
            }
            return fim;
        }
        _ => {}
    }
    None
}

/// ⭐ A cena 47: o traço de base grosso, e a pilha (do chão ao topo) sombra → traço de base →
/// preenchimento de base → contorno amarelo.
fn ordem_livre(p: &mut ph2d_vec_scene::VecPath) {
    use ph2d_vec_scene::{Place, StackItem};
    if let Some(s) = p.stroke.as_mut() {
        s.width = 0.16;
    }
    p.paints.remove(BRILHO);
    let por_baixo = p.move_paint(StackItem::Entry(0), Place::Below(StackItem::BaseFill));
    let cobre = p.move_paint(StackItem::BaseFill, Place::Above(StackItem::BaseStroke));
    eprintln!(
        "[cena 47] rectângulo: de baixo para cima — sombra deslocada (Fill), traço escuro grosso          de base, degradê azul→roxo de base POR CIMA do traço, contorno amarelo tracejado no topo          (ordem aplicada: {})",
        por_baixo && cobre
    );
    PASSO.store(2, Ordering::Relaxed);
}

/// ⭐ A cena 48 (report do dono 01–02/11, `img_F45/dono_linha_residual_1.png`): a 47 com o canto de baixo à direita
/// puxado para fora e o canto de baixo à esquerda PRESO por uma regra CAD — a forma passa a ter a cor dos lados (o
/// azul-claro do «livre»). O dono carrega no Smooth: os lados coloridos seguem a curva, nunca a recta antiga.
fn com_regra_e_um_ponto_puxado(scene: &mut VecScene, id: VecPathId) {
    use ph2d_vec_scene::cad::{Lei, PontoCad, RegraCad};
    let Some(p) = scene.path_mut(id) else { return };
    let Some(v) = p.verts.get_mut(1) else { return };
    v.anchor[0] += PUXAO;
    (v.in_handle, v.out_handle) = (v.anchor, v.anchor);
    let canto = p.verts[0];
    scene.regras_mut().push(RegraCad::nova(Lei::Fixo(
        PontoCad::Vert(id, canto.id),
        canto.anchor,
    )));
    scene.normaliza_ids();
    eprintln!(
        "[cena 48] a 47 com um canto puxado e outro preso por uma regra CAD: os quatro lados em azul-claro. \
         Carregue no Smooth da barra por cima da forma — as linhas seguem a curva"
    );
}

/// ⭐ A cena 49 (F47, dono 02/11): a 45 sem o brilho, com o traço de base GROSSO e marcado «Above
/// Effects», o contorno amarelo NÃO marcado, e nos Filtros um desfoque e uma sombra projectada. O
/// escuro fica nítido por cima do degradê borrado; o amarelo borra-se junto. Abre o ⚙ do traço.
fn por_cima_dos_filtros(
    vec: &VecState,
    scene: &mut VecScene,
    sim: &mut ph2d_ecs::SimWorld,
    id: VecPathId,
) {
    use ph2d_ecs::{FxOp, VecFilter};
    if let Some(p) = scene.path_mut(id) {
        p.paints.remove(BRILHO);
        if let Some(s) = p.stroke.as_mut() {
            s.width = 0.1;
            s.above_effects = true;
        }
    }
    let borra = FxOp {
        radius: 0.04,
        ..FxOp::new(FxOp::BLUR)
    };
    let sombra = FxOp {
        radius: 0.05,
        offset: [0.12, -0.12],
        opacity: 0.7,
        ..FxOp::new(FxOp::DROP_SHADOW)
    };
    let filtro = Some(VecFilter::from_ops(vec![borra, sombra]));
    ph2d_vec_entities::filter::set_filter(sim, &vec.entities, &[id], filtro);
    eprintln!(
        "[cena 49] rectângulo com Blur e Drop Shadow nos Filtros: o traço escuro de base com «Above \
         Effects» (nítido por cima) e o contorno amarelo sem (borrado com a forma). O ⚙ do traço abre-se"
    );
    PASSO.store(2, Ordering::Relaxed);
}

/// Rola o Inspector até a secção Fill & Stroke ficar no alto e abre o painel que a cena mostra (a
/// 47 não abre nenhum).
fn abre(hero: &mut HeroScreen, traco: bool, ordem: bool) -> Option<u8> {
    let painel = ph2d_editor_core::ids::VECTOR_PANEL;
    // O guião rola em LÓGICOS (`rect_for`, `panel_scroll`): pergunta pela porta física e volta.
    let area = hero
        .escala()
        .rect_to_logical(hero.panel_rect_fisico(painel)?);
    let cabeca = hero
        .hit_index
        .rect_for(ph2d_tool_vector::ids::VECTOR_SECTION_FILL)
        .filter(|r| r.h > 0.0);
    let alvo = area.y + area.h * 0.15;
    let Some(c) = cabeca else {
        let s = hero.store.panel_scroll(painel);
        hero.store.set_panel_scroll(painel, s + area.h * 0.5);
        return None;
    };
    if (c.y - alvo).abs() > area.h * 0.05 && ROLAGENS.fetch_add(1, Ordering::Relaxed) < MAX_ROLAGENS
    {
        let s = hero.store.panel_scroll(painel);
        hero.store
            .set_panel_scroll(painel, (s + c.y - alvo).max(0.0));
        return None;
    }
    if ordem {
        // `PH2D_OBJECT_MODE_GESTO=voo`: a foto do arrasto a meio — premir no ⚙ do preenchimento de
        // base e levá-lo ao topo, sem largar, pelo despacho do produto.
        if std::env::var("PH2D_OBJECT_MODE_GESTO").as_deref() == Ok("voo") {
            voo(hero)?;
        }
        return Some(9);
    }
    let i = if traco { FORA } else { BRILHO };
    hero.hit_index
        .rect_for(ph2d_panel_vector::ids::vector_paint_row_id(i))?;
    ph2d_panel_vector::open_paint_layer(&mut hero.store, i);
    Some(9)
}

/// Premir no ⚙ do preenchimento de base e levá-lo sobre o traço escuro, abaixo do meio da linha (a
/// que voa descola do vão, e o vão fica à vista).
fn voo(hero: &mut HeroScreen) -> Option<()> {
    let gear = hero
        .hit_index
        .rect_for(ph2d_panel_vector::ids::VECTOR_FILL_ROW)?;
    // A rolagem do Inspector ainda pode assentar: o gesto espera o ⚙ parado dois quadros seguidos
    // (medido 01/11: premido um quadro cedo, as linhas desceram 157 px por baixo do cursor).
    static ULTIMO_Y: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(u32::MAX);
    if ULTIMO_Y.swap(gear.y.to_bits(), Ordering::Relaxed) != gear.y.to_bits() {
        return None;
    }
    let alvo = hero
        .hit_index
        .rect_for(ph2d_panel_vector::ids::VECTOR_STROKE_SETTINGS)?;
    let x = gear.x + gear.w * 0.5;
    let arena = bumpalo::Bump::new();
    for (kind, y) in [
        (ph2d_host::PointerKind::Down, gear.y + gear.h * 0.5),
        (ph2d_host::PointerKind::Move, alvo.y + alvo.h * 0.8),
    ] {
        let event = ph2d_host::PointerEvent {
            kind,
            x,
            y,
            pressure: 1.0,
            source: ph2d_host::PointerSource::Mouse,
            button: ph2d_host::PointerButton::Primary,
            timestamp_ns: 0,
        };
        let _ = ph2d_editor_core::interaction::dispatch_pointer(
            &mut hero.store,
            &hero.hit_index,
            event,
            &arena,
        );
    }
    eprintln!("[cena 47] gesto: o ⚙ do Fill de base levado sobre o traço escuro (sem largar)");
    Some(())
}
