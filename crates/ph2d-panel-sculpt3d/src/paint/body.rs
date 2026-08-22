//! O corpo rolado: a pilha de seções dobráveis.
//!
//! Irmão do `paint.rs` porque o cap de LOC de painel é 600 e as duas metades
//! crescem por motivos diferentes (chrome × conteúdo).
//!
//! **A ordem das seções é a ordem em que a mão as procura:** a FERRAMENTA
//! primeiro (é o que se troca a cada minuto), o PINCEL logo abaixo (os knobs da
//! ferramenta em mãos), o ESPELHO, a TOPOLOGIA (a resolução do barro), o
//! SOMBREAMENTO (como a forma é lida) e a CENA por último — que é a ordem do
//! SculptGL, e por um motivo que se verifica: quanto mais raro o gesto, mais
//! fundo ele pode estar.

use ph2d_editor_core::ids;
use ph2d_editor_core::panel::PaintCtx;
use ph2d_i18n::tr;
use ph2d_tokens::{ROW_H_PX, Spacing};

use super::brush::{paint_brush_tail, paint_level_row};
use super::tool::paint_tool;
use super::widgets::{self, command, header, labelled_seg, readout, row_of_two, toggle};

use crate::rows;
use crate::state::Sculpt3dSnapshot;

/// Os rótulos dos três degraus de detalhe, na ordem do `DETAIL_STEPS` do shell.
const DETAIL_LABELS: [&str; 3] = [
    "panel.sculpt3d.detail.coarse",
    "panel.sculpt3d.detail.medium",
    "panel.sculpt3d.detail.fine",
];

/// Os rótulos das quatro primitivas, na ordem dos comandos `Add*`.
const ADD_LABELS: [&str; 4] = [
    "panel.sculpt3d.add.sphere",
    "panel.sculpt3d.add.cube",
    "panel.sculpt3d.add.cylinder",
    "panel.sculpt3d.add.torus",
];

/// Os rótulos das quatro operações de máscara, na ordem dos comandos `Mask*`.
pub(super) const MASK_LABELS: [&str; 4] = [
    "panel.sculpt3d.mask.clear",
    "panel.sculpt3d.mask.invert",
    "panel.sculpt3d.mask.blur",
    "panel.sculpt3d.mask.sharpen",
];

/// Pinta todas as seções. Devolve o `y` em que terminou.
///
/// ⚠️ **Uma chamada por seção, e não um corpo só.** Ele já cruzou o cap de 200
/// LOC de `fn` uma vez, e o corte que o gate pediu é o mesmo que a leitura pede:
/// cada seção é um ASSUNTO, e o orquestrador aqui é a ORDEM em que a mão os
/// procura.
pub(super) fn paint_sections(
    ctx: &mut PaintCtx,
    snap: &Sculpt3dSnapshot,
    x: f32,
    w: f32,
    y_in: f32,
) -> f32 {
    let mut y = paint_tool(ctx, snap, x, w, y_in);
    y = knob_section(
        ctx,
        snap,
        &rows::SECTIONS[0],
        x,
        w,
        y,
        paint_level_row,
        paint_brush_tail,
    );
    y = paint_symmetry(ctx, snap, x, w, y);
    y = paint_topology(ctx, snap, x, w, y);
    y = knob_section(
        ctx,
        snap,
        &rows::SECTIONS[1],
        x,
        w,
        y,
        no_head,
        paint_shading_tail,
    );
    y = paint_scene(ctx, snap, x, w, y);
    paint_bake(ctx, snap, x, w, y)
}

/// A seção não tem cabeça própria.
fn no_head(_: &mut PaintCtx, _: &Sculpt3dSnapshot, _: f32, _: f32, y: f32) -> f32 {
    y
}

