//! **O vidro no journal por tile** (ADR-0205): o passo do traço captura os tiles do vidro que a região
//! escrita toca, e o antes reconstruído (`journal.get(i).unwrap_or(vivo[i])`) é o plano de antes ao bit.

use crate::compositor::vidro::Vidro;
use crate::tool::PainterTool;
use crate::tool::paint::media::PaintMedia;
use crate::undo::journal::TILE;
use ph2d_editor_core::Tool;
use ph2d_editor_core::tool::{CanvasPaintTool, CanvasPointer, PointerPhase, RasterEditTool};

/// 4×3 tiles de 128, a última coluna (8 px) e a última linha (6 px) parciais.
pub(super) const W: u32 = 520;
pub(super) const H: u32 = 390;

pub(super) fn ev(t: &mut PainterTool, pos: [f32; 2], phase: PointerPhase) {
    t.on_canvas_pointer(CanvasPointer {
        pos,
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase,
    });
    t.on_tick(1000.0 / 60.0);
}

pub(super) fn plano(t: &PainterTool) -> Vec<Vidro> {
    let c = t.layers.active().expect("camada activa");
    t.vidros.get(&c).map(|p| p.to_vec()).unwrap_or_default()
}

/// Um traço de `a` a `b`; antes do pen-up confere o journal contra o plano de antes do traço.
fn traco_conferido(t: &mut PainterTool, a: [f32; 2], b: [f32; 2], nome: &str) {
    let antes = plano(t);
    ev(t, a, PointerPhase::Down);
    for k in 1..=10 {
        let s = k as f32 / 10.0;
        ev(
            t,
            [a[0] + (b[0] - a[0]) * s, a[1] + (b[1] - a[1]) * s],
            PointerPhase::Move,
        );
    }
    let vivo = plano(t);
    let ws = &t.undo.write_state;
    let (w, h) = (W as usize, H as usize);
    assert_eq!(antes.len(), w * h, "{nome}: o vidro existe antes do traço");
    let reconstruido: Vec<Vidro> = (0..w * h)
        .map(|i| ws.vidro_before(i).unwrap_or(vivo[i]))
        .collect();
    assert!(
        reconstruido == antes,
        "{nome}: o antes reconstruído do journal difere do plano de antes"
    );
    assert!(vivo != antes, "{nome}: o traço escreveu o vidro");
    // Os bytes = a soma dos tiles da caixa declarada (cada tile pelo tamanho real), nunca o plano inteiro.
    let r = ws.get().declarada().expect("a aguada declara a região");
    let (tx0, ty0) = (r.x as usize / TILE, r.y as usize / TILE);
    let (tx1, ty1) = (
        (r.x + r.w - 1) as usize / TILE,
        (r.y + r.h - 1) as usize / TILE,
    );
    let tile = |tx: usize, ty: usize| {
        (((tx + 1) * TILE).min(w) - tx * TILE) * (((ty + 1) * TILE).min(h) - ty * TILE)
    };
    let tecto: usize = (ty0..=ty1)
        .flat_map(|ty| (tx0..=tx1).map(move |tx| (tx, ty)))
        .map(|(tx, ty)| tile(tx, ty) * size_of::<Vidro>())
        .sum();
    let bytes = ws.vidro_journal_bytes();
    assert!(
        bytes <= tecto && bytes < w * h * size_of::<Vidro>(),
        "{nome}: o journal retém {bytes} B, a caixa declarada pede no máximo {tecto} B"
    );
    // Todo tile onde o vidro mudou foi capturado (senão o antes reconstruído já teria falhado acima).
    assert!(bytes > 0, "{nome}: o journal capturou o vidro");
    ev(t, b, PointerPhase::Up);
    for _ in 0..3 {
        t.on_tick(1000.0 / 60.0);
    }
}

