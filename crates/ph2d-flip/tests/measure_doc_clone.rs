//! ⭐ **O que custa CLONAR o documento Flip** — a MEDIÇÃO IRMÃ que a F8 nomeou e deixou por fazer.
//!
//! ```text
//! cargo test -p ph2d-flip --release --test measure_doc_clone -- --ignored --nocapture
//! ```
//!
//! ⚠️ **`#[ignore]`**: mede um relógio (família de flakes do `CLAUDE.md` §5.0) e imprime o `load`.
//!
//! # Porque ela existe, e porque é a IRMÃ
//!
//! O `ProjectState` guarda três geometrias: o mundo (`WorldSnapshot`), a cena vetorial
//! (`Arc<VecScene>`) e este `FlipDoc`. As duas primeiras já pagaram a conta — a F2 pôs um `Arc`
//! por linha no mundo, e a F8.1 pôs um `Arc` na cena depois de a medir. O `FlipDoc` **tem
//! exactamente a mesma forma** (um `Vec` de objectos, clonado inteiro por captura, guardado
//! `UNDO_CAP` vezes) e o plano diz, à letra: *«falta-lhe a medição irmã»*.
//!
//! ⛔⛔ **A ordem é a lei da fase.** O estudo de 2026-09-02 mediu ANTES de optimizar e concluiu que
//! **metade da F8 não se justifica** — o restauro custa `1,81 ms` uma vez por Ctrl+Z, que é
//! imperceptível. *Uma fase inteira apontava para a metade que não dói.* ⇒ aqui a pergunta é feita
//! antes de qualquer `Arc`: **se o número for pequeno, a resposta é uma recusa medida**, e ela vale
//! tanto como uma cura.
//!
//! # As DUAS perguntas, e porque a segunda é a que importa
//!
//! - **O RELÓGIO** (`clone` por quadro com input): foi o que ilibou a cena vetorial — `0,287 ms` a
//!   5 000 formas, irrelevante.
//! - **A RESIDÊNCIA** (`UNDO_CAP = 256` estados, cada um com um documento inteiro): foi o que a
//!   condenou — `303 MB` a 5 000 formas.
//!
//! ⚠️ **O tamanho SERIALIZADO é o piso honesto da residência, e não a residência.** Ele é cego à
//! partilha (`Arc`) por construção — e foi exactamente essa cegueira que fez a 1.ª medição da F8
//! responder à pergunta errada com um número grande e plausível. Aqui ele serve porque o `FlipDoc`
//! **não partilha nada**: cada clone é memória nova, e a serialização mede a mesma massa.
//!
//! # A fixtura é uma ANIMAÇÃO, não um número escolhido
//!
//! Um objecto Flip real é *quadros × traços × pontos*. A cena varre a dimensão que o artista de
//! facto move — o **número de quadros** — com traços e pontos parados no que um desenho comum tem
//! (⚠️ `20` traços de `60` pontos: uma figura simples, e por isso um **piso**).

use ph2d_core::Vec2;
use ph2d_flip::{FlipDoc, FlipStroke, Frame, Hold, KeyKind, Point};
use std::time::Instant;

const ITERS: usize = 25;
/// Traços por desenho — uma figura simples. ⚠️ Piso, não caso escolhido.
const STROKES: usize = 20;
/// Pontos por traço — um gesto curto a taxa de polling comum.
const POINTS: usize = 60;
/// A pilha de undo guarda tantos estados, cada um com um documento inteiro.
const UNDO_CAP: usize = 256;

fn load_average() -> f64 {
    std::fs::read_to_string("/proc/loadavg")
        .ok()
        .and_then(|s| s.split_whitespace().next()?.parse().ok())
        .unwrap_or(f64::NAN)
}

fn median(mut v: Vec<f64>) -> f64 {
    v.sort_by(|a, b| a.partial_cmp(b).expect("sem NaN"));
    v[v.len() / 2]
}

/// Um traço de `POINTS` pontos, com posições distintas (um zigue-zague) — pontos repetidos
/// comprimiriam noutro formato e dariam um número que o artista nunca vê.
fn stroke(seed: usize) -> FlipStroke {
    let mut s = FlipStroke::new();
    for i in 0..POINTS {
        let t = i as f32;
        s.push_point(Point::at(Vec2::new(
            seed as f32 + t * 0.37,
            (t * 0.21).sin() * 3.0,
        )));
    }
    s
}

