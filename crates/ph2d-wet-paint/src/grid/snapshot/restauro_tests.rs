//! O restauro por PLANO (tiles num job por plano) e as recontagens por pedaço dão o bit do restauro em série de antes
//! (o oráculo `ANTIGO`), no undo, no redo (com o `extra` = `desde`) e no inverso; e a sonda do Ctrl+Z a 4K.

use super::*;
use crate::painter::Engine;
use crate::sim::LeiDaTinta;
use std::hash::{Hash, Hasher};
use std::time::Instant;

fn h<T: std::fmt::Debug>(st: &mut Vec<u64>, v: &[T]) {
    // `Debug` de f32 distingue `-0.0`; o tamanho entra junto (um plano vazio ≠ um plano de zeros).
    let mut hs = std::collections::hash_map::DefaultHasher::new();
    v.len().hash(&mut hs);
    for x in v {
        format!("{x:?}").hash(&mut hs);
    }
    st.push(hs.finish());
}

/// Os bits de um plano de `f32` (`-0.0` ≠ `0.0`), sem o `Debug` célula a célula.
fn hf(st: &mut Vec<u64>, v: &[f32]) {
    let mut hs = std::collections::hash_map::DefaultHasher::new();
    v.len().hash(&mut hs);
    v.iter().for_each(|x| x.to_bits().hash(&mut hs));
    st.push(hs.finish());
}

fn hi<T: Hash>(st: &mut Vec<u64>, v: &[T]) {
    let mut hs = std::collections::hash_map::DefaultHasher::new();
    v.hash(&mut hs);
    st.push(hs.finish());
}

/// TODO o estado da grade que o restauro toca: os planos, o fluxo, as faixas vivas e os escalares contados.
fn estado(g: &Grid) -> Vec<u64> {
    let mut st = Vec::new();
    hf(&mut st, &g.film);
    hf(&mut st, &g.susp);
    hf(&mut st, g.susp_rgb.as_flattened());
    hf(&mut st, &g.sett);
    hf(&mut st, g.sett_rgb.as_flattened());
    hf(&mut st, &g.relevo);
    hf(&mut st, &g.altura);
    hf(&mut st, &g.espessura);
    hf(&mut st, &g.crista);
    hf(&mut st, &g.nivelado);
    hf(&mut st, &g.vela);
    h(&mut st, &g.material.para_vec());
    h(&mut st, &g.aparencia.para_vec());
    h(&mut st, &g.oleo.para_vec());
    h(&mut st, &g.tinta);
    h(&mut st, &g.pigmento);
    hi(&mut st, &g.wet);
    hi(&mut st, &g.active);
    hi(&mut st, &g.bloom);
    hf(&mut st, &g.vel_x);
    hf(&mut st, &g.vel_y);
    hf(&mut st, &g.flow_x);
    hf(&mut st, &g.flow_y);
    hi(&mut st, &g.row_lo);
    hi(&mut st, &g.row_hi);
    hi(&mut st, &g.row_tiles);
    h(
        &mut st,
        &[(
            g.bx0,
            g.by0,
            g.bx1,
            g.by1,
            g.has_fluid,
            g.paper_sheet,
            g.oleo_real,
            g.suave_teto.to_bits(),
            g.material.pedacos_vivos(),
            g.aparencia.pedacos_vivos(),
            g.oleo.pedacos_vivos(),
        )],
    );
    st
}

fn traco(e: &mut Engine, (x, y): (f64, f64), (dx, dy): (f64, f64), n: usize) {
    e.pointer_down(x, y, None);
    for i in 0..n {
        e.pointer_frame(x + dx * i as f64, y + dy * i as f64);
    }
    e.pointer_up();
    let fim = (x + dx * n as f64, y + dy * n as f64);
    while e.release_frame(fim.0, fim.1) {}
}

fn corre(e: &mut Engine, n: usize) {
    for _ in 0..n {
        e.step_simulation();
    }
}

fn monta(w: usize, h: usize, pigmento: bool, tamanho: f64) -> Engine {
    monta_lei(w, h, u8::from(pigmento), tamanho)
}

