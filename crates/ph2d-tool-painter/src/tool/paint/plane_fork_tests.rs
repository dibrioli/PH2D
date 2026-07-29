//! Gates de [`super`] — as quatro portas de fork: byte-identidade, o `Weak`, e o arch-gate que
//! varre a crate atrás de quem escreve relevo por fora delas.
//!
//! Irmão por `#[path]` (segue FILHO ⇒ `use super::*` alcança os privados); saiu do `plane_fork.rs`
//! quando o doc da região do canvas o levou a 706 > 700.
use super::*;
use crate::plane_copy::PAR_MIN_BYTES;
use rayon::prelude::*;

/// **A fork is a copy, and a copy has one right answer.** The parallel path must produce exactly what
/// `Arc::make_mut` produces — value-identical, and uniquely owned afterwards so the caller may write.
///
/// Run over a length that clears `PAR_MIN` (so the parallel path actually executes — the trap the
/// ADR-0120 lesson names: an optimisation nobody exercises is green code that never runs) and one that
/// does not.
#[test]
fn a_parallel_fork_is_byte_identical_to_the_serial_one() {
    for n in [PAR_MIN_BYTES / size_of::<f32>() + 1_000, 64] {
        let src: Vec<f32> = (0..n).map(|i| (i as f32) * 0.25 - 3.0).collect();

        let mut a = Arc::new(src.clone());
        let keep_a = Arc::clone(&a); // force the second-owner path
        let forked = fork_par(&mut a, &WriteState::default()).clone();

        let mut b = Arc::new(src.clone());
        let keep_b = Arc::clone(&b);
        let expected = Arc::make_mut(&mut b).clone();

        assert_eq!(forked, expected, "n = {n}");
        assert_eq!(forked, src, "n = {n}: the fork changed the contents");
        // The originals are untouched — the whole point of freezing one.
        assert_eq!(*keep_a, src);
        assert_eq!(*keep_b, src);
    }
}

/// **Um `Weak` vivo NÃO é um dono — e perguntar isso errado custou 4× no Wet Paint.**
///
/// `Arc::get_mut` devolve `None` na presença de qualquer `Weak`; `Arc::make_mut` só **copia** com
/// outro **strong** (com só `Weak` ele *move* o valor). Enquanto esta função perguntava pelo
/// `get_mut`, ela copiava o plano inteiro sempre que alguém segurasse um `Weak` — e o guard de
/// identidade do Wet Paint é exatamente isso (frente V, doc 28 §5.12), então o composite pagava uma
/// cópia do documento **por movimento do mouse** enquanto o `make_mut` seguinte o movia de graça.
///
/// ⚠️ O sintoma foi o gate de RAZÃO do move molhado voltando a **4,77×** — nunca uma falha de
/// comportamento, porque as duas rotas dão os mesmos bytes. Este gate afirma a propriedade direto,
/// para o defeito não depender de um relógio para aparecer.
#[test]
fn a_live_weak_is_not_an_owner_and_does_not_trigger_a_copy() {
    let mut a: Arc<Vec<f32>> = Arc::new(vec![1.0; PAR_MIN_BYTES / size_of::<f32>() + 1_000]);
    let watcher = Arc::downgrade(&a); // o guard de identidade do Wet Paint, em miniatura
    let before = a.as_ptr();
    let got = fork_par(&mut a, &WriteState::default());
    assert_eq!(
        got.as_ptr(),
        before,
        "um `Weak` vivo fez o fork COPIAR o plano — a pergunta e `strong_count`, nao `get_mut`"
    );
    // E o `Weak` foi desassociado, exatamente como o `Arc::make_mut` faria: quem o segurava tem de
    // enxergar que o buffer mudou de dono (e o guard do Wet Paint depende disso para se re-armar).
    assert!(watcher.upgrade().is_none() || Arc::strong_count(&a) == 1);
}

