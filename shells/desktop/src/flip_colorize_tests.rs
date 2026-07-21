//! Gates do **Colorize** no shell (`flip_colorize.rs`) — extraídos para o irmão pelo
//! teto de LOC do shell (HR-18, 600). A pilha de rabiscos, a posse do Ctrl+Z, o
//! re-Apply que SUBSTITUI em vez de empilhar, e a recusa que preserva o trabalho.

use super::*;
use ph2d_flip::Rgba;

/// Uma line-art de moldura FECHADA (um quadrado de tinta grossa) — o `boundaries` traça o
/// laço e o `colorize_with` preenche o interior onde a semente cai.
fn boxed_lineart() -> FlipDrawing {
    let mut s = FlipStroke::new();
    for &(x, y) in &[
        (0.0, 0.0),
        (100.0, 0.0),
        (100.0, 100.0),
        (0.0, 100.0),
        (0.0, 0.0),
    ] {
        s.push_point(Point {
            pos: Vec2::new(x, y),
            width: 4.0,
            opacity: 1.0,
            color: Rgba::BLACK,
        });
    }
    let mut d = FlipDrawing::default();
    d.strokes.push(s);
    d
}

/// Uma semente no centro do quadrado (rótulo 0), da paleta de uma cor.
fn center_seed() -> (Vec<[u8; 4]>, Vec<Scribble>) {
    (
        vec![[220, 70, 70, 255]],
        vec![Scribble {
            label: 0,
            points: vec![Vec2::new(40.0, 50.0), Vec2::new(60.0, 50.0)],
            width: 2.0,
        }],
    )
}

/// **O re-Apply ao vivo SUBSTITUI, não empilha** — o coração do Trap/Bleed em tempo real
/// (6º smoke, "faça ficar em tempo real"). A operação viva restaura a base congelada e
/// reinsere; se ela reinserisse SEM restaurar, cada tique do slider empilharia outra pilha
/// de fills e a borda ficaria opaca de camadas mortas.
///
/// Mutação que o derruba: **pular a restauração da base** antes do 2º `insert_regions` —
/// o desenho cresceria para `base + n1 + n2` em vez de `base + n2`.
#[test]
fn the_live_reapply_replaces_the_regions_it_does_not_stack_them() {
    let (palette, seeds) = center_seed();
    let base = boxed_lineart();
    let base_len = base.strokes.len();

    // 1ª aplicação (Bleed no default).
    let mut d = base.clone();
    let n1 = insert_regions(
        &mut d,
        &palette,
        &seeds,
        1.0,
        2.0,
        ph2d_flip_colorize::DEFAULT_SQUEEZE,
    );
    assert!(n1 >= 1, "a moldura fechada tem de dar ao menos 1 região");
    assert_eq!(
        d.strokes.len(),
        base_len + n1,
        "1ª inserção: base + regiões"
    );

    // Re-Apply ao vivo: RESTAURA a base, reinsere com outro Bleed (mais colado).
    d.strokes.clone_from(&base.strokes);
    let n2 = insert_regions(
        &mut d,
        &palette,
        &seeds,
        1.0,
        2.0,
        ph2d_flip_colorize::squeeze_from_bleed(0.0),
    );
    assert_eq!(
        d.strokes.len(),
        base_len + n2,
        "o re-Apply SUBSTITUI (base + n2), nunca empilha (base + n1 + n2)"
    );
}

fn scr(n: u8) -> ([u8; 4], Vec<Vec2>) {
    (
        [n, 0, 0, 255],
        vec![Vec2::new(0.0, 0.0), Vec2::new(1.0, f32::from(n))],
    )
}

/// **A disciplina da pilha de rabiscos** ("undo/redo ruim", 7º smoke): Ctrl+Z remove o
/// ÚLTIMO rabisco (LIFO), Ctrl+Shift+Z o devolve na ordem, e um rabisco NOVO descarta
/// os removidos — a lei de toda fila de redo (agir depois de desfazer mata o refazer).
#[test]
fn scribble_undo_pops_lifo_redo_restores_and_a_new_scribble_kills_the_redo() {
    let mut c = FlipColorize::default();
    for n in [1u8, 2, 3] {
        let (col, pts) = scr(n);
        c.push_scribble(col, pts);
    }
    assert!(c.can_undo_scribble() && !c.can_redo_scribble());

    c.undo_scribble(); // remove o 3
    c.undo_scribble(); // remove o 2
    assert_eq!(c.scribbles.len(), 1);
    assert_eq!(c.scribbles[0].0[0], 1, "o mais antigo fica");
    assert!(c.can_redo_scribble());

    c.redo_scribble(); // devolve o 2
    assert_eq!(c.scribbles.last().expect("redo").0[0], 2, "volta na ordem");

    // Um rabisco NOVO mata o redo pendente (o 3 nunca mais volta).
    let (col, pts) = scr(9);
    c.push_scribble(col, pts);
    assert!(
        !c.can_redo_scribble(),
        "agir depois de desfazer mata o refazer"
    );

    // Clear zera as DUAS filas.
    c.undo_scribble();
    c.clear();
    assert!(!c.can_undo_scribble() && !c.can_redo_scribble());
}