/// `lei`: 0 = a aguada, 1 = o pigmento com o Real Oil, 2 = o Óleo Real com material e aparência na célula (o Dabber):
/// os três planos esparsos vivos.
fn monta_lei(w: usize, h: usize, lei: u8, tamanho: f64) -> Engine {
    let mut e = Engine::new(w, h);
    if lei >= 1 {
        e.sim.set_tinta(LeiDaTinta::Pigmento);
        e.sim.oleo = true;
    }
    if lei == 2 {
        e.set_knob(crate::tuning::Knob::Metallic, 0.6);
        e.set_knob(crate::tuning::Knob::Opaque, 0.7);
        e.set_knob(crate::tuning::Knob::OilDepth, 1.8);
    }
    e.sliders.water = 1.0;
    e.sliders.size = tamanho;
    let (wf, hf) = (w as f64, h as f64);
    for (a, d) in [
        ((0.1, 0.2), (0.012, 0.004)),
        ((0.15, 0.7), (0.011, -0.008)),
        ((0.5, 0.1), (-0.004, 0.012)),
    ] {
        traco(&mut e, (a.0 * wf, a.1 * hf), (d.0 * wf, d.1 * hf), 60);
        corre(&mut e, 6);
    }
    e
}

/// Corre `f` com o oráculo ligado ou não, e devolve o que ela devolveu.
fn com<R>(antigo: bool, f: impl FnOnce() -> R) -> R {
    ANTIGO.with(|a| a.set(antigo));
    let r = f();
    ANTIGO.with(|a| a.set(false));
    r
}

/// A sequência que passa pelo `extra` (`desde`: a água anda entre um evento e o redo) e por um traço novo a meio.
#[test]
fn o_restauro_por_plano_da_o_bit_do_restauro_em_serie() {
    for pigmento in [false, true] {
        let (mut a, mut b) = (
            monta(960, 540, pigmento, 1.0),
            monta(960, 540, pigmento, 1.0),
        );
        let g = b.active_grid();
        assert_eq!(estado(a.active_grid()), estado(g), "partiram diferentes");
        assert!(
            !pigmento || !g.oleo.is_empty(),
            "o óleo nunca nasceu: a recontagem fica por medir"
        );
        let tiles = b.layers[b.active_layer]
            .undo
            .last()
            .map_or(0, |r| r.finos.len());
        assert!(tiles > 40, "o passo guarda só {tiles} tiles");
        for (k, passo) in [
            "undo", "corre", "undo", "corre", "redo", "corre", "redo", "undo", "traco", "undo",
            "redo", "undo", "undo", "corre", "redo", "redo",
        ]
        .into_iter()
        .enumerate()
        {
            let faz = |e: &mut Engine| match passo {
                "undo" => assert!(e.undo_sem_zerar_o_fluxo() || !e.can_undo()),
                "redo" => assert!(e.redo() || !e.can_redo()),
                "corre" => corre(e, 3),
                _ => traco(e, (400.0, 300.0), (9.0, 3.0), 30),
            };
            com(true, || faz(&mut a));
            com(false, || faz(&mut b));
            assert_eq!(
                estado(a.active_grid()),
                estado(b.active_grid()),
                "{k}: {passo} (pigmento {pigmento})"
            );
            assert_eq!(a.dirty, b.dirty, "{k}: {passo}: sujo");
        }
    }
}

/// As recontagens por pedaço dão o bit da célula a célula: pedaços vivos, ausentes, o último cortado, campo vazio.
#[test]
fn as_recontagens_por_pedaco_dao_o_bit_da_celula_a_celula() {
    let mut e = Engine::new(300, 200);
    let l = e.active_layer;
    let g = &mut e.layers[l].grid;
    let n = g.cells;
    let (viva, chao) = (
        aparencia::AparenciaCelula::NEUTRA,
        oleo_celula::OleoCelula::NEUTRA,
    );
    for caso in 0..4 {
        g.aparencia = campo_esparso::CampoEsparso::novo(viva, if caso == 3 { 0 } else { n });
        g.oleo = campo_esparso::CampoEsparso::novo(chao, if caso == 3 { 0 } else { n });
        if caso > 0 && caso < 3 {
            for (i, s) in [
                (5, 0.3),
                (campo_esparso::PEDACO + 7, 0.9),
                (n - 1, 0.5 * caso as f32),
            ] {
                g.aparencia[i] = aparencia::AparenciaCelula { suave: s, ..viva };
            }
            g.oleo[n - 1] = oleo_celula::OleoCelula {
                oleo: if caso == 1 { 0.2 } else { 0.0 },
                ..chao
            };
        }
        for teto in [0.0f32, 0.6, 2.0] {
            g.suave_teto = teto;
            aparencia::reconta_teto(g);
            let novo = (g.suave_teto.to_bits(), g.oleo_real);
            g.suave_teto = teto;
            aparencia::reconta_teto_antigo(g);
            oleo_celula::reconta_real(g);
            let real = g.oleo_real;
            oleo_celula::reconta_real_antigo(g);
            assert_eq!(novo.0, g.suave_teto.to_bits(), "caso {caso}, teto {teto}");
            assert_eq!(real, g.oleo_real, "caso {caso}, teto {teto}");
        }
    }
}

