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
//! ⛔⛔ **A PREMISSA DESTE FICHEIRO CAIU EM 2026-09-08, e quem a derrubou foi a própria medição.**
//!
//! Ele dizia: *«o tamanho SERIALIZADO é o piso honesto da residência … aqui ele serve porque o
//! `FlipDoc` **não partilha nada**: cada clone é memória nova»*. Era verdade — e a F8.3 pôs um
//! `Arc` **por DESENHO** precisamente porque a coluna do desperdício mostrou `99,0 %` a 96 quadros.
//! ⇒ *quem move o número que tornava algo verdadeiro tem de reconferir a nota* (§0.0).
//!
//! Hoje o clone do documento **não copia arte nenhuma** (só ponteiros), e o `to_bytes()` continua a
//! escrever a animação inteira. ⚠️ **As colunas de RESIDÊNCIA abaixo mediriam a massa de um mundo
//! que já não existe** — por isso elas passaram a dizer as DUAS coisas: o que o formato pesa (o
//! piso de um SAVE) e o que a pilha de facto ocupa (um desenho por passo). *O tamanho serializado é
//! cego à partilha por construção, e essa cegueira já respondeu à pergunta errada uma vez nesta
//! fase.*
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
        "  │ quadros │  clone   │ % de um quadro │ o SAVE pesa   │ a PILHA ocupa (x{UNDO_CAP}) │"
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
        // ⚠️ **A pilha ocupa UM DESENHO por passo, não o documento** — desde a F8.3 o clone
        // partilha tudo o que o passo não tocou, e a residência deixou de crescer com a animação.
        let base = FlipDoc::new().to_bytes().map(|v| v.len()).unwrap_or(0);
        let desenho = animation_of(1)
            .to_bytes()
            .map(|v| v.len())
            .unwrap_or(0)
            .saturating_sub(base);
        let mb = (desenho * UNDO_CAP) as f64 / (1024.0 * 1024.0);
        println!(
            "  │ {frames:>7} │ {ms:>6.3} ms │ {:>13.1} % │ {bytes:>13} │ {mb:>17.1} MB │",
            ms / 16.7 * 100.0
        );
    }
    println!("  └─────────┴──────────┴────────────────┴───────────────┴──────────────────────┘");
    println!("  (a cena vetorial, para comparar: 0,287 ms e 303 MB a 5 000 formas — F8.1, 02/09)");
    println!("  (o restauro inteiro, que a F8 apontava: 1,81 ms uma vez por Ctrl+Z — F8.0)\n");
}

/// ⭐⭐ **A pergunta que o relógio não faz: quantos PASSOS até a pilha comer a RAM?**
///
/// ⛔⛔ **A pergunta mudou de sujeito em 2026-09-08, e a mudança é o resultado da F8.3.** Ela era
/// *«quantos QUADROS DESENHADOS»* — porque cada passo copiava a animação inteira, e a residência
/// crescia com o tamanho dela (`1 GB` a ~108 quadros). Com a partilha por desenho, o que enche a
/// pilha é o número de **PASSOS**, e cada um custa **um desenho**: a residência deixou de depender
/// de quão longa a animação é.
///
/// ⚠️ **Ela é derivada da própria medição**, e não de um número escrito à mão. *Um teto escrito de
/// memória é um palpite; este diz de que recurso é.*
#[test]
#[ignore = "mede memoria; irmao do de cima"]
fn the_undo_stack_holds_this_many_frames_of_animation() {
    let um = animation_of(1);
    let doze = animation_of(12);
    let por_quadro = (doze.to_bytes().map(|v| v.len()).unwrap_or(0) as f64
        - um.to_bytes().map(|v| v.len()).unwrap_or(0) as f64)
        / 11.0;
    let por_passo_mb = por_quadro / (1024.0 * 1024.0);
    println!(
        "\n  um PASSO custa um desenho ({:.0} B); {UNDO_CAP} passos custam {:.2} MB — e o numero NAO",
        por_quadro,
        por_passo_mb * UNDO_CAP as f64
    );
    println!("  cresce com o tamanho da animacao (F8.3: era o documento INTEIRO por passo)");
    for gb in [1.0_f64, 4.0] {
        let passos = gb * 1024.0 / por_passo_mb;
        println!("  a pilha so' chegaria a {gb:.0} GB com ~{passos:.0} passos de desenho");
    }
    println!();
    assert!(
        por_quadro > 0.0,
        "a fixtura nao cresce com os quadros — ela nao esta' a medir uma animacao"
    );
}

