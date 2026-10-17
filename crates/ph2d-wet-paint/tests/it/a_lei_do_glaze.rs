//! **O Glaze do PINCEL** (prompt 42, doc `42_medicao_a_lei_da_tinta.md`): a lei da camada que filtra a de baixo.

use ph2d_wet_paint::assenta::{Ecra, assenta, sobre_branco};
use ph2d_wet_paint::colorops::{outra_tinta, vela_da_camada};
use ph2d_wet_paint::grid::{
    GridSnapshot, restore_grid, restore_grid_region, snapshot_grid, snapshot_grid_region,
};
use ph2d_wet_paint::opacity::alpha_of_mass;
use ph2d_wet_paint::painter::Engine;
use ph2d_wet_paint::trail::{Dab, Trail, TrailMode};
use ph2d_wet_paint::tuning::Knob;

const COBRE: Ecra = Ecra::Cobre {
    filme: 0.0,
    esconde: 5.0,
    molhada: 1.0,
    vela: 0.0,
};

/// A fracção multiplicada é a MEDIDA no Rebelle (16/10, sRGB por canal): a mesma tinta 0,48–0,51, amarelo sobre
/// laranja 0,61–0,89, vermelho sobre azul 0,98.
#[test]
fn a_fraccao_da_camada_e_a_medida_no_rebelle() {
    let azul = [23.0, 101.0, 179.0];
    let mesma = vela_da_camada(1.0, azul, azul);
    assert!((0.48..=0.51).contains(&mesma), "a mesma tinta: {mesma:.3}");
    let laranja = vela_da_camada(1.0, [240.0, 120.0, 20.0], [245.0, 205.0, 25.0]);
    assert!(
        (0.61..=0.89).contains(&laranja),
        "amarelo sobre laranja: {laranja:.3}"
    );
    let vermelho = vela_da_camada(1.0, azul, [191.0, 0.0, 18.0]);
    assert!(vermelho >= 0.95, "vermelho sobre azul: {vermelho:.3}");
    assert_eq!(vela_da_camada(0.0, azul, [191.0, 0.0, 18.0]), 0.0);
    // Os carimbos de um traço (a mesma tinta, diluída) não se empilham em camadas.
    assert_eq!(
        outra_tinta(azul, azul.map(|v| 255.0 - 0.4 * (255.0 - v))),
        0.0
    );
    assert!(outra_tinta(azul, [191.0, 0.0, 18.0]) >= 0.99);
}

/// Glaze 0 é a lei da sessão AO BIT — a porta que todo pincel sem Glaze atravessa.
#[test]
fn glaze_zero_e_a_lei_da_sessao() {
    assert_eq!(COBRE.com_vela(0.0), COBRE);
    assert_eq!(Ecra::Porto.com_vela(1.0), Ecra::Porto);
    let (sc, uc) = ([23.0, 101.0, 179.0], [191.0, 0.0, 18.0]);
    assert_eq!(
        sobre_branco(COBRE.com_vela(0.0), sc, uc, 0.8, 0.9),
        sobre_branco(COBRE, sc, uc, 0.8, 0.9)
    );
}

/// O vermelho de Glaze 1 sobre o azul assentado mostra o PRODUTO das duas camadas (o marcador do Rebelle:
/// (20,0,15) contra (191,0,18)⊙(23,101,179)/255 = (17,0,13)); sem Glaze, o vermelho tapa.
#[test]
fn glaze_um_mostra_o_produto_das_camadas() {
    let (sc, uc) = ([23.0, 101.0, 179.0], [191.0, 0.0, 18.0]);
    let tapa = sobre_branco(COBRE, sc, uc, 1.0, 1.0);
    let filtra = sobre_branco(COBRE.com_vela(1.0), sc, uc, 1.0, 1.0);
    assert!(tapa[0] > 180.0, "sem Glaze o vermelho tapa: {tapa:?}");
    for k in 0..3 {
        let produto = sc[k] * uc[k] / 255.0;
        assert!(
            (filtra[k] - produto).abs() < 1.0,
            "canal {k}: {filtra:?} contra {produto:.1}"
        );
    }
}

