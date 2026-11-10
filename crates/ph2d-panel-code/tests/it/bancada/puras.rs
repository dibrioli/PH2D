//! ⭐ **A Bancada, fora da grelha**: as partições das colunas, a coluna sob o cursor do texto, a frase
//! para cada alvo, e as marcas da margem que o `CodeView` pinta (◆, ⚡ e os pontos de quem mudou).

use super::{LIVES, SPEED, celula, coluna, meio, quem};
use crate::{Deco, area_para, frame, marcas, spans_de, ts};
use ph2d_code_buffer::{Buffer, Range as Cursor, Selection};
use ph2d_code_viewmodel::{CellKind, Entry};
use ph2d_editor_core::zones::Rect;
use ph2d_panel_code::bancada::{
    Aim, BancadaView, Who, caret_column, left_split, phrase_height, phrase_split, say,
};
use ph2d_panel_code::{Breakpoint, CodeView, DebugMarks};
use ph2d_text::TextSystem;
use ph2d_tokens::{ROW_H_PX, Spacing, Theme};
use ph2d_vector::VectorScene;
use std::ops::Range;

#[test]
fn a_coluna_da_lista_so_parte_quando_aberta_e_com_altura() {
    let col = Rect::new(10.0, 20.0, 200.0, 800.0);
    assert_eq!(
        left_split(col, 3, false),
        (col, None),
        "recolhida: só a lista"
    );
    let curta = Rect::new(10.0, 20.0, 200.0, ROW_H_PX * 4.0);
    assert_eq!(
        left_split(curta, 3, true),
        (curta, None),
        "curta: só a lista"
    );
    let (lista, fora) = left_split(col, 3, true);
    let fora = fora.expect("alta e aberta: o «In this script» por baixo");
    assert_eq!((lista.x, lista.y, lista.w), (col.x, col.y, col.w));
    assert_eq!((fora.x, fora.w), (col.x, col.w));
    assert!(
        lista.h > 0.0 && lista.h <= col.h * 0.5,
        "a lista até metade: {}",
        lista.h
    );
    assert!(
        fora.y >= lista.y + lista.h,
        "sem sobreposição: {fora:?} × {lista:?}"
    );
    assert!(
        ((fora.y + fora.h) - (col.y + col.h)).abs() < 1e-3,
        "o cartão vai até ao fundo"
    );
    assert!(fora.h >= ROW_H_PX * 4.0);
}

#[test]
fn a_frase_fica_no_fundo_sem_tapar_o_resto() {
    let doc = Rect::new(0.0, 50.0, 800.0, 600.0);
    let (resto, frase) = phrase_split(doc);
    assert!((frase.h - phrase_height()).abs() < 1e-3);
    assert!(((frase.y + frase.h) - (doc.y + doc.h)).abs() < 1e-3);
    assert!(resto.y + resto.h <= frase.y + 1e-3, "{resto:?} × {frase:?}");
    assert_eq!((resto.x, resto.y, resto.w), (doc.x, doc.y, doc.w));
}

fn cursor_em(texto: &str, agulha: &str, mais: usize) -> Buffer {
    let mut b = Buffer::new(texto);
    let at = texto.find(agulha).unwrap_or_else(|| panic!("{agulha}")) + mais;
    b.set_selection(Selection::single(Cursor::cursor(at)));
    b
}

#[test]
fn a_coluna_do_cursor_e_a_da_palavra_sob_ele() {
    let who = quem();
    let t = "local s = self.speed\nph2d.property(\"speed\", 220)\nprint(lives)";
    let casos: [(&str, usize, Option<usize>); 9] = [
        ("self.speed", 7, Some(SPEED)),
        ("self.speed", 10, Some(SPEED)),
        ("self.speed", 5, Some(SPEED)),
        ("\"speed\"", 3, Some(SPEED)),
        ("lives", 2, Some(LIVES)),
        ("local", 2, None),
        ("220", 1, None),
        ("property", 3, None),
        ("self.speed", 2, None),
    ];
    for (agulha, mais, esperado) in casos {
        let b = cursor_em(t, agulha, mais);
        assert_eq!(
            caret_column(&b, &who),
            esperado,
            "cursor em {agulha:?}+{mais}"
        );
    }
}