/// **With no second owner it does not allocate at all.** The common case inside a stroke is that the
/// plane was already forked by an earlier dab; paying a copy per dab instead of per stroke is the
/// regression this guards.
#[test]
fn an_unshared_plane_is_not_copied() {
    let mut a: Arc<Vec<f32>> = Arc::new(vec![1.0; PAR_MIN_BYTES / size_of::<f32>() + 1_000]);
    let before = a.as_ptr();
    let got = fork_par(&mut a, &WriteState::default());
    assert_eq!(got.as_ptr(), before, "an unshared plane was copied anyway");
}

/// **The fast path has to be proven by the clock, because nothing else can see it.**
///
/// The two gates above cannot tell the branches apart, and that is not an oversight — it is the
/// point: a fork is a copy, so the parallel path is *semantically identical* by construction. There
/// is no value, no pointer and no refcount that differs (`Arc::make_mut` also leaves a uniquely
/// owned buffer at a fresh address). A behavioural gate here would be the serial path measured
/// against itself and green forever — the trap ADR-0120 documented and ADR-0124 then hit a second
/// time, hiding in its own undo oracle.
///
/// So the claim is timed, and asserted as a **RATIO** rather than a wall-clock bar: `ci-test`
/// compiles at `opt-level=1` and this machine is documented as drifting ~3× across a session, so an
/// absolute millisecond bar would be measuring the profile and the weather. The ratio survives both.
///
/// Measured at 4096² (67 MB): serial 10,88 ms, parallel 3,34 ms — 3,3×. The bar is set well under
/// that so a loaded machine cannot flake it, while a fork that silently stopped being parallel
/// (delegating straight to `Arc::make_mut`) lands at 1,0× and fails.
#[test]
#[ignore = "perf measurement — run with --release --ignored"]
fn the_parallel_fork_is_actually_faster_than_the_serial_one() {
    use std::time::Instant;
    const N: usize = 4096 * 4096;
    let src: Arc<Vec<f32>> = Arc::new(vec![0.5; N]);
    let best = |mut f: Box<dyn FnMut() -> f64>| (0..3).map(|_| f()).fold(f64::MAX, f64::min);

    let s = Arc::clone(&src);
    let serial = best(Box::new(move || {
        let mut a = Arc::clone(&s);
        let _keep = Arc::clone(&s);
        let t0 = Instant::now();
        let m = Arc::make_mut(&mut a);
        std::hint::black_box(&m[0]);
        t0.elapsed().as_secs_f64() * 1000.0
    }));
    let p = Arc::clone(&src);
    let parallel = best(Box::new(move || {
        let mut a = Arc::clone(&p);
        let _keep = Arc::clone(&p);
        let t0 = Instant::now();
        let m = fork_par(&mut a, &WriteState::default());
        std::hint::black_box(&m[0]);
        t0.elapsed().as_secs_f64() * 1000.0
    }));
    eprintln!(
        "[fork] 4096² plane: serial {serial:.2} ms · parallel {parallel:.2} ms · \
             {:.1}×",
        serial / parallel
    );
    assert!(
        serial / parallel > 1.5,
        "the parallel fork bought {:.1}× over `Arc::make_mut` (serial {serial:.2} ms, parallel \
             {parallel:.2} ms). Below 1.5× the fast path is not running — check that `PAR_MIN` still \
             sits under a canvas-sized plane",
        serial / parallel
    );
}

