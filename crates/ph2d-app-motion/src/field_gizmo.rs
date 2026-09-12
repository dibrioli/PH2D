//! O gizmo de canvas de um **FIELD ESPACIAL** dos Motion Nodes (`field.box`, …).
//!
//! Um field espacial mascara/pesa as instâncias por POSIÇÃO, e a posição vive no mesmo
//! espaço de MUNDO que os sprites (as instâncias compõem no canvas via
//! `SpriteRenderer::render_with_extra`). Arrastar `center_x`/`center_y` num slider é
//! caçar a rotação; o idioma dos apps pro (C4D/Cavalry/Houdini mograph) é uma **alça na
//! tela** — mover/girar/escalar o field onde a ação está.
//!
//! # É o ESPELHO do [`ph2d_app_flip::selection_gizmo`], com um sink diferente
//!
//! O Flip já resolveu "um drag de gizmo de sprite que escreve num sink que **não é** um
//! `Transform` de entidade": a pose de uma chave (`FlipPose`) e a geometria de uma
//! seleção (`FlipSelection`) são [`ph2d_editor::GizmoTarget`]s próprios, com espaço de id
//! keyed, reconhecidos ANTES do caminho genérico de gizmo. Este módulo é mais um:
//! [`ph2d_editor::GizmoTarget::MotionField`], cujo apply escreve os **params do NÓ**
//! (`center_x`/`center_y`/`rotation`/`width`/`height` via `Graph::set_param`).
//!
//! **Isolamento por construção — a resposta ao *"não vai atrapalhar os sprites?"*:** um
//! field é um NÓ do grafo, não uma entidade ECS, então o gizmo de sprite (`hero.gizmo.view`,
//! chaveado em `hero.gizmo.selection` = bits de entidade) fica **intocado**. A view do
//! field mora no seu próprio slot (`hero.gizmo.field_view`), publicada SÓ com a tool Motion
//! ativa + um field espacial selecionado — modalidade que os torna mutuamente exclusivos
//! (você manipula sprites na tool de move/select, com a Motion desligada; ali o field não
//! tem hit-region nenhuma). Os dois nem compartilham modelo de seleção.
//!
//! **Seed = sample:** a caixa (`field_view`) e a semente do drag (`field_gizmo_down`) leem
//! os params pela MESMA porta que o painel de params usa
//! ([`crate::motion_bridge::params::param_value`], override→default), então a
//! alça concorda com os sliders. O writeback ([`params_from`]) é o inverso exato da semente
//! ([`seed_start`]) — um round-trip sob transform identidade devolve os mesmos params (a
//! lição recorrente `feedback_derived_coordinate_seed_must_match_sample`).

use ph2d_editor::screens::layout::CenterSplit;
use ph2d_editor::{
    GizmoCamera, GizmoDragState, GizmoModifiers, GizmoSnap, GizmoView, TransformSnapshot,
};
use ph2d_host::WindowSize;
use ph2d_nodegraph::graph::{Graph, NodeId};
use ph2d_nodegraph::node::NodeTypeId;
use ph2d_render::Camera2d;

use crate::motion_state::MotionState;

/// As dims `(w, h)` que a CENA de fato ocupa — o sub-retângulo do split (a porta única
/// [`CenterSplit::scene_viewport`]) quando a tool Motion divide o centro, ou a janela cheia
/// fora do split. ⚠️ **Todo mapeamento mundo↔tela do chrome da cena (a grade do mundo, o
/// gizmo de field e o drag dele) TEM de usar isto**, casando com o `uniform_for_subrect` +
/// `set_viewport` que o render usa (present.rs) — senão a cena renderiza na banda e o
/// chrome projeta a janela cheia, e um ponto de mundo cai em dois lugares (o drift crônico
/// do Motion, 2026-07-25). O sub-retângulo é ancorado em `(0,0)`, então só as DIMS mudam.
#[must_use]
pub fn scene_window_wh(center_split: CenterSplit, window: WindowSize) -> (f32, f32) {
    let (w, h) = (window.width as f32, window.height as f32);
    center_split
        .scene_viewport(w, h)
        .map_or((w, h), |r| (r[2], r[3]))
}

