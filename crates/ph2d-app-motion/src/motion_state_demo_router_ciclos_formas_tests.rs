//! ⭐⭐ **Toda cena de ciclo que ensina APARÊNCIA desenha FORMAS** (Bug #20; `CLAUDE.md` §0.8).
//!
//! Desde a lei do dono de 2026-09-19 (`72cdfa6f71`, `SinkStyle::so_com_forma`) uma corrente sem
//! forma desenha só os marcadores de ponto — e um `Tint`, uma `Color Ramp` ou um `Trail` sobre ela
//! pintam colunas que ninguém desenha. A `=118` (16/09) nasceu três dias antes da lei e ficou a
//! prometer laranja, arco-íris e rastos sobre cruzes azuis até 15/10.
//!
//! A régua é a do PRODUTO: cada cena coze pela porta do quadro (`o_quadro`, rota da CPU) e cada
//! sink cujo cone tem um cartão de aparência tem de sair com aparência ([`tem_aparencia`]: uma
//! geometria ou um ladrilho) — a mesma pergunta que a lei faz antes de desenhar.

use crate::motion_state::MotionState;
use ph2d_eval_motion::tem_aparencia;
use ph2d_nodegraph::graph::NodeId;

/// Os cartões que decidem como uma PEÇA se parece (o grupo do tutorial 7, menos o `Glow`, que é
/// um passe de tela e acende sem peças).
const APARENCIA: &[&str] = &[
    "motion.tint",
    "motion.color_ramp",
    "motion.color_array",
    "motion.trail",
    "motion.strobe",
    "motion.sub_uv",
    "fx.drop_shadow",
    "fx.rgb_split",
];

const QUADROS: u32 = 8;

/// `(sink, cartões de aparência no cone, desenha formas?)` de cada sink da cena `n`.
fn sinks_da_cena(n: &str) -> Vec<(NodeId, Vec<String>, bool)> {
    let mut m = MotionState::new();
    let _ = super::super::build_level(Some(n), &mut m.doc, &m.registry);
    let mut playhead = ph2d_core::Playhead::new(1.0 / 60.0);
    playhead.play();
    for _ in 0..QUADROS {
        playhead.advance_ticks(1);
        crate::motion_bridge::quadro::o_quadro(
            &mut m,
            None,
            &playhead,
            1.0 / 60.0,
            crate::motion_bridge::quadro::MaoDoEditor::default(),
        );
    }
    m.sinks
        .clone()
        .into_iter()
        .map(|s| {
            let g = &m.doc.graph;
            let efeitos: Vec<String> = ph2d_nodegraph::cook::upstream_cone(g, s)
                .into_iter()
                .filter_map(|k| g.node(k).map(|i| i.type_name.clone()))
                .filter(|t| APARENCIA.contains(&t.as_str()))
                .collect();
            let desenha = m
                .pump
                .cook
                .peek(s)
                .and_then(|v| v.first())
                .is_some_and(|v| tem_aparencia(v.as_stream()));
            (s, efeitos, desenha)
        })
        .collect()
}

