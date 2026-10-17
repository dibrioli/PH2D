//! As sondas da W7 (`docs/Macros/04_plano_W7.md` item 2): a MESMA idade da água.
//!
//! `cargo test -p ph2d-tool-painter --profile smoke --lib macro_track::tutorial::sonda::w7 -- --ignored --nocapture`

use super::*;

/// [`EnsinoDoPainter::medido`] com um ponto das curvas a cada quadro.
fn medido_a_cada_quadro(track: &Track) -> EnsinoDoPainter {
    let mut e = EnsinoDoPainter::de(track).expect("ensino");
    let mut ms = Vec::new();
    e.mede_com(track, &AtomicBool::new(false), 1, mede::fios(), |m| {
        ms.push(m);
    })
    .expect("mede");
    for m in ms {
        e.com_medida(m);
    }
    e
}

/// As curvas de `e` (um ponto por quadro) densas até à idade 64 (a reprodução, que `mede` lê a cada
/// evento: o corte da gravada a ~13, o da base a ~50) e depois com um ponto a cada `k` e o último —
/// perto das que `mede` com o passo `k` dá (lá a contagem começa no corte).
fn decimado(e: &EnsinoDoPainter, k: u32) -> EnsinoDoPainter {
    let mut d = e.clone();
    for c in &mut d.criterios {
        if let Some(r) = &mut c.referencia {
            for cv in [&mut r.gravada, &mut r.base] {
                let n = cv.0.len();
                cv.0 = (0..n)
                    .filter(|&i| cv.0[i].0 < 64 || cv.0[i].0 % k == 0 || i + 1 == n)
                    .map(|i| cv.0[i])
                    .collect();
            }
        }
    }
    d
}

const PASSOS: [u32; 5] = [1, 4, 8, 16, 32];

fn mm(v: &[u32]) -> String {
    let mut s = v.to_vec();
    s.sort_unstable();
    format!(
        "{}–{} (mediana {})",
        s.first().unwrap_or(&0),
        s.last().unwrap_or(&0),
        s.get(s.len() / 2).copied().unwrap_or(0)
    )
}

