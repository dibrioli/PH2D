//! Gates do [`super::CampoEsparso`] (B12.3): o mesmo valor do `Vec` denso em toda a célula, e o 1.º dab que não é
//! neutro escreve os pedaços das linhas dele, não a tela.

use super::*;
use crate::grid::aparencia::AparenciaCelula;
use crate::grid::oleo_celula::OleoCelula;
use crate::material::MaterialCelula;

fn mesmo_bit(a: &f32, b: &f32) -> bool {
    a.to_bits() == b.to_bits()
}

/// Um padrão denso com o chão a maior parte, `-0.0` (o `==` confunde-o com `0.0`) e valores nas fronteiras.
fn denso(n: usize) -> Vec<f32> {
    let mut v = vec![0.0f32; n];
    for (k, i) in [0, PEDACO - 1, PEDACO, 3 * PEDACO + 17, n - 1]
        .into_iter()
        .enumerate()
    {
        v[i] = k as f32 + 0.5;
    }
    v[2 * PEDACO + 5] = -0.0;
    v
}

#[test]
fn o_campo_esparso_le_o_mesmo_bit_do_denso() {
    let n = 5 * PEDACO + 123;
    let v = denso(n);
    let c = CampoEsparso::de_vec(0.0, &v, mesmo_bit);
    assert_eq!(c.len(), n);
    assert_eq!(c.pedacos_vivos(), 5, "o pedaço 4 é todo chão");
    for (i, x) in v.iter().enumerate() {
        assert_eq!(c[i].to_bits(), x.to_bits(), "célula {i}");
        assert_eq!(c.get(i).map(f32::to_bits), Some(x.to_bits()));
    }
    assert_eq!(c.get(n), None);
    let ida: Vec<u32> = c.para_vec().iter().map(|x| x.to_bits()).collect();
    let volta: Vec<u32> = v.iter().map(|x| x.to_bits()).collect();
    assert_eq!(ida, volta);
    assert!(c.iter().map(f32::to_bits).eq(volta.iter().copied()));
    // Os intervalos que atravessam pedaços (vivos e ausentes) dão a fatia do denso.
    for (a, b) in [
        (0, n - 1),
        (PEDACO - 3, PEDACO + 2),
        (4 * PEDACO - 1, 5 * PEDACO + 1),
        (7, 7),
    ] {
        let mut out = Vec::new();
        c.copia_intervalo(a, b, &mut out);
        assert!(
            out.iter()
                .map(|x| x.to_bits())
                .eq(v[a..=b].iter().map(|x| x.to_bits())),
            "{a}..={b}"
        );
    }
}

#[test]
fn ler_o_chao_nao_aloca_e_escrever_aloca_so_o_pedaco() {
    let n = 10 * PEDACO;
    let mut c = CampoEsparso::novo(1.0f32, n);
    assert!(c.iter().all(|x| x == 1.0));
    assert_eq!(c.pedacos_vivos(), 0);
    c[3 * PEDACO + 9] = 2.0;
    assert_eq!(c.pedacos_vivos(), 1);
    assert_eq!((c[3 * PEDACO + 8], c[3 * PEDACO + 9]), (1.0, 2.0));
    let mut d = vec![1.0f32; n];
    d[3 * PEDACO + 9] = 2.0;
    c.escreve_intervalo(PEDACO - 2, &[5.0; 4]);
    d[PEDACO - 2..PEDACO + 2].copy_from_slice(&[5.0; 4]);
    assert_eq!(c.pedacos_vivos(), 3);
    assert_eq!(c.para_vec(), d);
    assert_eq!(
        c,
        CampoEsparso::de_vec(1.0, &d, mesmo_bit),
        "igualdade lógica, não de alocação"
    );
    assert!(CampoEsparso::<f32>::vazio(0.0).is_empty());
}

/// A régua do B12.3: o 1.º dab fora da fábrica (raio 30 px) numa tela 3840×2160 — o `Vec` cheio de antes escrevia
/// os três campos inteiros (108 B × as células = 897 MB); o esparso escreve os pedaços das linhas do dab.
#[test]
fn o_primeiro_dab_nao_neutro_escreve_as_linhas_dele_e_nao_a_tela() {
    let (w, h, r) = (3840usize, 2160usize, 30usize);
    let s = w + 2;
    let n = s * (h + 2);
    let por_celula = std::mem::size_of::<MaterialCelula>()
        + std::mem::size_of::<OleoCelula>()
        + std::mem::size_of::<AparenciaCelula>();
    let denso = n * por_celula;
    let mut m = CampoEsparso::novo(MaterialCelula::NEUTRO, n);
    let mut o = CampoEsparso::novo(OleoCelula::NEUTRA, n);
    let mut a = CampoEsparso::novo(AparenciaCelula::NEUTRA, n);
    let (cx, cy) = (1900usize, 1000usize);
    for y in cy - r..=cy + r {
        for x in cx - r..=cx + r {
            let i = x + y * s;
            m[i].metal = 1.0;
            o[i].meio = 0.5;
            a[i].suave = 0.25;
        }
    }
    let esparso = m.bytes_vivos() + o.bytes_vivos() + a.bytes_vivos();
    let linhas = 2 * r + 1;
    // Cada linha do dab toca no máximo 2 pedaços (a linha a 4K tem menos de 2 pedaços).
    assert!(esparso <= 2 * linhas * PEDACO * por_celula, "{esparso} B");
    assert!(
        esparso * 30 < denso,
        "esparso {esparso} B × denso {denso} B"
    );
}