/// ⭐⭐ **O GATE.** ⚠️ Piso de população: pelo menos 10 sinks com aparência em 6 cenas (medido
/// 15/10: `=118`, `=133`–`=136`, `=138`, `=140`, `=142`) — `0 pedidos` lê-se igual a `0 defeitos`.
/// ⚠️ Controlo: a `=111` (o pivô, sem cartão de aparência) não desenha forma nenhuma — a régua
/// distingue um sink de pontos de um de formas.
#[test]
fn toda_cena_de_ciclo_que_pinta_aparencia_desenha_formas() {
    let mut mudos = Vec::new();
    let (mut sinks_com_efeito, mut cenas_com_efeito) = (0usize, 0usize);
    for n in super::CICLOS {
        let sinks = sinks_da_cena(n);
        assert!(!sinks.is_empty(), "a cena ={n} monta e tem sinks");
        let com: Vec<_> = sinks.iter().filter(|(_, e, _)| !e.is_empty()).collect();
        if !com.is_empty() {
            cenas_com_efeito += 1;
        }
        for (s, efeitos, desenha) in com {
            sinks_com_efeito += 1;
            if !desenha {
                mudos.push(format!("={n} sink {s:?} {efeitos:?}"));
            }
        }
    }
    assert!(
        mudos.is_empty(),
        "sinks de cena de ciclo com cartao de aparencia que desenham so' MARCADORES (sem forma): \
         {mudos:#?}"
    );
    assert!(
        sinks_com_efeito >= 10 && cenas_com_efeito >= 6,
        "piso: so' {sinks_com_efeito} sink(s) com aparencia em {cenas_com_efeito} cena(s) -- a \
         regua varre o vazio"
    );
    let controlo = sinks_da_cena("111");
    assert!(
        controlo.iter().all(|(_, e, d)| e.is_empty() && !d),
        "controlo: a =111 e' so' pontos e nao pinta aparencia: {controlo:?}"
    );
}

/// A fonte dos anúncios das cenas `=111`–`=120` (as de `=130` em diante anunciam no ficheiro de
/// cada uma, e entram no gate acima pelo grafo).
const ANUNCIOS: &str = include_str!("motion_state_demo_announce.rs");

/// As cenas cujo ANÚNCIO promete cor — o texto do `eprintln!` dela (do marcador `[cena N]` ao fim
/// da chamada), nunca o ficheiro inteiro, que tem os doc-comments das vizinhas.
fn anuncios_que_prometem_cor() -> Vec<String> {
    let mut fora = Vec::new();
    for n in super::CICLOS {
        let marca = format!("[cena {n}]");
        let Some(i) = ANUNCIOS.find(&marca) else {
            continue;
        };
        let resto = &ANUNCIOS[i..];
        let bloco = resto.find("\n    );").map_or(resto, |j| &resto[..j]);
        // ⚠️ Sem «rasto»: o `arrasto` da `=114` casava.
        if [
            "cor ",
            "COR ",
            "cores",
            "arco-iris",
            "laranja",
            "cauda",
            "CAUDA",
        ]
        .iter()
        .any(|w| bloco.contains(w))
        {
            fora.push((*n).to_string());
        }
    }
    fora
}

/// ⭐⭐ **A metade do TEXTO: toda cena cujo anúncio promete cor desenha formas em TODOS os sinks.**
/// ⚠️ Piso: a `=118` é uma delas — senão a extracção partiu-se e o gate varre o vazio.
#[test]
fn todo_anuncio_que_promete_cor_tem_formas_em_todo_pano() {
    let cenas = anuncios_que_prometem_cor();
    assert!(
        cenas.iter().any(|n| n == "118"),
        "piso: a =118 promete cor e a extraccao nao a viu: {cenas:?}"
    );
    for n in &cenas {
        let sinks = sinks_da_cena(n);
        let sem: Vec<_> = sinks.iter().filter(|(_, _, d)| !d).collect();
        assert!(
            sem.is_empty(),
            "o anuncio da ={n} promete cor e {} de {} pano(s) desenham so' marcadores: {sem:?}",
            sem.len(),
            sinks.len()
        );
    }
}

/// **SONDA — a família inteira:** por cena, quantos sinks, quantos desenham forma, e os cartões de
/// aparência de cada um.
#[test]
#[ignore = "sonda: corra à mão"]
fn sonda_formas_das_cenas_de_ciclo() {
    for n in super::CICLOS {
        let sinks = sinks_da_cena(n);
        let formas = sinks.iter().filter(|(_, _, d)| *d).count();
        let efeitos: Vec<_> = sinks
            .iter()
            .filter(|(_, e, _)| !e.is_empty())
            .map(|(_, e, d)| format!("{}{e:?}", if *d { "" } else { "MUDO " }))
            .collect();
        eprintln!(
            "={n}: {} sinks, {formas} com forma · {efeitos:?}",
            sinks.len()
        );
    }
}
