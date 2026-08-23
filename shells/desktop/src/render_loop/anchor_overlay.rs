//! **Os marcadores das âncoras no canvas** (spec
//! [`07_named_anchors.md`](../../../../docs/Sprite_projeto/07_named_anchors.md) §7.6).
//!
//! # Por que isto não é opcional
//!
//! ⚠️ Sem marcador, a §12 é um formulário: o artista escreve `pos = (28, -4)` e **não acontece
//! nada na tela**. Uma posição que não se vê não é uma posição — é a mesma lição que o realce de
//! seleção do Flip pagou («uma seleção que não se VÊ não existe»). É o marcador que fecha o
//! ciclo autoria → efeito, e sem ele a seção seria entregue morta.
//!
//! # A linguagem, derivada da FORMA
//!
//! - **Socket** (sem área) → uma **cruz**.
//! - **Slice** (com área) → a cruz mais o **retângulo**.
//! - **Região 9-slice** (área + miolo) → mais o **retângulo interno**.
//!
//! A cor vem do **hash do nome**, por isso é estável entre sessões e distinta entre âncoras: duas
//! âncoras coincidentes continuam a distinguir-se.
//!
//! ⚠️ **A espessura sai daqui em px de TELA, sob `Affine::IDENTITY`.** No Vello o transform do
//! `stroke` **multiplica** a espessura: entregar o afim mundo→tela como transform transforma
//! 2 px em `2 × px_por_unidade_de_mundo`. É o defeito que o realce do Flip apanhou num smoke em
//! 2026-07-13, e a razão de os pontos serem transformados e o traço não.
//!
//! # A decisão que a spec não fixa
//!
//! ⚠️ A spec diz que `bounds` é `[x, y, w, h]` e não diz **em relação a quê**. Aqui é **relativo
//! à própria âncora**, em pixels da fonte, com **+Y para cima** (a convenção do mundo, a mesma
//! do `QUAD_STRIP`). Motivo: uma âncora com área é «um socket que também é uma caixa» — a
//! hitbox da mão anda com a mão. Absoluto na imagem (a leitura do Aseprite) faria mover a âncora
//! deixar a caixa para trás.

use ph2d_core::Vec2;
use ph2d_ecs::{Entity, NamedAnchor, NamedAnchorList, Transform, World};
use ph2d_host::WindowSize;
use ph2d_render::Camera2d;
use ph2d_vector::{Affine, BezPath, Brush, Color, Point, Stroke, VectorScene};

/// Espessura do traço do marcador, em px de tela.
const MARK_PX: f64 = 1.5; // LITERAL-PX-OK: chrome de overlay, espessura de tela
/// Meio-braço da cruz de um socket, em px de tela.
const CROSS_PX: f64 = 7.0; // LITERAL-PX-OK: chrome de overlay, tamanho de tela
/// Meio-lado de uma ALÇA arrastável, em px de tela.
const HANDLE_PX: f64 = 4.0; // LITERAL-PX-OK: chrome de overlay, tamanho de tela
/// ⚠️ **A opacidade das âncoras que NÃO estão abertas.**
///
/// Elas continuam visíveis — são o «onde» do sprite, e escondê-las faria o artista perder a
/// noção do conjunto ao abrir uma. Mas ficam para trás: só a aberta tem alças, e uma marca com o
/// mesmo peso de uma que se pode agarrar seria um alvo que não é alvo.
const DIM_ALPHA: f32 = 0.4; // LITERAL-COLOR-OK: peso relativo do chrome, não uma cor
/// Corpo do rótulo do nome, em px de tela.
const LABEL_PX: f32 = 11.0; // LITERAL-PX-OK: chrome de overlay
/// Caixa do rótulo — larga o bastante para os 64 bytes que um nome pode ter sem cortar cedo.
const LABEL_BOX_W_PX: f32 = 160.0; // LITERAL-PX-OK: chrome de overlay
const LABEL_BOX_H_PX: f32 = 14.0; // LITERAL-PX-OK: chrome de overlay