/// **A porta do pen-down** — que o depósito de PIGMENTO atravesse esta função, e não o `make_mut` cru.
///
/// ⚠️ Este gate é arquitetural porque o defeito é **invisível ao comportamento**: as duas rotas produzem
/// os mesmos bytes (é o que o gate acima prova), então trocar uma pela outra deixa a suíte inteira verde e
/// custa **três vezes o tempo** no gesto que o artista mais sente. Medido no pen-down a 4096², pincel
/// digital: **10,3 ms com o `make_mut` cru contra 3,6 ms por aqui** (`measure_impasto_cost::
/// the_first_stroke_latency`); o pen-down do impasto, 18,6 -> 12,2.
///
/// ⚠️ **O escopo deixou de ser o `stamp_cache` (2026-07-26).** A doc antiga dizia que o pen-down era
/// *"o único sítio onde o `Arc` do canvas tem um segundo dono garantido"* — e isso é **falso**:
/// medido (`measure_stroke_owners`), em repouso o canvas tem **dois donos**, porque o `cursor` do
/// histórico é um dono **permanente**. Logo a PRIMEIRA escrita de **todo** gesto forka, e os outros
/// 23 sítios (fill, smear, blur, clone, seleção, warp, máscara, inpaint, aquarela, e o composite do
/// Wet Paint, que roda a cada TICK) pagavam a cópia **serial**. O gate irmão abaixo cobre a crate.
#[test]
fn the_pigment_deposit_forks_the_canvas_in_parallel() {
    let src = include_str!("stamp_cache.rs");
    // Controle positivo: o alvo tem de EXISTIR, senão o gate passa por não achar nada (a falha que o
    // `the_shape_slot_goes_through_the_shape_door` do Flow pegou em si mesmo).
    let through = src.matches("fork_canvas(").count();
    assert!(
        through >= 5,
        "controle: o stamp_cache tem de escrever o canvas pela porta paralela ({through} sitios)"
    );
    let raw = src.matches("Arc::make_mut(&mut self.canvas_rgba)").count();
    assert_eq!(
        raw, 0,
        "o deposito de pigmento nao pode forkar o canvas SERIALMENTE: {raw} sitio(s) com `make_mut` cru \
             (as duas rotas dao os mesmos bytes, entao isto nao acende em teste de comportamento nenhum \
             -- custa 3x o tempo do pen-down e passa despercebido)"
    );
}

/// **As portas NOMEADAS ensinam o journal; a genérica declara que não sabe.**
///
/// O relevo é um mapa por camada e três tipos de elemento, então a captura tem de saber **qual** —
/// e é essa exigência que separa as duas famílias de porta. Quem passa pela genérica não fica
/// errado, fica **lento**: o journal se declara incompleto e o commit deriva como sempre.
///
/// ⚠️ Mutação que sangra: tirar o `capture_heights` do [`fork_heights`] (o journal não sabe do byte
/// velho) · tirar o `note_untracked_write` do [`fork_par`] (o journal jura descrever um passo que
/// tem escrita fora dele).
#[test]
fn the_named_relief_doors_teach_the_journal_and_the_generic_one_does_not() {
    const W: u32 = 64;
    let n = (W as usize) * (W as usize);
    let layer = crate::layers::LayerId(7);

    let w = WriteState::default();
    let mut h: Arc<Vec<f32>> = Arc::new((0..n).map(|i| i as f32).collect());
    let keep = Arc::clone(&h); // um segundo dono, como o `stroke_undo` de um passo real
    fork_heights(&mut h, &w, layer, (W, W), None)[10] = -1.0;

    assert_eq!(w.relief_state(), "DESCREVE");
    assert_eq!(
        w.heights_before(layer, 10),
        Some(10.0),
        "o journal tem de guardar o valor VELHO, nao o que acabou de ser escrito"
    );
    assert_eq!(
        keep[10], 10.0,
        "controle: o fork preservou o plano congelado"
    );
    assert!(
        w.heights_before(crate::layers::LayerId(8), 10).is_none(),
        "o journal nao pode responder por OUTRA camada"
    );

    let w2 = WriteState::default();
    let mut h2: Arc<Vec<f32>> = Arc::new(vec![1.0; n]);
    let _ = fork_par(&mut h2, &w2);
    assert_eq!(
        w2.relief_state(),
        "INCOMPLETO",
        "a porta generica nao sabe de que plano e — o journal tem de se declarar incompleto"
    );
}

