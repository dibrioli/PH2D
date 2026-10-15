//! ⭐⭐ **A CENA ABRE COM TUDO O QUE PINTA DENTRO DO CANVAS VISTO, COM MARGEM** — em todas as cenas
//! do roteador que pintam alguma coisa.
//!
//! A régua é a das fichas que tapam ([`super::tapa`]): o que a bomba BAIXA em todo tique de
//! `2 · CICLO` s (sprites, formas, imagens, o gizmo de posições), as fichas com a largura que o
//! produto lhes mede, o canvas visto da foto de 1930×1040 sob o split da Motion, e a margem do
//! *Enquadrar tudo* (`float_bar::frame_margin`, 44 px). Cada cena é medida e:
//!
//! - um **CAMPO** (a figura em `t = 0` já contém o canvas visto da vista de omissão) abre na
//!   omissão, não tem ficha e não está na tabela;
//! - as outras têm a linha delas em [`EXTENSAO`](crate::motion_demo_enquadra::EXTENSAO) igual à
//!   medida (a figura do ciclo e a do arranque), e a vista que a lei tira dela põe dentro, com a
//!   margem, a figura que ela enquadra (a do ciclo, ou a do arranque quando o ciclo FOGE) e as fichas.
//!
//! ⚠️ O traço do gizmo (meio contorno + casing, ~3 px) entra no mundo à escala da omissão: numa vista
//! mais afastada ele vale menos do que isso no ecrã, coberto pela margem.

use super::tapa::{self, Caixa};
use crate::motion_demo_enquadra as e;
use crate::motion_state::MotionState;
use ph2d_render::Camera2d;

/// ⛔ **As cenas que a régua NÃO mede**, cada uma com o porquê medido. O gate confere que montam e
/// que não estão na tabela (abrem na omissão).
const SEM_REGUA: &[(&str, &str)] = &[(
    "11",
    "o Voronoi de 32 768 células custa 60,8 s POR TIQUE na CPU em debug (sonda de 15/10): os 481 \
     tiques não cabem num gate. Medido num tique: x ±12,5 · y ±7,5 m ⊇ o canvas visto — um campo",
)];

/// ⛔ **As cenas cuja figura não cabe na faixa do zoom** (`ZOOM_MAX_HEIGHT_WORLD`) — abrem centradas
/// nela, no zoom máximo. Re-medidas: se passarem a caber, reprovam.
const SEM_CABER: &[(&str, &str)] = &[
    (
        "15",
        "o caleidoscópio já nasce em x −351…−169 · y ±91 m: 181 m de altura",
    ),
    (
        "16",
        "o organismo já nasce em x −331…−189 · y −91…181 m: 271 m de altura",
    ),
];

/// Diferença tolerada entre a tabela e a medição, m.
const TOL_M: f32 = 1e-3;

/// O que se mediu numa cena.
struct Medida {
    /// A figura do ciclo e a do arranque.
    figura: Option<e::Extensao>,
    campo: bool,
    fundo: usize,
    fichas: Vec<e::Ficha>,
}

fn contem(a: Caixa, b: Caixa) -> bool {
    a[0] <= b[0] && a[1] <= b[1] && a[2] >= b[2] && a[3] >= b[3]
}

fn uniao(a: Option<Caixa>, p: Caixa) -> Caixa {
    a.map_or(p, |b| {
        [
            b[0].min(p[0]),
            b[1].min(p[1]),
            b[2].max(p[2]),
            b[3].max(p[3]),
        ]
    })
}

#[expect(clippy::cast_precision_loss, reason = "px de janela")]
fn k_de(h: f32) -> f32 {
    tapa::janela_da_cena().height as f32 / h
}