/// **The [`WindowSize`] the SCENE's world→screen camera must project into** — the porta
/// única for any WORLD geometry drawn into the Vello scene (the vector shapes, above all).
///
/// Under the Motion split the scene fills a top-left sub-rectangle (`scene_viewport` is
/// always `[0, 0, w, h·t]`), and the motion instances (`set_viewport`) and the world grid
/// already project it. A vector shape projected with the FULL window drifts off them —
/// shifted and shrunk — which is why a `motion.path`'s walkers sat on a displaced copy of
/// the drawn curve. Feed this to `Camera2d::world_to_screen_affine` and the curve lands
/// under its walkers. Not split ⇒ the full window ⇒ byte-identical.
pub fn scene_camera_window(center_split: CenterSplit, window: WindowSize) -> WindowSize {
    let (w, h) = scene_window_wh(center_split, window);
    WindowSize::new(w as u32, h as u32)
}

/// **O PAN da câmera, com o mundo-por-pixel da CENA** — a porta que faltava (report do
/// Enio, 2026-08-25: *«no modo motion a imagem de referência sofre um drift no pan com o
/// mouse»*).
///
/// ⚠️ **É a MESMA doença que o [`scene_camera_window`] cura, no terceiro consumidor.** O
/// doc do [`scene_window_wh`] já dizia *«todo mapeamento mundo↔tela do chrome da cena TEM
/// de usar isto»*, e o `CenterSplit::scene_viewport` já dizia que a projeção **muda** sob o
/// split (`view_proj_for_subrect(w,h)` ≡ `view_proj(WindowSize{w,h})`, não é um mero
/// recorte). A grade e o gizmo foram postos nessa porta em 2026-07-25; **o pan não**, e
/// continuou a derivar o mundo-por-pixel da JANELA CHEIA.
///
/// A conta do defeito: sob um split de fração `t`, um pixel de tela vale
/// `height_world / (h·t)` metros, e o pan movia `height_world / h`. ⇒ **a cena andava `t`
/// vezes o que o cursor andava** — a `t = 0,6`, arrastar 100 px movia o mundo 60, e a
/// imagem ficava para trás do rato. Fora do split, `t` não existe e a conta é a de sempre,
/// bit a bit.
///
/// *A regra estava escrita há um mês; o que faltava era o pan estar no caminho dela.*
pub fn pan_scene_camera(
    camera: &mut ph2d_render::Camera2d,
    center_split: CenterSplit,
    window: WindowSize,
    dx_px: f32,
    dy_px: f32,
) {
    let (w, h) = scene_window_wh(center_split, window);
    camera.pan_screen_delta(dx_px, dy_px, w, h);
}

/// Como a caixa do gizmo mapeia para os params de TAMANHO de um field — a única parte da
/// spec que varia por FORMA. `Rect` (a box) tem duas extensões cheias independentes;
/// `Disk` (o radial sweep) tem um `radius` (meia-extensão, isotrópico) e a caixa do gizmo
/// é o quadrado que o circunscreve. Todo o resto (centro + rotação) é comum, então o radar
/// **herda** a máquina do gizmo trocando só isto.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum FieldSize {
    /// Duas extensões CHEIAS independentes (meia = extensão/2): `field.box`.
    Rect {
        width: &'static str,
        height: &'static str,
    },
    /// Um `radius` (meia-extensão; a caixa do gizmo é o quadrado circunscrito, meia =
    /// `[radius, radius]`): `field.radial_sweep`. Isotrópico — todo handle redimensiona
    /// uniforme.
    Disk { radius: &'static str },
}

impl FieldSize {
    /// A meia-extensão `[hx, hy]` do gizmo a partir dos params do field. `Rect` divide cada
    /// extensão cheia; `Disk` é o quadrado `[radius, radius]` que circunscreve o disco.
    #[must_use]
    pub fn half(self, p: impl Fn(&str) -> f32) -> [f32; 2] {
        match self {
            Self::Rect { width, height } => [(p(width) * 0.5).abs(), (p(height) * 0.5).abs()],
            Self::Disk { radius } => {
                let r = p(radius).abs();
                [r, r]
            }
        }
    }