/// Cor estável a partir do nome — FNV-1a sobre os bytes, depois um passeio pelo círculo de
/// matiz. ⚠️ Determinística: a mesma âncora tem a mesma cor em todas as sessões e máquinas.
fn color_of(name: &str) -> [f32; 4] {
    let mut h: u32 = 0x811c_9dc5;
    for b in name.as_bytes() {
        h ^= u32::from(*b);
        h = h.wrapping_mul(0x0100_0193);
    }
    // Matiz em 6 setores, saturação e valor fixos e altos: chrome tem de ler sobre arte clara e
    // escura.
    let hue = (h % 360) as f32 / 60.0;
    let i = hue as u32 % 6;
    let f = hue - hue.floor();
    let (q, t) = (1.0 - f, f);
    let (r, g, b) = match i {
        0 => (1.0, t, 0.0),
        1 => (q, 1.0, 0.0),
        2 => (0.0, 1.0, t),
        3 => (0.0, q, 1.0),
        4 => (t, 0.0, 1.0),
        _ => (1.0, 0.0, q),
    };
    [r, g, b, 0.95] // LITERAL-COLOR-OK: chrome de overlay, opacidade de marcador
}

/// **Onde um ponto da âncora cai no MUNDO.**
///
/// `sprite_world` é a pose composta da sprite (pais incluídos); `local_px` é um deslocamento em
/// pixels da fonte relativo à âncora (`[0,0]` = o centro dela).
///
/// ⚠️ **Existe como função PURA porque o defeito morava exatamente aqui.** A primeira versão lia
/// `GlobalTransform` do mundo da SIMULAÇÃO — e `GlobalTransform` é componente de APRESENTAÇÃO,
/// reconstruído noutro mundo a cada quadro. A leitura devolvia sempre nada, caía no `Vec2::ZERO`,
/// e as âncoras ficavam **cravadas na origem do mundo**, sem seguir a sprite (smoke do Enio,
/// 2026-08-22). Uma leitura de componente enterrada num laço de desenho não é observável por
/// teste nenhum; com nome, ela responde.
pub(crate) fn anchor_world_point(
    sprite_world: Transform,
    anchor: &NamedAnchor,
    local_px: [f32; 2],
    pixels_per_meter: f32,
) -> Vec2 {
    let ppm = if pixels_per_meter.is_finite() && pixels_per_meter > 0.0 {
        pixels_per_meter
    } else {
        1.0
    };
    // A âncora sob a pose da sprite; depois o ponto sob a pose da âncora.
    //
    // ⚠️ **`anchor_pose_under` é a lei ÚNICA de «onde está esta âncora»**, e desde 2026-08-22 é
    // a mesma função que a montagem (`ph2d_ecs::mount_state`) e a API de runtime
    // (`anchor_world_pose`) usam. Ela é uma linha de álgebra — e é exatamente por ser uma linha
    // que se reimplementa sem ninguém reparar; aí a alça agarra num sítio e a espada monta
    // noutro. Rotação e escala vêm de graça, e por isso a caixa de dano roda com o objeto.
    let anchor_world = ph2d_ecs::anchor_pose_under(sprite_world, anchor);
    let offset = Transform {
        translation: Vec2::new(local_px[0] / ppm, local_px[1] / ppm),
        ..Transform::default()
    };
    Transform::compose(anchor_world, offset).translation
}

/// **Como esta passagem desenha as âncoras de uma entidade.**
///
/// ⚠️ Os dois modos existem porque as marcas respondem a **três** perguntas diferentes no mesmo
/// quadro, e só uma delas admite gesto: *o que estou a editar* (com alças), *de que ponto este
/// objeto parte* (contexto do filho montado) e *onde estão os pontos desta cena* (a caixa «Always
/// show anchors»). Um único modo faria as três parecerem a mesma coisa — e alças a mais são
/// alvos a disputar o mesmo pixel.
#[derive(Copy, Clone)]
enum Marks<'a> {
    /// A entidade **selecionada**: todas as âncoras, e alças na linha aberta.
    Editing { open_row: Option<usize> },
    /// Contexto: esmaecido, **sem alças**. `only` limita a UMA âncora, pelo nome.
    Context { only: Option<&'a str> },
}