/// Mede a cena `level` — `None` quando ela não monta sinks (não pinta nada).
fn mede(level: &str, ts: &mut ph2d_text::TextSystem) -> Option<Medida> {
    let mut m = MotionState::new();
    let (sinks, caps) = super::monta(level, &mut m.doc, &m.registry);
    if sinks.is_empty() {
        return None;
    }
    m.sinks = sinks;
    let fichas = e::fichas(&super::no_mundo_de(&caps, &m), ts);
    let omissao = Camera2d::default().view();
    let k0 = k_de(omissao.height_world);
    let vista0 = tapa::camara(omissao).1;
    let scopes = ph2d_nodegraph::cook::TimeScopes::new();
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "segundos inteiros"
    )]
    let tiques = (2.0 * crate::motion_state::ritmo_demo::CICLO * 60.0) as u64;
    let (mut md, mut ciclo, mut arranque) = (
        Medida {
            figura: None,
            campo: false,
            fundo: 0,
            fichas,
        },
        None,
        None,
    );
    let mut pecas = Vec::new();
    for t in 0..=tiques {
        #[expect(clippy::cast_precision_loss, reason = "um tique")]
        crate::motion_externals::publish_all(&mut m, t as f64 / 60.0);
        let sinks = m.sinks.clone();
        let _ = m.pump.advance_or_scrub_scoped(
            &m.doc.graph,
            &m.registry,
            &sinks,
            t,
            |t| t as f64 / 60.0,
            m.default_uv_rect,
            m.default_size,
            &scopes,
        );
        tapa::figura(&m, k0, &mut pecas);
        for p in &pecas {
            if e::e_fundo(*p) {
                md.fundo += 1;
            } else {
                ciclo = Some(uniao(ciclo, *p));
            }
        }
        if t == 0 {
            arranque = ciclo;
            // ⭐ Um campo em `t = 0` é campo no ciclo inteiro (a união só cresce): pára aqui.
            if ciclo.is_some_and(|f| contem(f, vista0)) {
                md.campo = true;
                break;
            }
        }
    }
    md.figura = ciclo.map(|c| e::Extensao {
        ciclo: c,
        arranque: arranque.unwrap_or(c),
    });
    Some(md)
}

/// Cabe o que a cena enquadra, com as fichas, na vista que a lei tira dela?
fn cabe_na_lei(x: e::Extensao, fichas: &[e::Ficha]) -> bool {
    let (area, janela, margem) = (tapa::area(), tapa::janela_da_cena(), tapa::margem());
    let v = e::vista_de(Some(x), fichas, area, janela, margem);
    let f = e::a_enquadrar(x, fichas, area, janela, margem);
    e::cabe(
        e::caixa(f, fichas, k_de(v.height_world)),
        v,
        area,
        janela,
        margem,
    ) && v.height_world >= Camera2d::default().height_world
}

/// A altura da vista do ciclo sobre a do arranque (com o canvas visto da omissão).
fn razao(x: e::Extensao, fichas: &[e::Ficha]) -> f32 {
    let (area, janela, margem) = (tapa::area(), tapa::janela_da_cena(), tapa::margem());
    let so_arranque = e::Extensao {
        ciclo: x.arranque,
        ..x
    };
    let h = |f| e::vista(f, fichas, area, janela, margem).height_world;
    h(x.ciclo) / h(e::a_enquadrar(so_arranque, fichas, area, janela, margem))
}

/// O veredito de uma cena medida contra a linha `tabela` dela — `None` = passa.
fn veredito(level: &str, md: &Medida, tabela: Option<e::Extensao>) -> Option<String> {
    let Some(f) = md.figura else {
        return tabela.map(|_| format!("={level} está na tabela e não pinta nada"));
    };
    if md.campo {
        if !md.fichas.is_empty() {
            return Some(format!(
                "={level} é um CAMPO (a figura em t = 0 contém o canvas visto) e tem {} ficha(s): \
                 a omissão não as garante à vista",
                md.fichas.len()
            ));
        }
        return tabela.map(|_| format!("={level} é um CAMPO e não pode estar na tabela"));
    }
    let Some(t) = tabela else {
        return Some(format!(
            "={level} não está na EXTENSAO — medido: (\"{level}\", {f:?}) (sonda \
             a_tabela_das_extensoes)"
        ));
    };
    let longe = |a: Caixa, b: Caixa| (0..4).any(|i| (a[i] - b[i]).abs() > TOL_M);
    if longe(t.ciclo, f.ciclo) || longe(t.arranque, f.arranque) {
        return Some(format!(
            "={level}: a EXTENSAO diz {t:?} e a cena pinta {f:?} — re-corra a sonda \
             a_tabela_das_extensoes"
        ));
    }
    let isenta = SEM_CABER.iter().any(|(n, _)| *n == level);
    match (cabe_na_lei(f, &md.fichas), isenta) {
        (true, true) => Some(format!(
            "={level} está em SEM_CABER e já abre inteira — retire-a"
        )),
        (false, false) => Some(format!(
            "={level} não abre inteira: {f:?} com {} ficha(s) (o zoom máximo é {} m)",
            md.fichas.len(),
            Camera2d::ZOOM_MAX_HEIGHT_WORLD
        )),
        _ => None,
    }
}

