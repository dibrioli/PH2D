//! Os gates do **LEQUE do radial** (F4) — o filho que se desenha é o que se escolhe, uma família
//! nunca executa, e nenhum rótulo tapa outro.

use super::*;
use crate::widget::radial_menu::label_rect;

const C: [f32; 2] = [800.0, 600.0];

fn leaf(i: u64) -> RadialItem {
    RadialItem::leaf(format!("L{i}"), NodeId(5000 + i))
}

/// Um anel de `n` sectores em que o sector `fam` é uma família de `k` filhos.
fn ring(n: usize, fam: usize, k: usize) -> RadialOpen {
    let items = (0..n)
        .map(|i| {
            if i == fam {
                RadialItem::family("F", (0..k as u64).map(|j| leaf(100 + j)).collect())
            } else {
                leaf(i as u64)
            }
        })
        .collect();
    RadialOpen {
        center: C,
        items,
        hot: None,
        fan: None,
    }
}

/// Onde o filho `j` assenta, no ecrã.
fn child_at(open: &RadialOpen, fam: usize, j: usize, k: usize) -> [f32; 2] {
    let fc = fan_center(open, fam);
    let o = fan_offset(j, k, outward(fam, open.items.len()));
    [fc[0] + o[0], fc[1] + o[1]]
}

/// ⭐ **O FILHO QUE SE DESENHA É O QUE SE ESCOLHE** — em toda contagem do anel, toda família, todo
/// tamanho de leque, e com o ponteiro a CHEGAR pelo caminho (do centro ao pai, do pai ao filho).
///
/// ⚠️ O caminho conta: os filhos das pontas assentam além da fronteira do sector vizinho, e um leque
/// que fugisse para o vizinho deixá-los-ia inalcançáveis — o gate atravessa essa fronteira.
#[test]
fn every_child_is_chosen_where_it_is_drawn() {
    for n in 2..=MAX_SECTORS {
        for fam in 0..n {
            for k in 1..=FAN_MAX {
                for j in 0..k {
                    let mut open = ring(n, fam, k);
                    let fc0 = fan_center(&open, fam);
                    point(&mut open, fc0);
                    let target = child_at(&open, fam, j, k);
                    for t in 1..=8u8 {
                        let f = f32::from(t) / 8.0;
                        let fc = fan_center(&open, fam);
                        let p = [
                            fc[0] + (target[0] - fc[0]) * f,
                            fc[1] + (target[1] - fc[1]) * f,
                        ];
                        point(&mut open, p);
                    }
                    let got = chosen(&open).map(|c| c.id);
                    assert_eq!(
                        got,
                        Some(NodeId(5100 + j as u64)),
                        "n={n} família={fam} k={k}: o filho {j} desenhado ali não é o escolhido"
                    );
                }
            }
        }
    }
}

/// ⛔ **UMA FAMÍLIA NUNCA EXECUTA** — soltar sobre ela antes de o leque abrir, ou na zona morta
/// do leque, cancela. ⚠️ Com o CONTROLO: a folha vizinha, no mesmo raio, escolhe-se.
#[test]
fn a_family_is_never_a_command() {
    let mut open = ring(7, 2, 4);
    let o = item_offset(2, 7);
    point(&mut open, [C[0] + o[0] * 0.6, C[1] + o[1] * 0.6]);
    assert_eq!(open.hot, Some(2), "a família não acendeu");
    assert_eq!(chosen(&open), None, "soltar na família executou alguma coisa");
    let fc = fan_center(&open, 2);
    point(&mut open, fc);
    assert!(open.fan.is_some(), "atravessar o anel não abriu o leque");
    assert_eq!(chosen(&open), None, "a zona morta do leque executou alguma coisa");
    let o = item_offset(1, 7);
    point(&mut open, [C[0] + o[0] * 0.6, C[1] + o[1] * 0.6]);
    assert_eq!(chosen(&open).map(|c| c.id), Some(NodeId(5001)), "o controlo");
}

/// **VOLTAR PARA DENTRO DO ANEL FECHA O LEQUE** — é o gesto de desistir da família.
#[test]
fn coming_back_inside_the_ring_closes_the_fan() {
    let mut open = ring(7, 3, 3);
    let t = child_at(&open, 3, 0, 3);
    let fc = fan_center(&open, 3);
    point(&mut open, fc);
    point(&mut open, t);
    assert!(open.fan.is_some_and(|f| f.hot == Some(0)));
    point(&mut open, C);
    assert_eq!(open.fan, None, "o leque ficou aberto com o ponteiro no centro");
    assert_eq!(chosen(&open), None, "o centro escolheu um filho");
}

/// ⛔ **NENHUM RÓTULO DO LEQUE TAPA OUTRO, NEM O DO PAI** — em toda direcção de pai e todo tamanho.
///
/// ⚠️ É o gate do raio derivado: no raio do anel (96 px) os dois filhos do meio de um leque virado
/// para cima ficam lado a lado a 73 px com 92 de rótulo.
#[test]
fn no_fan_label_covers_another() {
    let hit = |a: crate::zones::Rect, b: crate::zones::Rect| {
        a.x < b.x + b.w && b.x < a.x + a.w && a.y < b.y + b.h && b.y < a.y + a.h
    };
    for n in 2..=MAX_SECTORS {
        for fam in 0..n {
            for k in 1..=FAN_MAX {
                let open = ring(n, fam, k);
                let mut rects = vec![label_rect(fan_center(&open, fam))];
                rects.extend((0..k).map(|j| label_rect(child_at(&open, fam, j, k))));
                for a in 0..rects.len() {
                    for b in a + 1..rects.len() {
                        assert!(
                            !hit(rects[a], rects[b]),
                            "n={n} família={fam} k={k}: o rótulo {a} tapa o {b}"
                        );
                    }
                }
            }
        }
    }
}
