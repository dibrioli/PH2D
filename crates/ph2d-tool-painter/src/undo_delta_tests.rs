//! Gates do motor de delta do histórico (`super`) — split por LOC cap, e `tests` segue sendo FILHO
//! dele (`#[path]`), então `use super::*` alcança os itens privados que eles medem.

use super::*;

/// **De que é feito o scan do commit** — quanto cada plano canvas-shaped custa a 4096², e quanto a
/// paralelização comprou. Diagnóstico: a wave que paralelizou o `diff_window` precisa saber se o que
/// sobra ainda é o scan ou já é a extração da janela.
#[test]
#[ignore = "medicao — rode com --release --ignored"]
fn what_the_commit_scan_is_made_of() {
    use std::time::Instant;
    fn go<T: Clone + PartialEq + Sync + Copy>(name: &str, n: usize, stride: usize, v: T, w: T) {
        let a: Vec<T> = vec![v; n];
        let mut b = a.clone();
        // Uma janela de traço: ~200 linhas de ~200 elementos, no meio do plano.
        let rows = n / stride;
        for r in (rows / 2)..(rows / 2 + 200) {
            for c in (stride / 2)..(stride / 2 + 200) {
                b[r * stride + c] = w;
            }
        }
        let mut lo = f64::INFINITY;
        for _ in 0..5 {
            let t0 = Instant::now();
            let out = diff_window(&a, &b, stride);
            let dt = t0.elapsed().as_secs_f64() * 1000.0;
            std::hint::black_box(out);
            lo = lo.min(dt);
        }
        let mb = (n * size_of::<T>()) as f64 / (1024.0 * 1024.0);
        println!("{name:<10} {mb:>8.0} MB {lo:>10.3} ms");
    }
    let side = 4096usize;
    let n = side * side;
    println!("\n[diff_window] tela 4096x4096");
    go("canvas u8", n * 4, side * 4, 0u8, 9u8);
    go("covers u8", n, side, 0u8, 9u8);
    go("heights f32", n, side, 0.0f32, 1.0f32);
    go("mats [u8;7]", n, side, [0u8; 7], [9u8; 7]);
    println!();
}

/// **A varredura SERIAL, congelada como REFERÊNCIA** — é o código que rodava antes de a wave
/// paralelizar o `diff_window`, verbatim.
///
/// ⚠️ Ela mora sob `cfg(test)` de propósito. Um `fn` privado sem chamador de produção não é código
/// morto silencioso: é uma **segunda resposta** esperando alguém chamá-la — e sem o `cfg` o doc deste
/// gate (*"é o código que shipava"*) viraria falso no dia em que alguém a editasse.
fn serial_window<T: PartialEq>(
    a: &[T],
    b: &[T],
    stride: usize,
) -> Option<(usize, usize, usize, usize)> {
    let rows = a.len() / stride;
    let mut first = None;
    let mut last = 0usize;
    for r in 0..rows {
        let s = r * stride;
        if a[s..s + stride] != b[s..s + stride] {
            if first.is_none() {
                first = Some(r);
            }
            last = r;
        }
    }
    let row = first?;
    let mut col = stride;
    let mut end = 0usize;
    for r in row..=last {
        let s = r * stride;
        for c in 0..stride {
            if a[s + c] != b[s + c] {
                if c < col {
                    col = c;
                }
                if c + 1 > end {
                    end = c + 1;
                }
            }
        }
    }
    Some((row, last - row + 1, col, end.saturating_sub(col)))
}

