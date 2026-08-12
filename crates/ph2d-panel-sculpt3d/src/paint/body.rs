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
use ph2d_sculpt3d::{Alpha, Falloff};
use ph2d_tokens::{ROW_H_PX, Spacing};

use super::mask_tools::paint_mask_tools;
use super::tool::paint_tool;
use super::widgets::{command, header, labelled_seg, readout, row_of_two, toggle};

use crate::preview;
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
    y = knob_section(ctx, snap, &rows::SECTIONS[0], x, w, y, paint_brush_tail);
    y = paint_symmetry(ctx, snap, x, w, y);
    y = paint_topology(ctx, snap, x, w, y);
    y = knob_section(ctx, snap, &rows::SECTIONS[1], x, w, y, paint_shading_tail);
    y = paint_scene(ctx, snap, x, w, y);
    paint_bake(ctx, snap, x, w, y)
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
    let (open, mut y) = header(
        ctx,
        ids::SCULPT3D_SEC_BAKE,
        tr("panel.sculpt3d.section.bake"),
        x,
        w,
        y,
    );
    if !open {
        return y;
    }
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
    y + Spacing::Md.px()
}

/// O que vem DEPOIS dos knobs do pincel: a curva e as operações de máscara.
fn paint_brush_tail(ctx: &mut PaintCtx, snap: &Sculpt3dSnapshot, x: f32, w: f32, y: f32) -> f32 {
    // O falloff logo abaixo dos knobs: ele é a FORMA do peso, e a força é
    // quanto dele se aplica.
    let selected = Falloff::ALL
        .iter()
        .position(|&f| f == snap.ui.brush.falloff)
        .unwrap_or(0);
    let labels: Vec<&str> = Falloff::ALL.iter().map(|f| f.label()).collect();
    let y = labelled_seg(
        ctx,
        tr("panel.sculpt3d.falloff"),
        ids::SCULPT3D_SEC_BRUSH,
        &ids::SCULPT3D_FALLOFF,
        &labels,
        selected,
        x,
        w,
        y,
    );
    // **O PADRÃO**, logo abaixo do falloff — os dois moldam o MESMO peso: o
    // falloff diz como ele cai do centro à borda, o alpha diz onde ele age
    // dentro disso. A primeira opção é NENHUM, e o deslocamento de um é a mesma
    // aritmética que o seletor de matcap usa (o `event` a desfaz com
    // `checked_sub`; as duas metades vivem uma ao lado da outra de propósito).
    let mut labels: Vec<&str> = vec![tr("panel.sculpt3d.alpha.none")];
    labels.extend(Alpha::ALL.iter().map(|a| a.label()));
    // ⚠️ **O SLOT DE IMAGEM tem o nome do SPRITE**, e ele é o ÚLTIMO chip.
    //
    // ⚠️ **Isto REVOGA a decisão que estava escrita aqui.** A versão anterior
    // dizia que uma imagem nunca está na `ALL`, que o `position` devolve `None`
    // para ela e que cair em *nenhum* era honesto — *"o chip aceso não mente
    // sobre um padrão que aquela fileira não oferece"*. A premissa era que a
    // fileira não o oferecia; agora ela oferece, e o que sobrava era um painel
    // dizendo **None** com um padrão vivo e um preview desenhado logo abaixo:
    // o artista lia o painel como quebrado antes de olhar para a miniatura.
    //
    // ⚠️ **O nome vem do RETRATO, não do `Alpha`.** O motor guarda os pixels; de
    // onde eles vieram é proveniência da CENA. Um `label()` que devolvesse o
    // nome do sprite obrigaria o enum a carregar uma `String` que kernel nenhum
    // lê.
    if let Some(name) = snap.alpha_image_name.as_deref() {
        labels.push(name);
    }
    let selected = crate::state::alpha_chip_index(snap);
    let mut y = labelled_seg(
        ctx,
        tr("panel.sculpt3d.alpha"),
        ids::SCULPT3D_SEC_BRUSH,
        &ids::SCULPT3D_ALPHA,
        &labels,
        selected,
        x,
        w,
        y,
    );
    // **O ALPHA POR IMAGEM**, logo abaixo da fileira de nomes — e ele é um
    // BOTÃO, não um décimo chip. A fileira lista NOMES (as nove fórmulas); uma
    // imagem não é um nome, é uma coisa para a qual se aponta. Um chip "Image"
    // teria de existir antes de haver pixels, e é justamente esse estado que o
    // `Alpha::Image` torna inexprimível ao carregar a imagem dentro de si.
    //
    // ⚠️ **Sem sprite selecionado ele NÃO é pintado**, e não é dimming: um botão
    // que só pode falhar é como o artista aprende que ele não funciona. É a
    // mesma decisão do "Bake to Sprite" logo acima, que mostra uma dica no lugar.
    if snap.has_bake_target {
        y = command(
            ctx,
            ids::SCULPT3D_ALPHA_SPRITE,
            tr("panel.sculpt3d.alpha_sprite"),
            x,
            w,
            y,
        );
    }
    // ⚠️ **A pista de escala vem AQUI, colada nos chips que a governam** — e não
    // no bloco de knobs acima, que é onde ela nasceu e onde o smoke a perdeu:
    // lá ela aparecia do nada, separada do seletor pela fileira do Falloff, e o
    // artista lia um número sem saber de que ele era. A row continua na tabela
    // (ver `Row::place`); o que mudou é onde ela é desenhada.
    for row in rows::rows().filter(|r| r.place == rows::Place::AfterAlpha && (r.show)(&snap.ui)) {
        y = paint_one_row(ctx, snap, row, x, w, y);
    }
    // **O PREVIEW**, logo ABAIXO das pistas que o mudam — e a posição é a mesma
    // decisão que moveu a pista de escala para cá: um controle e o que ele
    // governa têm de estar no campo de visão um do outro, senão o artista arrasta
    // um número olhando para outro lugar.
    // **O interruptor do preview NO BARRO**, entre as pistas e o quadro — os
    // dois mostram o mesmo padrão e a caixa governa o de FORA, então ela fica
    // onde o olho já está. ⚠️ Só com padrão armado, pela mesma razão da pista de
    // escala: sem padrão ele é um interruptor de coisa nenhuma.
    if snap.ui.brush.alpha.is_some() {
        y = toggle(
            ctx,
            ids::SCULPT3D_ALPHA_PREVIEW,
            tr("panel.sculpt3d.alpha_preview"),
            snap.ui.alpha_preview,
            x,
            w,
            y,
        );
    }
    y = preview::paint(ctx, snap, x, w, y);
    // **ACUMULAR**, e só onde ele faz alguma coisa. ⚠️ A pergunta é feita à
    // PORTA do motor (`Verb::accumulates`) e não a uma lista de nomes aqui: o
    // aplicador pergunta à mesma para honrar o clique, e duas cópias divergiriam
    // num interruptor que aparece e não muda nada — quem tem âncora carrega o
    // gesto TOTAL desde o pen-down, e somar totais não significa nada.
    let y = if snap.ui.brush.verb.accumulates() {
        toggle(
            ctx,
            ids::SCULPT3D_ACCUMULATE,
            tr("panel.sculpt3d.accumulate"),
            snap.ui.brush.accumulate,
            x,
            w,
            y,
        ) + Spacing::Sm.px()
    } else {
        y
    };
    paint_mask_tools(ctx, snap, x, w, y)
}