    /// Escreve a meia-extensão nova (a `intrinsic_half` congelada × a `scale` do gizmo) de
    /// volta nos params deste tamanho. `Rect` escreve cada extensão cheia independente;
    /// `Disk` é isotrópico — MEDIA as escalas dos dois eixos, então um arrasto de CANTO
    /// (escala igual) redimensiona exato e um arrasto de BORDA a meio-passo (a caixa se
    /// re-quadra ao novo raio a cada frame — sem salto-de-volta). Um field é simétrico ⇒
    /// `abs` (flip é no-op, uma extensão nunca é negativa).
    fn write(self, g: &mut Graph, node: NodeId, intrinsic_half: [f32; 2], scale: [f32; 2]) {
        match self {
            Self::Rect { width, height } => {
                g.set_param(node, width, (2.0 * intrinsic_half[0] * scale[0]).abs());
                g.set_param(node, height, (2.0 * intrinsic_half[1] * scale[1]).abs());
            }
            Self::Disk { radius } => {
                let s = (scale[0].abs() + scale[1].abs()) * 0.5;
                g.set_param(node, radius, (intrinsic_half[0] * s).abs());
            }
        }
    }
}

/// Os nomes dos params de um field espacial que o gizmo dirige. Uma tabela por tipo de
/// field — cada um acrescenta uma [`FieldGizmoSpec`], sem tocar a máquina do gizmo (a
/// forma vive no [`FieldSize`]). `center`/`rotation`/`size` são a "Coordinates" da família
/// (doc 63 §0.1); um field NÃO-espacial (`index_range`, por rank) não tem spec e não ganha
/// gizmo.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct FieldGizmoSpec {
    pub center_x: &'static str,
    pub center_y: &'static str,
    /// Rotação em GRAUS, ou **`None` quando a coisa não tem ângulo**.
    ///
    /// ⛔⛔ **`None` é uma alça de rotação INERTE, e é por isso que nenhum nó do repo o usa
    /// hoje.** O `force.vortex` e o `force.attractor` põem-se no mundo e **não têm** param de
    /// ângulo (girar um vórtice em torno do próprio centro não move um texel), então uma spec
    /// deles teria de o pôr a `None` — e o artista arrastaria a argola de rodar sem nada
    /// acontecer. A cura certa é a [`GizmoView`](ph2d_editor::GizmoView) saber **suprimir** a
    /// argola, e o preço está MEDIDO: **28 sítios de construção** dela, em crates de outras
    /// linhas. ⇒ nomeado, não contrabandeado ([doc 108](../../../docs/Motion%20Nodes/108_ciclo_5_simulacao.md) §2).
    pub rotation: Option<&'static str>,
    /// Como o tamanho mapeia para os params (retângulo × disco).
    pub size: FieldSize,
}

/// A spec do `field.box`: retângulo de extensões cheias, rotação em graus.
const BOX_SPEC: FieldGizmoSpec = FieldGizmoSpec {
    center_x: "center_x",
    center_y: "center_y",
    rotation: Some("rotation"),
    size: FieldSize::Rect {
        width: "width",
        height: "height",
    },
};

/// A spec do `field.radial_sweep`: disco de raio único, rotação em graus (ver
/// `ph2d-node-field-radial-sweep`). O gizmo gira o setor e a escala dirige o `radius`.
const RADIAL_SWEEP_SPEC: FieldGizmoSpec = FieldGizmoSpec {
    center_x: "center_x",
    center_y: "center_y",
    rotation: Some("rotation"),
    size: FieldSize::Disk { radius: "radius" },
};