/// **O ESPELHO** — três botões INDEPENDENTES.
///
/// Não é um rádio: um segmented é *um de N* por construção, e o ZBrush espelha
/// em dois eixos ao mesmo tempo.
fn paint_symmetry(ctx: &mut PaintCtx, snap: &Sculpt3dSnapshot, x: f32, w: f32, y: f32) -> f32 {
    let gap = Spacing::Sm.px();
    let (fold, y) = header(
        ctx,
        ids::SCULPT3D_SEC_SYMMETRY,
        tr("panel.sculpt3d.section.symmetry"),
        x,
        w,
        y,
    );
    let Some(fold) = fold else {
        return y;
    };
    let third = (w - gap * 2.0) / 3.0; // LITERAL-PX-OK: sao TRES eixos de espelho, nao uma metrica
    for (i, (id, key, on)) in [
        (
            ids::SCULPT3D_SYM_X,
            "panel.sculpt3d.sym.x",
            snap.ui.symmetry.x,
        ),
        (
            ids::SCULPT3D_SYM_Y,
            "panel.sculpt3d.sym.y",
            snap.ui.symmetry.y,
        ),
        (
            ids::SCULPT3D_SYM_Z,
            "panel.sculpt3d.sym.z",
            snap.ui.symmetry.z,
        ),
    ]
    .into_iter()
    .enumerate()
    {
        let bx = (third + gap).mul_add(i as f32, x);
        toggle(ctx, id, tr(key), on, bx, third, y);
    }
    widgets::end_fold(ctx, fold, y + ROW_H_PX + Spacing::Md.px())
}

/// **COM QUE LUZ, e COM OU SEM A MALHA** — a cauda da seção de sombreamento.
///
/// ⚠️ Os dois são opções de VISTA e por isso ficam juntos, abaixo dos knobs: um
/// muda a lâmpada, o outro acrescenta uma anotação por cima. Nenhum deles toca a
/// escultura, e é isso que os separa de tudo que está acima na coluna.
fn paint_shading_tail(ctx: &mut PaintCtx, snap: &Sculpt3dSnapshot, x: f32, w: f32, y: f32) -> f32 {
    // **ASSAR O AO**, logo abaixo do slider que o mostra — a força só significa
    // alguma coisa depois de haver o que mostrar, e separar os dois faria o
    // artista arrastar um controle inerte sem nada dizendo por quê.
    let mut y = command(
        ctx,
        ids::SCULPT3D_BAKE_AO,
        tr("panel.sculpt3d.bake_ao"),
        x,
        w,
        y,
    );
    // ⚠️ **E a obsolescência é DITA, não deixada para o artista descobrir.** Um
    // AO velho não parece velho: parece uma escolha de iluminação. A linha só
    // existe quando há o que avisar — um aviso permanente vira moldura.
    if snap.ao_stale {
        y = readout(ctx, tr("panel.sculpt3d.ao_stale"), x, w, y);
    }
    let y = y + Spacing::Sm.px();
    // A primeira opção é o RIG e as seguintes são os materiais, então o índice
    // selecionado é `matcap + 1` — o mesmo deslocamento que o `ShadeRaw` faz
    // para o device. ⚠️ Ele é escrito aqui e lido no `event` pela mesma
    // aritmética; as duas metades vivem uma ao lado da outra de propósito.
    // ⚠️ **A lista de chips é a MENOR das duas** — os nomes que o host publicou e
    // os ids que existem. Um material sem id seria pintado sobre o chip do
    // vizinho; um id sem material seria um chip anônimo que despacha. Cortar
    // pelo mínimo faz das duas listas uma só, e o gate do shell é quem exige
    // que elas tenham o mesmo tamanho de verdade.
    let n = snap.matcaps.len().min(ids::SCULPT3D_MATCAP.len() - 1);
    let mut labels: Vec<&str> = vec![tr("panel.sculpt3d.matcap.rig")];
    labels.extend(&snap.matcaps[..n]);
    let options = &ids::SCULPT3D_MATCAP[..=n];
    let selected = snap.ui.matcap.map_or(0, |i| usize::from(i) + 1);
    let mut y = labelled_seg(
        ctx,
        tr("panel.sculpt3d.matcap"),
        ids::SCULPT3D_SEC_SHADING,
        options,
        &labels,
        selected.min(n),
        x,
        w,
        y,
    );
    y = toggle(
        ctx,
        ids::SCULPT3D_WIREFRAME,
        tr("panel.sculpt3d.wireframe"),
        snap.ui.wireframe,
        x,
        w,
        y,
    );
    y
}