/// ⭐ **Quanto o aluno espera e o Nada em cada idade** (W7): as curvas medidas a cada quadro e
/// dizimadas a cada [`PASSOS`]. O aluno exacto: quadros depois do `Up` até a pincelada se cumprir
/// (com [`mede::PASSO_DA_CURVA`]) e a razão no `Up`. O aluno que pula a pincelada e espera até a água
/// dele parar: a razão máxima enquanto as curvas mudam (idade ≤ a última amostrada) e depois.
#[test]
#[ignore = "sonda de medição (W7): a espera do aluno exacto e o Nada por idade"]
fn sonda_quanto_espera_o_aluno() {
    let ps30 = trinta();
    let casos: Vec<(PaintMedia, &[Pincelada], u32)> = MEIOS
        .iter()
        .map(|&m| (m, &SPLASH[..], 512))
        .chain([
            (PaintMedia::WetPaint, &SPLASH[..], 1024),
            (PaintMedia::WetPaint, &ps30[..], 512),
        ])
        .collect();
    let atual = PASSOS
        .iter()
        .position(|&k| k == mede::PASSO_DA_CURVA)
        .expect("o passo em vigor está nos passos medidos");
    println!("load antes: {}", carga());
    for (m, ps, lado) in casos {
        let track = trilha_no_meio(m, ps, lado);
        let t0 = std::time::Instant::now();
        let e1 = medido_a_cada_quadro(&track);
        let mede_s = t0.elapsed().as_secs_f32();
        let es: Vec<EnsinoDoPainter> = PASSOS.iter().map(|&k| decimado(&e1, k)).collect();
        #[expect(clippy::cast_precision_loss, reason = "o lado cabe num f32")]
        let l = lado as f32;
        // (k, o critério, o aluno com as anteriores certas), cada um num fio.
        let mut alunos = Vec::new();
        let mut t = inicial::tool_no_inicial(&track.initial).expect("inicial");
        for (k, p) in ps.iter().enumerate() {
            if let Some(ci) = criterio_de(&e1, &track, k)
                .and_then(|c| e1.criterios.iter().position(|x| x.regiao.k == c.regiao.k))
            {
                alunos.push((k, ci, t.instantaneo().expect("instantâneo do aluno")));
            }
            pinta(&mut t, p, l);
        }
        let es = &es;
        let razao = |e: &EnsinoDoPainter, a: &PainterTool, ci: usize| {
            e.razao(a, &e.criterios[ci]).expect("medida")
        };
        // Por aluno: (quadros até cumprir, a razão no Up, (máx. nas curvas, máx. depois) por passo, o
        // pior nas curvas (razão, idade)).
        let corre = |k: usize, ci: usize, t: PainterTool| {
            let mut a = t.instantaneo().expect("instantâneo do aluno");
            traca(&mut a, &ps[k], l);
            let no_up = razao(&es[atual], &a, ci);
            let mut q = 0;
            let ate_cumprir = loop {
                if razao(&es[atual], &a, ci) >= FRACAO_DA_GRAVADA_QUE_CUMPRE {
                    break Some(q);
                }
                if q >= base::QUADROS_ATE_ASSENTAR {
                    break None;
                }
                quadro(&mut a);
                q += 1;
            };
            let ate = es[0].criterios[ci]
                .referencia
                .as_ref()
                .expect("medida")
                .ate();
            let mut a = t;
            let mut nada = vec![(0.0f32, 0.0f32); PASSOS.len()];
            let mut pior = (0.0f32, 0);
            let mut q = 0u32;
            loop {
                let dentro = idade(&a) <= ate;
                for (j, e) in es.iter().enumerate() {
                    let r = razao(e, &a, ci);
                    let x = if dentro {
                        &mut nada[j].0
                    } else {
                        &mut nada[j].1
                    };
                    if j == atual && dentro && r > pior.0 {
                        pior = (r, idade(&a));
                    }
                    *x = x.max(r);
                }
                if !dentro && (a.wet_assentada() || q >= 4_000) {
                    break;
                }
                quadro(&mut a);
                q += 1;
            }
            (k, ate_cumprir, no_up, nada, pior)
        };
        let feitos: Vec<_> = std::thread::scope(|sc| {
            let fios: Vec<_> = alunos
                .into_iter()
                .map(|(k, ci, a)| sc.spawn(move || corre(k, ci, a)))
                .collect();
            fios.into_iter()
                .map(|f| f.join().expect("o aluno"))
                .collect()
        });
        let (mut ate_cumprir, mut nunca, mut no_up) = (Vec::new(), Vec::new(), f32::MAX);
        let mut nada = vec![(0.0f32, 0.0f32); PASSOS.len()];
        let mut nada_onde = (0usize, 0u32, 0.0f32);
        for (k, ac, nu, na, pior) in feitos {
            match ac {
                Some(q) => ate_cumprir.push(q),
                None => nunca.push(k),
            }
            no_up = no_up.min(nu);
            for (x, y) in nada.iter_mut().zip(&na) {
                *x = (x.0.max(y.0), x.1.max(y.1));
            }
            if pior.0 > nada_onde.2 {
                nada_onde = (k, pior.1, pior.0);
            }
        }
        let nada_s: Vec<String> = PASSOS
            .iter()
            .zip(&nada)
            .map(|(k, (d, f))| format!("passo {k}: {:.1} / {:.1}", 100.0 * d, 100.0 * f))
            .collect();
        println!(
            "{m:?} {} pinceladas {lado}² (curvas a cada quadro em {mede_s:.1} s): exacto — quadros depois \
             do Up até cumprir {} · nunca {nunca:?} · razão no Up (mín.) {:.1} % | Nada (máx. %, nas \
             curvas / depois) {} · o pior nas curvas: k = {}, idade {}",
            ps.len(),
            mm(&ate_cumprir),
            100.0 * no_up,
            nada_s.join(" · "),
            nada_onde.0,
            nada_onde.1
        );
    }
    println!("load depois: {}", carga());
}