/// ⭐⭐⭐ **A spec do `motion.falloff`** (ciclo 4, W1 — [doc 107](../../../docs/Motion%20Nodes/107_ciclo_4_foco_os_campos.md)).
///
/// ⛔ **Ele é o TERCEIRO campo espacial e era o único sem alça — sendo o que o artista
/// encontra primeiro:** é o único do grupo no namespace `motion.*`, é o que os deformadores
/// leem, e é o da primeira página de qualquer tutorial. Arrastar `center_x`/`center_y` num
/// slider é caçar a rotação, e era isso que ele obrigava.
///
/// ⭐ **Uma spec serve as TRÊS formas dele, e a geometria já encaixava:** a
/// [`FieldSize::Disk`] devolve a meia-extensão `[r, r]`, que é **o disco** do `Circle`, **o
/// quadrado** do `Rect` (a lei dele é Chebyshev, `max(|dx|,|dy|)/radius`) e **o vão** do
/// `Linear` (a rampa corre em `±radius`). ⚠️ E a escala isotrópica do `Disk` é a certa nas
/// três: o nó tem **um** `radius`, não dois.
///
/// ⚠️⚠️ **A alça de ROTAÇÃO fica, e o painel esconde a linha dela num círculo — a divergência
/// é deliberada e medida.** Concordar com o painel (não escrever `rotation` quando a linha
/// está gateada) faria a caixa **girar na tela e voltar atrás no quadro seguinte**, porque a
/// view lê o param que o arrasto não escreveu: *um controlo que se mexe e desfaz é pior que um
/// cujo efeito espera pelo modo*. O param é real e guardado — num `Circle` ele não move um
/// texel (o campo é isotrópico) e passa a valer no instante em que a forma vira `Rect` ou
/// `Linear`. ⛔ A terceira saída — suprimir a alça — pedia um campo novo na
/// [`GizmoView`](ph2d_editor::GizmoView), que é partilhada por **todos** os gizmos do app.
const FALLOFF_SPEC: FieldGizmoSpec = FieldGizmoSpec {
    center_x: "center_x",
    center_y: "center_y",
    rotation: Some("rotation"),
    size: FieldSize::Disk { radius: "radius" },
};

/// A [`FieldGizmoSpec`] de um tipo de nó, ou `None` se o nó **não** é um field
/// ESPACIAL (o `index_range` é por rank, sem geometria; o `combine` compõe dois fields,
/// sem geometria própria). Porta única — a view, o down e o gate perguntam à mesma.
#[must_use]
/// ⭐⭐⭐ **A spec do `sim.collide`** (ciclo 5, W1 — [doc 108](../../../docs/Motion%20Nodes/108_ciclo_5_simulacao.md)).
///
/// ⛔ **O colisor é literalmente uma coisa que se põe num sítio** — um chão, um prato, uma caixa
/// — e punha-se com dois sliders. É o achado do ciclo 4 um grupo adiante, e aqui morde mais.
///
/// ⚠️⚠️ **Ele é o primeiro nó cuja spec depende dos PARAMS e não só do tipo:** as quatro formas
/// (`Plane · Disc · Bowl · Box`) medem-se com params **diferentes** — a caixa tem
/// `box_width`/`box_height`, o prato e a tigela têm `radius`. O `motion.falloff` escapou a isto
/// porque uma `Disk` servia as três formas dele; aqui não serve, e é por isso que a
/// [`spec_for`] passou a receber o leitor de params.
///
/// ⚠️ **O `Plane` recebe a caixa do `Box`** de propósito: um plano é infinito e não tem extensão
/// para agarrar, mas o `angle` e o `height` dele **são** o que a alça move — e a alça de tamanho
/// escreve num param que aquela forma não lê, o que é inerte e não mentiroso (o artista vê a
/// caixa, arrasta o canto, e a linha do chão não muda de sítio). ⛔ A alternativa — não dar spec
/// nenhuma ao `Plane` — tiraria também o MOVER e o GIRAR, que são exactamente o que ele precisa.
fn collide_spec(p: &dyn Fn(&str) -> f32) -> FieldGizmoSpec {
    /// `0` Plane · `1` Disc · `2` Bowl · `3` Box — a mesma escada do `shape` do nó.
    const BOX: f32 = 3.0;
    FieldGizmoSpec {
        center_x: "center_x",
        center_y: "center_y",
        rotation: Some("angle"),
        size: if (p("shape") - BOX).abs() < 0.5 {
            FieldSize::Rect {
                width: "box_width",
                height: "box_height",
            }
        } else {
            FieldSize::Disk { radius: "radius" }
        },
    }
}

