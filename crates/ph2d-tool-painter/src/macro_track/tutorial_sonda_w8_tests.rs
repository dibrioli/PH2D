//! **Partilhar uma corrida entre a gravada da anterior e a base da `k`?** (W8, `docs/Macros/05_plano_W8.md`
//! item 3). As duas são «a trilha até ao toque de `k`, depois nada»: a base reproduz pela fita a espera
//! (o tempo da `k` devolvido à etapa seguinte) e só depois assenta no relógio fixo; a gravada assenta
//! no relógio fixo logo no corte. Aqui, a base que o produto mede ([`EnsinoDoPainter::mede`]) contra
//! a gravada da anterior continuada no relógio fixo até à idade em que a base acaba, lida no critério
//! da `k` — o erro em % da régua da `k` (a margem da [`FRACAO_DA_GRAVADA_QUE_CUMPRE`]: 6,55 pontos).
//! Duas máquinas: o relógio fixo dos roteiros e uma gravação com a água no worker ritmado e quadros
//! irregulares (a de um artista).
//!
//! `bash scripts/ph2d-run.sh cargo test -p ph2d-tool-painter --profile smoke --lib sonda_a_corrida_partilhada -- --ignored --nocapture`

use super::*;
use crate::macro_track::RitmoDaAgua;
use crate::macro_track::roteiro::grava_pinceladas_como_a_maquina;
use std::collections::BTreeMap;

/// A gravada da anterior por assentar: o critério da `k`, o instantâneo no toque de `k`, as leituras
/// desde o último `Up` e os quadros do relógio fixo até à idade em que a base acaba.
struct Corte {
    c: usize,
    toc: TocadorDoPainter,
    antes: base::Amostras,
    quadros: u32,
}

/// A curva partilhada de cada critério com uma pincelada antes (`c` → curva).
fn partilhadas(
    e: &EnsinoDoPainter,
    track: &Track,
    bases: &BTreeMap<usize, Referencia>,
) -> BTreeMap<usize, Curva> {
    let tocavel = track.tocavel(&LeitorDoPainter).expect("tocável");
    let ps = pinceladas_em(track, &tocavel).expect("pinceladas");
    let ups: Vec<usize> = tocavel
        .events
        .iter()
        .enumerate()
        .filter(|(_, ev)| {
            matches!(
                Passo::descodifica(&ev.payload),
                Ok(Passo::Ponteiro {
                    fase: PointerPhase::Up,
                    ..
                })
            )
        })
        .map(|(i, _)| i)
        .collect();
    // (desde, corte, c, leituras)
    let mut janelas: Vec<(usize, usize, usize, base::Amostras)> = ps
        .iter()
        .skip(1)
        .filter_map(|p| {
            let c = e.criterios.iter().position(|c| c.regiao.k == p.k)?;
            let desde = ups.iter().rev().find(|&&u| u < p.inicio).copied()?;
            Some((desde, p.inicio, c, base::Amostras::default()))
        })
        .collect();
    let mut toc = TocadorDoPainter::novo(
        &tocavel.initial,
        tocavel.events.iter().map(|ev| ev.payload.as_slice()),
    )
    .expect("tocador");
    let mut cortes = Vec::new();
    for ev in 0..=tocavel.events.len() {
        for j in janelas.iter_mut().filter(|j| j.1 == ev) {
            let copia = toc.instantaneo().expect("instantâneo");
            let idade = idade_de(copia.tool()).unwrap_or(0);
            let ate = bases[&e.criterios[j.2].regiao.k].base.ate();
            cortes.push(Corte {
                c: j.2,
                toc: copia,
                antes: std::mem::take(&mut j.3),
                quadros: base::QUADROS_ATE_ASSENTAR.max(ate.saturating_sub(idade)),
            });
        }
        let Some(x) = tocavel.events.get(ev) else {
            break;
        };
        toca(&mut toc, &x.payload).expect("toca");
        for j in janelas.iter_mut().filter(|j| j.0 <= ev && ev < j.1) {
            j.3.le(e, toc.tool(), j.2);
        }
    }
    let n = mede::fios();
    let mut grupos: Vec<Vec<Corte>> = (0..n).map(|_| Vec::new()).collect();
    for (i, c) in cortes.into_iter().enumerate() {
        grupos[i % n].push(c);
    }
    let (fora, daqui) = (AtomicBool::new(false), AtomicBool::new(false));
    let p = mede::Paragem::de(&fora, &daqui);
    std::thread::scope(|sc| {
        let fios: Vec<_> = grupos
            .into_iter()
            .map(|g| {
                sc.spawn(move || {
                    g.into_iter()
                        .map(|x| {
                            let curva = base::assentada_por(
                                e,
                                x.toc,
                                x.c,
                                x.antes,
                                mede::PASSO_DA_CURVA,
                                p,
                                x.quadros,
                            )
                            .expect("não parou");
                            (x.c, curva)
                        })
                        .collect::<Vec<_>>()
                })
            })
            .collect();
        fios.into_iter()
            .flat_map(|f| f.join().expect("o fio da sonda"))
            .collect()
    })
}