/// **A varredura paralela acha EXATAMENTE a mesma janela que a serial.**
///
/// Uma janela é um bbox, e um bbox tem uma resposta certa — então o oráculo é a implementação que
/// shipava, e não um número escrito à mão que envelheceria junto com ela.
///
/// ⚠️ As fixtures **atravessam o [`PAR_MIN`]** de propósito: abaixo dele o produto roda a rota serial,
/// e um gate que só a exercitasse estaria comparando o caminho antigo com ele mesmo — verde para
/// sempre, sobre uma paralelização que nunca rodou (a armadilha que o ADR-0120 documentou e o
/// ADR-0124 repetiu dentro do próprio oráculo de undo).
#[test]
fn the_parallel_scan_finds_the_same_window_as_the_serial_one() {
    let stride = 1024usize;
    // Duas alturas: uma sob o limiar (rota serial) e outra bem acima dele (rota paralela).
    for rows in [8usize, 2048] {
        let n = rows * stride;
        assert_eq!(
            n >= PAR_MIN,
            rows == 2048,
            "a fixture tem de cruzar o PAR_MIN"
        );
        for (name, marks) in [
            ("um texel no meio", vec![(rows / 2, stride / 2)]),
            ("dois cantos opostos", vec![(0, 0), (rows - 1, stride - 1)]),
            ("primeira linha so", vec![(0, 3), (0, 900)]),
            ("ultima linha so", vec![(rows - 1, 7)]),
            (
                "faixa larga",
                (0..rows).step_by(3).map(|r| (r, r % stride)).collect(),
            ),
            // ⚠️ Uma banda ALTA (a rota paralela das colunas) cujas colunas ficam LONGE do zero.
            // Sem ela a identidade da redução de coluna é invisível: toda outra fixture larga aqui
            // acerta a coluna 0 por acidente, e `min(0, c)` = 0 concorda com a resposta certa.
            (
                "faixa alta, colunas longe do zero",
                (0..rows).step_by(2).map(|r| (r, 400 + (r % 64))).collect(),
            ),
        ] {
            let a = vec![0u8; n];
            let mut b = a.clone();
            for (r, c) in &marks {
                b[r * stride + c] = 9;
            }
            let got = diff_window(&a, &b, stride).map(|w| (w.row, w.rows, w.col, w.cols));
            let want = serial_window(&a, &b, stride);
            assert_eq!(got, want, "{name} (rows = {rows})");
        }
        // E o caso em que NÃO diferem: os dois têm de dizer `None` (é o que vira `Unchanged`, e
        // confundi-lo com "não sei medir" perderia a edição em silêncio).
        let a = vec![0u8; n];
        let b = a.clone();
        assert_eq!(diff_window(&a, &b, stride), None, "iguais (rows = {rows})");
        assert_eq!(serial_window(&a, &b, stride), None);
    }
}

/// **O mesmo, nos TIPOS que o produto de fato guarda** — `f32` e `[u8; 7]` não comparam por memcmp,
/// então percorrem outro código no `PartialEq` de slice; é neles que o custo mora e é neles que uma
/// redução errada apareceria diferente.
#[test]
fn the_parallel_scan_agrees_on_every_plane_type_the_history_stores() {
    let stride = 1024usize;
    let rows = 1200usize; // acima do PAR_MIN nos três
    let n = rows * stride;
    // ⚠️ Nenhum acerto na coluna 0, e a banda cobre quase todas as linhas: é essa combinação que
    // torna VISÍVEL uma identidade errada na redução paralela das colunas.
    let hit = [(3usize, 200usize), (rows / 2, stride - 1), (rows - 2, 700)];

    let a = vec![0.5f32; n];
    let mut b = a.clone();
    for (r, c) in hit {
        b[r * stride + c] = -1.0;
    }
    assert_eq!(
        diff_window(&a, &b, stride).map(|w| (w.row, w.rows, w.col, w.cols)),
        serial_window(&a, &b, stride),
        "heights f32"
    );

    let a = vec![[1u8; 7]; n];
    let mut b = a.clone();
    for (r, c) in hit {
        b[r * stride + c] = [2u8; 7];
    }
    assert_eq!(
        diff_window(&a, &b, stride).map(|w| (w.row, w.rows, w.col, w.cols)),
        serial_window(&a, &b, stride),
        "mats [u8; 7]"
    );

    let a = vec![[0.0f32, 0.0]; n];
    let mut b = a.clone();
    for (r, c) in hit {
        b[r * stride + c] = [3.0, -4.0];
    }
    assert_eq!(
        diff_window(&a, &b, stride).map(|w| (w.row, w.rows, w.col, w.cols)),
        serial_window(&a, &b, stride),
        "deform disp [f32; 2]"
    );
}