/// O que um bloco de cenas deu.
#[derive(Default)]
struct Resumo {
    medidas: usize,
    com_fichas: usize,
    campos: usize,
    maus: Vec<String>,
}

fn confere(niveis: std::ops::RangeInclusive<u32>) -> Resumo {
    let mut ts = ph2d_text::TextSystem::without_system_fonts();
    let mut r = Resumo::default();
    for n in niveis {
        let level = n.to_string();
        let tabela = e::extensao(Some(&level));
        if let Some((_, porque)) = SEM_REGUA.iter().find(|(l, _)| *l == level) {
            let mut m = MotionState::new();
            if super::monta(&level, &mut m.doc, &m.registry).0.is_empty() || tabela.is_some() {
                r.maus.push(format!(
                    "={level} está em SEM_REGUA ({porque}) e já não monta ou está na tabela"
                ));
            }
            continue;
        }
        let Some(md) = mede(&level, &mut ts) else {
            if tabela.is_some() {
                r.maus.push(format!("={level} está na tabela e não monta"));
            }
            continue;
        };
        if let Some(m) = veredito(&level, &md, tabela) {
            r.maus.push(m);
        }
        if md.campo {
            r.campos += 1;
        } else if md.figura.is_some() {
            r.medidas += 1;
            r.com_fichas += usize::from(!md.fichas.is_empty());
        }
    }
    r
}

fn reprova_se(r: &Resumo, faixa: &str, piso: (usize, usize, usize)) {
    assert!(
        r.medidas >= piso.0 && r.com_fichas >= piso.1 && r.campos >= piso.2,
        "{faixa}: {} cenas medidas ({} com fichas) e {} campos contra o piso {piso:?} — ou as \
         cenas mudaram, ou a montagem partiu-se",
        r.medidas,
        r.com_fichas,
        r.campos
    );
    assert!(
        r.maus.is_empty(),
        "{faixa}: {} cena(s) não abrem inteiras (motion_demo_enquadra):\n  {}",
        r.maus.len(),
        r.maus.join("\n  ")
    );
}

// ⭐⭐⭐ **OS GATES** — o roteador em blocos, para correrem em paralelo. Pisos medidos a 15/10
// (cenas medidas, com fichas, campos): 131 medidas (42 com fichas), 10 campos e a `=11` sem régua.

#[test]
fn a_cena_abre_inteira_1_a_40() {
    reprova_se(&confere(1..=40), "=1..=40", (30, 0, 9));
}

#[test]
fn a_cena_abre_inteira_41_a_99() {
    reprova_se(&confere(41..=99), "=41..=99", (59, 17, 0));
}

#[test]
fn a_cena_abre_inteira_100_a_max() {
    let max = crate::motion_state::demo_router::MAX_DEMO_LEVEL;
    reprova_se(&confere(100..=max), "=100..", (42, 25, 1));
}

/// ⭐ **CONTROLO POSITIVO**: a `=82` com a câmara de 10 m (a de antes) tem as linhas de cima fora do
/// canvas visto; a vista da lei põe-nas dentro. E a tabela adulterada (meio metro a mais em cima,
/// ou sem a linha) reprova.
#[test]
fn a_82_na_camara_de_10_m_reprova_e_a_tabela_velha_tambem() {
    let mut ts = ph2d_text::TextSystem::without_system_fonts();
    let md = mede("82", &mut ts).expect("a =82 monta");
    let f = md.figura.expect("a =82 pinta");
    let (area, janela, margem) = (tapa::area(), tapa::janela_da_cena(), tapa::margem());
    let omissao = Camera2d::default().view();
    assert!(
        !e::cabe(
            e::caixa(f.ciclo, &md.fichas, k_de(omissao.height_world)),
            omissao,
            area,
            janela,
            margem
        ),
        "na câmara de 10 m a =82 tinha as linhas 1–2 sob a barra (foto de 15/10)"
    );
    assert!(
        cabe_na_lei(f, &md.fichas),
        "a vista da lei põe a =82 inteira"
    );
    let t = e::extensao(Some("82")).expect("a =82 está na tabela");
    assert_eq!(veredito("82", &md, Some(t)), None, "a tabela certa passa");
    let mut velha = t;
    velha.ciclo[3] += 0.5;
    assert!(
        veredito("82", &md, Some(velha)).is_some(),
        "meio metro a mais reprova"
    );
    assert!(veredito("82", &md, None).is_some(), "sem a linha reprova");
}

