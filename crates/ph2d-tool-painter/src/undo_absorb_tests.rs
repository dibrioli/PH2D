//! Gates do **caminho barato da absorção** (`undo_delta_absorb.rs`) — filho de `undo_tests` para usar
//! o `model` de lá.
//!
//! A promessa é forte e é por isso que o gate é forte: o caminho barato guarda **a MESMA entrada** que
//! o caro, campo a campo — não uma parecida, não uma com os mesmos pixels. Cada cenário corre duas
//! vezes na mesma sequência de passos (uma com [`ONLY_THE_FULL_PATH`] ligado) e compara a pilha
//! inteira de undo, o cursor e o livro de bytes. Um gate de pixels ao desfazer passaria com uma janela
//! maior que a exacta, e o custo disso é memória, que ninguém vê sem medir.
//!
//! ⚠️ E cada cenário afirma **qual caminho correu** ([`CHEAP_FIRED`]): uma igualdade entre dois lados
//! que foram ambos o caminho caro é verdadeira e não diz nada.

use super::super::absorb::{ABSORB_FIRED, CHEAP_FIRED, DETECTED_WITHIN, ONLY_THE_FULL_PATH};
use super::super::window::WriteWindow;
use super::*;

/// Largura e altura da tela dos cenários: grande o bastante para que uma janela de poucos pixels seja
/// muito menos de meio plano, que é onde o caminho barato vive.
const LADO: u32 = 16;

/// Pixels trocados, `((x, y), valor)` — os quatro canais recebem o valor.
type Marcas = Vec<((usize, usize), u8)>;

/// Uma tela `LADO × LADO` de fundo `0x11`, com os pixels de `marcas` trocados.
fn tela(marcas: &[((usize, usize), u8)]) -> ModelSnapshot {
    let mut m = model(0x11);
    m.canvas_size = (LADO, LADO);
    let lado = LADO as usize;
    let mut px = vec![0x11u8; lado * lado * 4];
    for &((x, y), v) in marcas {
        let i = (y * lado + x) * 4;
        px[i..i + 4].fill(v);
    }
    m.canvas_rgba = Arc::new(px);
    m
}

/// Um bloco `[x0, x1) × [y0, y1)` pintado com `v`.
fn bloco(x0: usize, x1: usize, y0: usize, y1: usize, v: u8) -> Marcas {
    (y0..y1)
        .flat_map(|y| (x0..x1).map(move |x| ((x, y), v)))
        .collect()
}

/// O que uma sequência de passos deixa no controller: a pilha de undo, o cursor e o livro de bytes.
fn estado(c: &UndoController) -> (String, String, usize) {
    (format!("{:?}", c.undo), format!("{:?}", c.cursor), c.bytes)
}

/// Corre `passos` pelos dois caminhos e devolve quantas absorções o caminho BARATO fez.
fn pelos_dois_caminhos(passos: impl Fn(&mut UndoController)) -> u32 {
    pelos_dois_caminhos_contando(passos).0
}

/// O mesmo, devolvendo também quantas vezes o detector leu só a janela DECLARADA.
fn pelos_dois_caminhos_contando(passos: impl Fn(&mut UndoController)) -> (u32, u32) {
    let correr = |so_o_caro: bool| {
        ONLY_THE_FULL_PATH.with(|f| f.set(so_o_caro));
        CHEAP_FIRED.with(|f| f.set(0));
        DETECTED_WITHIN.with(|f| f.set(0));
        let mut c = UndoController::new(DEFAULT_MAX_BYTES);
        passos(&mut c);
        ONLY_THE_FULL_PATH.with(|f| f.set(false));
        (
            estado(&c),
            CHEAP_FIRED.with(std::cell::Cell::get),
            DETECTED_WITHIN.with(std::cell::Cell::get),
        )
    };
    let (caro, baratas_no_caro, dentro_no_caro) = correr(true);
    let (barato, baratas, dentro) = correr(false);
    assert_eq!(
        (baratas_no_caro, dentro_no_caro),
        (0, 0),
        "o controlo tem de correr SÓ o caminho caro"
    );
    assert_eq!(
        barato.2, caro.2,
        "o livro de bytes diverge: a janela guardada não é a exacta"
    );
    assert_eq!(barato.1, caro.1, "o cursor diverge");
    assert_eq!(
        barato.0, caro.0,
        "a pilha de undo diverge entre o caminho barato e o caro"
    );
    (baratas, dentro)
}