#[test]
fn a_frase_nunca_fica_vazia_nem_com_marcadores_por_trocar() {
    let who = quem();
    let mut aims = vec![Aim::Nothing];
    for h in 0..who.holders.len() {
        aims.extend([Aim::Holder(h), Aim::Select(h)]);
        aims.extend((0..who.columns.len()).map(|c| Aim::Cell(h, c)));
    }
    for (c, col) in who.columns.iter().enumerate() {
        aims.extend([Aim::Column(c), Aim::Mark(col.line), Aim::Entry(col.line)]);
    }
    for e in who.hears.iter().chain(&who.functions) {
        aims.push(Aim::Entry(e.1));
    }
    aims.extend(who.hears.iter().map(|e| Aim::Mark(e.1)));
    // Controlo: todas as variantes estão na lista (uma variante nova parte esta compilação).
    let mut vistas = [false; 7];
    for a in &aims {
        let i = match a {
            Aim::Nothing => 0,
            Aim::Cell(..) => 1,
            Aim::Holder(_) => 2,
            Aim::Column(_) => 3,
            Aim::Mark(_) => 4,
            Aim::Select(_) => 5,
            Aim::Entry(_) => 6,
        };
        vistas[i] = true;
    }
    assert!(vistas.iter().all(|v| *v), "variantes: {vistas:?}");
    let ninguem = Who::default();
    let mut n = 0;
    for aim in &aims {
        for editing in [None, Some((1, SPEED))] {
            for peek in [true, false] {
                for (w, a) in [(&who, *aim), (&ninguem, Aim::Nothing)] {
                    let s = say(
                        w,
                        a,
                        editing.filter(|_| std::ptr::eq(w, &who)),
                        "hero.luau",
                        peek,
                    );
                    assert!(!s.trim().is_empty(), "{a:?} {editing:?} {peek}: vazia");
                    assert!(
                        !s.contains('{') && !s.contains('}'),
                        "{a:?}: marcador por trocar em {s:?}"
                    );
                    n += 1;
                }
            }
        }
    }
    assert!(n >= aims.len() * 8, "{n} frases medidas");
    // E cada variante diz coisas DIFERENTES (a frase não é uma só para tudo).
    let ditas: std::collections::BTreeSet<String> = aims
        .iter()
        .map(|a| say(&who, *a, None, "hero.luau", true))
        .collect();
    assert!(
        ditas.len() > aims.len() / 2,
        "{} frases distintas para {} alvos",
        ditas.len(),
        aims.len()
    );
}

const SCRIPT: &str = "ph2d.property(\"speed\", 220)\nph2d.property(\"lives\", 3)\nlocal function on_signal(name)\n  if name == \"coin\" then\n    print(name)\n  end\nend";

/// O `who` das marcas: `speed` na linha 0 (mudada por «Hero 2»), `lives` na 1 (ninguém a mudou),
/// e o sinal «coin» ouvido na linha 3.
fn quem_das_marcas() -> Who {
    Who {
        holders: quem().holders,
        columns: vec![
            coluna("speed", 0, Some(CellKind::Number), "220", &[]),
            coluna("lives", 1, Some(CellKind::Number), "3", &[]),
        ],
        cells: vec![
            celula("220", false),
            celula("3", false),
            celula("180", true),
            celula("3", false),
        ],
        hears: vec![Entry("coin".into(), 3)],
        ..Who::default()
    }
}

fn pinta_codigo(
    v: &mut CodeView,
    buf: &Buffer,
    ts: &mut TextSystem,
    area: Rect,
    who: Option<&Who>,
    debug: DebugMarks<'_>,
) {
    let d = Deco::default();
    let mut f = |r: Range<usize>| spans_de(buf, r);
    let mut m = marcas(&d);
    let mut fr = frame(buf, &d, &mut f, &mut m);
    fr.who = who;
    fr.debug = debug;
    let mut scene = VectorScene::new();
    v.paint(fr, area, &mut scene, ts, &Theme::default());
}

/// Os alvos da linha `line`, da esquerda para a direita (a marca da margem e, se há, os pontos).
fn alvos(v: &CodeView, line: usize) -> Vec<Rect> {
    let mut r: Vec<Rect> = v
        .bancada_marks()
        .iter()
        .filter(|(l, _)| *l == line)
        .map(|(_, r)| *r)
        .collect();
    r.sort_by(|a, b| a.x.total_cmp(&b.x));
    assert!(
        !r.is_empty(),
        "a linha {line} tem marca: {:?}",
        v.bancada_marks()
    );
    r
}

fn marca(v: &CodeView, line: usize) -> Rect {
    alvos(v, line)[0]
}