fn ms(a: Instant) -> f64 {
    a.elapsed().as_secs_f64() * 1e3
}

fn resumo(nome: &str, v: &mut [f64]) {
    v.sort_by(f64::total_cmp);
    eprintln!(
        "  {nome:<28} min {:7.2} ms · mediana {:7.2} ms",
        v[0],
        v[v.len() / 2]
    );
}

/// **Sonda (ignorada):** o Ctrl+Z/redo do motor a 3840×2160 e as fases do restauro, antigo × novo intercalados no
/// MESMO processo (a ordem roda a cada rodada); o resumo é o MÍNIMO.
#[test]
#[ignore = "sonda — --profile smoke -- --ignored --nocapture"]
fn sonda_o_ctrl_z_wet_a_4k() {
    let tam = std::env::var("PH2D_SONDA_TAMANHO")
        .ok()
        .and_then(|v| v.parse().ok());
    // `PH2D_SONDA_LEI`: 0 aguada, 1 pigmento + Real Oil, 2 o Óleo Real com material e aparência (`monta_lei`).
    let lei = std::env::var("PH2D_SONDA_LEI")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(u8::from(std::env::var("PH2D_SONDA_PIGMENTO").is_ok()));
    let mut e = monta_lei(3840, 2160, lei, tam.unwrap_or(1.0));
    let l = e.active_layer;
    let tiles = e.layers[l].undo.last().map_or(0, |r| r.finos.len());
    let g = &e.layers[l].grid;
    eprintln!(
        "4K: {tiles} tiles no passo do topo · material {} · aparência {} · óleo {} pedaços vivos · tinta {} · pigmento {}",
        g.material.pedacos_vivos(),
        g.aparencia.pedacos_vivos(),
        g.oleo.pedacos_vivos(),
        g.tinta.len(),
        g.pigmento.len()
    );
    const N: usize = 9;
    // 0 = o restauro em série de antes, 1 = a troca só dos densos (os esparsos copiam), 2 = a troca de todos.
    let nomes = ["antigo", "troca densa", "troca total"];
    let mut t = vec![
        [
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new()
        ];
        3
    ];
    for r in 0..N {
        for v in [r % 3, (r + 1) % 3, (r + 2) % 3] {
            let antigo = v == 0;
            super::troca::ESPARSOS_COPIAM.with(|c| c.set(v == 1));
            let a = Instant::now();
            com(antigo, || assert!(e.undo_sem_zerar_o_fluxo()));
            t[v][0].push(ms(a));
            let a = Instant::now();
            com(antigo, || assert!(e.redo_sem_zerar_o_fluxo()));
            t[v][1].push(ms(a));
            super::troca::ESPARSOS_COPIAM.with(|c| c.set(false));
            // As fases, sobre o passo do topo: o inverso, os tiles postos e repostos, as duas recontagens.
            let layer = &mut e.layers[l];
            let reg = layer.undo.last().expect("o passo");
            let a = Instant::now();
            let inv = com(antigo, || Registo::inverso(&layer.grid, reg, None));
            t[v][2].push(ms(a));
            let (s, rows) = (layer.grid.s, layer.grid.rows);
            let a = Instant::now();
            for rg in [reg, &inv] {
                let ts: Vec<(Ret, &TileFino)> = rg
                    .finos
                    .iter()
                    .map(|(k, t)| (ret_do_tile(*k, s, rows), t))
                    .collect();
                if antigo {
                    for (r, t) in &ts {
                        t.poe(&mut layer.grid, *r);
                    }
                } else {
                    TileFino::poe_no_pool(&mut layer.grid, &ts);
                }
            }
            t[v][3].push(ms(a) / 2.0);
            let a = Instant::now();
            if antigo {
                aparencia::reconta_teto_antigo(&mut layer.grid);
            } else {
                aparencia::reconta_teto(&mut layer.grid);
            }
            t[v][4].push(ms(a));
            let a = Instant::now();
            if antigo {
                oleo_celula::reconta_real_antigo(&mut layer.grid);
            } else {
                oleo_celula::reconta_real(&mut layer.grid);
            }
            t[v][5].push(ms(a));
            // A troca sozinha (sem o `drain_step` nem o sujo do motor): o registo do topo vai e volta.
            let mut reg = layer.undo.pop().expect("o passo");
            super::troca::ESPARSOS_COPIAM.with(|c| c.set(v == 1));
            let a = Instant::now();
            let (mut inv, _) = com(antigo, || reg.troca(&mut layer.grid, None, false));
            let (mut volta, _) = com(antigo, || inv.troca(&mut layer.grid, None, false));
            t[v][6].push(ms(a) / 2.0);
            (volta.passo, volta.dabs) = (reg.passo, reg.dabs);
            super::troca::ESPARSOS_COPIAM.with(|c| c.set(false));
            layer.undo.push(volta);
        }
    }
    for (v, nome) in nomes.iter().enumerate() {
        eprintln!("{nome}:");
        let f = [
            "undo do motor",
            "redo do motor",
            "inverso",
            "tiles postos (1 restauro)",
            "reconta_teto",
            "reconta_real",
            "troca só (1 restauro)",
        ];
        for (i, f) in f.iter().enumerate() {
            resumo(f, &mut t[v][i]);
        }
    }
}