/// Os instantes (quadros depois da variante) em que [`vigia`] lê a fracção.
const INSTANTES: [u32; 6] = [60, 120, 240, 480, 960, 1920];

/// A fracção de `c` vigiada até a água PARAR: a de cada um dos [`INSTANTES`] e a final.
fn vigia(t: &mut PainterTool, e: &EnsinoDoPainter, c: &CriterioDaPincelada) -> ([f32; 6], f32) {
    let mut em = [0.0f32; 6];
    let mut q = 0u32;
    while q < 20_000 && !t.wet_assentada() {
        quadro(t);
        q += 1;
        if let Some(i) = INSTANTES.iter().position(|&x| x == q) {
            em[i] = e.fracao_pintada(t, c);
        }
    }
    let f = e.fracao_pintada(t, c);
    for (i, &x) in INSTANTES.iter().enumerate() {
        if q < x {
            em[i] = f;
        }
    }
    (em, f)
}

/// ⭐ **Em que instante medir a base e a gravada** (W7, `docs/Macros/04_plano_W7.md`): esperar a água
/// PARAR custa 2 645–3 562 quadros por medida em Wet Paint, e a fracção muda até ~2 600 quadros com
/// pausas de até 413 (a 1.ª rodada desta sonda). Medidas num instante T, o aluno que pula a pincelada e
/// espera PARA SEMPRE chega a `(Nada_final − Nada_T) / (Exacta_T − Nada_T)` da régua — o erro da lei
/// em T. Exacta e Nada (= a gravada e a base), Wet Paint: o Splash a 512² e 1024², e as 8 primeiras das
/// 30 pinceladas a 512². Imprime o máximo do erro por T.
#[test]
#[ignore = "sonda de medição (W7): o erro da régua por instante de medida"]
fn sonda_quando_a_fracao_assenta() {
    use Variante::{Exacta, Nada};
    let ps30 = trinta();
    println!("load antes: {}", carga());
    for (ps, lado) in [
        (&SPLASH[..], 512u32),
        (&SPLASH[..], 1024),
        (&ps30[..8], 512),
    ] {
        let track = trilha_no_meio(PaintMedia::WetPaint, ps, lado);
        let e = EnsinoDoPainter::de(&track).expect("ensino");
        #[expect(clippy::cast_precision_loss, reason = "o lado cabe num f32")]
        let l = lado as f32;
        let mut t = inicial::tool_no_inicial(&track.initial).expect("inicial");
        let mut erro = [0.0f32; 6];
        let mut creep_exacta = [0.0f32; 6];
        for (k, p) in ps.iter().enumerate() {
            if let Some(c) = criterio_de(&e, &track, k) {
                let corre = |v| {
                    let mut a = t.instantaneo().expect("instantâneo do aluno");
                    faz_sem_assentar(&mut a, &track, k, p, v, l);
                    vigia(&mut a, &e, c)
                };
                let (ex, exf) = corre(Exacta);
                let (na, naf) = corre(Nada);
                for i in 0..INSTANTES.len() {
                    let delta = (ex[i] - na[i]).max(1e-6);
                    erro[i] = erro[i].max((naf - na[i]) / delta);
                    creep_exacta[i] = creep_exacta[i].max((exf - ex[i]) / delta);
                }
            }
            pinta(&mut t, p, l);
        }
        let fmt = |v: &[f32; 6]| {
            INSTANTES
                .iter()
                .zip(v)
                .map(|(q, x)| format!("T={q}: {:.2}", 100.0 * x))
                .collect::<Vec<_>>()
                .join(" · ")
        };
        println!(
            "WetPaint {} pinceladas {lado}²: erro do Nada (máx., % da régua) {} | a Exacta ainda cresce {}",
            ps.len(),
            fmt(&erro),
            fmt(&creep_exacta)
        );
    }
    println!("load depois: {}", carga());
}