/// **O escorrido LONGE do traço, PERTO dele e DENTRO dele — a mesma entrada pelos dois caminhos.**
#[test]
fn the_cheap_absorption_stores_the_same_entry_as_the_full_one() {
    let traco = bloco(3, 6, 3, 6, 0x22);
    let casos: [(&str, Marcas); 3] = [
        ("longe", bloco(11, 13, 10, 12, 0x33)),
        ("a encostar", bloco(5, 8, 5, 7, 0x33)),
        ("dentro", bloco(4, 5, 4, 5, 0x33)),
    ];
    for (nome, pinga) in casos {
        let baratas = pelos_dois_caminhos(|c| {
            let depois = tela(&traco);
            c.record_structural(tela(&[]), depois);
            let mut escorrido = traco.clone();
            escorrido.extend(pinga.iter().copied());
            let mut seguinte = escorrido.clone();
            seguinte.push(((8, 1), 0x44));
            c.record_structural(tela(&escorrido), tela(&seguinte));
        });
        assert_eq!(baratas, 1, "{nome}: o caminho barato não correu");
    }
}

/// **O escorrido que DEVOLVE texels do traço ao fundo** — a janela exacta ENCOLHE, e é ela que tem de
/// sair, não a caixa `U`. É exactamente o que a evaporação do Wet Paint faz (o composite escreve a base
/// onde o pigmento secou).
#[test]
fn a_drip_that_undoes_part_of_the_stroke_shrinks_the_window_the_same_way() {
    let baratas = pelos_dois_caminhos(|c| {
        let traco = bloco(3, 7, 3, 7, 0x22);
        c.record_structural(tela(&[]), tela(&traco));
        // A borda de cima e a da esquerda do traço secaram de volta ao fundo.
        let escorrido: Vec<_> = traco
            .iter()
            .copied()
            .filter(|&((x, y), _)| x > 3 && y > 3)
            .collect();
        c.record_structural(tela(&escorrido), tela(&escorrido));
    });
    assert_eq!(baratas, 1, "o caminho barato não correu");
}

/// **O escorrido que apaga o traço INTEIRO** — o re-split não guarda nada no canvas.
#[test]
fn a_drip_that_undoes_the_whole_stroke_leaves_the_canvas_unchanged_the_same_way() {
    let baratas = pelos_dois_caminhos(|c| {
        c.record_structural(tela(&[]), tela(&bloco(3, 6, 3, 6, 0x22)));
        c.record_structural(tela(&[]), tela(&[((9, 9), 0x44)]));
    });
    assert_eq!(baratas, 1, "o caminho barato não correu");
}

/// **O topo que NÃO mexeu no canvas** (só metadados) — a janela é só a do escorrido.
#[test]
fn a_top_that_left_the_canvas_alone_absorbs_the_same_way() {
    let baratas = pelos_dois_caminhos(|c| {
        let mut depois = tela(&[]);
        depois.offset_norm = 0.75;
        c.record_structural(tela(&[]), depois.clone());
        let mut escorrido = tela(&bloco(2, 4, 2, 4, 0x33));
        escorrido.offset_norm = 0.75;
        c.record_structural(escorrido.clone(), escorrido);
    });
    assert_eq!(baratas, 1, "o caminho barato não correu");
}

/// **O topo gravado com uma janela DECLARADA maior que a exacta** — o re-split do caminho caro deriva
/// a exacta, e o barato tem de derivar a MESMA, não herdar a declarada.
#[test]
fn a_hinted_top_is_re_split_to_the_exact_window_the_same_way() {
    let baratas = pelos_dois_caminhos(|c| {
        let dica = crate::compositor::Region {
            x: 1,
            y: 1,
            w: 8,
            h: 8,
        };
        c.record_structural_hinted(tela(&[]), tela(&bloco(3, 5, 3, 5, 0x22)), Some(dica));
        let mut escorrido = bloco(3, 5, 3, 5, 0x22);
        escorrido.push(((6, 6), 0x33));
        c.record_structural(tela(&escorrido), tela(&escorrido));
    });
    assert_eq!(baratas, 1, "o caminho barato não correu");
}