/// As chaves e os bytes de cada registo do undo, e os bytes da história toda (redo e `desde` incluídos): o inverso da
/// troca guarda os mesmos tiles.
fn pilha(e: &Engine) -> (Vec<(Vec<u32>, Vec<u32>, usize)>, usize) {
    let l = &e.layers[e.active_layer];
    let undo = l
        .undo
        .iter()
        .map(|r| {
            (
                r.finos.keys().copied().collect(),
                r.fluxo.keys().copied().collect(),
                r.bytes_retidos(),
            )
        })
        .collect();
    (undo, e.bytes_do_historico(e.active_layer))
}

/// A troca (o registo que sai da pilha troca os tiles com a grade) dá o bit do inverso + restauro em série: o estado
/// todo a cada passo, a pilha (chaves e bytes), e no fim a história inteira desfeita e refeita — cada inverso que a
/// troca deixou volta a ser posto e comparado. Nas duas leis, com o `extra` do redo e um traço a meio.
#[test]
fn a_troca_da_o_bit_do_inverso_e_do_restauro() {
    for lei in [0, 1, 2] {
        let (mut a, mut b) = (monta_lei(700, 420, lei, 1.0), monta_lei(700, 420, lei, 1.0));
        if lei == 2 {
            let g = b.active_grid();
            let vivos = [
                g.material.pedacos_vivos(),
                g.aparencia.pedacos_vivos(),
                g.oleo.pedacos_vivos(),
            ];
            assert!(
                vivos.iter().all(|&n| n > 4),
                "os esparsos não nasceram: {vivos:?}"
            );
        }
        let passos = [
            "undo", "corre", "redo", "undo", "undo", "traco", "corre", "undo", "redo", "corre",
            "undo", "redo", "redo",
        ];
        for (k, passo) in passos.into_iter().enumerate() {
            let faz = |e: &mut Engine| match passo {
                "undo" => assert!(e.undo() || !e.can_undo()),
                "redo" => assert!(e.redo_sem_zerar_o_fluxo() || !e.can_redo()),
                "corre" => corre(e, 2),
                _ => traco(e, (300.0, 200.0), (7.0, 2.0), 25),
            };
            com(true, || faz(&mut a));
            com(false, || faz(&mut b));
            let at = format!("{k}: {passo} (lei {lei})");
            assert_eq!(estado(a.active_grid()), estado(b.active_grid()), "{at}");
            assert_eq!(pilha(&a), pilha(&b), "{at}: a pilha");
            assert_eq!(a.dirty, b.dirty, "{at}: sujo");
        }
        let mut n = 0;
        for desfaz in [true, false] {
            loop {
                let feito = |e: &mut Engine| if desfaz { e.undo() } else { e.redo() };
                let (fa, fb) = (com(true, || feito(&mut a)), com(false, || feito(&mut b)));
                assert_eq!(fa, fb);
                if !fa {
                    break;
                }
                n += 1;
                assert_eq!(
                    estado(a.active_grid()),
                    estado(b.active_grid()),
                    "volta {n}"
                );
                assert_eq!(pilha(&a), pilha(&b), "volta {n}: a pilha");
            }
        }
        assert!(n >= 4, "a história inteira tinha só {n} passos");
    }
}

