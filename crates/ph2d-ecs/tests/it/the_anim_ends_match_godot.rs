//! **A régua das pontas:** Once e Ping-pong do nosso motor contra o Godot 4.7.2 (MIT), corrido sem
//! interface. Fixtures, comando e leitura: `tests/fixtures/anim_godot/README.md`.
//!
//! O par comparável é `AnimatedSprite2D` (`SpriteFrames.LOOP_NONE` / `LOOP_PINGPONG`): lista de quadros
//! com uma duração. O `AnimationPlayer` só entra em ordem e período, pela razão escrita no README.
//!
//! Os `#[ignore]` são divergências MEDIDAS e recusadas: a duração do nosso quadro não depende do passo do relógio.

use std::collections::BTreeMap;
use std::fs;

use ph2d_ecs::{AnimDirection, AnimationTag, SpriteAnimator, advance};

const FIXTURES: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/anim_godot");
const D100_25: &str = "d100_passo25ms.csv";
const D110_25: &str = "d110_passo25ms.csv";
const D100_12: &str = "d100_passo12_5ms.csv";

#[derive(Clone, Copy, PartialEq)]
enum Modo {
    Once,
    PingPong,
}

struct Linha {
    quadro: u32,
    tocando: bool,
    fim: i64,
}

struct Corrida {
    casos: BTreeMap<String, Vec<Linha>>,
    dur_ms: u32,
    dt_us: u64,
}

impl Corrida {
    fn caso(&self, nome: &str) -> &[Linha] {
        self.casos
            .get(nome)
            .unwrap_or_else(|| panic!("caso {nome} fora da fixture"))
    }

    fn quadros(&self, nome: &str) -> Vec<u32> {
        self.caso(nome).iter().map(|l| l.quadro).collect()
    }
}

fn valor(linha: &str, chave: &str) -> f64 {
    let bruto = linha
        .split_whitespace()
        .find_map(|p| p.strip_prefix(chave))
        .unwrap_or_else(|| panic!("{chave} fora de: {linha}"));
    bruto.parse().unwrap()
}

/// Lê a fixture e confere o cabeçalho: uma fixture regerada noutra versão ou noutro relógio reprova.
fn ler(ficheiro: &str, dur_ms: u32, dt_us: u64) -> Corrida {
    let texto = fs::read_to_string(format!("{FIXTURES}/{ficheiro}")).unwrap();
    assert!(
        texto.contains("# oraculo=Godot 4.7.2"),
        "{ficheiro}: versão"
    );
    assert!(texto.contains("# data=2026-11-07"), "{ficheiro}: data");
    assert!(
        texto.contains(&format!("--fixed-fps {} ", 1_000_000 / dt_us)),
        "{ficheiro}: comando"
    );
    let mut casos: BTreeMap<String, Vec<Linha>> = BTreeMap::new();
    for l in texto.lines() {
        if let Some(c) = l.strip_prefix("# dur_ms=") {
            let lido: f64 = c.split_whitespace().next().unwrap().parse().unwrap();
            assert_eq!(lido.round() as u32, dur_ms, "{ficheiro}: dur_ms");
        } else if l.starts_with("# delta_min=") {
            let esperado = dt_us as f64 / 1e6;
            for chave in ["delta_min=", "delta_max="] {
                let d = valor(l, chave);
                assert!((d - esperado).abs() < 1e-12, "{ficheiro}: relógio {d}");
            }
        }
        let p: Vec<&str> = l.split(',').collect();
        if p.len() == 7 && p[0] != "caso" {
            let linhas = casos.entry(p[0].to_owned()).or_default();
            assert_eq!(p[2].parse::<usize>().unwrap(), linhas.len(), "{l}");
            linhas.push(Linha {
                quadro: p[3].parse().unwrap(),
                tocando: p[5] == "1",
                fim: p[6].parse().unwrap(),
            });
        }
    }
    assert_eq!(casos.len(), 26, "{ficheiro}: casos");
    Corrida {
        casos,
        dur_ms,
        dt_us,
    }
}