/// **O ESPELHO** — três botões INDEPENDENTES.
///
/// Não é um rádio: um segmented é *um de N* por construção, e o ZBrush espelha
/// em dois eixos ao mesmo tempo.
fn paint_symmetry(ctx: &mut PaintCtx, snap: &Sculpt3dSnapshot, x: f32, w: f32, y: f32) -> f32 {
    let gap = Spacing::Sm.px();
    let (open, y) = header(
        ctx,
        ids::SCULPT3D_SEC_SYMMETRY,
        tr("panel.sculpt3d.section.symmetry"),
        x,
        w,
        y,
    );
    if !open {
        return y;
    }
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
    y + ROW_H_PX + Spacing::Md.px()
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
    let (open, mut y) = header(
        ctx,
        ids::SCULPT3D_SEC_TOPOLOGY,
        tr("panel.sculpt3d.section.topology"),
        x,
        w,
        y,
    );
    if !open {
        return y;
    }
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
    // ⚠️ **A pista fica LOGO ABAIXO do botão que a lê**, e não no alto da seção:
    // ela é argumento do Remesh, e separá-los faria dela um número que aparece
    // do nada e não se liga ao gesto que o artista acabou de dar — a mesma
    // lição que o `Alpha Scale` custou um smoke (ver `Row::place`).
    for row in rows::TOPOLOGY {
        y = paint_one_row(ctx, snap, row, x, w, y);
    }
    y + Spacing::Md.px()
}

/// **A CENA** — a lista de peças e os verbos que a mexem.
fn paint_scene(ctx: &mut PaintCtx, snap: &Sculpt3dSnapshot, x: f32, w: f32, y: f32) -> f32 {
    let gap = Spacing::Sm.px();
    let (open, mut y) = header(
        ctx,
        ids::SCULPT3D_SEC_SCENE,
        tr("panel.sculpt3d.section.scene"),
        x,
        w,
        y,
    );
    if !open {
        return y;
    }
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
    y + gap
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

fn knob_section(
    ctx: &mut PaintCtx,
    snap: &Sculpt3dSnapshot,
    section: &rows::Section,
    x: f32,
    w: f32,
    y_in: f32,
    tail: impl Fn(&mut PaintCtx, &Sculpt3dSnapshot, f32, f32, f32) -> f32,
) -> f32 {
    let (open, mut y) = header(ctx, section.id, tr(section.title), x, w, y_in);
    if !open {
        return y;
    }
    for row in section.rows {
        // ⚠️ A row condicional é PULADA, não desenhada apagada: um controle
        // apagado que ainda despacha mente, e um que não despacha é a affordance
        // morta que esta casa varre.
        // E a row de CAUDA é pulada aqui porque quem a desenha é o `tail`, ao
        // lado do controle que a governa — ver `Row::place`.
        if row.place != rows::Place::Knobs || !(row.show)(&snap.ui) {
            continue;
        }
        y = paint_one_row(ctx, snap, row, x, w, y);
    }
    y = tail(ctx, snap, x, w, y);
    y + Spacing::Md.px()
}

/// Ver [`crate::paint::readout_at`] — a porta única do readout para o preview.
pub(super) fn readout_for(ctx: &mut PaintCtx, text: &str, x: f32, w: f32, y: f32) -> f32 {
    readout(ctx, text, x, w, y)
}