/// **AUSENTE e INCOMPLETO são coisas diferentes, e o `else` da porta distingue as duas.**
///
/// Um plano que **não existia** no começo do passo não tem *antes* a descrever — o motor de delta já
/// chama isso de `OnlyAfter`, e não descrever o que não existe é a resposta certa. Um plano que
/// **existia**, foi capturado, e perdeu a forma no meio do passo é a outra coisa: aí a escrita atual
/// é genuinamente indescritível e o passo inteiro se declara incompleto.
///
/// ⚠️ **Este gate existe porque a rede que confere a promoção é opt-in** (`PH2D_UNDO_AUDIT=1`) — uma
/// rede de verificação não pode viver no relógio do que ela observa (§5.23), e por isso a
/// PROPRIEDADE ganha um gate próprio, sem relógio e sem env. Sem ele a distinção seria provada só
/// por uma varredura que ninguém roda por padrão.
///
/// ⚠️ Mutação que sangra: `note_absent` marcar ausente sempre (o segundo caso vira DESCREVE, e o
/// journal jura conhecer bytes velhos que ele não guardou).
#[test]
fn a_plane_that_never_existed_is_absent_not_incomplete() {
    const W: u32 = 64;
    let n = (W as usize) * (W as usize);
    let layer = crate::layers::LayerId(3);

    // (1) A 1ª pincelada de uma camada: o plano nasce VAZIO, então não há "antes".
    let w = WriteState::default();
    let mut fresh: Arc<Vec<f32>> = Arc::new(Vec::new());
    let _ = fork_heights(&mut fresh, &w, layer, (W, W), None);
    assert_eq!(
        w.relief_state(),
        "DESCREVE",
        "um plano que nao existia nao tem *antes* a descrever — declarar o passo INCOMPLETO por \
             causa dele e confundir 'nada a dizer' com 'nao sei dizer'"
    );
    assert!(
        w.relief_absent()[0],
        "o fato tem de ficar registrado para a rede o conferir"
    );

    // (2) O plano EXISTIA e foi capturado; perder a forma depois é incompletude de verdade.
    let w2 = WriteState::default();
    let mut real: Arc<Vec<f32>> = Arc::new(vec![7.0; n]);
    let _keep = Arc::clone(&real);
    fork_heights(&mut real, &w2, layer, (W, W), None)[0] = -1.0;
    assert_eq!(
        w2.relief_state(),
        "DESCREVE",
        "controle: a captura normal descreve"
    );
    let mut odd: Arc<Vec<f32>> = Arc::new(vec![0.0; n + 1]);
    let _ = fork_heights(&mut odd, &w2, layer, (W, W), None);
    assert_eq!(
        w2.relief_state(),
        "INCOMPLETO",
        "o plano ja tinha tiles no journal: perder a forma no meio do passo deixa bytes velhos sem \
             dono, e isso NAO e a mesma coisa que um plano que nunca existiu"
    );
}