/// O pior erro e quantos estados mudam o ecrã > 1,5 níveis ao assentar, na grelha de estados (cores, massas e a
/// massa que desce — as coberturas pela conta do motor).
fn erro_ao_assentar(vela: f32) -> (f64, usize, String) {
    let cores = [
        [23.0, 101.0, 179.0],
        [191.0, 0.0, 18.0],
        [245.0, 205.0, 25.0],
        [240.0, 120.0, 20.0],
        [40.0, 160.0, 60.0],
    ];
    let e = COBRE.com_vela(vela);
    let (mut pior, mut acima, mut onde) = (0.0f64, 0, String::new());
    for sc in cores {
        for uc in cores {
            for (ms, mf) in [
                (200.0, 400.0),
                (800.0, 2000.0),
                (3000.0, 600.0),
                (500.0, 5000.0),
            ] {
                for desce in [0.1, 0.3, 0.6] {
                    let dm = mf * desce;
                    let (a_s, a_f) = (alpha_of_mass(ms), alpha_of_mass(mf));
                    let (a_s2, a_f2) = (alpha_of_mass(ms + dm), alpha_of_mass(mf - dm));
                    let alvo = sobre_branco(e, sc, uc, a_s, a_f);
                    let novo = assenta(e, sc, uc, a_s, a_f, a_s2, a_f2);
                    let depois = sobre_branco(e, novo, uc, a_s2, a_f2);
                    let d = (0..3)
                        .map(|k| (depois[k] - alvo[k]).abs())
                        .fold(0.0, f64::max);
                    if d > 1.5 {
                        acima += 1;
                    }
                    if d > pior {
                        pior = d;
                        onde = format!("sc {sc:?} uc {uc:?} massas {ms}/{mf} desce {desce}");
                    }
                }
            }
        }
    }
    (pior, acima, onde)
}

/// ⭐ **Assentar não muda o que se vê — e o Glaze não o faz pior que a lei da obra** (dono 10/10). O CONTROLO é a
/// mesma grelha com Glaze 0 (a lei de hoje: 41 de 300 estados > 1,5, o pior 42,6 — o aberto (b) do 05d).
#[test]
fn assentar_guarda_o_ecra_com_o_glaze() {
    let (p0, n0, _) = erro_ao_assentar(0.0);
    for vela in [0.5, 1.0] {
        let (p, n, onde) = erro_ao_assentar(vela);
        assert!(
            p <= p0 && n <= n0,
            "Glaze {vela}: {n} estados > 1,5 e o pior {p:.2} ({onde}) — a lei da obra: {n0} e {p0:.2}"
        );
    }
}

#[test]
/// SONDA: existe uma cor de assentado que guarda o ecrã? (força bruta na grelha 3³ do RGB) — prompt 42.
#[ignore = "sonda: o alcance da inversa do Glaze"]
fn sonda_alcance() {
    let e = COBRE.com_vela(1.0);
    for (sc, uc, ms, mf, desce) in [
        (
            [245.0, 205.0, 25.0],
            [40.0, 160.0, 60.0],
            500.0,
            5000.0,
            0.6,
        ),
        ([23.0, 101.0, 179.0], [191.0, 0.0, 18.0], 500.0, 5000.0, 0.6),
        ([23.0, 101.0, 179.0], [191.0, 0.0, 18.0], 3000.0, 600.0, 0.3),
    ] {
        let dm = mf * desce;
        let (a_s, a_f) = (alpha_of_mass(ms), alpha_of_mass(mf));
        let (a_s2, a_f2) = (alpha_of_mass(ms + dm), alpha_of_mass(mf - dm));
        let alvo = sobre_branco(e, sc, uc, a_s, a_f);
        let novo = assenta(e, sc, uc, a_s, a_f, a_s2, a_f2);
        let mut melhor = (f64::MAX, [0.0; 3]);
        for r in (0..=255).step_by(3) {
            for g in (0..=255).step_by(3) {
                for b in (0..=255).step_by(3) {
                    let x = [r as f64, g as f64, b as f64];
                    let y = sobre_branco(e, x, uc, a_s2, a_f2);
                    let d = (0..3).map(|k| (y[k] - alvo[k]).abs()).fold(0.0, f64::max);
                    if d < melhor.0 {
                        melhor = (d, x);
                    }
                }
            }
        }
        let y = sobre_branco(e, novo, uc, a_s2, a_f2);
        println!(
            "alvo {alvo:?} · assenta {novo:?} → {y:?} · melhor da grelha {:.2} em {:?} · a {a_s:.3}/{a_f:.3} → {a_s2:.3}/{a_f2:.3}",
            melhor.0, melhor.1
        );
    }
}

