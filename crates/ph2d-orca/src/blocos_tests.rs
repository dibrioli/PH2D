//! (W11) As paredes por blocos = [`Walls::from_walkable_walls`] das paredes de todos os blocos em fila,
//! campo a campo, ao bit — a frio e depois de cada mudança.

use std::collections::BTreeMap;

use super::{Chave, ParedesPorBlocos, bits};
use crate::v2::V2;
use crate::walls::Walls;

struct Lcg(u64);
impl Lcg {
    fn f(&mut self) -> f64 {
        self.0 = self
            .0
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        (self.0 >> 11) as f64 / (1u64 << 53) as f64
    }
    fn n(&mut self, n: usize) -> usize {
        (self.f() * n as f64) as usize % n
    }
}

const LADO: f64 = 10.0;

fn rect((x, y): Chave) -> (V2, V2) {
    let lo = [x as f64 * LADO, y as f64 * LADO];
    (lo, [lo[0] + LADO, lo[1] + LADO])
}

/// Um ponto do bloco numa grelha GROSSA (passo `2,5`): muitos pontos repetem-se, logo há vértices com
/// várias entradas a começar (a fronteira que se toca) e cadeias que passam de um bloco ao vizinho.
fn ponto(r: &mut Lcg, k: Chave) -> V2 {
    let (lo, _) = rect(k);
    [lo[0] + 2.5 * r.n(5) as f64, lo[1] + 2.5 * r.n(5) as f64]
}

/// As paredes de um bloco: segmentos ao calhas e cadeias em anel, algumas a encostar à borda.
fn paredes(r: &mut Lcg, k: Chave) -> Vec<(V2, V2)> {
    let mut out = Vec::new();
    for _ in 0..r.n(4) {
        let m = 2 + r.n(4);
        let anel: Vec<V2> = (0..m).map(|_| ponto(r, k)).collect();
        for i in 0..m {
            let (a, b) = (anel[i], anel[(i + 1) % m]);
            if a != b {
                out.push((a, b));
            }
        }
    }
    for _ in 0..r.n(6) {
        let (a, b) = (ponto(r, k), ponto(r, k));
        if a != b {
            out.push((a, b));
        }
    }
    out
}

#[derive(Default)]
struct Populacao {
    fora: usize,
    tocam: usize,
    sozinhas: usize,
    iguais: usize,
    tiradas: usize,
}

/// O oráculo: os vértices numerados pelo ponto, as paredes em fila pela ordem da chave.
fn oraculo(blocos: &BTreeMap<Chave, Vec<(V2, V2)>>) -> Walls {
    let mut id: BTreeMap<u128, u32> = BTreeMap::new();
    let mut verts = Vec::new();
    let mut walls = Vec::new();
    let mut v = |p: V2| {
        *id.entry(bits(p)).or_insert_with(|| {
            verts.push(p);
            (verts.len() - 1) as u32
        })
    };
    for ps in blocos.values() {
        for &(de, para) in ps {
            walls.push((v(de), v(para)));
        }
    }
    Walls::from_walkable_walls(&verts, &walls)
}

fn confere(w: &Walls, o: &Walls, r: &mut Lcg, pop: &mut Populacao, de_quem: &[usize]) {
    assert_eq!(w.len(), o.len());
    for i in 0..o.len() {
        assert_eq!(bits(w.point(i)), bits(o.point(i)), "point {i}");
        assert_eq!(w.next(i), o.next(i), "next {i}");
        assert_eq!(w.prev(i), o.prev(i), "prev {i}");
        assert_eq!(bits(w.dir(i)), bits(o.dir(i)), "dir {i}");
        assert_eq!(w.convex(i), o.convex(i), "convex {i}");
        pop.fora += usize::from(de_quem[i] != de_quem[o.next(i)]);
        pop.sozinhas += usize::from(o.next(i) == i);
    }
    let (mut a, mut b) = (Vec::new(), Vec::new());
    for k in 0..300 {
        let pos = [r.f() * 50.0 - 5.0, r.f() * 40.0 - 5.0];
        let range = match k % 7 {
            0 => f64::INFINITY,
            1 => 0.0,
            _ => r.f() * 8.0,
        };
        w.near(pos, range, &mut a);
        o.near_todas(pos, range, &mut b);
        assert_eq!(a, b, "pos {pos:?}, alcance {range}");
    }
}