/// ⭐ **O dreno de cada quadro da assentada** (W7): o Splash em Wet Paint, a reprodução até ao toque da
/// 2.ª pincelada e 960 quadros sem eventos — drenando com a composição ([`PainterTool::take_preview_arc`],
/// o editor) ou sem ela (`drena_sem_compor`, o passo `Preview` da macro tocada: o mesmo estado). O tempo
/// (o mínimo de 2, intercaladas) e a fracção da 1.ª pincelada no fim (tem de ser a mesma).
#[test]
#[ignore = "sonda de medição (W7): o custo do dreno na assentada"]
fn sonda_o_dreno_da_assentada() {
    for lado in [512u32, 1024] {
        let track = trilha_no_meio(PaintMedia::WetPaint, &SPLASH, lado);
        let e = EnsinoDoPainter::de(&track).expect("ensino");
        let tocavel = track.tocavel(&LeitorDoPainter).expect("tocável");
        let corte = pinceladas_em(&track, &tocavel).expect("pinceladas")[1].inicio;
        let mut toc = TocadorDoPainter::novo(
            &tocavel.initial,
            tocavel.events.iter().map(|e| e.payload.as_slice()),
        )
        .expect("tocador");
        for ev in &tocavel.events[..corte] {
            toca(&mut toc, &ev.payload).expect("toca");
        }
        let mut ms = [u128::MAX; 2];
        let mut fs = [0.0f32; 2];
        for volta in 0..2 {
            for j in if volta == 0 { [0, 1] } else { [1, 0] } {
                let mut c = toc.instantaneo().expect("instantâneo");
                c.passa_ao_relogio_fixo();
                let t = c.tool_mut();
                let t0 = std::time::Instant::now();
                for _ in 0..base::QUADROS_ATE_ASSENTAR {
                    if j == 0 {
                        drop(t.take_preview_arc());
                    } else {
                        t.drena_sem_compor();
                    }
                    t.on_tick(DT_MS);
                }
                ms[j] = ms[j].min(t0.elapsed().as_millis());
                fs[j] = e.fracao_pintada(t, &e.criterios[0]);
            }
        }
        println!(
            "{lado}²: 960 quadros com a composição {} ms (fracção {}) · sem {} ms (fracção {}) · load {}",
            ms[0],
            fs[0],
            ms[1],
            fs[1],
            carga().trim()
        );
    }
}

/// ⭐ **O custo do passo da curva** (W7): [`EnsinoDoPainter::mede_com`] do Splash em Wet Paint a 512²
/// com um ponto a cada quadro e a cada 8 — o mesmo número de fios, alternadas, o mínimo de 3.
#[test]
#[ignore = "sonda de medição (W7): o custo de ler a fracção a cada quadro"]
fn sonda_o_custo_do_passo() {
    let track = trilha_no_meio(PaintMedia::WetPaint, &SPLASH, 512);
    let e = EnsinoDoPainter::de(&track).expect("ensino");
    let mut ms = [u128::MAX; 2];
    println!("load antes: {}", carga());
    for volta in 0..3 {
        for j in if volta % 2 == 0 { [0, 1] } else { [1, 0] } {
            let passo = [1, 8][j];
            let t0 = std::time::Instant::now();
            e.mede_com(&track, &AtomicBool::new(false), passo, mede::fios(), |_| {})
                .expect("mede");
            ms[j] = ms[j].min(t0.elapsed().as_millis());
        }
    }
    println!(
        "Splash Wet Paint 512², {} fios: passo 1 {} ms · passo 8 {} ms · load {}",
        mede::fios(),
        ms[0],
        ms[1],
        carga().trim()
    );
}