/// **O run COALESCIDO também absorve pela porta barata** — os dois `record_*` entram pela absorção.
#[test]
fn a_coalesced_run_absorbs_the_same_way() {
    let baratas = pelos_dois_caminhos(|c| {
        let s1 = bloco(3, 5, 3, 5, 0x22);
        c.record_structural_coalesced(CoalesceKind::Simplify, tela(&[]), tela(&s1));
        let mut escorrido = s1.clone();
        escorrido.push(((10, 10), 0x33));
        let mut s2 = escorrido.clone();
        s2.push(((12, 2), 0x44));
        c.record_structural_coalesced(CoalesceKind::Simplify, tela(&escorrido), tela(&s2));
    });
    assert_eq!(baratas, 1, "o caminho barato não correu");
}

/// **Traço e escorrido em cantos OPOSTOS: a caixa `U` é meio plano ou mais, e o barato RECUSA.**
///
/// ⚠️ É a metade que prova a cerca: aí a janela exacta pode ser metade do plano, e o caminho caro
/// guarda `Whole` — um barato sem cerca guardaria `Patch` e a pilha divergiria.
#[test]
fn a_box_of_half_the_plane_or_more_falls_back_to_the_full_path() {
    let baratas = pelos_dois_caminhos(|c| {
        let traco = bloco(0, 2, 0, 2, 0x22);
        c.record_structural(tela(&[]), tela(&traco));
        let mut escorrido = traco.clone();
        escorrido.extend(bloco(14, 16, 14, 16, 0x33));
        c.record_structural(tela(&escorrido), tela(&escorrido));
    });
    assert_eq!(
        baratas, 0,
        "a caixa de meio plano tinha de cair no caminho caro"
    );
}

/// **E desfazer tudo depois de uma absorção barata devolve a tela pristina** — a metade de PIXELS,
/// que é o que o artista vê (as de cima são a de entrada, que é o que a memória paga).
#[test]
fn undoing_through_a_cheap_absorption_gives_back_the_pristine_canvas() {
    CHEAP_FIRED.with(|f| f.set(0));
    let mut c = UndoController::new(DEFAULT_MAX_BYTES);
    let traco = bloco(3, 6, 3, 6, 0x22);
    c.record_structural(tela(&[]), tela(&traco));
    let mut escorrido = traco.clone();
    escorrido.extend(bloco(5, 9, 7, 9, 0x33));
    let mut seguinte = escorrido.clone();
    seguinte.push(((12, 12), 0x44));
    c.record_structural(tela(&escorrido), tela(&seguinte));
    assert_eq!(
        CHEAP_FIRED.with(std::cell::Cell::get),
        1,
        "o caminho barato não correu"
    );
    assert!(c.undo_here().is_some(), "desfaz o 2º passo");
    let back = c.undo_here().expect("desfaz o 1º passo");
    assert_eq!(
        back.canvas_rgba.as_ref(),
        tela(&[]).canvas_rgba.as_ref(),
        "desfazer tudo tem de devolver a tela pristina"
    );
    let fwd = c.redo_here().expect("refaz o 1º passo");
    assert_eq!(
        fwd.canvas_rgba.as_ref(),
        tela(&escorrido).canvas_rgba.as_ref(),
        "refazer o 1º passo devolve o traço COM o escorrido, que ele absorveu"
    );
}

/// A janela que o escorrido DECLAROU desde o último commit — o que o `wetpaint::composite` escreve.
fn declara(c: &UndoController, x: u32, y: u32, w: u32, h: u32) {
    let mut win = WriteWindow::default();
    win.open_write();
    win.mark(Some(crate::compositor::Region { x, y, w, h }));
    c.write_state.set(win);
}

/// **O detector que lê só a janela DECLARADA guarda a MESMA entrada** — com a janela justa ao
/// escorrido e com uma folgada à volta dele (ela é um superconjunto, e é a exacta que tem de sair).
#[test]
fn the_declared_detector_absorbs_the_same_way_as_the_full_scan() {
    let traco = bloco(3, 6, 3, 6, 0x22);
    for (nome, janela) in [("justa", (10, 9, 3, 2)), ("folgada", (8, 7, 6, 6))] {
        let (baratas, dentro) = pelos_dois_caminhos_contando(|c| {
            c.record_structural(tela(&[]), tela(&traco));
            let mut escorrido = traco.clone();
            escorrido.extend(bloco(10, 13, 9, 11, 0x33));
            let (x, y, w, h) = janela;
            declara(c, x, y, w, h);
            c.record_structural(tela(&escorrido), tela(&escorrido));
        });
        assert_eq!(dentro, 1, "{nome}: o detector não leu a janela declarada");
        assert_eq!(baratas, 1, "{nome}: o caminho barato não correu");
    }
}