/// **A TOPOLOGIA** — a resolução do barro.
fn paint_topology(ctx: &mut PaintCtx, snap: &Sculpt3dSnapshot, x: f32, w: f32, y: f32) -> f32 {
    let gap = Spacing::Sm.px();
    let (fold, mut y) = header(
        ctx,
        ids::SCULPT3D_SEC_TOPOLOGY,
        tr("panel.sculpt3d.section.topology"),
        x,
        w,
        y,
    );
    let Some(fold) = fold else {
        return y;
    };
    y = toggle(
        ctx,
        ids::SCULPT3D_DYNTOPO,
        tr("panel.sculpt3d.dyntopo"),
        snap.dyntopo,
        x,
        w,
        y,
    ) + gap;
    let detail: Vec<&str> = DETAIL_LABELS.iter().map(|k| tr(k)).collect();
    y = labelled_seg(
        ctx,
        tr("panel.sculpt3d.detail"),
        ids::SCULPT3D_SEC_TOPOLOGY,
        &ids::SCULPT3D_DETAIL,
        &detail,
        snap.ui.detail as usize,
        x,
        w,
        y,
    );
    // O nível vivo é um FATO, e ele fica entre os dois botões que o movem — sem
    // ele, descer e subir são dois botões que não dizem onde você está (a malha
    // de baixo se PARECE com a de cima alisada).
    y = readout(
        ctx,
        &format!(
            "{}: {} / {}",
            tr("panel.sculpt3d.level"),
            snap.level,
            snap.level_count.saturating_sub(1)
        ),
        x,
        w,
        y,
    );
    y = row_of_two(
        ctx,
        (ids::SCULPT3D_LEVEL_DOWN, "-"),
        (ids::SCULPT3D_LEVEL_UP, "+"),
        x,
        w,
        y,
    ) + gap;
    y = row_of_two(
        ctx,
        (ids::SCULPT3D_SUBDIVIDE, tr("panel.sculpt3d.subdivide")),
        (ids::SCULPT3D_REVERSE, tr("panel.sculpt3d.reverse")),
        x,
        w,
        y,
    ) + gap;
    // ⚠️ **Só com a pilha MONTADA.** Com um nível o achatar é um no-op, e um
    // botão que não faz nada é pior que um botão que falta — a mesma lei que
    // esconde as rows de um verbo que não as lê. E ele fica LOGO ABAIXO dos dois
    // que constroem a pilha, porque é deles que ele é o inverso.
    if snap.level_count > 1 {
        y = command(
            ctx,
            ids::SCULPT3D_FLATTEN,
            tr("panel.sculpt3d.flatten"),
            x,
            w,
            y,
        ) + gap;
    }
    y = row_of_two(
        ctx,
        (ids::SCULPT3D_REMESH, tr("panel.sculpt3d.remesh")),
        (ids::SCULPT3D_CLOSE_HOLES, tr("panel.sculpt3d.close_holes")),
        x,
        w,
        y,
    ) + gap;
    // ⚠️ **A retopologia fica ao LADO do voxel remesh de propósito:** as duas
    // reconstroem a malha, e o artista escolhe entre elas pela pergunta que faz —
    // *arrumar o que a escultura destruiu* contra *pôr a grade a correr ao longo
    // da forma* (ADR-0160 §1).
    //
    // ⭐ **E QUAL dos dois motores, logo ACIMA do botão que os chama.** Ver
    // [`crate::state::RetopoMode`]: o `Global` entrega 100 % de quads e paga em
    // relógio; o `Local` é o porte do Instant Meshes, sub-segundo e robusto.
    // ⛔ Ele viveu atrás de `PH2D_RETOPO_LEGACY=1` a wave inteira — *um motor que o
    // painel não oferece não existe para o artista.*
    {
        let selected = crate::state::RetopoMode::ALL
            .iter()
            .position(|&m| m == snap.ui.retopo_mode)
            .unwrap_or(0);
        let labels: Vec<&str> = crate::state::RetopoMode::ALL
            .iter()
            .map(|m| m.label())
            .collect();
        y = widgets::labelled_seg(
            ctx,
            tr("panel.sculpt3d.retopo_mode"),
            ids::SCULPT3D_SEC_TOPOLOGY,
            &ids::SCULPT3D_RETOPO_MODE,
            &labels,
            selected,
            x,
            w,
            y,
        ) + gap;
    }
    y = command(
        ctx,
        ids::SCULPT3D_QUAD_REMESH,
        tr("panel.sculpt3d.quad_remesh"),
        x,
        w,
        y,
    ) + gap;
    // ⚠️ **A pista fica LOGO ABAIXO do botão que a lê**, e não no alto da seção:
    // ela é argumento do Remesh, e separá-los faria dela um número que aparece
    // do nada e não se liga ao gesto que o artista acabou de dar — a mesma
    // lição que o `Alpha Scale` custou um smoke (ver `Row::place`).
    for row in rows::TOPOLOGY {
        y = paint_one_row(ctx, snap, row, x, w, y);
    }
    widgets::end_fold(ctx, fold, y + Spacing::Md.px())
}