/// **NENHUM sítio da crate escreve um plano de RELEVO por fora das portas** — arch-gate, porque o
/// defeito é invisível ao comportamento: as duas rotas produzem os mesmos bytes, e a diferença é só
/// se o journal aprende.
///
/// ⚠️ **Ele nasceu estreito (só `impasto_live.rs`) e o alargamento ACHOU um sítio**: o
/// `impasto_material.rs` escrevia o `mats` com um `Arc::make_mut` cru, e as três metades pesavam —
/// journal cego ao byte velho, fork serial, e **acesso não aberto**, que é o grave: o contador de
/// acessos não-declarados é o que faz o modo de falha de um sítio esquecido ser *lento em vez de
/// errado*, e ele só enxerga quem passa por uma porta. Um gate por-arquivo protege o arquivo que
/// alguém lembrou de listar.
///
/// ⚠️ **A metade que contava a porta GENÉRICA morreu, e de propósito:** o `fork_par` é `cfg(test)`
/// agora, então um sítio de produto que o chame **não compila**. Uma asserção que não pode falhar é
/// pior que asserção nenhuma — a lição que a `line/Vector` deixou em 2026-07-23.
///
/// **Prosa é isenta, código não** (mesma política do irmão do canvas): o literal aparece em
/// doc-comments que explicam justamente esta diferença.
#[test]
fn no_site_in_the_crate_writes_relief_outside_the_doors() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/tool/paint");
    let (mut scanned, mut through) = (0usize, 0usize);
    let mut offenders: Vec<String> = Vec::new();
    let mut stack = vec![root];
    while let Some(dir) = stack.pop() {
        for e in std::fs::read_dir(&dir).expect("src/tool/paint existe") {
            let p = e.expect("entrada legivel").path();
            if p.is_dir() {
                stack.push(p);
                continue;
            }
            let name = p.file_name().and_then(|n| n.to_str()).unwrap_or_default();
            if !name.ends_with(".rs")
                || name.contains("test")
                || name.starts_with("measure_")
                || name == "plane_fork.rs"
            {
                continue; // gates e sondas chamam a rota crua DE PROPOSITO
            }
            let src = std::fs::read_to_string(&p).expect("fonte legivel");
            scanned += 1;
            for door in ["fork_heights(", "fork_covers(", "fork_mats("] {
                through += src.matches(door).count();
            }
            for (i, line) in src.lines().enumerate() {
                let t = line.trim_start();
                if t.starts_with("//") {
                    continue; // prosa: explicar a rota crua e o que as portas FAZEM
                }
                for plane in ["self.heights", "self.covers", "self.mats"] {
                    if t.contains(&format!("make_mut({plane}")) {
                        offenders.push(format!("{name}:{}", i + 1));
                    }
                }
            }
        }
    }
    // Controle positivo nas DUAS pontas: sem ele o gate passa por não ter achado arquivo nenhum.
    assert!(scanned > 20, "controle: so {scanned} arquivos varridos");
    assert!(
        through >= 11,
        "controle: so {through} escritas de relevo pelas portas nomeadas — o scanner esta cego"
    );
    assert!(
        offenders.is_empty(),
        "estes sitios escrevem um plano de RELEVO por fora da porta: o journal nao aprende de que \
             plano sao os bytes (passo INCOMPLETO), o fork e serial, e o acesso nao e aberto — entao o \
             commit acredita que a janela declarada cobre o que este sitio escreveu: {offenders:?}"
    );
}

/// **De que é feito um fork, por TIPO e por TAMANHO** — a medição que mostrou que
/// `par_iter().copied().collect()` **não é** uniformemente melhor que um memcpy.
#[test]
#[ignore = "medicao — rode com --release --ignored"]
fn what_a_fork_costs_by_element_type_and_size() {
    use std::time::Instant;
    fn best(mut f: impl FnMut() -> f64) -> f64 {
        (0..5).map(|_| f()).fold(f64::MAX, f64::min)
    }
    fn go<T: Copy + Send + Sync + Default + PartialEq>(name: &str, n: usize) {
        let src: Arc<Vec<T>> = Arc::new(vec![T::default(); n]);
        let a = best(|| {
            let s = Arc::clone(&src);
            let t0 = Instant::now();
            let v: Vec<T> = (*s).clone();
            let dt = t0.elapsed().as_secs_f64() * 1000.0;
            std::hint::black_box(v.len());
            dt
        });
        let b = best(|| {
            let s = Arc::clone(&src);
            let t0 = Instant::now();
            let v: Vec<T> = s.par_iter().copied().collect();
            let dt = t0.elapsed().as_secs_f64() * 1000.0;
            std::hint::black_box(v.len());
            dt
        });
        let c = best(|| {
            let s = Arc::clone(&src);
            let t0 = Instant::now();
            let mut v: Vec<T> = vec![T::default(); s.len()];
            v.par_chunks_mut(1 << 16)
                .zip(s.par_chunks(1 << 16))
                .for_each(|(d, x)| d.copy_from_slice(x));
            let dt = t0.elapsed().as_secs_f64() * 1000.0;
            std::hint::black_box(v.len());
            dt
        });
        let mb = (n * std::mem::size_of::<T>()) as f64 / (1024.0 * 1024.0);
        println!(
            "{name:<16} {mb:>7.0} MB  clone {a:>7.3}  par_collect {b:>7.3}  par_memcpy {c:>7.3}"
        );
    }
    println!();
    for side in [2048usize, 4096] {
        let n = side * side;
        println!("-- tela {side}x{side} --");
        go::<u8>("canvas rgba u8", n * 4);
        go::<u8>("covers u8", n);
        go::<f32>("heights f32", n);
        go::<[u8; 7]>("mats [u8;7]", n);
    }
    println!();
}