/// ⭐ **Onde o filme escuro esconde um canal, assentar guarda o ecrã** (a inversa FECHADA do `assenta`): os
/// estados onde o Newton, sozinho, ficava no palpite — o verde sob o azul-noite a Glaze 0,5 e o amarelo sob o
/// azul a Glaze 1. Medido: com a inversa 0,009 e 0,000 níveis; sem ela 10,87 e 2,83.
#[test]
fn assentar_sob_o_filme_que_esconde_um_canal_guarda_o_ecra() {
    for (vela, sc, uc, ms, mf, desce) in [
        (
            0.5,
            [40.0, 160.0, 60.0],
            [20.0, 20.0, 60.0],
            3000.0,
            600.0,
            0.3,
        ),
        (
            1.0,
            [245.0, 205.0, 25.0],
            [23.0, 101.0, 179.0],
            1500.0,
            8000.0,
            0.6,
        ),
    ] {
        let e = COBRE.com_vela(vela);
        let dm = mf * desce;
        let (a_s, a_f) = (alpha_of_mass(ms), alpha_of_mass(mf));
        let (a_s2, a_f2) = (alpha_of_mass(ms + dm), alpha_of_mass(mf - dm));
        let alvo = sobre_branco(e, sc, uc, a_s, a_f);
        let depois = sobre_branco(e, assenta(e, sc, uc, a_s, a_f, a_s2, a_f2), uc, a_s2, a_f2);
        let d = (0..3)
            .map(|k| (depois[k] - alvo[k]).abs())
            .fold(0.0, f64::max);
        assert!(
            d < 1.0,
            "Glaze {vela}, sc {sc:?} sob uc {uc:?}: o ecrã mudou {d:.2} níveis ao assentar"
        );
    }
}

const AZUL: [f64; 3] = [23.0, 101.0, 179.0];
const VERMELHO: [f64; 3] = [191.0, 0.0, 18.0];

/// O motor de 200² sem arrasto (o pouso de cada célula lê só a própria célula).
fn motor() -> Engine {
    let mut e = Engine::new(200, 200);
    e.tuning.set(Knob::Drag, 0.0);
    e
}

/// Um pouso da tinta `cor` com o Glaze `glaze`: dabs de raio 20 em `(100, 100)` até a janela encher, e o
/// transfer. Devolve o grid de ANTES do transfer.
fn pouso(e: &mut Engine, cor: [f64; 3], glaze: f64) -> GridSnapshot {
    e.tuning.set(Knob::Glaze, glaze);
    let p = e.sim.gather_params(&e.tuning);
    let tex = e.bristle_texture_for_measure();
    let mut t = Trail::default();
    t.start_stroke(100.0, 100.0, cor, TrailMode::Paint);
    t.on_segment(12.0, 12.0);
    let g = e.active_grid_mut();
    let d = Dab {
        x: 100.0,
        y: 100.0,
        r: 20.0,
        hardness: 0.5,
        intensity: 40.0,
        water_amount: 0.5,
        dry_gate: 0.2,
        reach: 1.0,
        window_reach: 1.0,
        opacity: 1.0,
    };
    assert!(
        (0..8).any(|_| t.accumulate_paint(g, &p, &tex, &d, false).window_full),
        "a janela enche"
    );
    let antes = snapshot_grid(g);
    t.transfer_paint(g, &p);
    antes
}