/// Desenha as marcas de UMA entidade. `true` = desenhou alguma coisa.
#[allow(clippy::too_many_arguments)]
fn draw_entity_marks(
    sim: &World,
    entity: Entity,
    marks: Marks<'_>,
    ppm: f32,
    to_screen: Affine,
    vector_scene: &mut VectorScene,
    text_system: &mut ph2d_text::TextSystem,
) -> bool {
    let Some(list) = sim.get::<NamedAnchorList>(entity) else {
        return false;
    };
    if list.is_empty() {
        return false;
    }
    // ⚠️ **`world_transform` do mundo da SIMULAÇÃO, não `GlobalTransform`.** O `GlobalTransform`
    // é `PresentComponent` — vive no mundo de apresentação, reconstruído a cada quadro. Lê-lo
    // daqui devolvia sempre `None`.
    let Some(sprite_world) = ph2d_ecs::world_transform(sim, entity) else {
        return false;
    };
    let mut drew = false;
    for (row, a) in list.iter().enumerate() {
        // **O modo decide as três coisas de uma vez**: se esta âncora entra, se leva alças, e com
        // que peso. ⚠️ `Context { only: Some(n) }` é o filtro que a passagem do filho montado usa
        // — desenhar a lista inteira do pai ali afogaria a única âncora que interessa.
        let open = match marks {
            Marks::Editing { open_row } => open_row == Some(row),
            Marks::Context { only } => {
                if only.is_some_and(|n| n != a.name) {
                    continue;
                }
                false
            }
        };
        drew = true;
        let mut rgba = color_of(&a.name);
        if !open {
            rgba[3] *= DIM_ALPHA;
        }
        let brush = Brush::Solid(Color::new(rgba));
        let width = if open { MARK_PX * 1.6 } else { MARK_PX };
        // O centro da âncora, em MUNDO, depois em tela.
        let world = anchor_world_point(sprite_world, a, [0.0, 0.0], ppm);
        let c = to_screen * Point::new(f64::from(world.x), f64::from(world.y));

        // A cruz: sempre, para toda âncora. É o «onde» — e o retângulo, quando existe, é o
        // «quanto».
        let mut cross = BezPath::new();
        cross.move_to(Point::new(c.x - CROSS_PX, c.y));
        cross.line_to(Point::new(c.x + CROSS_PX, c.y));
        cross.move_to(Point::new(c.x, c.y - CROSS_PX));
        cross.line_to(Point::new(c.x, c.y + CROSS_PX));
        vector_scene.inner_mut().stroke(
            &Stroke::new(width),
            Affine::IDENTITY,
            &brush,
            None,
            &cross,
        );

        // **O NOME, ao lado da cruz.** Sem ele, cinco âncoras são cinco cruzes coloridas e o
        // artista tem de contar linhas no painel para saber qual é qual — a cor sozinha diz que
        // são diferentes, não QUAIS são.
        ph2d_editor::paint::paint_text_centered(
            text_system,
            vector_scene,
            &a.name,
            ph2d_editor::zones::Rect::new(
                c.x as f32 + CROSS_PX as f32,
                c.y as f32 - CROSS_PX as f32 - LABEL_BOX_H_PX,
                LABEL_BOX_W_PX,
                LABEL_BOX_H_PX,
            ),
            LABEL_PX,
            Color::new(rgba),
        );

        // A área e o miolo, em px da fonte relativos à âncora. ⚠️ **+Y para cima**: o `h` de um
        // rect cresce para cima, como no mundo.
        for (rect, dash) in [(a.bounds, false), (a.center, true)] {
            let Some([rx, ry, rw, rh]) = rect else {
                continue;
            };
            if rw <= 0.0 || rh <= 0.0 {
                continue;
            }
            let p = |px: f32, py: f32| {
                let w = anchor_world_point(sprite_world, a, [px, py], ppm);
                to_screen * Point::new(f64::from(w.x), f64::from(w.y))
            };
            let (a0, a1, a2, a3) = (
                p(rx, ry),
                p(rx + rw, ry),
                p(rx + rw, ry + rh),
                p(rx, ry + rh),
            );
            let mut path = BezPath::new();
            path.move_to(a0);
            path.line_to(a1);
            path.line_to(a2);
            path.line_to(a3);
            path.close_path();
            // O miolo desenha-se mais fino: ele é uma subdivisão da área, não outra área.
            let width = if dash { MARK_PX * 0.6 } else { MARK_PX };
            let rect_width = if dash { width * 0.6 } else { width };
            vector_scene.inner_mut().stroke(
                &Stroke::new(rect_width),
                Affine::IDENTITY,
                &brush,
                None,
                &path,
            );
        }

        // ⚠️ **AS ALÇAS — e só na âncora aberta.** Elas vêm do mesmo `handles` que o arrasto
        // consulta, e não de uma segunda cópia da geometria: se o desenho e o teste de acerto
        // divergissem, o artista veria uma alça onde não há e agarraria onde não vê. *A alça
        // pintada É a alça que agarra.*
        if !open {
            continue;
        }
        let (hs, n) = super::anchor_gizmo::handles(sprite_world, a, ppm);
        for h in hs.iter().take(n).flatten() {
            let p = to_screen * Point::new(f64::from(h.world.x), f64::from(h.world.y));
            // A de ROTAÇÃO leva uma haste até ao centro: é ela que faz o gesto ler-se como rodar.
            if h.kind == super::anchor_gizmo::AnchorHandleKind::Rotate {
                let mut arm = BezPath::new();
                arm.move_to(c);
                arm.line_to(p);
                vector_scene.inner_mut().stroke(
                    &Stroke::new(width * 0.7),
                    Affine::IDENTITY,
                    &brush,
                    None,
                    &arm,
                );
            }
            let mut box_path = BezPath::new();
            box_path.move_to(Point::new(p.x - HANDLE_PX, p.y - HANDLE_PX));
            box_path.line_to(Point::new(p.x + HANDLE_PX, p.y - HANDLE_PX));
            box_path.line_to(Point::new(p.x + HANDLE_PX, p.y + HANDLE_PX));
            box_path.line_to(Point::new(p.x - HANDLE_PX, p.y + HANDLE_PX));
            box_path.close_path();
            vector_scene.inner_mut().fill(
                ph2d_vector::Fill::NonZero,
                Affine::IDENTITY,
                &brush,
                None,
                &box_path,
            );
        }
    }

    drew
}