#[test]
fn as_paredes_por_blocos_sao_a_construcao_inteira_ao_bit() {
    let mut r = Lcg(0x5EED_0011);
    let mut pb = ParedesPorBlocos::new();
    let mut agora: BTreeMap<Chave, Vec<(V2, V2)>> = BTreeMap::new();
    let mut pop = Populacao::default();
    let chaves: Vec<Chave> = (0..4).flat_map(|x| (0..3).map(move |y| (x, y))).collect();
    for passo in 0..120 {
        // A frio, todos; depois, um a três blocos mudam, saem, voltam — ou ficam iguais.
        let mexe: Vec<Chave> = if passo == 0 {
            chaves.clone()
        } else {
            (0..1 + r.n(3)).map(|_| chaves[r.n(chaves.len())]).collect()
        };
        for k in mexe {
            let (lo, hi) = rect(k);
            match r.n(6) {
                0 if agora.contains_key(&k) => {
                    pb.tira(k);
                    agora.remove(&k);
                    pop.tiradas += 1;
                }
                1 if agora.contains_key(&k) => {
                    pb.poe(k, lo, hi, agora[&k].clone());
                    pop.iguais += 1;
                }
                _ => {
                    let ps = paredes(&mut r, k);
                    pb.poe(k, lo, hi, ps.clone());
                    agora.insert(k, ps);
                }
            }
        }
        let o = oraculo(&agora);
        let de_quem: Vec<usize> = agora
            .values()
            .enumerate()
            .flat_map(|(b, ps)| std::iter::repeat_n(b, ps.len()))
            .collect();
        let mut comecos: BTreeMap<u128, usize> = BTreeMap::new();
        for ps in agora.values() {
            for &(_, para) in ps {
                *comecos.entry(bits(para)).or_default() += 1;
            }
        }
        pop.tocam += comecos.values().filter(|&&c| c > 1).count();
        let w = pb.monta();
        confere(&w, &o, &mut r, &mut pop, &de_quem);
    }
    // A fixtura contém os fenómenos (CONTROLOS de população; medidos: 663 · 1 656 · 1 908 · 36 · 32).
    assert!(
        pop.fora > 300,
        "{} entradas seguem para outro bloco",
        pop.fora
    );
    assert!(pop.tocam > 800, "{} pontos onde começam várias", pop.tocam);
    assert!(pop.sozinhas > 900, "{} entradas sem seguinte", pop.sozinhas);
    assert!(pop.iguais > 10, "{} blocos postos iguais", pop.iguais);
    assert!(pop.tiradas > 10, "{} blocos tirados", pop.tiradas);
}

#[test]
#[should_panic(expected = "fora do rectângulo")]
fn uma_parede_fora_do_bloco_e_recusada() {
    let mut pb = ParedesPorBlocos::new();
    pb.poe(
        (0, 0),
        [0.0, 0.0],
        [10.0, 10.0],
        [([1.0, 1.0], [10.5, 1.0])],
    );
}

#[test]
#[should_panic(expected = "não é uma célula")]
fn blocos_que_nao_formam_uma_grelha_sao_recusados() {
    let mut pb = ParedesPorBlocos::new();
    pb.poe((0, 0), [0.0, 0.0], [10.0, 10.0], [([1.0, 1.0], [2.0, 1.0])]);
    pb.poe((1, 0), [5.0, 0.0], [20.0, 10.0], [([6.0, 1.0], [7.0, 1.0])]);
    let _ = pb.monta();
}

#[test]
fn sem_blocos_nao_ha_paredes() {
    let mut pb = ParedesPorBlocos::new();
    let w = pb.monta();
    assert!(w.is_empty());
    let mut out = vec![7];
    w.near([0.0, 0.0], f64::INFINITY, &mut out);
    assert!(out.is_empty());
}

/// O CUSTO é proporcional ao que mudou: um bloco posto IGUAL não se refaz, e um mudado refaz-se a si
/// (L1) e recose-se com os vizinhos (L2) — nunca a malha inteira.
#[test]
fn so_se_refaz_o_bloco_que_mudou_e_os_vizinhos() {
    use super::Contas;
    let mut r = Lcg(77);
    let mut pb = ParedesPorBlocos::new();
    let chaves: Vec<Chave> = (0..5).flat_map(|x| (0..5).map(move |y| (x, y))).collect();
    let mut agora: BTreeMap<Chave, Vec<(V2, V2)>> = BTreeMap::new();
    for &k in &chaves {
        let (lo, hi) = rect(k);
        let ps = paredes(&mut r, k);
        pb.poe(k, lo, hi, ps.clone());
        agora.insert(k, ps);
    }
    let _ = pb.monta();
    assert_eq!(pb.contas(), Contas { l1: 25, l2: 25 });
    for (&k, ps) in &agora {
        let (lo, hi) = rect(k);
        pb.poe(k, lo, hi, ps.clone());
    }
    let _ = pb.monta();
    assert_eq!(
        pb.contas(),
        Contas { l1: 0, l2: 0 },
        "tudo igual: nada se refaz"
    );
    let k = (2, 2);
    let (lo, hi) = rect(k);
    let mut ps = agora[&k].clone();
    ps.push(([lo[0] + 1.0, lo[1] + 1.0], [lo[0] + 2.0, lo[1] + 1.0]));
    pb.poe(k, lo, hi, ps);
    let _ = pb.monta();
    assert_eq!(
        pb.contas(),
        Contas { l1: 1, l2: 9 },
        "um bloco do meio mudou"
    );
    pb.tira((0, 0));
    let _ = pb.monta();
    assert_eq!(
        pb.contas(),
        Contas { l1: 0, l2: 3 },
        "o canto saiu: os 3 vizinhos dele"
    );
}