/// **A rota paralela é de fato mais rápida que a serial** — e isto só pode ser afirmado pelo
/// RELÓGIO, porque as duas produzem a mesma janela por construção (é o que o gate acima prova).
///
/// Um gate de comportamento aqui seria a rota serial medida contra ela mesma, verde para sempre. A
/// barra é uma **RAZÃO** e não um wall-clock: `ci-test` compila em `opt-level=1` e esta máquina deriva
/// ~3× ao longo de uma sessão, então um limite em milissegundos mediria o perfil e o tempo.
///
/// Medido a 4096² (o plano de `mats`, 112 MB, o mais caro dos quatro): serial **8,31 ms**, paralelo
/// **4,25 ms** — **2,0×**. A barra fica bem abaixo para que máquina carregada não a faça flakar,
/// enquanto um `diff_window` que voltasse a ser serial pousa em 1,0× e falha.
#[test]
#[ignore = "perf — rode com --release --ignored"]
fn the_parallel_scan_is_actually_faster_than_the_serial_one() {
    use std::time::Instant;
    let stride = 4096usize;
    let rows = 4096usize;
    let n = rows * stride;
    let a = vec![[1u8; 7]; n];
    let mut b = a.clone();
    for r in (rows / 2)..(rows / 2 + 200) {
        for c in (stride / 2)..(stride / 2 + 200) {
            b[r * stride + c] = [2u8; 7];
        }
    }
    let best = |mut f: Box<dyn FnMut() -> f64>| (0..5).map(|_| f()).fold(f64::MAX, f64::min);
    let (sa, sb) = (a.clone(), b.clone());
    let serial = best(Box::new(move || {
        let t0 = Instant::now();
        let out = serial_window(&sa, &sb, stride);
        let dt = t0.elapsed().as_secs_f64() * 1000.0;
        std::hint::black_box(out);
        dt
    }));
    let parallel = best(Box::new(move || {
        let t0 = Instant::now();
        let out = diff_window(&a, &b, stride);
        let dt = t0.elapsed().as_secs_f64() * 1000.0;
        std::hint::black_box(out);
        dt
    }));
    eprintln!(
        "[diff] mats 4096²: serial {serial:.2} ms · paralelo {parallel:.2} ms · {:.1}x",
        serial / parallel
    );
    assert!(
        serial / parallel > 1.5,
        "o scan paralelo comprou {:.1}x sobre o serial (serial {serial:.2} ms, paralelo \
         {parallel:.2} ms). Abaixo de 1,5x a rota rapida nao esta rodando — confira se o PAR_MIN \
         ainda cabe sob um plano canvas-shaped",
        serial / parallel
    );
}

/// A janela é o bbox EXATO da diferença — nem uma linha a mais.
#[test]
fn the_window_is_the_exact_bbox_of_the_difference() {
    let stride = 8;
    let mut a = vec![0u8; stride * 6];
    let b = {
        let mut b = a.clone();
        b[2 * stride + 3] = 9;
        b[4 * stride + 5] = 7;
        b
    };
    let w = diff_window(&a, &b, stride).expect("difere");
    assert_eq!((w.row, w.rows, w.col, w.cols), (2, 3, 3, 3));
    // …e ela reconstrói o outro lado exatamente.
    let patch = w.extract(&b);
    w.blit(&patch, &mut a);
    assert_eq!(a, b);
}

/// Planos idênticos em CONTEÚDO (ponteiros diferentes) não custam nada.
#[test]
fn identical_content_costs_nothing_even_with_different_pointers() {
    let mut a = Arc::new(vec![3u8; 64]);
    let mut b = Arc::new(vec![3u8; 64]);
    assert!(!Arc::ptr_eq(&a, &b));
    let p = StoredPlane::split(&mut a, &mut b, 8);
    assert!(matches!(p, StoredPlane::Unchanged));
    assert_eq!(p.heap_bytes(), 0);
}

/// Uma janela grande demais NÃO vira patch: dois lados de meia-tela custam mais que os dois buffers.
#[test]
fn a_window_that_does_not_pay_for_itself_falls_back_to_whole() {
    let stride = 8;
    let mut a = Arc::new(vec![0u8; stride * 8]);
    let mut b = Arc::new({
        let mut v = vec![0u8; stride * 8];
        for (i, x) in v.iter_mut().enumerate() {
            if i % stride < 6 {
                *x = 1;
            }
        }
        v
    });
    let p = StoredPlane::split(&mut a, &mut b, stride);
    assert!(matches!(p, StoredPlane::Whole { .. }), "esperava Whole");
}

/// Um cursor de outro tamanho RECUSA em vez de escrever pixels em lugares que ninguém autorou.
#[test]
fn a_cursor_of_the_wrong_size_is_refused_not_patched() {
    let stride = 8;
    let mut a = Arc::new(vec![0u8; stride * 8]);
    let mut b = Arc::new({
        let mut v = vec![0u8; stride * 8];
        v[9] = 5;
        v
    });
    let p = StoredPlane::split(&mut a, &mut b, stride);
    assert!(matches!(p, StoredPlane::Patch { .. }));
    let wrong = Arc::new(vec![0u8; stride * 4]);
    assert!(p.side(&wrong, true).is_none());
}