/// **Uma janela declarada sobre bytes IGUAIS não faz a absorção disparar** — o caminho de sempre não
/// dispara ali (o `Arc` é outro, o conteúdo é o mesmo), e o `split` COM janela dispararia: ele guarda a
/// declarada tal como veio. É a metade que separa `split_exact_within` do `split` com dica.
#[test]
fn a_declared_window_over_unchanged_bytes_does_not_fire_the_absorption() {
    let (_, dentro) = pelos_dois_caminhos_contando(|c| {
        let traco = bloco(3, 6, 3, 6, 0x22);
        c.record_structural(tela(&[]), tela(&traco));
        declara(c, 2, 2, 5, 5);
        ABSORB_FIRED.with(|f| f.set(0));
        // Um `Arc` NOVO com o mesmo conteúdo do cursor.
        c.record_structural(tela(&traco), tela(&traco));
        assert_eq!(
            ABSORB_FIRED.with(std::cell::Cell::get),
            0,
            "nada mudou: a absorção não pode disparar"
        );
    });
    assert_eq!(dentro, 1, "o detector não leu a janela declarada");
}

/// **Uma janela declarada de meio plano ou mais cai no detector de sempre** — aí o escorrido pode ser
/// meio plano, o detector de sempre guarda `Whole`, e a resposta de dentro da janela seria `Patch`.
#[test]
fn a_declared_window_of_half_the_plane_falls_back_to_the_full_scan() {
    let (_, dentro) = pelos_dois_caminhos_contando(|c| {
        c.record_structural(tela(&[]), tela(&bloco(3, 6, 3, 6, 0x22)));
        declara(c, 0, 0, LADO, LADO);
        // Um escorrido de DEZ linhas inteiras: mais de meio plano.
        let escorrido = bloco(0, LADO as usize, 0, 10, 0x33);
        c.record_structural(tela(&escorrido), tela(&escorrido));
    });
    assert_eq!(
        dentro, 0,
        "a janela de meio plano tinha de cair no detector de sempre"
    );
}

/// **E a rede: uma janela declarada que NÃO contém o escorrido reprova em DEBUG** — é ela que torna
/// seguro ler só a janela, e sem ela um sítio que declarasse mal deixaria escorrido sem dono.
#[cfg(debug_assertions)]
#[test]
#[should_panic(expected = "detector da absorcao")]
fn a_declared_window_that_misses_the_drip_is_caught_in_debug() {
    let mut c = UndoController::new(DEFAULT_MAX_BYTES);
    let traco = bloco(3, 6, 3, 6, 0x22);
    c.record_structural(tela(&[]), tela(&traco));
    let mut escorrido = traco.clone();
    escorrido.push(((12, 12), 0x33));
    declara(&c, 0, 0, 2, 2);
    c.record_structural(tela(&escorrido), tela(&escorrido));
}

/// **Uma janela mais NOVA que o cursor não é lida** — ela acumula desde o último commit, e o que foi
/// escrito entre o cursor e esse commit não está nela. É a metade de PROVENIÊNCIA do `hint_for`: sem
/// ela o detector leria só a janela e o escorrido de antes do commit ficaria sem dono.
#[test]
fn a_declared_window_newer_than_the_cursor_is_not_read() {
    let (_, dentro) = pelos_dois_caminhos_contando(|c| {
        let traco = bloco(3, 6, 3, 6, 0x22);
        c.record_structural(tela(&[]), tela(&traco)); // o cursor nasce com `writes = 0`
        let mut escorrido = traco.clone();
        escorrido.push(((12, 12), 0x33)); // escrito ANTES de a janela ser zerada…
        escorrido.push(((1, 1), 0x44)); // …e este DEPOIS, o único que ela viu
        let mut win = WriteWindow::default();
        win.reset(10);
        win.open_write();
        win.mark(Some(crate::compositor::Region {
            x: 1,
            y: 1,
            w: 1,
            h: 1,
        }));
        c.write_state.set(win);
        c.record_structural(tela(&escorrido), tela(&escorrido));
    });
    assert_eq!(
        dentro, 0,
        "a janela mais nova que o cursor não pode responder por ele"
    );
}
