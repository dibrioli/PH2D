//! **AS FILEIRAS PRÓPRIAS DE CADA PINCEL** — os chips que só existem com um
//! verbo na mão: o contorno, a densidade, a pose e o tecido.
//!
//! Irmão (`#[path]`-menos) do [`super::brush`], e o corte é de
//! RESPONSABILIDADE: lá mora *a moldura do pincel* — a linha do nível, a cauda
//! partilhada, os interruptores por verbo —, aqui *o vocabulário PRÓPRIO de
//! cada ferramenta*, que é a lista que cresce um bloco a cada pincel novo. Foi
//! ela que levou o ficheiro ao tecto de 600 LOC do `panel_files_under_loc_cap`
//! quando a fileira da densidade chegou (2026-09-14).
//!
//! ⛔ **Curado por CORTE e nunca por uma entrada no `FILE_OVERAGE_OK`** — a lei
//! do §5.0 do roteador, que esta casa já pagou em cinco painéis.
//!
//! ⚠️ **Nenhum pixel muda de sítio:** as quatro funções são chamadas na mesma
//! ordem, do mesmo sítio, com os mesmos argumentos. O que muda é onde a lista
//! cresce.

use ph2d_editor_core::panel::PaintCtx;
use ph2d_i18n::tr;
use ph2d_sculpt3d::{ClothArea, ClothForceFalloff, ClothMode, Verb};
use ph2d_tokens::Spacing;

use super::widgets::{command, labelled_seg, toggle};
use crate::state::Sculpt3dSnapshot;

/// **AS DUAS FILEIRAS DO PINCEL DE CONTORNO.**
///
/// ⚠️⚠️ **Elas são DUAS perguntas diferentes e é fácil lê-las como uma:** a
/// primeira escolhe **o que a borda faz** (dobrar, expandir, inflar, agarrar,
/// torcer, alisar) e a segunda **como isso esmorece AO LONGO da borda**. Uma
/// terceira, a curva do pincel, gradua a profundidade **para dentro** da peça —
/// e essa já vive na secção do pincel, partilhada com todos os verbos.
///
/// ⛔ **Sem a primeira o artista alcança UM dos seis gestos**, que é o defeito
/// que o pincel de tecido pagou: *um motor vivo sem botão nenhum.*
pub(super) fn paint_boundary_rows(
    ctx: &mut PaintCtx,
    snap: &Sculpt3dSnapshot,
    x: f32,
    w: f32,
    y: f32,
) -> f32 {
    if !snap.ui.brush.offers_boundary_controls() {
        return y;
    }
    let modos = ph2d_sculpt3d::BoundaryModo::ALL;
    let selected = modos
        .iter()
        .position(|&m| m == snap.ui.brush.boundary.modo)
        .unwrap_or(0);
    let labels: Vec<&str> = modos.iter().map(|m| m.label()).collect();
    let y = labelled_seg(
        ctx,
        tr("panel.sculpt3d.boundary_mode"),
        crate::ids::SCULPT3D_SEC_BRUSH,
        &crate::ids::SCULPT3D_BOUNDARY_MODE,
        &labels,
        selected,
        x,
        w,
        y,
    );
    let quedas = ph2d_sculpt3d::BoundaryQueda::ALL;
    let selected = quedas
        .iter()
        .position(|&q| q == snap.ui.brush.boundary.queda_no_contorno)
        .unwrap_or(0);
    let labels: Vec<&str> = quedas.iter().map(|q| q.label()).collect();
    labelled_seg(
        ctx,
        tr("panel.sculpt3d.boundary_falloff"),
        crate::ids::SCULPT3D_SEC_BRUSH,
        &crate::ids::SCULPT3D_BOUNDARY_FALLOFF,
        &labels,
        selected,
        x,
        w,
        y,
    ) + Spacing::Sm.px()
}

/// **A FILEIRA E AS DUAS CAIXAS DO PINCEL DE POSE.**
///
/// ⚠️ **A fileira de modos NÃO é um luxo: sem ela o artista alcança UM dos três
/// gestos.** Girar/torcer, escalar/transladar e espremer/esticar são
/// deformações diferentes — e cada barra tem duas metades, porque o modificador
/// de inversão **troca de deformação** em vez de trocar o sinal da força.
///
/// ⚠️ **A trava aparece SÓ no modo de escala** — ver
/// [`ph2d_sculpt3d::Brush::offers_pose_rotation_lock`]: nos outros dois ela não
/// tem o que travar, e um interruptor inerte é pior que um ausente.
pub(super) fn paint_pose_rows(
    ctx: &mut PaintCtx,
    snap: &Sculpt3dSnapshot,
    x: f32,
    w: f32,
    y: f32,
) -> f32 {
    if !snap.ui.brush.offers_pose_controls() {
        return y;
    }
    let modos = ph2d_sculpt3d::PoseModo::ALL;
    let selected = modos
        .iter()
        .position(|&m| m == snap.ui.brush.pose.modo)
        .unwrap_or(0);
    let labels: Vec<&str> = modos.iter().map(|m| m.label()).collect();
    let y = labelled_seg(
        ctx,
        tr("panel.sculpt3d.pose_mode"),
        crate::ids::SCULPT3D_SEC_BRUSH,
        &crate::ids::SCULPT3D_POSE_MODE,
        &labels,
        selected,
        x,
        w,
        y,
    );
    let y = toggle(
        ctx,
        crate::ids::SCULPT3D_POSE_ANCHORED,
        tr("panel.sculpt3d.pose_anchored"),
        snap.ui.brush.pose.ancorado,
        x,
        w,
        y,
    ) + Spacing::Sm.px();
    if snap.ui.brush.offers_pose_rotation_lock() {
        toggle(
            ctx,
            crate::ids::SCULPT3D_POSE_ROT_LOCK,
            tr("panel.sculpt3d.pose_rot_lock"),
            snap.ui.brush.pose.trava_rotacao,
            x,
            w,
            y,
        ) + Spacing::Sm.px()
    } else {
        y
    }
}