#[test]
fn o_traco_que_cruza_tiles_guarda_so_os_tiles_tocados() {
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; (W * H * 4) as usize], W, H);
    t.set_paint_media(PaintMedia::Watercolor);
    crate::tool::paint::tests::aquarela_neutra(&mut t);
    crate::tool::paint::tests::watercolor_basic_de_antes(&mut t);
    t.set_brush_size_px(10.0);
    t.set_brush_color_srgb8([30, 60, 220]);
    // O 1.º traço faz nascer o vidro (sem antes a descrever).
    ev(&mut t, [40.0, 40.0], PointerPhase::Down);
    for k in 1..=8 {
        ev(
            &mut t,
            [40.0 + 50.0 * k as f32, 40.0 + 35.0 * k as f32],
            PointerPhase::Move,
        );
    }
    let ws = &t.undo.write_state;
    assert!(
        ws.vidro_absent() && ws.vidro_journal_bytes() == 0,
        "o vidro que nasce no traço é `absent` (sem antes), não capturado: absent={} bytes={} plano={}",
        ws.vidro_absent(),
        ws.vidro_journal_bytes(),
        plano(&t).len()
    );
    ev(&mut t, [440.0, 320.0], PointerPhase::Up);
    for _ in 0..3 {
        t.on_tick(1000.0 / 60.0);
    }
    traco_conferido(&mut t, [60.0, 20.0], [400.0, 300.0], "diagonal");
    traco_conferido(
        &mut t,
        [127.5, 30.0],
        [128.5, 200.0],
        "colado a x = 127|128",
    );
    traco_conferido(&mut t, [505.0, 375.0], [517.0, 387.0], "canto parcial");
}

/// **ADR-0205 passo 0 — quem segura o plano do vidro no 2.º pen-down** (a contagem não depende do tamanho da
/// tela). Imprime, por dono, quem segura o plano vivo antes e depois do 1.º quadro do 2.º traço, e as cópias.
/// Asserta só que a conta fecha: nenhum dono sem nome.
#[test]
fn quem_segura_o_vidro_no_segundo_pen_down() {
    use super::watercolor_vidro::COPIAS_DO_VIDRO;
    use std::sync::Arc;
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; (W * H * 4) as usize], W, H);
    t.set_paint_media(PaintMedia::Watercolor);
    crate::tool::paint::tests::aquarela_neutra(&mut t);
    crate::tool::paint::tests::watercolor_basic_de_antes(&mut t);
    t.set_brush_size_px(10.0);
    let traco = |t: &mut PainterTool, y: f32| {
        ev(t, [40.0, y], PointerPhase::Down);
        for k in 1..=6 {
            ev(t, [40.0 + 20.0 * k as f32, y], PointerPhase::Move);
        }
        ev(t, [160.0, y], PointerPhase::Up);
        for _ in 0..3 {
            t.on_tick(1000.0 / 60.0);
        }
    };
    traco(&mut t, 60.0);
    let c = t.layers.active().expect("camada");
    let donos = |t: &PainterTool, quando: &str| {
        let p = t.vidros.get(&c).expect("o vidro existe").clone();
        let total = Arc::strong_count(&p) - 1; // menos o clone desta linha
        let antes = t
            .paint
            .stroke_undo
            .as_ref()
            .and_then(|s| s.vidros.get(&c))
            .is_some_and(|v| Arc::ptr_eq(v, &p));
        let cursor = t.undo.cursor_segura_o_vidro(c, &p);
        let congelado = false; // D3: o congelado guarda o journal da sessão, não o plano
        let nomeados = 1 + usize::from(antes) + usize::from(cursor) + usize::from(congelado);
        println!(
            "{quando}: strong={total} · mapa=1 · before={antes} · cursor={cursor} · congelado={congelado} · sem nome={}",
            total as isize - nomeados as isize
        );
        (total, nomeados)
    };
    let repouso = donos(&t, "repouso depois do 1.º traço");
    COPIAS_DO_VIDRO.set(0);
    traco(&mut t, 110.0);
    println!("cópias do plano no 2.º traço: {}", COPIAS_DO_VIDRO.get());
    let repouso_3 = donos(&t, "repouso depois do 2.º traço");
    COPIAS_DO_VIDRO.set(0);
    traco(&mut t, 140.0);
    println!("cópias do plano no 3.º traço: {}", COPIAS_DO_VIDRO.get());
    // O 1.º traço CRIOU o plano: sem a chave a entrada dele (`OnlyAfter`) segura-o até ao 2.º traço; com
    // ela guarda só a janela (`Aparece`, ADR-0205) e não segura nada.
    let da_entrada = usize::from(!crate::undo::window::vidro_pelo_journal());
    assert_eq!(
        repouso.0,
        repouso.1 + da_entrada,
        "depois do 1.º traço: mapa + cursor (+ a entrada que o criou, sem a chave)"
    );
    assert_eq!(
        repouso_3.0, repouso_3.1,
        "depois do 2.º traço: só mapa + cursor"
    );
}