/// ⭐ **A camada que desce leva a sua água** (doc 42 §6, o halo branco do Ink Brush): o vermelho de Glaze 0,5
/// sobre o azul molhado assenta a massa `dm`, e o filme da célula cai na MESMA proporção, `film × dm/susp` —
/// medido contra o mesmo pouso a Glaze 0 (que não assenta nada), célula a célula.
#[test]
fn a_camada_que_desce_leva_a_sua_agua() {
    let mut e = motor();
    pouso(&mut e, AZUL, 0.0);
    let molhado = snapshot_grid(e.active_grid());
    let mut corre = |glaze: f64| {
        restore_grid(e.active_grid_mut(), &molhado);
        let antes = pouso(&mut e, VERMELHO, glaze);
        (antes, snapshot_grid(e.active_grid()))
    };
    let (antes, sem) = corre(0.0);
    let (antes_com, com) = corre(0.5);
    assert_eq!(antes.film, antes_com.film, "o Glaze não muda o carimbo");
    let (mut n, mut pior) = (0, 0.0f64);
    for i in 0..antes.film.len() {
        let dm = f64::from(com.sett[i]) - f64::from(sem.sett[i]);
        if dm < 1.0 {
            continue;
        }
        let esperado = f64::from(antes.film[i]) * dm / f64::from(antes.susp[i]);
        let caiu = f64::from(sem.film[i]) - f64::from(com.film[i]);
        pior = pior.max((caiu - esperado).abs() / esperado);
        n += 1;
    }
    assert!(n > 50, "o pouso assentou em {n} células");
    assert!(
        pior < 1e-3,
        "o filme não caiu na proporção do que assentou: erro relativo {pior:.4} em {n} células"
    );
}

/// O azul pousado, e por cima o vermelho de Glaze 1 — que escreve a fracção que filtra (`vela`).
fn glaze_sobre_azul(e: &mut Engine, entre: impl FnOnce(&mut Engine)) -> GridSnapshot {
    pouso(e, AZUL, 0.0);
    entre(e);
    let antes = snapshot_grid(e.active_grid());
    pouso(e, VERMELHO, 1.0);
    let n = e.active_grid().vela.iter().filter(|v| **v > 0.0).count();
    assert!(n > 50, "o Glaze escreveu a vela em {n} células");
    antes
}

/// **Desfazer o Glaze devolve a vela** — o Ctrl+Z do motor (`capture_history`/`undo`) e o recorte regional
/// (`snapshot_grid_region`, o rascunho das figuras): sem ela a tinta de baixo continuava filtrada.
#[test]
fn desfazer_o_glaze_devolve_a_vela() {
    let mut e = motor();
    let antes = glaze_sobre_azul(&mut e, Engine::capture_history);
    assert!(e.undo());
    let g = e.active_grid();
    let n = g
        .vela
        .iter()
        .zip(&antes.vela)
        .filter(|(a, b)| a != b)
        .count();
    assert_eq!(n, 0, "undo: {n} células com a vela do Glaze desfeito");
    assert_eq!(g.sett, antes.sett);

    let mut e = motor();
    let mut recorte = None;
    let antes = glaze_sobre_azul(&mut e, |e| {
        recorte = snapshot_grid_region(e.active_grid(), 50, 50, 150, 150);
    });
    restore_grid_region(e.active_grid_mut(), &recorte.expect("o recorte"));
    let g = e.active_grid();
    let n = g
        .vela
        .iter()
        .zip(&antes.vela)
        .filter(|(a, b)| a != b)
        .count();
    assert_eq!(n, 0, "recorte: {n} células com a vela do Glaze desfeito");
}

/// **SONDA (smoke do dono 17/10): a linha clara na orla** — um filme FINO da mesma tinta sobre um assentado escuro
/// de Glaze (a célula da orla de um traço novo sobre várias camadas: `sonda_a_celula_da_borda`) secando aos poucos:
/// o ecrã a cada passe do assentar, e o melhor da força bruta (existe uma cor que o guarda?).
#[test]
#[ignore = "sonda: a linha clara na orla (smoke 17/10)"]
fn sonda_a_linha_clara() {
    for (vela, sc, ms, mf) in [
        (0.535f32, [166.0, 34.0, 34.0], 3307.0, 348.0),
        (0.5, [190.0, 55.0, 55.0], 1962.0, 4533.0),
        (0.5, [166.0, 34.0, 34.0], 3307.0, 1500.0),
        (0.5, [166.0, 34.0, 34.0], 3307.0, 100.0),
    ] {
        let uc = [190.0, 55.0, 55.0];
        let e = COBRE.com_vela(vela);
        let alvo = sobre_branco(e, sc, uc, alpha_of_mass(ms), alpha_of_mass(mf));
        let (mut s, mut m_s, mut m_f) = (sc, ms, mf);
        let mut linha = format!(
            "vela {vela} sc {sc:?} {ms}/{mf}: alvo {:?}",
            alvo.map(f64::round)
        );
        for passo in 0..80 {
            let dm = if m_f < 1.0 { m_f } else { m_f * 0.08 };
            let (a_s, a_f) = (alpha_of_mass(m_s), alpha_of_mass(m_f));
            let (a_s2, a_f2) = (alpha_of_mass(m_s + dm), alpha_of_mass(m_f - dm));
            s = assenta(e, s, uc, a_s, a_f, a_s2, a_f2);
            m_s += dm;
            m_f -= dm;
            if passo % 10 == 9 || m_f <= 0.0 {
                let y = sobre_branco(e, s, uc, alpha_of_mass(m_s), alpha_of_mass(m_f));
                linha += &format!(" | {passo}: f {m_f:.0} {:?}", y.map(f64::round));
            }
            if m_f <= 0.0 {
                break;
            }
        }
        println!("{linha}");
    }
}