/// B18.11: a troca não reescreve a linha de tile que já é igual ao bit à grade (o papel `relevo`, o que a água não
/// tocou) — e o bit é o da troca cega (o `TROCA_CEGA`, a de antes) e o do oráculo (`ANTIGO`).
#[test]
fn a_troca_poupa_as_linhas_iguais_e_da_o_bit_da_troca_cega() {
    use super::troca::{POUPADAS, TROCA_CEGA};
    use std::sync::atomic::Ordering;
    for lei in [0, 1, 2] {
        let mut e = [0, 1, 2].map(|_| monta_lei(700, 420, lei, 1.0));
        let antes = POUPADAS.load(Ordering::Relaxed);
        for passo in ["undo", "redo", "corre", "undo", "undo", "redo"] {
            for (k, e) in e.iter_mut().enumerate() {
                let faz = || match passo {
                    "undo" => assert!(e.undo_sem_zerar_o_fluxo() || !e.can_undo()),
                    "redo" => assert!(e.redo_sem_zerar_o_fluxo() || !e.can_redo()),
                    _ => corre(e, 2),
                };
                TROCA_CEGA.with(|c| c.set(k == 1));
                com(k == 2, faz);
                TROCA_CEGA.with(|c| c.set(false));
            }
            let at = format!("{passo} (lei {lei})");
            assert_eq!(
                estado(e[0].active_grid()),
                estado(e[1].active_grid()),
                "{at}: × cega"
            );
            assert_eq!(
                estado(e[0].active_grid()),
                estado(e[2].active_grid()),
                "{at}: × oráculo"
            );
            assert_eq!(pilha(&e[0]), pilha(&e[1]), "{at}: a pilha");
        }
        assert!(
            POUPADAS.load(Ordering::Relaxed) > antes,
            "lei {lei}: nenhuma linha igual poupada"
        );
    }
}

/// B18.11: os tiles de fluxo (`vel_x`/`vel_y`) trocam no pool e dão o bit do fluxo em série de antes
/// (`FLUXO_EM_SERIE`) e do oráculo (`ANTIGO`), nas três leis, com a água a correr entre os passos.
#[test]
fn o_fluxo_no_pool_da_o_bit_do_fluxo_em_serie() {
    use super::troca::FLUXO_EM_SERIE;
    for lei in [0, 1, 2] {
        let mut e = [0, 1, 2].map(|_| monta_lei(700, 420, lei, 1.0));
        assert!(
            e[0].active_grid().vel_x.iter().any(|v| *v != 0.0),
            "lei {lei}: sem velocidade, o gate não mede o fluxo"
        );
        for passo in ["undo", "redo", "corre", "undo", "undo", "redo", "redo"] {
            for (k, e) in e.iter_mut().enumerate() {
                let faz = || match passo {
                    "undo" => assert!(e.undo_sem_zerar_o_fluxo() || !e.can_undo()),
                    "redo" => assert!(e.redo_sem_zerar_o_fluxo() || !e.can_redo()),
                    _ => corre(e, 2),
                };
                FLUXO_EM_SERIE.with(|c| c.set(k == 1));
                com(k == 2, faz);
                FLUXO_EM_SERIE.with(|c| c.set(false));
            }
            let at = format!("{passo} (lei {lei})");
            assert_eq!(
                estado(e[0].active_grid()),
                estado(e[1].active_grid()),
                "{at}: × série"
            );
            assert_eq!(
                estado(e[0].active_grid()),
                estado(e[2].active_grid()),
                "{at}: × oráculo"
            );
            assert_eq!(pilha(&e[0]), pilha(&e[1]), "{at}: a pilha");
        }
    }
}
