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

/// ⭐ **As cenas que FOGEM — a decisão** (abrem no arranque, [`e::FOGE_ACIMA_DE`]). O gate confere
/// que a razão da lei as põe deste lado e as outras do outro, em cada uma de [`JANELAS`]: uma cena
/// que muda de modo de abrir reprova até alguém a pôr ou tirar daqui.
const FOGEM: &[&str] = &[
    "3", "5", "38", "50", "52", "71", "74", "95", "103", "113", "138",
];

/// As janelas em que a classe se confere: a foto do smoke, o monitor do dono e dois extremos comuns.
/// O canvas visto em cada uma é o da foto com os painéis de largura fixa e o grafo sob o split
/// ([`area_em`]). Medido a 15/10: entre elas a razão da `=138` anda 0,6 % e a da `=128` nada, mas a
/// da `=139` anda 9 % e a da `=3` 38 % — a classe tem de valer em todas.
const JANELAS: [(u32, u32); 4] = [(1930, 1040), (2560, 1080), (2560, 1440), (1366, 768)];

/// ⭐ **A folga mínima ao corte** (relativa, em qualquer de [`JANELAS`]). Não é um recurso: é o
/// fio-de-tropeço. Medido a 15/10: a mais perto é a `=138`, 4,1 % acima (a 1366×768), e a `=128`
/// 3,9 % abaixo; 3 % reprova a cena que chegar mais perto do que qualquer uma de hoje antes de o vão
/// fechar.
const PERTO_DO_CORTE: f32 = 0.03;

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
    mede_com(level, ts, |_, _| {})
}