/// **A CENA** — a lista de peças e os verbos que a mexem.
fn paint_scene(ctx: &mut PaintCtx, snap: &Sculpt3dSnapshot, x: f32, w: f32, y: f32) -> f32 {
    let gap = Spacing::Sm.px();
    let (fold, mut y) = header(
        ctx,
        ids::SCULPT3D_SEC_SCENE,
        tr("panel.sculpt3d.section.scene"),
        x,
        w,
        y,
    );
    let Some(fold) = fold else {
        return y;
    };
    let add: Vec<&str> = ADD_LABELS.iter().map(|k| tr(k)).collect();
    y = labelled_seg(
        ctx,
        tr("panel.sculpt3d.add"),
        ids::SCULPT3D_SEC_SCENE,
        &ids::SCULPT3D_ADD,
        &add,
        usize::MAX, // gestos, não um modo
        x,
        w,
        y,
    );
    y = row_of_two(
        ctx,
        (ids::SCULPT3D_DUPLICATE, tr("panel.sculpt3d.duplicate")),
        (ids::SCULPT3D_DELETE, tr("panel.sculpt3d.delete")),
        x,
        w,
        y,
    ) + gap;
    // O Isolate é o único desta fileira com ESTADO — ele fica aceso enquanto a
    // cena está reduzida a uma peça, senão o artista perde quatro objetos e não
    // tem na tela nada que explique por quê.
    let half = (w - gap) * 0.5;
    toggle(
        ctx,
        ids::SCULPT3D_ISOLATE,
        tr("panel.sculpt3d.isolate"),
        snap.isolated,
        x,
        half,
        y,
    );
    y = command(
        ctx,
        ids::SCULPT3D_MERGE,
        tr("panel.sculpt3d.merge"),
        x + half + gap,
        half,
        y,
    ) + gap;
    y = readout(
        ctx,
        &format!(
            "{}: {}   {}: {}",
            tr("panel.sculpt3d.pieces"),
            snap.pieces,
            tr("panel.sculpt3d.verts"),
            snap.verts
        ),
        x,
        w,
        y,
    );
    widgets::end_fold(ctx, fold, y + gap)
}

/// Uma seção de knobs da tabela, com um sufixo opcional (o falloff, a máscara).
/// Uma row, onde quer que ela seja desenhada. **Porta única** — o bloco de knobs
/// e a cauda a chamam, então uma row de cauda não pode nascer com espaçamento ou
/// leitura diferentes das irmãs.
pub(super) fn paint_one_row(
    ctx: &mut PaintCtx,
    snap: &Sculpt3dSnapshot,
    row: &rows::Row,
    x: f32,
    w: f32,
    y: f32,
) -> f32 {
    let value = (row.get)(&snap.ui);
    let used = super::paint_row(ctx, row, value, x, w, y);
    y + used + Spacing::Sm.px()
}