/// A [`FieldGizmoSpec`] de um tipo de nó, ou `None` se o nó **não** é um field
/// ESPACIAL (o `index_range` é por rank, sem geometria; o `combine` compõe dois fields,
/// sem geometria própria). Porta única — a view, o down e o gate perguntam à mesma.
///
/// ⚠️ **Recebe o leitor de PARAMS desde o ciclo 5:** um nó cuja forma muda os params de tamanho
/// (o `sim.collide`) não é exprimível por uma tabela indexada só pelo tipo.
#[must_use]
pub fn spec_for(type_id: NodeTypeId, p: &dyn Fn(&str) -> f32) -> Option<FieldGizmoSpec> {
    if type_id == NodeTypeId::of("field.box") {
        Some(BOX_SPEC)
    } else if type_id == NodeTypeId::of("field.radial_sweep") {
        Some(RADIAL_SWEEP_SPEC)
    } else if type_id == NodeTypeId::of("motion.falloff") {
        Some(FALLOFF_SPEC)
    } else if type_id == NodeTypeId::of("sim.collide") {
        Some(collide_spec(p))
    } else {
        None
    }
}

/// Embrulha graus em `[-180, 180]` — a faixa declarada do param `rotation` do field
/// (`PARAM_HINTS`). Puro aritmético (`rem_euclid` é o módulo do livro; sem transcendental,
/// HR-5). Não afeta o drag vivo (que usa `start_transform` + cursor); só normaliza o valor
/// ARMAZENADO para bater com o slider do painel.
#[must_use]
fn wrap180(deg: f32) -> f32 {
    (deg + 180.0).rem_euclid(360.0) - 180.0
}

/// Monta a [`GizmoView`] de um field a partir dos seus params — a MESMA álgebra do gizmo
/// de sprite (caixa não-rotacionada center±half, rotação carregada à parte e aplicada em
/// torno do pivô). Para um field o pivô É o centro (âncora zero), então
/// `bbox = center ± half` direto.
// Um builder de `GizmoView` legitimamente carrega a pose (5) + câmera + dims da cena +
// cursor; espelha o `gizmo_view_from` do `vec_gizmo_view` (mesmo domínio). ⚠️ `win_w`/
// `win_h` são as dims da CENA ([`scene_window_wh`]), NÃO da janela cheia — é o que casa o
// gizmo com o `set_viewport` do render sob o split (o fix do drift). O `canvas` (scissor)
// usa as mesmas dims, então o gizmo é recortado na banda em vez de invadir o painel do grafo.
#[allow(clippy::too_many_arguments)]
#[must_use]
fn view_from_params(
    cx: f32,
    cy: f32,
    half: [f32; 2],
    rot_deg: f32,
    camera: &Camera2d,
    win_w: f32,
    win_h: f32,
    last_pointer: (f32, f32),
) -> GizmoView {
    GizmoView {
        bbox_min_world: [cx - half[0], cy - half[1]],
        bbox_max_world: [cx + half[0], cy + half[1]],
        pivot_world: [cx, cy],
        pivot_tool_active: false,
        rotation: rot_deg.to_radians(),
        camera_center: camera.center,
        camera_height_world: camera.height_world,
        window_w: win_w,
        window_h: win_h,
        canvas: ph2d_editor::zones::Rect::new(0.0, 0.0, win_w, win_h),
        cursor_screen: Some(last_pointer),
    }
}

/// O `TransformSnapshot` de PARTIDA (Down) de um field na linguagem do gizmo: centro na
/// translation, rotação em radianos, escala `1`. A meia-extensão intrínseca (pré-escala) é
/// a `half` da [`FieldSize`], congelada à parte pelo chamador — o writeback ([`FieldSize::write`])
/// multiplica a escala do gizmo por ela.
#[must_use]
pub fn seed_start(cx: f32, cy: f32, rot_deg: f32) -> TransformSnapshot {
    TransformSnapshot {
        translation: [cx, cy],
        rotation: rot_deg.to_radians(),
        scale: [1.0, 1.0],
    }
}

