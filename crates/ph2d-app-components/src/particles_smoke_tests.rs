//! O que as duas cenas do emissor ENSINAM — e as três coisas que as tornariam mentira.

use ph2d_ecs::{Name, ParticleEmitter, ParticleSpace, Timers, Transform, World};
use ph2d_physics_ecs::ProjectileMotion;
use ph2d_render::Sprite;

use super::{CENAS, montar};

fn mundo(nivel: u32) -> World {
    let mut w = World::new();
    let m = montar(&mut w, nivel);
    assert_eq!(m.nivel, nivel);
    // ⚠️ **A cena abre com uma fonte ESCOLHIDA** — sem isso o painel está vazio quando o dono lê a
    // instrução que lhe manda ver a secção.
    assert_ne!(m.escolhido, 0, "a cena ={nivel} não escolheu fonte nenhuma");
    w
}

/// Os emissores da cena, com nome, pose e tamanho da sprite.
fn fontes(w: &mut World) -> Vec<(String, [f32; 2], [f32; 2])> {
    w.query::<(&Name, &ParticleEmitter, &Transform, &Sprite)>()
        .iter(w)
        .map(|(n, _, t, s)| {
            (
                n.0.clone(),
                [t.translation.x, t.translation.y],
                [s.size[0], s.size[1]],
            )
        })
        .collect()
}

/// ⚠️ **O `CENAS` é CONTADO do `match`** — um roteador que promete um nível que não existe manda o
/// dono a uma cena vazia.
#[test]
fn o_roteador_serve_exactamente_as_cenas_que_declara() {
    assert_eq!(CENAS, 2);
    for n in 1..=CENAS {
        let mut w = mundo(n);
        assert!(
            !fontes(&mut w).is_empty(),
            "a cena ={n} não montou emissor nenhum"
        );
    }
}

/// ⭐⭐⭐ **Cada fonte é um CORPO visível** — o pick só devolve quem emite uma sprite, e uma fonte
/// sem corpo é impossível de escolher: o roteiro manda clicar nela e a secção nunca aparece.
///
/// **Mutação que deve sangrar:** tirar a `Sprite` do `fonte`.
#[test]
fn quem_emite_tem_corpo() {
    for n in 1..=CENAS {
        let mut w = mundo(n);
        let com_corpo = fontes(&mut w).len();
        let total = w.query::<&ParticleEmitter>().iter(&w).count();
        assert_eq!(com_corpo, total, "cena ={n}: um emissor sem corpo");
    }
}

/// ⭐⭐ **As placas ENCOSTAM sem se sobrepor** — entre arquétipos diferentes a ordem de iteração do
/// pick é indefinida por escrito, logo duas caixas sobrepostas tornam o clique um sorteio.
#[test]
fn nenhuma_fonte_tapa_outra() {
    for n in 1..=CENAS {
        let mut w = mundo(n);
        let f = fontes(&mut w);
        for (i, a) in f.iter().enumerate() {
            for b in f.iter().skip(i + 1) {
                let dx = (a.1[0] - b.1[0]).abs() - (a.2[0] + b.2[0]) * 0.5;
                let dy = (a.1[1] - b.1[1]).abs() - (a.2[1] + b.2[1]) * 0.5;
                assert!(
                    dx >= 0.0 || dy >= 0.0,
                    "cena ={n}: «{}» e «{}» sobrepõem-se",
                    a.0,
                    b.0
                );
            }
        }
    }
}