/// O nosso motor, com o que o segmentado do cartão escreve em Once e Ping-pong (`AnimCycleMode`).
/// Devolve o quadro a cada tique, o tique em que acabou (se acabou) e se ainda toca no fim.
fn nosso(
    modo: Modo,
    n: u32,
    dur_ms: u32,
    dt_us: u64,
    ticks: usize,
) -> (Vec<u32>, Option<usize>, bool) {
    let mut tag = AnimationTag::new("a", 0, n - 1);
    tag.frame_ms = dur_ms;
    (tag.direction, tag.repeat) = match modo {
        Modo::Once => (AnimDirection::Forward, Some(1)),
        Modo::PingPong => (AnimDirection::PingPong, None),
    };
    let mut a = SpriteAnimator::new("a");
    a.playing = true;
    let mut f = 0;
    let mut seq = vec![f];
    let mut fim = None;
    for k in 1..ticks {
        if advance(&mut a, &tag, &mut f, n, dt_us).finished {
            fim = Some(k);
        }
        seq.push(f);
    }
    (seq, fim, a.playing)
}

/// Os quadros, sem as repetições: só a ORDEM em que aparecem.
fn ordem(seq: &[u32]) -> Vec<u32> {
    let mut o = seq.to_vec();
    o.dedup();
    o
}

/// O primeiro passo em que as duas sequências diferem, escrito para o relatório.
fn diferenca(nome: &str, dt_us: u64, nos: &[u32], godot: &[u32]) -> Option<String> {
    assert_eq!(nos.len(), godot.len());
    let k = nos.iter().zip(godot).position(|(a, b)| a != b)?;
    let total = nos.iter().zip(godot).filter(|(a, b)| a != b).count();
    Some(format!(
        "{nome}: tique {k} (t = {} us): nosso {} × Godot {}; {total} de {} tiques diferem\n nosso {}\n Godot {}",
        k as u64 * dt_us,
        nos[k],
        godot[k],
        nos.len(),
        corridas(nos),
        corridas(godot)
    ))
}

/// `0x4 1x4 …`: cada quadro e quantos tiques ficou.
fn corridas(seq: &[u32]) -> String {
    let mut s: Vec<(u32, u32)> = Vec::new();
    for q in seq {
        match s.last_mut() {
            Some((u, c)) if u == q => *c += 1,
            _ => s.push((*q, 1)),
        }
    }
    let partes: Vec<String> = s.iter().map(|(q, c)| format!("{q}x{c}")).collect();
    partes.join(" ")
}

fn fim_godot(linhas: &[Linha]) -> Option<usize> {
    usize::try_from(linhas.last().unwrap().fim).ok()
}

/// Sem empate no relógio (110 ms / 25 ms), Once casa em todos os tiques, no fim e no quadro parado.
#[test]
fn once_without_a_tie_matches_step_by_step() {
    let c = ler(D110_25, 110, 25_000);
    for n in [2, 4] {
        let nome = format!("as2d_once_n{n}");
        let godot = c.caso(&nome);
        let g: Vec<u32> = godot.iter().map(|l| l.quadro).collect();
        let (nos, fim, toca) = nosso(Modo::Once, n, c.dur_ms, c.dt_us, g.len());
        assert_eq!(diferenca(&nome, c.dt_us, &nos, &g), None);
        assert_eq!(fim, fim_godot(godot), "{nome}: tique do fim");
        assert_eq!(toca, godot.last().unwrap().tocando, "{nome}: parou");
        assert_eq!(*nos.last().unwrap(), n - 1, "{nome}: pára no último");
    }
}

/// Once pára no ÚLTIMO quadro nos dois, em todas as grades e nos dois nós.
#[test]
fn once_stops_on_the_last_frame() {
    for c in [
        ler(D100_25, 100, 25_000),
        ler(D110_25, 110, 25_000),
        ler(D100_12, 100, 12_500),
    ] {
        for n in [1u32, 2, 4] {
            for nome in [
                format!("as2d_once_n{n}"),
                format!("ap_once_n{n}_len{n}_disc"),
            ] {
                let g = c.caso(&nome).last().unwrap();
                assert_eq!(g.quadro, n - 1, "{nome}");
                let (nos, ..) = nosso(Modo::Once, n, c.dur_ms, c.dt_us, c.quadros(&nome).len());
                assert_eq!(*nos.last().unwrap(), g.quadro, "{nome}");
            }
        }
    }
}