/// Uma animação de `frames` quadros numa camada, cada quadro com o seu desenho.
fn animation_of(frames: u32) -> FlipDoc {
    let mut doc = FlipDoc::new();
    let oid = doc.push_object("Personagem");
    let obj = doc.object_mut(oid).expect("o objecto acabou de nascer");
    let layer = obj.add_layer("Traço");
    for f in 0..frames {
        let Some(d) = obj.insert_frame(layer, f as Frame, Hold::default(), KeyKind::default())
        else {
            continue;
        };
        let draw = obj.drawing_mut(d).expect("o desenho acabou de nascer");
        for k in 0..STROKES {
            draw.strokes.push(stroke(k));
        }
    }
    doc
}

#[test]
#[ignore = "mede um relogio; rode com a maquina calma e --release"]
fn cloning_the_flip_doc_costs_the_animation_and_this_is_the_number() {
    println!(
        "\n  clone do FlipDoc (o que a captura faz em TODO quadro com input) — mediana de {ITERS}, load = {:.2}",
        load_average()
    );
    println!("  fixtura: {STROKES} tracos x {POINTS} pontos por quadro (uma figura simples)");
    println!("  ┌─────────┬──────────┬────────────────┬───────────────┬──────────────────────┐");
    println!(
        "  │ quadros │  clone   │ % de um quadro │ bytes/estado  │ x{UNDO_CAP} na pilha      │"
    );
    println!("  ├─────────┼──────────┼────────────────┼───────────────┼──────────────────────┤");
    for frames in [1u32, 12, 24, 96] {
        let doc = animation_of(frames);
        let mut samples = Vec::with_capacity(ITERS);
        for _ in 0..ITERS {
            let t0 = Instant::now();
            let c = doc.clone();
            samples.push(t0.elapsed().as_secs_f64() * 1000.0);
            std::hint::black_box(&c);
        }
        let ms = median(samples);
        let bytes = doc.to_bytes().map(|v| v.len()).unwrap_or(0);
        let mb = (bytes * UNDO_CAP) as f64 / (1024.0 * 1024.0);
        println!(
            "  │ {frames:>7} │ {ms:>6.3} ms │ {:>13.1} % │ {bytes:>13} │ {mb:>17.1} MB │",
            ms / 16.7 * 100.0
        );
    }
    println!("  └─────────┴──────────┴────────────────┴───────────────┴──────────────────────┘");
    println!("  (a cena vetorial, para comparar: 0,287 ms e 303 MB a 5 000 formas — F8.1, 02/09)");
    println!("  (o restauro inteiro, que a F8 apontava: 1,81 ms uma vez por Ctrl+Z — F8.0)\n");
}

/// ⭐⭐ **A pergunta que o relógio não faz: quantos quadros até a pilha comer a RAM?**
///
/// ⚠️ **Ela é derivada da própria medição**, e não de um número escrito à mão: a residência por
/// quadro sai da fixtura, e o limite é o que a máquina do dono tem. *Um teto escrito de memória é
/// um palpite; este diz de que recurso é.*
#[test]
#[ignore = "mede memoria; irmao do de cima"]
fn the_undo_stack_holds_this_many_frames_of_animation() {
    let um = animation_of(1);
    let doze = animation_of(12);
    let por_quadro = (doze.to_bytes().map(|v| v.len()).unwrap_or(0) as f64
        - um.to_bytes().map(|v| v.len()).unwrap_or(0) as f64)
        / 11.0;
    let por_estado_mb = por_quadro / (1024.0 * 1024.0);
    println!(
        "\n  um quadro de animacao custa {:.0} B; {UNDO_CAP} estados custam {:.2} MB por quadro desenhado",
        por_quadro,
        por_estado_mb * UNDO_CAP as f64
    );
    for gb in [1.0_f64, 4.0] {
        let quadros = gb * 1024.0 / (por_estado_mb * UNDO_CAP as f64);
        println!("  a pilha cheia chega a {gb:.0} GB com ~{quadros:.0} quadros desenhados");
    }
    println!();
    assert!(
        por_quadro > 0.0,
        "a fixtura nao cresce com os quadros — ela nao esta' a medir uma animacao"
    );
}