/// **O PLANO de desenho deste quadro** — que entidade, em que modo, e por que ordem.
///
/// ⚠️ **Puro de propósito: sem cena, sem texto, sem câmara.** É a mesma escolha (e a mesma razão)
/// do [`super::anchor_gizmo`]: as três passagens abaixo decidem *quem aparece*, e uma decisão
/// enterrada num laço de desenho é inalcançável por teste — que é exatamente onde os erros de
/// overlay moram (a marca que não aparece, a que aparece duas vezes, a que rouba o destaque).
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum PlanMode {
    /// Esmaecida, todas as âncoras — a caixa «Always show anchors» do dono.
    AlwaysVisible,
    /// Esmaecida, **uma só** âncora pelo nome — a que o objeto selecionado monta.
    RiddenAnchor(String),
    /// A selecionada: todas, com alças na linha aberta.
    Editing(Option<usize>),
}

/// Constrói o plano. **A ORDEM da lista é a ordem de pintura**, e quem vem depois fica por cima.
///
/// # As TRÊS passagens, e por que a ordem é esta
///
/// 1. **«Always show anchors»** (`ph2d_ecs::AnchorVisibility::in_editor`) — toda entidade que o
///    peça, esteja ou não selecionada, esteja a §12 aberta ou fechada. É o que permite montar uma
///    cena com várias peças presas sem ter de selecionar cada dono para ver onde os pontos estão.
/// 2. **A âncora que o objeto SELECIONADO monta** (Enio, 2026-08-23: *«ao mover/rot/escalonar o
///    filho ancorado, a âncora a que está ligado deve ficar visível»*). Ela é do **pai**, e
///    aparece **mesmo com a §12 fechada** — é a referência de que o deslocamento do filho é
///    medido, e sem ela um arrasto é às cegas.
/// 3. **A entidade selecionada**, com as alças na linha aberta — o que já existia. Vai por último
///    porque é a única que aceita gesto, e o que se pode agarrar tem de estar por cima.
///
/// ⚠️ **A dedup importa e é assimétrica.** Um dono «always visible» que seja também o selecionado
/// não se repete (duas passagens de alfa esmaecido somam num traço que se lê como destaque, e a
/// marca mentiria sobre qual é a importante). Mas a passagem (2) desenha **uma** âncora e a (3)
/// desenha **todas**: se o pai for o próprio selecionado, as duas têm de correr.
#[must_use]
pub(crate) fn marks_plan(
    sim: &World,
    expanded: bool,
    selected: Option<u64>,
    open_row: Option<usize>,
) -> Vec<(Entity, PlanMode)> {
    let mut plan: Vec<(Entity, PlanMode)> = Vec::new();

    // **Quem vai em modo `Editing`, decidido ANTES de tudo — porque ele GANHA.**
    //
    // ⚠️ **Isto foi um DEFEITO, e o gate afirmava-o** (Enio, 2026-08-23: *«Always show anchors
    // retira o destaque da âncora/slot selecionado»*). A primeira versão deixava a passagem (1)
    // reclamar a entidade selecionada e **desistia** da (3), com a justificação de que desenhar
    // duas vezes soma o alfa e finge destaque. A observação sobre o alfa estava certa; a cura
    // estava ao contrário.
    //
    // `Editing` é **superset** de `AlwaysVisible`: desenha as MESMAS âncoras, mais o realce da
    // linha aberta, mais as alças. Quem tem de sair é a (1) — e o efeito da caixa passa a ser
    // exatamente o que o nome dela promete: *manter visível quando NÃO está selecionada*.
    // Desselecionar devolve a entidade à passagem (1), e o destaque some, que é a outra metade
    // do pedido.
    //
    // ⚠️ **Com a §12 FECHADA não há `Editing`**, e aí a caixa volta a mandar mesmo na
    // selecionada: sem seção aberta não há linha aberta nem alças para destacar.
    let editing = if expanded {
        selected.map(Entity::from_bits)
    } else {
        None
    };

    // (1) — quem pediu para ficar sempre visível.
    //
    // ⚠️ **`try_query` porque só há `&World` aqui**, e o `World::query` pede `&mut`. Ele também
    // dá o atalho certo de graça: devolve `None` enquanto **nenhuma** entidade tiver tocado o
    // componente, que é o caso da esmagadora maioria das cenas — e aí esta passagem custa uma
    // comparação. Quando há, construir o estado é O(arquétipos), não O(entidades).
    //
    // ⚠️ Quem decide **é** `anchors_draw_in_editor`, e não um `vis.in_editor` solto: a consulta
    // aqui só ENUMERA. O default «só quando selecionada» tem de viver num sítio só.
    if let Some(q) = sim.try_query::<(Entity, &ph2d_ecs::AnchorVisibility)>() {
        for (id, _) in q.iter_manual(sim) {
            // ⛔ **A entidade que vai em modo `Editing` NÃO entra aqui** — ver `editing` acima.
            if Some(id) != editing && ph2d_ecs::anchors_draw_in_editor(sim, id) {
                plan.push((id, PlanMode::AlwaysVisible));
            }
        }
        // ⚠️ A ordem de iteração de um arquétipo não é a da cena. Ordenar pelos bits torna o
        // plano **determinístico** — sem isto, dois quadros iguais podiam pintar por ordens
        // diferentes, e um gate de ordem seria uma flake à espera.
        plan.sort_unstable_by_key(|(e, _)| e.to_bits());
    }
    let already = |plan: &[(Entity, PlanMode)], e: Entity| plan.iter().any(|(p, _)| *p == e);

    // (2) — a âncora de que o selecionado parte.
    if let Some(bits) = selected {
        let child = Entity::from_bits(bits);
        if let Some(mount) = sim
            .get::<ph2d_ecs::AnchorMount>(child)
            .filter(|m| m.is_bound())
            && let Some(parent) = sim.get::<ph2d_ecs::ChildOf>(child).map(|c| c.parent())
            && !already(&plan, parent)
        {
            plan.push((parent, PlanMode::RiddenAnchor(mount.anchor.clone())));
        }
    }

    // (3) — a selecionada, com alças, por cima de tudo. Ela entra **sempre** que existe: a
    // passagem (1) já lhe cedeu o lugar, e a (2) nunca a pode ter apanhado (uma entidade não é
    // pai de si própria).
    if let Some(e) = editing {
        plan.push((e, PlanMode::Editing(open_row)));
    }
    plan
}