/// [`mede`], com `olha(tique, peças sem o fundo)` em cada tique.
fn mede_com(
    level: &str,
    ts: &mut ph2d_text::TextSystem,
    mut olha: impl FnMut(u64, &[Caixa]),
) -> Option<Medida> {
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
        pecas.retain(|p| !e::e_fundo(*p));
        olha(t, &pecas);
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

/// A razão da fuga da lei ([`e::razao_da_fuga`]) na janela da foto.
fn razao(x: e::Extensao, fichas: &[e::Ficha]) -> f32 {
    let (area, janela, margem) = (tapa::area(), tapa::janela_da_cena(), tapa::margem());
    e::razao_da_fuga(x, fichas, area, janela, margem)
}

/// A janela da cena e o canvas visto numa janela `w × h`: o da foto, com o Hierarchy, o Inspector e a
/// barra de cima de largura fixa e o fundo do canvas a seguir ao split. Na foto dá o canvas da foto.
#[expect(clippy::cast_precision_loss, reason = "px de janela")]
fn area_em(w: u32, h: u32) -> (ph2d_render::camera_frame::Area, ph2d_host::WindowSize) {
    let split = ph2d_editor_core::screens::layout::CenterSplit::Horizontal {
        t: ph2d_editor_core::screens::layout::CenterSplit::T_DEFAULT,
    };
    let (j0, a0) = (tapa::janela_da_cena(), tapa::area());
    let (dir, baixo) = (
        j0.width as f32 - (a0.x + a0.w),
        j0.height as f32 - (a0.y + a0.h),
    );
    let j = crate::field_gizmo::scene_camera_window(split, ph2d_host::WindowSize::new(w, h));
    let area = ph2d_render::camera_frame::Area {
        w: j.width as f32 - dir - a0.x,
        h: j.height as f32 - baixo - a0.y,
        ..a0
    };
    (area, j)
}

/// A razão da fuga da cena em cada uma de [`JANELAS`].
fn razoes(x: e::Extensao, fichas: &[e::Ficha]) -> [f32; 4] {
    JANELAS.map(|(w, h)| {
        let (area, janela) = area_em(w, h);
        e::razao_da_fuga(x, fichas, area, janela, tapa::margem())
    })
}

/// O veredito da classe da cena `level` com as razões `r` (uma por janela) — `None` = passa.
fn veredito_da_fuga(level: &str, r: &[f32; 4]) -> Option<String> {
    let decidida = FOGEM.contains(&level);
    let corte = e::FOGE_ACIMA_DE;
    JANELAS.iter().zip(r).find_map(|(&(w, h), &r)| {
        if (r > corte) != decidida {
            Some(format!(
                "={level} a {w}×{h}: a razão da fuga é {r:.4} e o corte {corte} — a cena {} FOGEM, \
                 e o modo de abrir mudou: decida (a lista FOGEM) e re-meça o vão (sonda \
                 candidatos_da_fuga)",
                if decidida {
                    "está em"
                } else {
                    "não está em"
                }
            ))
        } else if (r / corte).ln().abs() < PERTO_DO_CORTE.ln_1p() {
            Some(format!(
                "={level} a {w}×{h}: a razão da fuga {r:.4} está a menos de {} % do corte {corte} \
                 — o vão fechou: re-meça-o (sonda candidatos_da_fuga) antes de mover o corte",
                PERTO_DO_CORTE * 100.0
            ))
        } else {
            None
        }
    })
}

/// A razão que este gate media até 15/10 — sobre o arranque SEM o canvas (só a sonda a imprime).
fn razao_sem_canvas(x: e::Extensao, fichas: &[e::Ficha]) -> f32 {
    let (area, janela, margem) = (tapa::area(), tapa::janela_da_cena(), tapa::margem());
    let h = |f| e::vista(f, fichas, area, janela, margem).height_world;
    h(x.ciclo) / h(x.arranque)
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
    if let Some(m) = veredito_da_fuga(level, &razoes(f, &md.fichas)) {
        return Some(m);
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
/// num canto de um canvas vazio (foto de 15/10), e a `=138` (os discos a cair, a que FOGE mais perto
/// do corte) abrem no arranque; a `=139` (a galáxia que CRESCE com os gatilhos) e a `=128` (a que
/// cresce mais perto do corte) ficam no ciclo — em todas as [`JANELAS`]. E a régua da classe reprova
/// a cena do lado errado e a cena colada ao corte.
#[test]
fn o_que_foge_nao_afasta_a_vista_e_o_que_cresce_sim() {
    let mut ts = ph2d_text::TextSystem::without_system_fonts();
    for (level, foge) in [("50", true), ("138", true), ("139", false), ("128", false)] {
        let md = mede(level, &mut ts).expect("monta");
        let x = md.figura.expect("pinta");
        let r = razoes(x, &md.fichas);
        assert_eq!(FOGEM.contains(&level), foge, "={level} na lista");
        assert_eq!(veredito_da_fuga(level, &r), None, "={level}: {r:?}");
        for (&(w, h), r) in JANELAS.iter().zip(r) {
            let (area, janela) = area_em(w, h);
            let f = e::a_enquadrar(x, &md.fichas, area, janela, tapa::margem());
            assert_eq!(r > e::FOGE_ACIMA_DE, foge, "={level} a {w}×{h}: {r}");
            assert_eq!(f == x.ciclo, !foge, "={level} a {w}×{h} abre no ciclo?");
        }
    }
    // A régua: a cena do outro lado do corte, ou colada a ele, reprova.
    let c = e::FOGE_ACIMA_DE;
    assert!(
        veredito_da_fuga("139", &[c * 1.2; 4]).is_some(),
        "cresce e passou a fugir"
    );
    assert!(
        veredito_da_fuga("138", &[c / 1.2; 4]).is_some(),
        "fugia e passou a crescer"
    );
    let perto = 1.0 + 0.5 * PERTO_DO_CORTE;
    assert!(veredito_da_fuga("138", &[c * 1.2, c * 1.2, c * perto, c * 1.2]).is_some());
    assert!(
        veredito_da_fuga("128", &[c / perto; 4]).is_some(),
        "colada por baixo"
    );
    let o = 1.0 + 2.0 * PERTO_DO_CORTE;
    assert_eq!(veredito_da_fuga("138", &[c * o; 4]), None, "longe passa");
    assert_eq!(veredito_da_fuga("128", &[c / o; 4]), None, "longe passa");
}

/// SONDA: a razão da fuga de cada cena da tabela em cada uma de [`JANELAS`] (só a tabela e as
/// fichas: não corre o ciclo).
/// `cargo test -p ph2d-app-motion --lib a_razao_noutras_janelas -- --ignored --nocapture`
#[test]
#[ignore = "sonda: imprime razões"]
fn a_razao_noutras_janelas() {
    let mut ts = ph2d_text::TextSystem::without_system_fonts();
    println!("JANELAS {JANELAS:?} · corte {}", e::FOGE_ACIMA_DE);
    for &(level, ciclo, arranque) in e::EXTENSAO {
        let mut m = MotionState::new();
        let (sinks, caps) = super::monta(level, &mut m.doc, &m.registry);
        m.sinks = sinks;
        let fichas = e::fichas(&super::no_mundo_de(&caps, &m), &mut ts);
        let r = razoes(e::Extensao { ciclo, arranque }, &fichas);
        println!("JAN,{level},{r:?}");
    }
}

/// ⭐ **SONDA dos candidatos a discriminador da fuga** (a tabela do [`e::FOGE_ACIMA_DE`]) — uma linha
/// `CAND` por cena da tabela: cena, razão da lei, crescimento do alcance no 2.º ciclo ÷ no 1.º,
/// alcance no fim ÷ no meio, instante dos 95 % do alcance, a caixa em `t = CICLO` contra a de
/// `t = 0`, peças que batem o recorde do 1.º ciclo, peças fora da união do 1.º, alcance em
/// `0`/`CICLO`/`2·CICLO`, peças, preenchimento (mediana, máximo, de área), altura da vista do ciclo e
/// do arranque, razão sem o canvas. Uns 2 min em três fatias:
/// `PH2D_LAYOUT_DE=a PH2D_LAYOUT_ATE=b cargo test … candidatos_da_fuga -- --ignored --nocapture`
#[test]
#[ignore = "sonda: imprime candidatos"]
#[expect(clippy::cast_precision_loss, reason = "contagens")]
fn candidatos_da_fuga() {
    let de: u32 = std::env::var("PH2D_LAYOUT_DE").map_or(1, |s| s.parse().unwrap());
    let ate: u32 = std::env::var("PH2D_LAYOUT_ATE")
        .map_or(crate::motion_state::demo_router::MAX_DEMO_LEVEL, |s| {
            s.parse().unwrap()
        });
    let mut ts = ph2d_text::TextSystem::without_system_fonts();
    let centro = |b: &Caixa| [0.5 * (b[0] + b[2]), 0.5 * (b[1] + b[3])];
    let longe = |c: [f32; 2], b: &Caixa| {
        let dx = (b[0] - c[0]).abs().max((b[2] - c[0]).abs());
        let dy = (b[1] - c[1]).abs().max((b[3] - c[1]).abs());
        dx.hypot(dy)
    };
    for n in de..=ate {
        let level = n.to_string();
        let Some(tab) = e::extensao(Some(&level)) else {
            continue;
        };
        let c0 = centro(&tab.arranque);
        // Por tique: o alcance (peça mais longe do centro do arranque) e as distâncias por índice.
        let mut alc: Vec<f32> = Vec::new();
        let mut dist: Vec<Vec<f32>> = Vec::new();
        let mut caixa: Vec<Option<Caixa>> = Vec::new();
        let mut fim: Vec<Caixa> = Vec::new();
        let Some(md) = mede_com(&level, &mut ts, |_, p| {
            fim.clear();
            fim.extend_from_slice(p);
            let d: Vec<f32> = p.iter().map(|b| longe(c0, b)).collect();
            alc.push(d.iter().copied().fold(0.0, f32::max));
            caixa.push(p.iter().fold(None, |a, b| Some(uniao(a, *b))));
            dist.push(d);
        }) else {
            continue;
        };
        let Some(x) = md.figura else { continue };
        let t_fim = alc.len() - 1;
        let t_c = t_fim / 2;
        let rmax = |t: usize| alc[..=t].iter().copied().fold(0.0, f32::max);
        let (r0, rc, rt) = (rmax(0), rmax(t_c), rmax(t_fim));
        let g2 = (rt - rc) / (rc - r0).max(1e-3);
        let fim_meio = alc[t_fim] / alc[t_c].max(1e-3);
        let sat =
            (0..=t_fim).find(|&t| rmax(t) >= 0.95 * rt).unwrap_or(t_fim) as f32 / t_fim as f32;
        // Volta ao arranque ao fim de um CICLO? (a caixa do instante C contra a de t = 0)
        let per = match (caixa[0], caixa[t_c]) {
            (Some(a), Some(b)) => {
                let s = (a[2] - a[0]).max(a[3] - a[1]).max(1e-3);
                (0..4).map(|i| (a[i] - b[i]).abs()).fold(0.0, f32::max) / s
            }
            _ => f32::NAN,
        };
        // Fracção das peças do fim que batem, no 2.º ciclo, o recorde delas do 1.º.
        let (mut foge, mut vivas) = (0usize, 0usize);
        for (i, d) in dist[t_fim].iter().enumerate() {
            let rec = dist[..=t_c]
                .iter()
                .filter_map(|v| v.get(i))
                .copied()
                .fold(f32::NAN, f32::max);
            if rec.is_nan() {
                continue;
            }
            vivas += 1;
            foge += usize::from(*d > rec * 1.001 + 1e-4);
        }
        let f_rec = foge as f32 / vivas.max(1) as f32;
        // Fracção das peças do fim FORA da união do 1.º ciclo.
        let u_c = caixa[..=t_c]
            .iter()
            .flatten()
            .fold(None, |a, b| Some(uniao(a, *b)));
        let f_fora = u_c.map_or(f32::NAN, |u| {
            let fora = fim
                .iter()
                .filter(|b| {
                    let c = centro(b);
                    c[0] < u[0] || c[0] > u[2] || c[1] < u[1] || c[1] > u[3]
                })
                .count();
            fora as f32 / fim.len().max(1) as f32
        });
        // Quanto da figura do ciclo o instante PREENCHE (lado maior), mediana e máximo sobre os tiques.
        let lado = |b: &Caixa| (b[2] - b[0]).max(b[3] - b[1]);
        let mut fill: Vec<f32> = caixa
            .iter()
            .map(|b| b.map_or(0.0, |b| lado(&b) / lado(&x.ciclo).max(1e-3)))
            .collect();
        let fill_max = fill.iter().copied().fold(0.0, f32::max);
        fill.sort_by(f32::total_cmp);
        let fill_med = fill[fill.len() / 2];
        let area = |b: &Caixa| (b[2] - b[0]).max(1e-3) * (b[3] - b[1]).max(1e-3);
        let afill_max = caixa
            .iter()
            .flatten()
            .map(|b| area(b) / area(&x.ciclo))
            .fold(0.0, f32::max);
        let (ar, ja, mg) = (tapa::area(), tapa::janela_da_cena(), tapa::margem());
        let hc = e::vista(x.ciclo, &md.fichas, ar, ja, mg).height_world;
        let hs = e::vista(x.arranque, &md.fichas, ar, ja, mg).height_world;
        println!(
            "CAND,{level},{:.4},{g2:.4},{fim_meio:.4},{sat:.4},{per:.4},{f_rec:.4},{f_fora:.4},{r0:.3},{rc:.3},{rt:.3},{},{fill_med:.4},{fill_max:.4},{afill_max:.4},{hc:.3},{hs:.3},{:.4}",
            razao(x, &md.fichas),
            dist[t_fim].len(),
            razao_sem_canvas(x, &md.fichas)
        );
    }
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