/// 🔴 **Um Apply RECUSADO não pode comer os rabiscos do artista** (auditoria 2026-07-20).
///
/// O `flip_colorize_apply` tem CINCO saídas que recusam o trabalho, e três delas mandam o
/// artista *corrigir e tentar de novo* ("a camada está travada", "desenhe a line-art
/// primeiro", "rabisque dentro das formas fechadas"). Enquanto o `mem::take` das sementes
/// morava no TOPO da função, obedecer era impossível: os rabiscos já tinham ido embora — e
/// o **Ctrl+Z não os devolvia**, porque a fila de removidos era limpa na linha seguinte e
/// o `undo_route` só é dono do atalho enquanto `can_undo_scribble()`. Uma recusa custava o
/// trabalho.
///
/// O fixture usa a recusa mais barata de alcançar headless (`gfx` ausente), mas a lei que
/// ele pina vale para as cinco: **só o SUCESSO consome**.
///
/// Mutação que sangra: devolver o `mem::take` (ou o `scribbles.clear()`) para antes das
/// validações ⇒ sobra 0 rabisco.
#[test]
fn a_refused_apply_keeps_the_scribbles_the_artist_drew() {
    let mut app = crate::App::new();
    app.flip_active = true;
    app.flip_style = Some(ph2d_tool_flip::FlipStyleSnapshot {
        mode: FlipMode::Colorize,
        ..Default::default()
    });
    for n in [1u8, 2] {
        let (col, pts) = scr(n);
        app.flip_colorize.push_scribble(col, pts);
    }
    assert_eq!(app.flip_colorize.scribbles.len(), 2, "semeado");

    // `App::new()` é headless (`gfx: None`), então o Apply RECUSA na 1ª saída.
    app.flip_colorize_apply();

    assert_eq!(
        app.flip_colorize.scribbles.len(),
        2,
        "um Apply recusado devolveu {} rabiscos em vez de 2 — o artista perdeu o \
         trabalho e o Ctrl+Z não o traz de volta",
        app.flip_colorize.scribbles.len()
    );
    // E o Ctrl+Z continua alcançável (o Colorize ainda é dono do atalho).
    assert!(
        app.flip_colorize.can_undo_scribble(),
        "com rabisco pendente o Colorize tem de seguir dono do Ctrl+Z"
    );
}

/// **Vazio, o Colorize CEDE o atalho** — é o que deixa o Ctrl+Z pós-Apply cair no
/// Global (o dono do Apply). O contrato dos `can_*` é exatamente este gate de posse.
#[test]
fn an_empty_scribble_buffer_yields_the_chord() {
    let c = FlipColorize::default();
    assert!(!c.can_undo_scribble());
    assert!(!c.can_redo_scribble());
}