/// Um tool WB numa tela branca, com a chave `VIDRO_PELO_JOURNAL` em `journal`.
pub(super) fn tool_wb(journal: bool) -> PainterTool {
    crate::undo::window::set_vidro_pelo_journal(journal);
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; (W * H * 4) as usize], W, H);
    t.set_paint_media(PaintMedia::Watercolor);
    crate::tool::paint::tests::aquarela_neutra(&mut t);
    crate::tool::paint::tests::watercolor_basic_de_antes(&mut t);
    t.set_brush_size_px(14.0);
    t
}

/// O traço `k` (sobrepostos e disjuntos, a cruzar fronteiras de tile).
pub(super) fn traco_k(t: &mut PainterTool, k: usize) {
    let y = 40.0 + 41.0 * (k % 8) as f32;
    let (x0, x1) = if k.is_multiple_of(2) {
        (30.0, 300.0)
    } else {
        (110.0, 480.0)
    };
    ev(t, [x0, y], PointerPhase::Down);
    for s in 1..=8 {
        let f = s as f32 / 8.0;
        ev(t, [x0 + (x1 - x0) * f, y + 30.0 * f], PointerPhase::Move);
    }
    ev(t, [x1, y + 30.0], PointerPhase::Up);
    for _ in 0..3 {
        t.on_tick(1000.0 / 60.0);
    }
}

/// ADR-0205 D4: com o `before` e o cursor a DESCREVER o vidro (journal por tile), o desfazer e o refazer do
/// 1.º, 2.º e N-ésimo traço dão o vidro E a tela ao bit do oráculo (a chave desligada), no mesmo processo.
#[test]
fn o_vidro_do_traco_volta_ao_bit_no_desfazer_e_no_refazer() {
    use crate::undo_planes::VIDRO_FROM_JOURNAL;
    let mut j = tool_wb(true);
    let mut o = tool_wb(false);
    let confere = |j: &PainterTool, o: &PainterTool, quando: &str| {
        assert!(plano(j) == plano(o), "{quando}: o vidro difere do oráculo");
        assert!(
            j.canvas_rgba.as_slice() == o.canvas_rgba.as_slice(),
            "{quando}: a tela difere do oráculo"
        );
    };
    // Cada rota corre com a SUA chave.
    let em = |t: &mut PainterTool, journal: bool, f: &mut dyn FnMut(&mut PainterTool)| {
        crate::undo::window::set_vidro_pelo_journal(journal);
        f(t);
    };
    VIDRO_FROM_JOURNAL.set(0);
    const N: usize = 8;
    // O vidro e a tela depois de cada traço: a verdade absoluta (o `side` é partilhado pelas duas rotas).
    let mut gravado = vec![(plano(&j), j.canvas_rgba.to_vec())];
    for k in 0..N {
        em(&mut j, true, &mut |t| traco_k(t, k));
        em(&mut o, false, &mut |t| traco_k(t, k));
        confere(&j, &o, &format!("depois do traço {}", k + 1));
        gravado.push((plano(&j), j.canvas_rgba.to_vec()));
    }
    let no_gravado = |j: &PainterTool, i: usize, quando: &str| {
        assert!(
            plano(j) == gravado[i].0,
            "{quando}: o vidro não é o de depois do traço {i}"
        );
        assert!(
            j.canvas_rgba.as_slice() == gravado[i].1.as_slice(),
            "{quando}: a tela não é a de depois do traço {i}"
        );
    };
    assert!(
        VIDRO_FROM_JOURNAL.get() >= (N - 1) as u32,
        "o vidro saiu do journal em {} de {N} traços (verde por vácuo?)",
        VIDRO_FROM_JOURNAL.get()
    );
    assert!(!plano(&j).is_empty(), "a aguada criou o vidro");
    for k in (0..N).rev() {
        let (a, b) = (
            {
                crate::undo::window::set_vidro_pelo_journal(true);
                j.undo_last()
            },
            {
                crate::undo::window::set_vidro_pelo_journal(false);
                o.undo_last()
            },
        );
        assert!(a && b, "desfazer o traço {}", k + 1);
        confere(&j, &o, &format!("desfeito o traço {}", k + 1));
        no_gravado(&j, k, &format!("desfeito o traço {}", k + 1));
    }
    assert!(
        plano(&j).is_empty(),
        "desfazer tudo tira o vidro que o 1.º traço criou"
    );
    for k in 0..N {
        crate::undo::window::set_vidro_pelo_journal(true);
        let a = j.redo_last();
        crate::undo::window::set_vidro_pelo_journal(false);
        let b = o.redo_last();
        assert!(a && b, "refazer o traço {}", k + 1);
        confere(&j, &o, &format!("refeito o traço {}", k + 1));
        no_gravado(&j, k + 1, &format!("refeito o traço {}", k + 1));
    }
    for k in 0..N / 2 {
        crate::undo::window::set_vidro_pelo_journal(true);
        j.undo_last();
        crate::undo::window::set_vidro_pelo_journal(false);
        o.undo_last();
        confere(&j, &o, &format!("desfeita a metade, passo {}", k + 1));
    }
    em(&mut j, true, &mut |t| traco_k(t, 3));
    em(&mut o, false, &mut |t| traco_k(t, 3));
    confere(&j, &o, "pintado outro traço depois da metade");
    crate::undo::window::set_vidro_pelo_journal(true);
    assert!(j.undo_last());
    crate::undo::window::set_vidro_pelo_journal(false);
    assert!(o.undo_last());
    confere(&j, &o, "desfeito o traço novo");
    crate::undo::window::set_vidro_pelo_journal(true);
}