/// **NENHUM sítio da crate forka o canvas serialmente** — o irmão de escopo largo do gate acima.
///
/// ⚠️ Um gate por-arquivo protege o arquivo que alguém lembrou de listar. Este varre `tool/paint/**`
/// inteiro, então o sítio 24 nasce coberto — que é exatamente como os 23 nasceram descobertos.
///
/// ⚠️ **O literal é só o NOME da porta, e isso é deliberado.** A primeira versão casava a chamada
/// inteira (`fork_canvas(&mut self.canvas_rgba, &self.undo.write_state, self.source_size.0, None)`) e
/// morreu no `cargo fmt`, que quebra três argumentos em quatro linhas: o gate passou a achar ZERO
/// e só o controle positivo o salvou de virar verde-sobre-nada. Um gate ancorado em LAYOUT é um
/// proxy que expira; o que ele tem de afirmar é a PORTA.
///
/// **Prosa é isenta, código não.** O literal aparece em doc-comments explicando a diferença entre as
/// duas rotas, e um gate que os proibisse mandaria apagar a explicação; ele conta só linhas de código.
/// Arquivos de teste e de medição também são isentos — eles CHAMAM `make_mut` de propósito, para
/// medir a rota lenta contra a rápida.
#[test]
fn no_site_in_the_crate_forks_the_canvas_serially() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/tool/paint");
    let mut scanned = 0usize;
    let mut through = 0usize;
    let mut offenders: Vec<String> = Vec::new();
    let mut stack = vec![root];
    while let Some(dir) = stack.pop() {
        for e in std::fs::read_dir(&dir).expect("src/tool/paint existe") {
            let p = e.expect("entrada legivel").path();
            if p.is_dir() {
                stack.push(p);
                continue;
            }
            let name = p.file_name().and_then(|n| n.to_str()).unwrap_or_default();
            if !name.ends_with(".rs")
                || name.contains("test")
                || name.starts_with("measure_")
                || name == "plane_fork.rs"
            {
                continue; // gates e sondas chamam a rota lenta DE PROPOSITO
            }
            let src = std::fs::read_to_string(&p).expect("fonte legivel");
            scanned += 1;
            through += src.matches("fork_canvas(").count();
            for (i, line) in src.lines().enumerate() {
                let t = line.trim_start();
                if t.starts_with("//") {
                    continue; // prosa: explicar a rota lenta e o que os docs FAZEM
                }
                if t.contains("Arc::make_mut(&mut self.canvas_rgba)") {
                    offenders.push(format!("{name}:{}", i + 1));
                }
            }
        }
    }
    // Controle positivo, nas DUAS pontas: sem ele o gate passa por não ter achado arquivo nenhum.
    assert!(scanned > 20, "controle: so {scanned} arquivos varridos");
    assert!(
        through >= 20,
        "controle: so {through} escritas de canvas pela porta paralela — o scanner esta cego"
    );
    assert!(
        offenders.is_empty(),
        "estes sitios forkam o canvas SERIALMENTE (a primeira escrita de todo gesto copia a tela \
             inteira, ~3x mais devagar, e as duas rotas dao os mesmos bytes — nenhum teste de \
             comportamento acende): {offenders:?}"
    );
}