/// ⭐⭐⭐ **O QUE UM PASSO DE DESENHO DE FACTO MUDA** — a pergunta que a F8.2 deixou aberta.
///
/// ```text
/// cargo test -p ph2d-flip --release --test measure_doc_clone -- --ignored --nocapture
/// ```
///
/// # Porque ela é a pergunta certa
///
/// A F8.1 pôs um `Arc` no **documento inteiro**: dois passos que não tocam o Flip partilham um
/// documento e a captura não move um byte. ⚠️ **E o próprio handoff nomeou o resíduo:** *numa
/// sessão de desenho **cada passo muda o documento**, e aí o custo volta ao de hoje* — que é
/// precisamente o caso comum do módulo, porque desenhar **é** o gesto dele.
///
/// A cura de fundo proposta era `Arc` **por DESENHO** (o grão que o mundo usa desde a F2). Antes de
/// a construir, o §0.0 manda medir **o que ela pouparia**: se o desenho tocado for a maior parte do
/// documento, não há nada a partilhar e a resposta é uma **recusa medida**.
///
/// ⚠️ **A régua é o tamanho SERIALIZADO**, e aqui ele é honesto pela mesma razão da medição irmã: o
/// `FlipDoc` de hoje **não partilha nada**, logo cada byte do formato é um byte de memória nova.
#[test]
#[ignore = "mede memoria; irmao dos de cima"]
fn only_the_touched_drawing_needs_to_be_new() {
    println!("\n  o que um PASSO de desenho muda, contra o que ele CLONA hoje");
    println!("  fixtura: {STROKES} tracos x {POINTS} pontos por quadro (uma figura simples)");
    println!("  ┌─────────┬───────────────┬───────────────┬──────────────┬──────────────────────┐");
    println!(
        "  │ quadros │ o doc INTEIRO │ UM desenho    │ desperdicio  │ x{UNDO_CAP} na pilha      │"
    );
    println!("  ├─────────┼───────────────┼───────────────┼──────────────┼──────────────────────┤");
    for frames in [12u32, 24, 96, 240] {
        let doc = animation_of(frames);
        let inteiro = doc.to_bytes().map(|v| v.len()).unwrap_or(0);
        // Um desenho só: a mesma fixtura com UM quadro é o piso do que um passo tem de ser novo.
        let um = animation_of(1).to_bytes().map(|v| v.len()).unwrap_or(0);
        let base = FlipDoc::new().to_bytes().map(|v| v.len()).unwrap_or(0);
        let desenho = um.saturating_sub(base);
        let pct = if inteiro > 0 {
            100.0 - (desenho as f64 / inteiro as f64) * 100.0
        } else {
            0.0
        };
        let mb_hoje = (inteiro * UNDO_CAP) as f64 / (1024.0 * 1024.0);
        let mb_partilhado = (desenho * UNDO_CAP) as f64 / (1024.0 * 1024.0);
        println!(
            "  │ {frames:>7} │ {inteiro:>13} │ {desenho:>13} │ {pct:>11.1} % │ {mb_hoje:>7.1} → {mb_partilhado:>6.1} MB │"
        );
    }
    println!("  └─────────┴───────────────┴───────────────┴──────────────┴──────────────────────┘");
    println!("  (a coluna do desperdicio e' o que a partilha por DESENHO evitaria copiar)\n");
}