/// **ADR-0205: o 2.º traço (e cada um depois) não copia o plano do vidro.** A entrada do 1.º traço guarda só a
/// janela fora do neutro (`StoredEntry::Aparece`); o cursor e o `before` descrevem o vidro pelo journal. O
/// controlo é a chave desligada no mesmo processo: lá o 2.º traço copia (a régua mede alguma coisa).
#[test]
fn o_segundo_traco_nao_copia_o_plano_do_vidro() {
    use super::watercolor_vidro::COPIAS_DO_VIDRO;
    const N: usize = 8;
    let copias = |journal: bool| {
        let mut t = tool_wb(journal);
        traco_k(&mut t, 0);
        let c = t.layers.active().expect("camada");
        assert!(t.vidros.contains_key(&c), "o 1.º traço criou o vidro");
        (1..N)
            .map(|k| {
                COPIAS_DO_VIDRO.set(0);
                traco_k(&mut t, k);
                COPIAS_DO_VIDRO.get()
            })
            .collect::<Vec<_>>()
    };
    let com = copias(true);
    let sem = copias(false);
    crate::undo::window::set_vidro_pelo_journal(true);
    println!("cópias por traço (2.º..{N}.º): com a chave {com:?} · sem {sem:?}");
    assert!(
        sem[0] >= 1,
        "controlo: sem a chave o 2.º traço copia o plano ({sem:?}) — senão a régua não mede nada"
    );
    assert!(
        com.iter().all(|&n| n == 0),
        "com a chave nenhum traço do 2.º ao {N}.º copia o plano do vidro: {com:?}"
    );
}

/// **ADR-0205 D2: o traço Digital sobre a aguada continua o vidro lendo o journal** (o `before` já não segura
/// o plano: `continua_o_vidro_do_traco` lê o antes pelo journal sobre o vivo). Três aguadas, depois Digital por
/// cima (sobrepostos e a cruzar tiles); depois de cada traço, de desfazer tudo o Digital e de o refazer, o vidro
/// e a tela são os do oráculo (a chave desligada) ao bit, no mesmo processo.
///
/// Duas cenas: a aguada sobre o papel branco (alfa saturado) e a aguada numa camada TRANSPARENTE (alfa < 255)
/// sobre papel de cor (o vidro segue o traço quadro a quadro), onde reler o vivo já selado no quadro anterior NÃO dá o selo (`alfas(o, v)` cai no ramo «o alfa desceu»): é
/// esta que separa o journal do vivo.
#[test]
fn o_traco_digital_sobre_a_aguada_continua_o_vidro_lendo_o_journal() {
    cena_digital_sobre_aguada(false);
    cena_digital_sobre_aguada(true);
}

