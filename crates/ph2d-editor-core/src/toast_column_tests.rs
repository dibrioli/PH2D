//! Os gates da coluna do topo e do tempo de leitura (doc 132 §5, P5).

use super::*;
use crate::toast_reading::{MIN_TTL_S, READING_WPM, rows_for_height, shown};
use ph2d_text::{FontWeight, TextSystem};

/// O caso do dono: a recusa do Motion com a cura (`motion_bridge_subgraph.rs`).
fn aviso_do_dono() -> String {
    let cura = ph2d_i18n::tr_with(
        "app.motion.motion_bridge_subgraph.insert_a_to_read_one_number_from_it",
        &[("ty", &"value.attribute")],
    );
    ph2d_i18n::tr_with(
        "app.motion.motion_bridge_subgraph.can_t_drive_that_output_is_a_per_element_stream",
        &[("cura", &cura)],
    )
}

fn sobrepoe(a: Rect, b: Rect) -> bool {
    a.x < b.x + b.w && b.x < a.x + a.w && a.y < b.y + b.h && b.y < a.y + a.h
}

/// ⭐⭐ **Um balão de k linhas não tapa o balão seguinte nem a barra de trabalho de baixo** — e o
/// texto dele cabe no cartão. As barras descem pelas LINHAS ocupadas, não pelo número de balões.
#[test]
fn um_balao_longo_nao_tapa_o_seguinte_nem_a_barra_de_baixo() {
    let mut ts = TextSystem::without_system_fonts();
    let viewport = Rect::new(0.0, 0.0, 1280.0, 800.0);
    let mut q = ToastQueue::new();
    let longo = "There are 12 skeletons in the scene and no bone selected. Pick a bone of the \
                 skeleton you want too (Ctrl+click in the Hierarchy) - otherwise the shape would \
                 be bound to all 12 at once.";
    q.push(Toast::warning(aviso_do_dono()));
    q.push(Toast::info("Saved"));
    q.push(Toast::error(longo));
    let col = top_column(&q, 2, &mut ts, viewport);
    assert_eq!(col.toasts.len(), 3);
    assert_eq!(col.bars.len(), 2);
    let pad = ph2d_tokens::Spacing::Md.px();
    for (i, (r, s)) in col.toasts.iter().enumerate() {
        assert!(
            s.height <= r.h - 2.0 * pad + 0.01,
            "o texto do balão {i} ({} linhas, {:.1} px) não cabe no cartão de {:.1} px",
            s.lines,
            s.height,
            r.h
        );
    }
    assert!(
        col.toasts[0].1.lines >= 2,
        "o aviso do dono quebrou: {:?}",
        col.toasts[0].1
    );
    assert!(
        col.toasts[2].1.lines >= 4,
        "o longo quebrou: {:?}",
        col.toasts[2].1
    );
    assert!(
        col.rows > q.len(),
        "os balões longos abarcam mais de uma linha da coluna"
    );
    let todos: Vec<Rect> = col
        .toasts
        .iter()
        .map(|(r, _)| *r)
        .chain(col.bars.iter().copied())
        .collect();
    for (i, a) in todos.iter().enumerate() {
        for b in &todos[i + 1..] {
            assert!(
                !sobrepoe(*a, *b),
                "dois cartões da coluna sobrepõem-se: {a:?} × {b:?}"
            );
            assert!(a.y < b.y, "a coluna não desce pela ordem: {a:?} × {b:?}");
        }
    }
    // A régua continua uniforme: a 1.ª barra está na linha a seguir à última ocupada.
    assert_eq!(col.bars[0], crate::progress::column_row(viewport, col.rows));
}

/// ⭐⭐ **Um orçamento só**: uma frase que cabe numa linha à largura da PORTA, mas não à largura
/// com o segundo desconto (`label_budget`), sai inteira numa linha e num cartão de uma linha.
#[test]
fn a_frase_quebra_a_largura_da_porta_sem_segundo_desconto() {
    let mut ts = TextSystem::without_system_fonts();
    let porta = text_budget_px();
    let com_desconto = crate::paint::label_budget(porta);
    let f = crate::toast_reading::font_px();
    let mut frase = String::from("Saved");
    while ts.prefix_width_weighted(&frase, f, FontWeight::MEDIUM) <= com_desconto {
        frase.push_str(" ok");
    }
    let w = ts.prefix_width_weighted(&frase, f, FontWeight::MEDIUM);
    assert!(
        w > com_desconto && w <= porta,
        "a frase de prova não está entre as duas larguras: {w}"
    );
    let s = shown(&mut ts, &frase);
    assert_eq!(s.text, frase);
    assert_eq!(
        s.lines, 1,
        "{frase:?} ({w:.1} px) quebrou a {:.1} px",
        s.width
    );
    assert!(
        (s.width - porta).abs() < 0.01,
        "a largura de quebra não é a da porta"
    );
    assert_eq!(rows_for_height(s.height), 1);
}

/// ⭐ **O tempo de vida de uma frase longa ≥ o tempo de a LER; uma curta fica nos 3 s**, e o
/// `ttl_seconds` do chamador manda.
#[test]
fn o_tempo_de_vida_e_o_de_ler_a_frase() {
    let curta = Toast::info("Saved");
    assert!((curta.ttl_s - MIN_TTL_S).abs() < f32::EPSILON);
    let msg = aviso_do_dono();
    #[expect(clippy::cast_precision_loss, reason = "palavras de uma frase")]
    let leitura = msg.split_whitespace().count() as f32 * 60.0 / READING_WPM;
    let longa = Toast::warning(msg.clone());
    assert!(
        leitura > MIN_TTL_S,
        "a frase de prova é curta demais: {leitura} s"
    );
    assert!(
        longa.ttl_s >= leitura,
        "vive {} s e lê-se em {leitura} s: {msg:?}",
        longa.ttl_s
    );
    assert!((Toast::warning(msg).ttl_seconds(1.5).ttl_s - 1.5).abs() < f32::EPSILON);

    // Pela fila: o curto atrás do longo sai aos 3 s; o longo fica até ser lido.
    let mut q = ToastQueue::new();
    q.push(longa);
    q.push(curta);
    let dt = 1.0 / 60.0;
    let mut t = 0.0;
    while t < MIN_TTL_S + 0.1 {
        q.tick(dt);
        t += dt;
    }
    assert_eq!(
        q.len(),
        1,
        "o curto não saiu aos 3 s (ou o longo saiu cedo)"
    );
    assert_eq!(
        q.iter().next().map(|x| x.severity),
        Some(ToastSeverity::Warning)
    );
    while t < leitura + 0.1 {
        q.tick(dt);
        t += dt;
    }
    assert!(q.is_empty(), "o longo sobreviveu ao tempo de leitura");
}