#[test]
fn a_marca_da_propriedade_vai_da_margem_aos_pontos_de_quem_mudou() {
    let buf = Buffer::new(SCRIPT);
    let (mut v, mut ts) = (CodeView::new(), ts());
    let area = area_para(&mut v, &buf, &mut ts, 20);
    let who = quem_das_marcas();
    pinta_codigo(
        &mut v,
        &buf,
        &mut ts,
        area,
        Some(&who),
        DebugMarks::default(),
    );
    let m = *v.metrics();
    let fim = v
        .caret_rect(&buf, buf.text().line_end(0))
        .expect("o fim da linha 0 está à vista")
        .x;
    let speed = alvos(&v, 0);
    assert_eq!(speed.len(), 2, "a marca da margem e os pontos: {speed:?}");
    assert!(
        (speed[0].x - area.x).abs() < 0.5 && (speed[0].w - m.mark_w).abs() < 0.5,
        "a marca é só a margem: {speed:?}"
    );
    assert!(
        speed[1].x > fim && speed[1].x + speed[1].w > fim + Spacing::Lg.px(),
        "os pontos ficam depois do texto ({fim}): {speed:?}"
    );
    // ⛔ O texto da linha NÃO é alvo da marca: o clique nele é do cursor (o defeito que uma caixa
    // só, da margem aos pontos, tinha — o renomear e o clique na linha da propriedade morriam).
    let meio = (area.x + m.mark_w + fim) * 0.5;
    assert!(
        !speed.iter().any(|r| r.contains(meio, r.y + r.h * 0.5)),
        "o meio do texto da linha 0 não é da marca: {speed:?}"
    );
    // Controlo: «lives» ninguém mudou — só a marca da margem, sem pontos.
    let lives = marca(&v, 1);
    assert!((lives.x - area.x).abs() < 0.5);
    assert!(
        (lives.w - m.mark_w).abs() < 0.5,
        "sem pontos a marca é só a margem: {lives:?}"
    );
    let coin = marca(&v, 3);
    assert!(
        (coin.w - m.mark_w).abs() < 0.5,
        "o ⚡ do sinal ouvido: {coin:?}"
    );
    assert_eq!(
        v.bancada_marks().len(),
        4,
        "◆+pontos de speed, ◆ de lives, ⚡ de coin"
    );
    // O ponto de parada ganha à marca na pintura, mas o alvo do balão fica.
    let bp = [Breakpoint {
        line: 0,
        armed: true,
    }];
    pinta_codigo(
        &mut v,
        &buf,
        &mut ts,
        area,
        Some(&who),
        DebugMarks {
            breakpoints: &bp,
            ..DebugMarks::default()
        },
    );
    let com_bp = alvos(&v, 0);
    assert_eq!(
        com_bp, speed,
        "com o ponto de parada os alvos do balão ficam os mesmos"
    );
    // Sem a Bancada, nenhuma marca.
    pinta_codigo(&mut v, &buf, &mut ts, area, None, DebugMarks::default());
    assert!(v.bancada_marks().is_empty());
}

#[test]
fn a_marca_pintada_abre_o_balao_da_vista() {
    let buf = Buffer::new(SCRIPT);
    let (mut v, mut ts) = (CodeView::new(), ts());
    let area = area_para(&mut v, &buf, &mut ts, 20);
    let who = quem_das_marcas();
    pinta_codigo(
        &mut v,
        &buf,
        &mut ts,
        area,
        Some(&who),
        DebugMarks::default(),
    );
    let mut b = BancadaView::default();
    b.set_marks(v.bancada_marks());
    let speed = marca(&v, 0);
    // Sobre os pontos (à direita do texto), não só sobre o ◆.
    let (x, y) = (speed.x + speed.w - 1.0, speed.y + speed.h * 0.5);
    assert!(b.hover(x, y, 0));
    assert_eq!(b.aim(), Aim::Mark(0));
    assert_eq!(b.popover_line(), Some(0));
    let mut scene = VectorScene::new();
    b.paint_popover(
        &who,
        area,
        &mut CodeView::new(),
        &mut scene,
        &mut ts,
        &Theme::default(),
    );
    assert!(
        b.target(Aim::Select(1)).is_some(),
        "o «Select Hero 2» do balão"
    );
    let (cx, cy) = meio(marca(&v, 3));
    b.hover(cx, cy, 1);
    assert_eq!(b.popover_line(), Some(3), "o balão do sinal ouvido");
}