/// ⭐⭐⭐ **As quatro da `=1` partilham o que NÃO está em causa** — é isso que torna cada coluna
/// legível (a régua que o #14 pagou): a quantidade e a vida são as MESMAS nas quatro, e a rapidez
/// é a mesma nas três que têm gravidade.
///
/// ⚠️ **O anel é a excepção NOMEADA, e ela saiu de uma foto:** sem gravidade uma partícula viaja
/// `v × vida` e não volta, logo à rapidez das outras ele saía do ecrã.
///
/// **Mutação que deve sangrar:** dar outra `amount` a uma das fontes, ou rapidez própria a uma das
/// três com gravidade.
#[test]
fn as_quatro_da_um_partilham_o_que_nao_esta_em_causa() {
    let mut w = mundo(1);
    let cfgs: Vec<ParticleEmitter> = w.query::<&ParticleEmitter>().iter(&w).cloned().collect();
    assert_eq!(cfgs.len(), 4);
    let primeiro = &cfgs[0];
    for c in &cfgs {
        assert_eq!(c.amount, primeiro.amount, "a quantidade não é a mesma");
        assert!(
            (c.life - primeiro.life).abs() < 1e-6,
            "a vida não é a mesma"
        );
    }
    let com_peso: Vec<&ParticleEmitter> = cfgs.iter().filter(|c| c.gravity[1] != 0.0).collect();
    assert_eq!(com_peso.len(), 3, "só o anel é que não tem gravidade");
    for c in &com_peso {
        assert!(
            (c.speed - com_peso[0].speed).abs() < 1e-6,
            "a rapidez das que têm gravidade não é a mesma"
        );
    }
}

/// ⭐⭐ **Tudo o que nasce CABE na banda que o dono vê** — a régua saiu da foto (≈ `11 × 6 m` com os
/// painéis abertos), e o raio de uma partícula é `velocidade × vida` (ou o topo do arco, com peso).
///
/// ⚠️ **Uma fonte que VIAJA mede-se só no ponto de partida** — os dois da `=2` atravessam a cena de
/// propósito, e exigir-lhes que o penacho nunca saia do ecrã seria exigir que eles não andassem.
///
/// **Mutação que deve sangrar:** pôr a fila de volta a `±7,5 m`, dar ao anel a rapidez das outras,
/// ou pôr a rajada a `180°` — as três coisas que a foto apanhou e nenhum outro gate via.
#[test]
fn o_que_nasce_cabe_no_ecra() {
    // ⚠️ Metades MEDIDAS na régua do canvas, com margem: a cena nunca as pode encher toda.
    const MEIA_LARGURA: f32 = 5.2;
    const MEIO_ALTO: f32 = 2.8;
    for n in 1..=CENAS {
        let mut w = mundo(n);
        let cenas: Vec<([f32; 2], ParticleEmitter, bool)> = w
            .query::<(&Transform, &ParticleEmitter, Option<&ProjectileMotion>)>()
            .iter(&w)
            .map(|(t, c, voa)| ([t.translation.x, t.translation.y], c.clone(), voa.is_some()))
            .collect();
        for (p, c, voa) in cenas {
            assert!(
                p[0].abs() <= MEIA_LARGURA && p[1].abs() <= MEIO_ALTO,
                "cena ={n}: uma fonte nasce fora do ecrã"
            );
            if voa {
                continue;
            }
            // ⚠️ **De LADO uma partícula só anda o seno da abertura** — medir o alcance inteiro
            // acusaria um jacto estreito de sair pelo lado quando ele vai todo para cima. ⛔ E
            // acima dos `90°` o seno DESCE: a `180°` ele dá zero, que leria uma esfera como um
            // fio. A partir dali a fracção é `1`.
            let fracao = if c.spread >= 90.0 {
                1.0
            } else {
                c.spread.to_radians().sin()
            };
            let alcance = c.speed * c.life;
            // O topo de um arco é `v²/2g` quando há peso; sem peso, o alcance inteiro.
            let alto = if c.gravity[1] < 0.0 {
                c.speed * c.speed / (2.0 * -c.gravity[1])
            } else {
                alcance
            };
            assert!(
                p[0].abs() + alcance * fracao <= MEIA_LARGURA,
                "cena ={n}: uma partícula sai pelo lado ({:.2} m)",
                p[0].abs() + alcance * fracao
            );
            assert!(
                p[1] + alto <= MEIO_ALTO,
                "cena ={n}: uma partícula sai por cima ({:.2} m)",
                p[1] + alto
            );
        }
    }
}