// ⚠️ **Oito, e o oitavo é o irmão simétrico do `tail`.** A alternativa era
// perguntar `section.id == SCULPT3D_SEC_BRUSH` aqui dentro — uma ENUMERAÇÃO
// dentro da função genérica, que é precisamente a forma que apodrece quando a
// segunda seção ganha cabeça. Precedente do `body_desc` da física.
#[allow(clippy::too_many_arguments)]
fn knob_section(
    ctx: &mut PaintCtx,
    snap: &Sculpt3dSnapshot,
    section: &rows::Section,
    x: f32,
    w: f32,
    y_in: f32,
    head: impl Fn(&mut PaintCtx, &Sculpt3dSnapshot, f32, f32, f32) -> f32,
    tail: impl Fn(&mut PaintCtx, &Sculpt3dSnapshot, f32, f32, f32) -> f32,
) -> f32 {
    let (fold, mut y) = header(ctx, section.id, tr(section.title), x, w, y_in);
    let Some(fold) = fold else {
        return y;
    };
    y = head(ctx, snap, x, w, y);
    for row in section.rows {
        // ⚠️ A row condicional é PULADA, não desenhada apagada: um controle
        // apagado que ainda despacha mente, e um que não despacha é a affordance
        // morta que esta casa varre.
        // E a row de CAUDA é pulada aqui porque quem a desenha é o `tail`, ao
        // lado do controle que a governa — ver `Row::place`.
        if row.place != rows::Place::Knobs || !row.visible(&snap.ui) {
            continue;
        }
        y = paint_one_row(ctx, snap, row, x, w, y);
    }
    y = tail(ctx, snap, x, w, y);
    widgets::end_fold(ctx, fold, y + Spacing::Md.px())
}

/// Ver [`crate::paint::readout_at`] — a porta única do readout para o preview.
pub(super) fn readout_for(ctx: &mut PaintCtx, text: &str, x: f32, w: f32, y: f32) -> f32 {
    readout(ctx, text, x, w, y)
}

/// **A ENTREGA** — a forma escrita num objeto da cena 2D (`docs/3D/02.2`, o
/// objetivo 2 do módulo).
///
/// ⚠️ **Seção própria, e por último.** As cinco de cima descrevem *como a
/// escultura é*; esta descreve *o que sai dela*, e é o gesto mais raro do painel
/// — que é exatamente a lei de ordenação que o doc do topo declara. Uma linha na
/// cauda do sombreamento a colaria no *Bake Occlusion*, e os dois carregam a
/// palavra **bake** significando coisas diferentes: aquele mede um canal e o
/// escreve na MALHA, este escreve a forma inteira num SPRITE.
///
/// ⚠️ **O botão é SEMPRE pintado, e a dica é que some.** Esconder o botão sem
/// alvo tornaria a única entrega do módulo invisível justamente para quem ainda
/// não sabe que ela existe — que é a queixa que ele veio resolver (até aqui o
/// gesto tinha uma porta só, o `Shift+B`, e nada na tela a mencionava). A
/// condição é DITA, no molde do `ao_stale`: a linha só existe quando há o que
/// avisar, porque um aviso permanente vira moldura.
fn paint_bake(ctx: &mut PaintCtx, snap: &Sculpt3dSnapshot, x: f32, w: f32, y: f32) -> f32 {
    let (fold, mut y) = header(
        ctx,
        ids::SCULPT3D_SEC_BAKE,
        tr("panel.sculpt3d.section.bake"),
        x,
        w,
        y,
    );
    let Some(fold) = fold else {
        return y;
    };
    y = command(
        ctx,
        ids::SCULPT3D_BAKE_SPRITE,
        tr("panel.sculpt3d.bake_sprite"),
        x,
        w,
        y,
    );
    if !snap.has_bake_target {
        y = readout(ctx, tr("panel.sculpt3d.bake_sprite.hint"), x, w, y);
    }
    widgets::end_fold(ctx, fold, y + Spacing::Md.px())
}
