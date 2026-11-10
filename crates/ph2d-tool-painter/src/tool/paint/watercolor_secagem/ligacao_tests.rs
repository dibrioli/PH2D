//! B16.A1: a ligação das poças funde os troços de cada linha antes de ligar as linhas — o mesmo
//! veredicto que a ligação troço a troço (o oráculo), com as uniões em `O(linhas·elo)` nas listras.

use super::{largos_secos, pocas_secas_troco_a_troco, trocos_da_linha, Troco, UNIOES};

/// Uma cobertura: por linha, `(ocupado, molhado)` de cada texel.
type Cobertura = Vec<Vec<(bool, bool)>>;

fn troços(c: &Cobertura, elo: usize) -> (Vec<Troco>, Vec<usize>) {
    let (mut t, mut inicio) = (Vec::new(), Vec::new());
    for (y, l) in c.iter().enumerate() {
        inicio.push(t.len());
        t.extend(trocos_da_linha(y, 0, l.len(), elo, |x| l[x].0, |x| l[x].1));
    }
    inicio.push(t.len());
    (t, inicio)
}

/// O caminho novo: os largos, a ligação deles, e os estreitos de cada largo seco.
fn os_que_assam(c: &Cobertura, elo: usize) -> Vec<(usize, usize, usize)> {
    let (largos, inicio) = troços(c, elo);
    let seco = largos_secos(&largos, &inicio, elo);
    let mut v = Vec::new();
    for (l, _) in largos.iter().zip(&seco).filter(|(_, s)| **s) {
        let linha = &c[l.y];
        v.extend(
            trocos_da_linha(l.y, l.a, l.b, 0, |x| linha[x].0, |x| linha[x].1)
                .iter()
                .map(|t| (t.y, t.a, t.b)),
        );
    }
    v
}

/// O oráculo: os estreitos da tela inteira e a ligação troço a troço de antes.
fn os_que_assam_no_oraculo(c: &Cobertura, elo: usize) -> Vec<(usize, usize, usize)> {
    let (t, inicio) = troços(c, 0);
    let seca = pocas_secas_troco_a_troco(&t, &inicio, elo);
    t.iter()
        .zip(&seca)
        .filter(|(_, s)| **s)
        .map(|(t, _)| (t.y, t.a, t.b))
        .collect()
}

/// ⭐ Ao bit contra o oráculo: 400 coberturas pseudo-aleatórias (troços de 1–40 px, vãos de 1–60 px,
/// linhas vazias, papel molhado raro) × os elos 1, 2, 5, 16 e 28 — os mesmos troços assam.
#[test]
fn a_ligacao_das_pocas_funde_os_trocos_e_da_o_veredicto_do_oraculo() {
    let mut s = 0x9E37_79B9_7F4A_7C15u64;
    let mut r = |m: u64| {
        s ^= s << 13;
        s ^= s >> 7;
        s ^= s << 17;
        (s % m) as usize
    };
    let mut assados = 0usize;
    for caso in 0..400 {
        let largura = 64 + r(400);
        let vao_max = 1 + r(60) as u64;
        let raro = 20 + r(400) as u64;
        let c: Cobertura = (0..8 + r(80))
            .map(|_| {
                let mut l = vec![(false, false); largura];
                if r(6) == 0 {
                    return l;
                }
                let mut x = r(20);
                while x < largura {
                    let b = (x + 1 + r(40)).min(largura);
                    for p in &mut l[x..b] {
                        *p = (true, r(raro * 8) == 0);
                    }
                    x = b + 1 + r(vao_max);
                }
                l
            })
            .collect();
        for elo in [1, 2, 5, 16, 28] {
            let novo = os_que_assam(&c, elo);
            assert_eq!(novo, os_que_assam_no_oraculo(&c, elo), "caso {caso}, elo {elo}");
            assados += novo.len();
        }
    }
    assert!(assados > 0, "a régua: alguns casos assam");
}

/// ⭐ Listras de 2 px (o pior caso, `x % 4 < 2`) em 512 linhas × 1 024 px, elo 16, um texel molhado:
/// a ligação troço a troço fazia ~`3·10⁷` uniões; a dos largos, no máximo `linhas · elo`.
#[test]
fn as_listras_de_dois_px_ligam_se_em_linhas_vezes_elo_unioes() {
    let (linhas, largura, elo) = (512usize, 1024usize, 16usize);
    let c: Cobertura = (0..linhas)
        .map(|y| (0..largura).map(|x| (x % 4 < 2, y == 0 && x == 0)).collect())
        .collect();
    let conta = |f: &dyn Fn() -> Vec<bool>| {
        UNIOES.with(|u| u.set(0));
        let v = f();
        (v, UNIOES.with(std::cell::Cell::get))
    };
    let (largos, il) = troços(&c, elo);
    let (estreitos, ie) = troços(&c, 0);
    let (seco, n_novo) = conta(&|| largos_secos(&largos, &il, elo));
    let (seca, n_velho) = conta(&|| pocas_secas_troco_a_troco(&estreitos, &ie, elo));
    assert!(seco.iter().all(|&v| !v) && seca.iter().all(|&v| !v), "uma poça só, molhada");
    let tecto = (linhas * elo) as u64;
    assert!(n_novo <= tecto, "{n_novo} uniões > {tecto}");
    assert!(n_velho > 100 * tecto, "a régua morde: o oráculo fazia {n_velho}");
}