/// ⭐⭐⭐ **Nenhuma coluna invade a vizinha** — a galeria só se lê se cada knob ficar na coluna dele.
///
/// ⚠️⚠️ **Esta lei nasceu de uma MUTAÇÃO SOBREVIVENTE:** devolver ao anel a rapidez das outras
/// deixava o gate do enquadramento VERDE (ele cabia na banda) e punha os penachos uns por cima dos
/// outros. *Caber no ecrã e ser legível são duas grandezas, e só uma delas estava medida.*
///
/// **Mutação que deve sangrar:** dar ao anel a rapidez das outras, ou abrir a rajada a `45°`.
#[test]
fn cada_coluna_fica_na_coluna_dela() {
    let mut w = mundo(1);
    let fontes: Vec<(f32, f32)> = w
        .query::<(&Transform, &ParticleEmitter)>()
        .iter(&w)
        .map(|(t, c)| {
            let fracao = if c.spread >= 90.0 {
                1.0
            } else {
                c.spread.to_radians().sin()
            };
            (t.translation.x, c.speed * c.life * fracao)
        })
        .collect();
    for (i, a) in fontes.iter().enumerate() {
        for b in fontes.iter().skip(i + 1) {
            assert!(
                (a.0 - b.0).abs() >= a.1 + b.1,
                "duas colunas tocam-se: {:.2} m de vão contra {:.2} m de penacho",
                (a.0 - b.0).abs(),
                a.1 + b.1
            );
        }
    }
}

/// ⭐⭐⭐ **A rajada nasce DESLIGADA e o sinal que a acende é o que o relógio grita** — as duas
/// pontas do fio, e é a única coisa da cena que liga o emissor ao resto da casa.
///
/// **Mutação que deve sangrar:** deixar a rajada a emitir de partida, ou trocar um dos dois nomes.
#[test]
fn a_rajada_nasce_desligada_e_o_relogio_e_que_a_acende() {
    let mut w = mundo(1);
    let rajada = w
        .query::<(&Name, &ParticleEmitter)>()
        .iter(&w)
        .find(|(n, _)| n.0 == "Burst")
        .map(|(_, c)| c.clone())
        .expect("a cena tem uma rajada");
    assert!(!rajada.emitting, "a rajada não pode nascer a emitir");
    assert!(rajada.one_shot);
    assert!(!rajada.restart_on.is_empty());
    let gritos: Vec<String> = w
        .query::<&Timers>()
        .iter(&w)
        .flat_map(|t| t.0.iter().map(|x| x.signal.clone()))
        .collect();
    assert!(
        gritos.contains(&rajada.restart_on),
        "ninguém grita «{}» — a rajada nunca dispara",
        rajada.restart_on
    );
}

/// ⭐⭐⭐ **Na `=2` a ÚNICA diferença é o espaço** — sem isso o artista não distingue *«o espaço
/// funciona»* de *«é assim que partículas são»*.
///
/// **Mutação que deve sangrar:** dar outra cor, outra vida ou outra velocidade a um dos dois.
#[test]
fn na_dois_a_unica_diferenca_e_o_espaco() {
    let mut w = mundo(2);
    let cfgs: Vec<ParticleEmitter> = w.query::<&ParticleEmitter>().iter(&w).cloned().collect();
    assert_eq!(cfgs.len(), 2);
    let (a, b) = (&cfgs[0], &cfgs[1]);
    assert_ne!(a.space, b.space, "os dois têm o mesmo espaço");
    let igualado = ParticleEmitter {
        space: a.space,
        ..b.clone()
    };
    assert_eq!(
        *a, igualado,
        "os dois diferem em mais do que o espaço — a cena deixa de ser um controlo"
    );
    assert!(cfgs.iter().any(|c| c.space == ParticleSpace::World));
    assert!(cfgs.iter().any(|c| c.space == ParticleSpace::Local));
}