fn cena_digital_sobre_aguada(transparente: bool) {
    use super::watercolor_vidro::COPIAS_DO_VIDRO;
    let mut j = tool_wb(true);
    let mut o = tool_wb(false);
    if transparente {
        for (t, journal) in [(&mut j, true), (&mut o, false)] {
            crate::undo::window::set_vidro_pelo_journal(journal);
            t.add_raster_layer("aguada").expect("camada transparente");
            // O papel de cor faz o vidro seguir o traço QUADRO a QUADRO (`segue_o_vidro_no_quadro`); sobre o
            // branco só o pen-up o continua, uma vez, e o vivo ainda é o antes.
            t.papel = Some([200, 170, 120]);
        }
    }
    let confere = |j: &PainterTool, o: &PainterTool, quando: &str| {
        assert!(plano(j) == plano(o), "{quando}: o vidro difere do oráculo");
        assert!(
            j.canvas_rgba.as_slice() == o.canvas_rgba.as_slice(),
            "{quando}: a tela difere do oráculo"
        );
    };
    let em = |t: &mut PainterTool, journal: bool, f: &mut dyn FnMut(&mut PainterTool)| {
        crate::undo::window::set_vidro_pelo_journal(journal);
        f(t);
    };
    for k in 0..3 {
        em(&mut j, true, &mut |t| traco_k(t, k));
        em(&mut o, false, &mut |t| traco_k(t, k));
    }
    confere(&j, &o, "depois das aguadas");
    if transparente {
        assert!(
            j.papel_atravessa_vidro(),
            "controlo: o papel de cor não atravessa o vidro"
        );
        let alfas: Vec<u8> = j
            .canvas_rgba
            .chunks(4)
            .map(|p| p[3])
            .filter(|&a| a > 0)
            .collect();
        assert!(
            !alfas.is_empty() && alfas.iter().any(|&a| a < 255),
            "controlo: a aguada na camada transparente não ficou com alfa < 255"
        );
    }
    let aguada = plano(&j);
    for (t, journal) in [(&mut j, true), (&mut o, false)] {
        em(t, journal, &mut |t| {
            t.set_paint_media(PaintMedia::Digital);
            t.set_brush_color_srgb8([200, 40, 30]);
            // Fluxo baixo: o mesmo texel recebe vários toques no MESMO traço, e o antes que cada quadro lê
            // (journal) já não é o vivo que o quadro anterior selou.
            t.set_brush_flow(0.3);
        });
    }
    const D: usize = 4;
    let mut copias = Vec::new();
    for k in 0..D {
        COPIAS_DO_VIDRO.set(0);
        em(&mut j, true, &mut |t| vai_e_volta(t, k, transparente));
        copias.push(COPIAS_DO_VIDRO.get());
        em(&mut o, false, &mut |t| vai_e_volta(t, k, transparente));
        confere(
            &j,
            &o,
            &format!("{transparente}: depois do Digital {}", k + 1),
        );
    }
    assert!(
        plano(&j) != aguada,
        "o Digital por cima mudou o vidro (senão o gate é verde por vácuo)"
    );
    println!("cópias do vidro por traço Digital, com a chave: {copias:?}");
    for k in (0..D).rev() {
        crate::undo::window::set_vidro_pelo_journal(true);
        let a = j.undo_last();
        crate::undo::window::set_vidro_pelo_journal(false);
        let b = o.undo_last();
        assert!(a && b, "desfazer o Digital {}", k + 1);
        confere(&j, &o, &format!("desfeito o Digital {}", k + 1));
    }
    assert!(
        plano(&j) == aguada,
        "desfeito todo o Digital, o vidro é o das aguadas"
    );
    for k in 0..D {
        crate::undo::window::set_vidro_pelo_journal(true);
        let a = j.redo_last();
        crate::undo::window::set_vidro_pelo_journal(false);
        let b = o.redo_last();
        assert!(a && b, "refazer o Digital {}", k + 1);
        confere(&j, &o, &format!("refeito o Digital {}", k + 1));
    }
    crate::undo::window::set_vidro_pelo_journal(true);
}

/// Um traço que vai e volta sobre si mesmo (cada texel tocado em vários quadros do mesmo traço); com `drena`,
/// cada evento passa pelo dreno da pré-visualização, como o quadro do app (o vidro segue o traço nele).
pub(super) fn vai_e_volta(t: &mut PainterTool, k: usize, drena: bool) {
    let ev = |t: &mut PainterTool, pos, phase| {
        ev(t, pos, phase);
        if drena {
            let _ = t.take_preview_arc();
        }
    };
    let y = 60.0 + 70.0 * k as f32;
    let xs = [40.0, 300.0, 60.0, 280.0, 90.0];
    ev(t, [xs[0], y], PointerPhase::Down);
    for par in xs.windows(2) {
        for s in 1..=6 {
            let f = s as f32 / 6.0;
            ev(
                t,
                [par[0] + (par[1] - par[0]) * f, y + 8.0 * f],
                PointerPhase::Move,
            );
        }
    }
    ev(t, [xs[4], y + 8.0], PointerPhase::Up);
    for _ in 0..3 {
        t.on_tick(1000.0 / 60.0);
    }
}