/// **SONDA: a catraca da orla** — um assentado escuro de Glaze que recebe, a cada ciclo, um pouco da MESMA tinta
/// (`entra`, o escoamento do traço vizinho, sem guardar o ecrã) e a assenta toda (guardando-o): o ecrã ao fim de N
/// ciclos. Uma lei monótona só escurece (ou fica).
#[test]
#[ignore = "sonda: a catraca da orla (smoke 17/10)"]
fn sonda_a_catraca_da_orla() {
    let uc = [190.0, 55.0, 55.0];
    for (vela, sc, ms, entra) in [
        (0.535f32, [166.0, 34.0, 34.0], 3307.0, 100.0),
        (0.535, [166.0, 34.0, 34.0], 3307.0, 30.0),
        (0.5, [190.0, 55.0, 55.0], 3307.0, 100.0),
        (0.5, [150.0, 28.0, 28.0], 6000.0, 100.0),
        (0.535, [166.0, 34.0, 34.0], 900.0, 100.0),
    ] {
        let e = COBRE.com_vela(vela);
        let inicio = sobre_branco(e, sc, uc, alpha_of_mass(ms), 0.0);
        let (mut s, mut m_s) = (sc, ms);
        let mut linha = format!(
            "vela {vela} sc {sc:?} m {ms} entra {entra}: {:?}",
            inicio.map(f64::round)
        );
        for ciclo in 1..=40 {
            let (a_s, a_f) = (alpha_of_mass(m_s), alpha_of_mass(entra));
            let molhado = sobre_branco(e, s, uc, a_s, a_f);
            s = assenta(e, s, uc, a_s, a_f, alpha_of_mass(m_s + entra), 0.0);
            m_s += entra;
            if ciclo % 10 == 0 || ciclo == 1 {
                linha += &format!(
                    " | {ciclo}: molhado {:?} seco {:?}",
                    molhado.map(f64::round),
                    sobre_branco(e, s, uc, alpha_of_mass(m_s), 0.0).map(f64::round)
                );
            }
        }
        println!("{linha}");
    }
}

/// ⭐ **A mesma tinta que ENTRA não clareia o Glaze** (smoke do dono 17/10, doc 42 §8): um assentado escuro de Glaze
/// (o vermelho sobre si) que recebe, ciclo a ciclo, um filme fino da mesma tinta e o assenta — o escoamento na orla
/// de um traço — só escurece até o ponto fixo das camadas. Com o tapar extra da tinta diluída em branco a contar o
/// produto como OUTRA tinta, o verde subia 34 → 52 (a linha esbranquiçada).
#[test]
fn a_mesma_tinta_que_entra_nao_clareia_o_glaze() {
    let uc = [190.0, 55.0, 55.0];
    let e = COBRE.com_vela(0.535);
    let (mut s, mut m_s) = ([166.0, 34.0, 34.0], 3307.0);
    let mut antes = sobre_branco(e, s, uc, alpha_of_mass(m_s), 0.0);
    for ciclo in 0..40 {
        let a_s = alpha_of_mass(m_s);
        s = assenta(
            e,
            s,
            uc,
            a_s,
            alpha_of_mass(100.0),
            alpha_of_mass(m_s + 100.0),
            0.0,
        );
        m_s += 100.0;
        let agora = sobre_branco(e, s, uc, alpha_of_mass(m_s), 0.0);
        for k in 0..3 {
            assert!(
                agora[k] <= antes[k] + 0.5,
                "ciclo {ciclo}, canal {k}: {antes:?} → {agora:?}"
            );
        }
        antes = agora;
    }
}