/// O nó selecionado no grafo SE ele for um field espacial (+ sua [`FieldGizmoSpec`]).
/// `None` quando: nada selecionado (ou multi-seleção), o nó não é um field espacial, ou o
/// nó sumiu do grafo. Porta única para a view, o down e o gate.
#[must_use]
pub fn selected_field(motion: &MotionState) -> Option<(NodeId, FieldGizmoSpec)> {
    let nid = crate::motion_bridge::params::selected_motion_node().map(NodeId)?;
    let type_id = motion.doc.graph.node(nid)?.type_id();
    let p = |name: &str| crate::motion_bridge::params::param_value(motion, nid, name);
    Some((nid, spec_for(type_id, &p)?))
}

/// A [`GizmoView`] do field espacial selecionado, ou `None`. O chamador (render_loop) já
/// gateia a tool Motion ativa; aqui recusa quando não há um field espacial selecionado. Lê
/// os params pela MESMA porta do painel (`param_value`), então a caixa concorda com os
/// sliders (seed = sample).
#[must_use]
pub fn field_view(
    motion: &MotionState,
    camera: &Camera2d,
    win_w: f32,
    win_h: f32,
    last_pointer: (f32, f32),
) -> Option<GizmoView> {
    let (nid, spec) = selected_field(motion)?;
    let p = |name: &str| crate::motion_bridge::params::param_value(motion, nid, name);
    let half = spec.size.half(p);
    Some(view_from_params(
        p(spec.center_x),
        p(spec.center_y),
        half,
        spec.rotation.map_or(0.0, p),
        camera,
        win_w,
        win_h,
        last_pointer,
    ))
}

/// O arrasto de gizmo de field em curso: o estado genérico do gizmo + o NÓ alvo + a spec
/// e a meia-extensão intrínseca congelada no Down. `Copy` como o [`GizmoDragState`] (nada
/// de `Vec`, ≠ `FlipSelectionDrag`).
#[derive(Copy, Clone, Debug)]
pub struct FieldGizmoDrag {
    pub drag: GizmoDragState,
    pub node: NodeId,
    pub spec: FieldGizmoSpec,
    pub intrinsic_half: [f32; 2],
}

/// O núcleo do writeback de um arrasto de field, SEM a janela: avança o cursor pelo drag
/// (o contador de voltas do Rotate mora aí), recomputa o TRS pelo motor canônico e escreve
/// os params do NÓ — centro + rotação (universais) + o(s) param(s) de TAMANHO pela porta
/// [`FieldSize::write`] (retângulo × disco). ⚠️ **A única escrita é `motion.doc.graph.set_param`**
/// — esta função nem recebe um `SimWorld`, então por CONSTRUÇÃO não pode tocar nenhum
/// `Transform` de entidade (a prova de que o gizmo de field não interfere na manipulação de
/// sprites). Devolve o `TransformSnapshot` computado. Porta única do
/// [`crate::App::field_gizmo_move`] e do gate.
pub fn apply_field_drag(
    motion: &mut MotionState,
    fgd: &mut FieldGizmoDrag,
    cursor: (f32, f32),
    cam: &GizmoCamera,
    mods: GizmoModifiers,
    snap: GizmoSnap,
) -> TransformSnapshot {
    fgd.drag.advance_cursor(cursor, cam);
    let new_t = ph2d_editor::compute_gizmo_transform(&fgd.drag, cam, mods, snap, None);
    let g = &mut motion.doc.graph;
    g.set_param(fgd.node, fgd.spec.center_x, new_t.translation[0]);
    g.set_param(fgd.node, fgd.spec.center_y, new_t.translation[1]);
    if let Some(r) = fgd.spec.rotation {
        g.set_param(fgd.node, r, wrap180(new_t.rotation.to_degrees()));
    }
    fgd.spec
        .size
        .write(g, fgd.node, fgd.intrinsic_half, new_t.scale);
    motion.pump.mark_dirty();
    new_t
}

/// Os gates da LEI deste módulo — eles seguem o SUJEITO, não o ficheiro (HOWTO §1.2).
#[cfg(test)]
#[path = "field_gizmo_tests.rs"]
mod tests;