/// Desenha os marcadores das âncoras — a metade que o artista VÊ da §12.
///
/// `expanded` é a seção §12 estar aberta. ⚠️ Ela governa só a **terceira** passagem do
/// [`marks_plan`]; as outras duas existem precisamente para responder a perguntas que não
/// dependem de o painel estar aberto.
#[allow(clippy::too_many_arguments)]
pub(super) fn draw_anchor_marks(
    expanded: bool,
    sim: &World,
    selected: Option<u64>,
    open_row: Option<usize>,
    pixels_per_meter: f32,
    camera: &Camera2d,
    window: WindowSize,
    vector_scene: &mut VectorScene,
    text_system: &mut ph2d_text::TextSystem,
) {
    let ppm = pixels_per_meter.max(crate::EPS_PIXELS_PER_METER);
    let to_screen = camera.world_to_screen_affine(window);
    for (entity, mode) in marks_plan(sim, expanded, selected, open_row) {
        let marks = match &mode {
            PlanMode::AlwaysVisible => Marks::Context { only: None },
            PlanMode::RiddenAnchor(n) => Marks::Context {
                only: Some(n.as_str()),
            },
            PlanMode::Editing(row) => Marks::Editing { open_row: *row },
        };
        draw_entity_marks(
            sim,
            entity,
            marks,
            ppm,
            to_screen,
            vector_scene,
            text_system,
        );
    }
}

#[cfg(test)]
#[path = "anchor_overlay_tests.rs"]
mod tests;