/// (máx. |base − partilhada|, máx. em % da régua, as duas iguais ao bit) em todas as idades.
fn compara(r: &Referencia, s: &Curva) -> (f32, f32, bool) {
    let (mut d, mut pct) = (0.0f32, 0.0f32);
    for q in 0..=r.base.ate().max(s.ate()) {
        let x = (r.base.em(q) - s.em(q)).abs();
        d = d.max(x);
        pct = pct.max(100.0 * x / r.em(q).1);
    }
    (d, pct, r.base == *s)
}

#[test]
#[ignore = "sonda de medição (W8): partilhar a corrida da gravada da anterior com a base"]
fn sonda_a_corrida_partilhada() {
    let maquina = RitmoDaAgua {
        estagios: vec![3, 5, 2, 7, 4, 1, 6],
        nao_volta: vec![false, false, false, true],
    };
    let dts = [16.667, 12.5, 20.8, 16.667, 18.9, 14.4];
    let ps = trinta();
    let casos: [(&str, Box<dyn Fn() -> Option<(Vec<u8>, f64)>>); 3] = [
        (
            "512², relógio fixo",
            Box::new(|| grava_pinceladas(512, 0, &ps, &[])),
        ),
        (
            "1024², relógio fixo",
            Box::new(|| grava_pinceladas(1024, 0, &ps, &[])),
        ),
        (
            "512², a máquina (worker ritmado, quadros irregulares)",
            Box::new(|| grava_pinceladas_como_a_maquina(512, &ps, &maquina, &dts)),
        ),
    ];
    for (nome, grava) in &casos {
        let (b, _) = grava().expect("grava");
        let track = ph2d_macro::codec::read(&b).expect("lê").tracks.remove(0);
        let e = EnsinoDoPainter::de(&track).expect("ensino");
        let antes = carga();
        let t0 = std::time::Instant::now();
        let mut refs = BTreeMap::new();
        e.mede_com(
            &track,
            &AtomicBool::new(false),
            mede::PASSO_DA_CURVA,
            mede::fios(),
            |m| {
                refs.insert(m.k, m.referencia);
            },
        )
        .expect("mede");
        let t_mede = t0.elapsed().as_secs_f32();
        let t0 = std::time::Instant::now();
        let s = partilhadas(&e, &track, &refs);
        let t_partilha = t0.elapsed().as_secs_f32();
        println!(
            "30 pinceladas {nome} ({} eventos, {} critérios): mede {t_mede:.1} s · as partilhadas \
             {t_partilha:.1} s · load {} → {}",
            track.events.len(),
            e.criterios.len(),
            antes.trim(),
            carga().trim()
        );
        let ks: Vec<usize> = e.criterios.iter().map(|c| c.regiao.k).collect();
        let (mut pior, mut pior_util, mut iguais) = (0.0f32, 0.0f32, 0);
        for (c, curva) in &s {
            let k = e.criterios[*c].regiao.k;
            let anterior_medida = k > 0 && ks.contains(&(k - 1));
            let (d, pct, igual) = compara(&refs[&k], curva);
            pior = pior.max(pct);
            if anterior_medida {
                pior_util = pior_util.max(pct);
            }
            iguais += usize::from(igual);
            println!(
                "  k {k:2}: a anterior medida {anterior_medida:5} · iguais ao bit {igual:5} · máx |Δ| \
                 {d:.4} · {pct:6.2} % da régua · idades base {} / partilhada {}",
                refs[&k].base.ate(),
                curva.ate()
            );
        }
        println!(
            "  ⇒ {iguais} de {} iguais ao bit; o pior {pior:.2} % da régua, {pior_util:.2} % onde a \
             partilha poupa (a margem: 6,55)",
            s.len()
        );
    }
}