/// Ping-pong: a mesma ORDEM de quadros: a ponta aparece uma vez, sem repetir ao virar.
#[test]
fn pingpong_visits_the_frames_in_the_same_order() {
    for c in [
        ler(D100_25, 100, 25_000),
        ler(D110_25, 110, 25_000),
        ler(D100_12, 100, 12_500),
    ] {
        for n in [1u32, 2, 4] {
            let nome = format!("as2d_pingpong_n{n}");
            let g = ordem(&c.quadros(&nome));
            // 8 quadros bastam para duas voltas de n = 4 e não dependem do relógio de cada um.
            let corte = g.len().min(8);
            let (nos, ..) = nosso(Modo::PingPong, n, c.dur_ms, c.dt_us, c.quadros(&nome).len());
            assert_eq!(ordem(&nos)[..corte], g[..corte], "{nome}");
        }
    }
    let g = ordem(&ler(D100_25, 100, 25_000).quadros("as2d_pingpong_n4"));
    assert_eq!(g[..9], [0, 1, 2, 3, 2, 1, 0, 1, 2]);
}

/// O `AnimationPlayer` só casa em ordem e período com `length = (n-1)·d` e trilha discreta; os outros
/// dois arranjos estão na fixture e mostram porque a equivalência não é automática.
#[test]
fn animation_player_needs_a_chosen_authoring_to_match() {
    let c = ler(D100_25, 100, 25_000);
    let g = c.quadros("ap_pingpong_n4_len3_disc");
    let (nos, ..) = nosso(Modo::PingPong, 4, c.dur_ms, c.dt_us, g.len());
    assert_eq!(ordem(&g)[..9], ordem(&nos)[..9]);
    for k in 5..g.len() - 24 {
        assert_eq!(g[k], g[k + 24], "o período do Godot é 24 tiques em {k}");
        assert_eq!(nos[k], nos[k + 24], "o nosso período é 24 tiques em {k}");
    }
    // `length = n·d`: a ponta 3 fica 300 ms. Trilha contínua: a ponta 3 pisca um tique só.
    let duracao_do_3 = |s: &[u32]| s.iter().take(30).filter(|q| **q == 3).count();
    assert_eq!(duracao_do_3(&c.quadros("ap_pingpong_n4_len4_disc")), 12);
    assert_eq!(duracao_do_3(&c.quadros("ap_pingpong_n4_len3_cont")), 1);
}

/// Empate: `t` cai exatamente na troca. O Godot troca só no tique seguinte; nós trocamos nele.
#[test]
#[ignore = "decidido 07/11, não copiar: no empate (100 ms / 25 ms) o Godot troca 1 tique depois; tique 4 nosso 1 × Godot 0, fim 16 × 17"]
fn once_on_a_tie_matches_step_by_step() {
    let c = ler(D100_25, 100, 25_000);
    let godot = c.caso("as2d_once_n4");
    let g = c.quadros("as2d_once_n4");
    let (nos, fim, _) = nosso(Modo::Once, 4, c.dur_ms, c.dt_us, g.len());
    assert_eq!(diferenca("once n4", c.dt_us, &nos, &g), None);
    assert_eq!(fim, fim_godot(godot));
}

/// A virada do Ping-pong: a ponta dura `d` em nós; no Godot dura mais, e o quanto depende do relógio.
#[test]
#[ignore = "decidido 07/11, não copiar: a 25 ms a ponta do Godot dura 150 ms, e encolhe com o passo; o nosso dura 100 ms em qualquer passo"]
fn pingpong_matches_step_by_step() {
    let mut difs = Vec::new();
    for (ficheiro, dur, dt) in [
        (D100_25, 100, 25_000),
        (D110_25, 110, 25_000),
        (D100_12, 100, 12_500),
    ] {
        let c = ler(ficheiro, dur, dt);
        for n in [2, 4] {
            let nome = format!("as2d_pingpong_n{n}");
            let g = c.quadros(&nome);
            let (nos, ..) = nosso(Modo::PingPong, n, dur, dt, g.len());
            difs.extend(diferenca(&format!("{ficheiro} {nome}"), dt, &nos, &g));
        }
    }
    assert!(difs.is_empty(), "{}", difs.join("\n"));
}

/// Um quadro só: o Godot avisa o fim passado `d`; o nosso nunca acaba e continua «a tocar».
#[test]
fn once_of_a_single_frame_finishes() {
    let c = ler(D110_25, 110, 25_000);
    let godot = c.caso("as2d_once_n1");
    let (_, fim, toca) = nosso(Modo::Once, 1, c.dur_ms, c.dt_us, godot.len());
    assert_eq!(fim, fim_godot(godot), "tique do fim");
    assert_eq!(toca, godot.last().unwrap().tocando, "parou");
}