/// 🔴 **A COSTURA do painel até o motor** (auditoria 2026-07-21): os dois sliders chegam ao
/// `colorize` por UMA função, e ela não tinha gate NENHUM — a auditoria tirou o `.max(seal_px)`
/// e **862 testes de shell ficaram verdes** enquanto a entrega do 6º smoke (*"Bleed 0 sela o
/// vão"*) morria. Um motor gateado atrás de uma costura não-gateada é um motor que o produto
/// não alcança.
///
/// Três leis, uma por camada:
/// 1. **O selo do `Bleed 0` chega** (`seal_from_bleed` × precisão) — a entrega escolhida pelo
///    Enio em 2026-07-20;
/// 2. **o Trap chega**, e cruza a precisão (BUGS #11: sem isso subir a Precision encolheria a
///    bola em silêncio — um raio em px de TELA lido como px de BUFFER);
/// 3. **os dois COMPÕEM por `max`** — são a mesma grandeza (força de selagem), não duas
///    perguntas: no Bleed 0 com Trap alto manda o Trap, e vice-versa.
///
/// Mutações que sangram: tirar o `.max(seal_px)` (1) · tirar o `* precision` do `trap_px` (2)
/// · trocar o `max` pelo `seal_px` sozinho (3).
#[test]
fn both_sealing_sliders_reach_the_engine_and_compose() {
    let style = |trap: f64, bleed: f64| ph2d_tool_flip::FlipStyleSnapshot {
        mode: FlipMode::Colorize,
        trap,
        colorize_bleed: bleed,
        ..Default::default()
    };
    // A câmera do produto: 10 unidades de mundo numa janela de 1080p, objeto sem escala.
    let (px_to_world, obj_scale) = (10.0 / 1080.0, 1.0);
    let precision = 1.6 / (px_to_world * obj_scale);

    // (1) Bleed 0, Trap 0: só o selo — e ele TEM de chegar em px de buffer.
    let (p, trap_px) = precision_and_trap(&style(0.0, 0.0), px_to_world, obj_scale);
    let seal = ph2d_flip_colorize::seal_from_bleed(0.0) * precision;
    assert!(
        seal > 0.0,
        "controle positivo: o Bleed 0 SELA (senão nada a testar)"
    );
    assert!((p - precision).abs() < 1e-3, "a precisão é a do balde");
    assert!(
        (trap_px - seal).abs() < 1e-3,
        "o selo do Bleed 0 tem de chegar ao motor: {trap_px} ≠ {seal}"
    );

    // (2) Bleed default (acima do joelho ⇒ sem selo), Trap 8 px de tela: o Trap sozinho,
    //     cruzando a precisão. Um `trap_px` que não a cruzasse ficaria em 8.
    let (_, trap_only) = precision_and_trap(&style(8.0, 0.5), px_to_world, obj_scale);
    assert_eq!(
        ph2d_flip_colorize::seal_from_bleed(0.5),
        0.0,
        "controle positivo: no Bleed default o selo não engaja"
    );
    assert!(
        trap_only > 8.0 * 1.5,
        "o Trap tem de cruzar a precisão (BUGS #11) — chegou {trap_only} px de buffer"
    );

    // (3) Os dois juntos COMPÕEM: manda o maior, nunca o último a ser escrito.
    let (_, big_trap) = precision_and_trap(&style(8.0, 0.0), px_to_world, obj_scale);
    assert!(
        (big_trap - trap_only.max(seal)).abs() < 1e-3,
        "Trap e selo compõem por max: {big_trap} ≠ max({trap_only}, {seal})"
    );
    let (_, small_trap) = precision_and_trap(&style(0.01, 0.0), px_to_world, obj_scale);
    assert!(
        (small_trap - seal).abs() < 1e-3,
        "com Trap minúsculo manda o SELO: {small_trap} ≠ {seal}"
    );
}

/// 🔴 **O ONION FILL: o MESMO rabisco colore os quadros selecionados** (fatia C3,
/// `09 §5.2`).
///
/// O que a C3 acrescenta ao multiframe do balde **não é o range** — esse já existia
/// (`flip_fill.rs`, W7) — é a **SEMENTE**: o balde replica um PONTO nos N desenhos, e aqui o
/// artista rabisca por cima das poses EMPILHADAS e cada quadro é semeado pelo TRAÇO inteiro.
///
/// A fixture põe a MESMA moldura em dois quadros, mas com a linha **deslocada** entre eles —
/// é o que um desenho animado faz, e é o que obriga cada quadro a resolver por conta própria
/// (a região muda de forma; não há contorno a reaproveitar). Um rabisco que cai dentro dos
/// DOIS tem de colorir os dois.
///
/// ⚠️ O oráculo é a APARÊNCIA (cada quadro ganhou uma região de preenchimento), nunca a
/// contagem de chamadas do `insert_regions` — um espelho da implementação ficaria verde com
/// as regiões saindo no desenho errado.
///
/// Mutação que sangra: `colorize_frames` iterando `dids.iter().take(0)` — o vizinho fica sem
/// cor e o gate nomeia qual quadro ficou.
#[test]
fn the_same_scribble_colours_every_selected_frame() {
    // Uma moldura fechada deslocada de `dx` — a "pose" daquele quadro.
    let framed = |dx: f32| -> FlipDrawing {
        let mut s = FlipStroke::new();
        for &(x, y) in &[
            (0.0, 0.0),
            (100.0, 0.0),
            (100.0, 100.0),
            (0.0, 100.0),
            (0.0, 0.0),
        ] {
            s.push_point(Point {
                pos: Vec2::new(x + dx, y),
                width: 4.0,
                opacity: 1.0,
                color: Rgba::BLACK,
            });
        }
        let mut d = FlipDrawing::default();
        d.strokes.push(s);
        d
    };
    let (palette, _) = center_seed();
    // Um RABISCO (não um ponto) largo o bastante para cair dentro das duas poses.
    let seeds = vec![Scribble {
        label: 0,
        points: vec![Vec2::new(45.0, 50.0), Vec2::new(60.0, 50.0)],
        width: 2.0,
    }];

    let mut produced = Vec::new();
    for dx in [0.0f32, 8.0] {
        let mut d = framed(dx);
        let before = d.strokes.len();
        let n = insert_regions(
            &mut d,
            &palette,
            &seeds,
            1.0,
            2.0,
            ph2d_flip_colorize::DEFAULT_SQUEEZE,
        );
        assert_eq!(
            d.strokes.len(),
            before + n,
            "cada quadro é um solve independente sobre a PRÓPRIA base"
        );
        produced.push(n);
    }
    assert!(
        produced.iter().all(|n| *n >= 1),
        "controle positivo: o MESMO rabisco tem de colorir as DUAS poses, não só a ativa \
         (regiões por quadro: {produced:?})"
    );
}