/// **AS DUAS FILEIRAS DO PINCEL DE TECIDO** — *Deformation* e *Simulation Area*,
/// na ordem em que o painel da referência as põe (espec §8.4).
///
/// ⚠️ **Elas só existem com o verbo Cloth na mão**, e a pergunta é ao VERBO — a
/// mesma cerca da lâmina do `MultiplaneScrape` acima: uma lista paralela aqui
/// seria uma fileira que aparece noutra ferramenta e não move um vértice.
///
/// ⚠️ **Antes de 2026-09-06 os dois selectores eram VARIÁVEIS DE AMBIENTE.** O
/// motor respondia aos oito modos e às três áreas desde que a lei da referência
/// nasceu, e o artista chegava a UM. *Não era um botão morto: era um motor vivo
/// sem botão nenhum.*
pub(super) fn paint_cloth_rows(
    ctx: &mut PaintCtx,
    snap: &Sculpt3dSnapshot,
    x: f32,
    w: f32,
    y: f32,
) -> f32 {
    if snap.ui.brush.verb != Verb::Cloth {
        return y;
    }
    let selected = ClothMode::ALL
        .iter()
        .position(|&m| m == snap.ui.brush.cloth_mode)
        .unwrap_or(0);
    let labels: Vec<&str> = ClothMode::ALL.iter().map(|m| m.label()).collect();
    let y = labelled_seg(
        ctx,
        tr("panel.sculpt3d.cloth_mode"),
        crate::ids::SCULPT3D_SEC_BRUSH,
        &crate::ids::SCULPT3D_CLOTH_MODE,
        &labels,
        selected,
        x,
        w,
        y,
    );
    let selected = ClothArea::ALL
        .iter()
        .position(|&a| a == snap.ui.brush.cloth_area)
        .unwrap_or(0);
    let labels: Vec<&str> = ClothArea::ALL.iter().map(|a| a.label()).collect();
    let y = labelled_seg(
        ctx,
        tr("panel.sculpt3d.cloth_area"),
        crate::ids::SCULPT3D_SEC_BRUSH,
        &crate::ids::SCULPT3D_CLOTH_AREA,
        &labels,
        selected,
        x,
        w,
        y,
    );
    let selected = ClothForceFalloff::ALL
        .iter()
        .position(|&f| f == snap.ui.brush.cloth_force_falloff)
        .unwrap_or(0);
    let labels: Vec<&str> = ClothForceFalloff::ALL.iter().map(|f| f.label()).collect();
    let y = labelled_seg(
        ctx,
        tr("panel.sculpt3d.cloth_force_falloff"),
        crate::ids::SCULPT3D_SEC_BRUSH,
        &crate::ids::SCULPT3D_CLOTH_FORCE_FALLOFF,
        &labels,
        selected,
        x,
        w,
        y,
    );
    // **A BASE PERSISTENTE** — o interruptor e o botão que o torna observável.
    //
    // ⚠️⚠️ **Os DOIS, e nenhum basta sozinho:** ligar a opção sem base gravada é
    // um **no-op exacto** (a construção cai no repouso do traço), e gravar a base
    // com a opção desligada não muda nada. *Um controlo sozinho aqui seria um
    // botão que o artista carrega e não vê acontecer nada.*
    let y = toggle(
        ctx,
        crate::ids::SCULPT3D_CLOTH_PERSISTENT,
        tr("panel.sculpt3d.cloth_persistent"),
        snap.ui.brush.cloth_persistent,
        x,
        w,
        y,
    );
    let y = command(
        ctx,
        crate::ids::SCULPT3D_CLOTH_SET_BASE,
        tr("panel.sculpt3d.cloth_set_base"),
        x,
        w,
        y,
    );
    // **A COLISÃO** — o pano pára nas outras peças da cena.
    //
    // ⚠️ **Ela nasce desligada e o painel não a esconde:** o custo é um raio por
    // vértice activo, por colisor e por passo, e é o artista que decide se o
    // paga. *Um controlo caro escondido é um controlo que ninguém sabe que tem.*
    let y = toggle(
        ctx,
        crate::ids::SCULPT3D_CLOTH_COLLISIONS,
        tr("panel.sculpt3d.cloth_collisions"),
        snap.ui.brush.cloth_collisions,
        x,
        w,
        y,
    );
    // **O PINO DA FRONTEIRA**, e só onde a lei existe. ⚠️ A pergunta é a mesma
    // que [`ph2d_sculpt3d`] faz ao construir o pincel — a lei recusa o pino fora
    // da área *Local* (espec §2.3), e a caixa nos outros dois terços do selector
    // seria um interruptor de coisa nenhuma. ⛔ Duas cópias da pergunta
    // divergiriam; esta pergunta ao MOTOR, pelo mesmo `area()`.
    if !snap.ui.brush.cloth_area.offers_pin() {
        return y;
    }
    toggle(
        ctx,
        crate::ids::SCULPT3D_CLOTH_PIN,
        tr("panel.sculpt3d.cloth_pin"),
        snap.ui.brush.cloth_pin,
        x,
        w,
        y,
    )
}