/// ⭐ **CONTROLO da fuga**: a `=50` (a corda solta cai) com o ciclo INTEIRO abria a ~79 m, a figura
/// num canto de um canvas vazio (foto de 15/10); a lei abre-a no arranque. E a `=139` (a galáxia
/// que CRESCE com os gatilhos) fica no ciclo.
#[test]
fn o_que_foge_nao_afasta_a_vista_e_o_que_cresce_sim() {
    let mut ts = ph2d_text::TextSystem::without_system_fonts();
    let (area, janela, margem) = (tapa::area(), tapa::janela_da_cena(), tapa::margem());
    let md = mede("50", &mut ts).expect("a =50 monta");
    let x = md.figura.expect("a =50 pinta");
    assert!(razao(x, &md.fichas) > e::FOGE_ACIMA_DE, "a =50 foge");
    assert_ne!(
        e::a_enquadrar(x, &md.fichas, area, janela, margem),
        x.ciclo,
        "abre no arranque"
    );
    let md = mede("139", &mut ts).expect("a =139 monta");
    let x = md.figura.expect("a =139 pinta");
    assert!(
        razao(x, &md.fichas) <= e::FOGE_ACIMA_DE,
        "a =139 cresce no sítio"
    );
    assert_eq!(
        e::a_enquadrar(x, &md.fichas, area, janela, margem),
        x.ciclo,
        "abre no ciclo"
    );
}

/// ⭐ **A SONDA** — mede cada cena e imprime a tabela `EXTENSAO` (Rust), os campos e o tempo.
///
/// `cargo test -p ph2d-app-motion --lib a_tabela_das_extensoes -- --ignored --nocapture`
/// (`PH2D_LAYOUT_LEVEL=<n>` mede uma cena só).
#[test]
#[ignore = "sonda: imprime a tabela, não afirma"]
fn a_tabela_das_extensoes() {
    let only = std::env::var("PH2D_LAYOUT_LEVEL").ok();
    let mut ts = ph2d_text::TextSystem::without_system_fonts();
    let (area, janela, margem) = (tapa::area(), tapa::janela_da_cena(), tapa::margem());
    for n in 1..=crate::motion_state::demo_router::MAX_DEMO_LEVEL {
        let level = n.to_string();
        if only.as_deref().is_some_and(|w| w != level) || SEM_REGUA.iter().any(|(l, _)| *l == level)
        {
            continue;
        }
        let t0 = std::time::Instant::now();
        let Some(md) = mede(&level, &mut ts) else {
            continue;
        };
        let ms = t0.elapsed().as_secs_f64() * 1e3;
        match md.figura {
            Some(f) if !md.campo => {
                let v = e::vista_de(Some(f), &md.fichas, area, janela, margem);
                println!(
                    "    (\"{level}\", {:?}, {:?}), // {} fichas · {} fundo · razão {:.2} · abre a \
                     {:.2} m em ({:.2}, {:.2}){} · {ms:.0} ms",
                    f.ciclo,
                    f.arranque,
                    md.fichas.len(),
                    md.fundo,
                    razao(f, &md.fichas),
                    v.height_world,
                    v.center[0],
                    v.center[1],
                    if cabe_na_lei(f, &md.fichas) {
                        ""
                    } else {
                        " · NÃO CABE"
                    }
                );
            }
            Some(f) => println!(
                "    // ={level} CAMPO {f:?} · {} fichas · {ms:.0} ms",
                md.fichas.len()
            ),
            None => println!("    // ={level} não pinta nada · {ms:.0} ms"),
        }
    }
}