/// 🔴 **O fan-out ESCREVE em cada quadro selecionado** (fatia C3) — dirigindo o
/// `colorize_frames` sobre um `FlipDoc` de verdade, com DUAS chaves e a linha deslocada
/// entre elas.
///
/// ⚠️ Este gate existe porque o irmão de engine (`the_same_scribble_colours_every_selected
/// _frame`) chama o `insert_regions` **ele mesmo**, quadro a quadro — ele prova que uma
/// semente só resolve em duas poses, e ficaria VERDE com o laço do produto deletado. E o
/// arch-gate irmão lê o FONTE, então uma mutação que preserva a forma e neutraliza o laço
/// (`take(0)`) passa por ele. Só este aqui roda o laço do produto e conta o resultado.
///
/// Mutações que sangram: `dids.iter().take(0)` (nenhum vizinho colorido) · devolver o
/// `LiveFrame` mesmo com `produced == 0` (a base do quadro que não fechou entraria na sessão
/// viva e o Trap seguinte reescreveria um desenho que o gesto nunca tocou).
#[test]
fn the_fan_out_writes_a_region_into_every_frame_it_is_given() {
    use ph2d_flip::{Hold, KeyKind};

    let framed = |dx: f32| -> FlipStroke {
        let mut s = FlipStroke::new();
        for &(x, y) in &[
            (0.0, 0.0),
            (100.0, 0.0),
            (100.0, 100.0),
            (0.0, 100.0),
            (0.0, 0.0),
        ] {
            s.push_point(Point {
                pos: Vec2::new(x + dx, y),
                width: 4.0,
                opacity: 1.0,
                color: Rgba::BLACK,
            });
        }
        s
    };

    let mut flip = ph2d_flip::FlipDoc::default();
    let oid = flip.push_object("C3");
    let obj = flip.object_mut(oid).expect("objeto");
    let lid = obj.add_layer("L");
    // Duas chaves, poses DIFERENTES — é o que obriga cada quadro a resolver por conta.
    let mut dids = Vec::new();
    for (frame, dx) in [(0i32, 0.0f32), (4, 8.0)] {
        let did = obj
            .insert_frame(lid, frame, Hold::Implicit, KeyKind::Keyframe)
            .expect("chave");
        obj.drawing_mut(did)
            .expect("desenho")
            .strokes
            .push(framed(dx));
        dids.push(did);
    }
    // Um quadro EXTRA sem line-art nenhuma: ele não fecha, e tem de falhar em SILÊNCIO
    // sem derrubar os outros nem entrar na sessão viva.
    let empty = obj
        .insert_frame(lid, 8, Hold::Implicit, KeyKind::Keyframe)
        .expect("chave vazia");
    dids.push(empty);

    let (palette, _) = center_seed();
    let seeds = vec![Scribble {
        label: 0,
        points: vec![Vec2::new(45.0, 50.0), Vec2::new(60.0, 50.0)],
        width: 2.0,
    }];
    let before: Vec<usize> = dids
        .iter()
        .map(|d| {
            flip.object(oid)
                .and_then(|o| o.drawing(*d))
                .map_or(0, |dr| dr.strokes.len())
        })
        .collect();

    let frames = colorize_frames(
        &mut flip,
        oid,
        &dids,
        &palette,
        &seeds,
        1.0,
        2.0,
        ph2d_flip_colorize::DEFAULT_SQUEEZE,
    );

    assert_eq!(
        frames.len(),
        2,
        "os DOIS quadros com arte têm de ganhar região; o vazio falha em silêncio \
         (saíram {} sessões)",
        frames.len()
    );
    for (i, d) in dids.iter().take(2).enumerate() {
        let now = flip
            .object(oid)
            .and_then(|o| o.drawing(*d))
            .map_or(0, |dr| dr.strokes.len());
        assert!(
            now > before[i],
            "o quadro {i} tem de ter ganho preenchimento ({} -> {now})",
            before[i]
        );
    }
    // O quadro que não fechou volta INTOCADO — e não entra na sessão viva, senão o Trap
    // seguinte restauraria uma base que este gesto nunca escreveu.
    let empty_now = flip
        .object(oid)
        .and_then(|o| o.drawing(empty))
        .map_or(0, |dr| dr.strokes.len());
    assert_eq!(empty_now, before[2], "o quadro sem arte volta intocado");
    assert!(
        !frames.iter().any(|f| f.did == empty),
        "um quadro que não produziu região não pode entrar na sessão de ajuste ao vivo"
    );
}
