//! Gates do campo guardado (filho de [`super`]): B14.A1 — outra revisão das grades não realoca a grade inteira.

use super::*;

const LADO: usize = 96;

/// Uma grade com uma mancha de tinta que cresce com `n` (a revisão seguinte pinta mais).
fn grade(n: usize) -> Grid {
    let mut g = Grid::new(LADO, LADO);
    for y in 1..=LADO {
        for x in 1..=LADO {
            let (dx, dy) = (x as f64 - 48.0, y as f64 - 48.0);
            if dx * dx + dy * dy < (20.0 + 6.0 * n as f64).powi(2) {
                g.sett[x + y * g.s] = 2000.0 + 37.0 * ((x * 7 + y * 13 + n) % 11) as f32;
            }
        }
    }
    g.sett_rgb.fill([200.0, 110.0, 50.0]);
    g
}

fn ponteiros(c: &CacheCampo) -> [*const f32; 3] {
    [
        c.campo.v.as_ptr(),
        c.campo.detalhe.as_ptr(),
        c.campo.sombra.as_ptr(),
    ]
}

/// ⭐ B14.A1 (AU-12-3): no Wet a revisão muda a cada imagem; com a mesma chave e a mesma grade a cache REUSA os
/// planos (os mesmos ponteiros) e só esquece as linhas — as que a nova revisão traz servem-se ao bit, as que não
/// trouxe não se servem (nada da revisão velha passa).
#[test]
fn a_cache_nao_realoca_com_a_revisao() {
    let o = OleoReal::default();
    let mut cache: Option<CacheCampo> = None;
    let (g1, g2) = (grade(0), grade(1));
    let chave = |g: &Grid| ChaveCampo::de(&o, &[(g, true)], 1);
    let todo = (1, 1, LADO, LADO);
    let c1 = CampoAltura::de(&[(&g1, true)], &o, 1, 1, 1, LADO, LADO);
    CacheCampo::guarda(&mut cache, chave(&g1), 1, &c1, (LADO, LADO), todo);
    let antes = ponteiros(cache.as_ref().unwrap());
    // A revisão 2 recompõe só uma faixa de largura inteira.
    let faixa = (1, 30, LADO, 60);
    let c2 = CampoAltura::de(&[(&g2, true)], &o, 1, 1, 30, LADO, 60);
    CacheCampo::guarda(&mut cache, chave(&g2), 2, &c2, (LADO, LADO), faixa);
    let depois = ponteiros(cache.as_ref().unwrap());
    assert_eq!(antes, depois, "a revisão 2 realocou os planos da cache");
    let a = alcance(&o);
    let servido = CacheCampo::serve(cache.as_ref(), &chave(&g2), 2, &o, (30 + a, 60 - a))
        .expect("as linhas da faixa servem-se");
    let inteiro = CampoAltura::de(&[(&g2, true)], &o, 1, 1, 1, LADO, LADO);
    for y in 30..=60 {
        for x in 1..=LADO {
            let (xi, yi) = (x as isize, y as isize);
            assert_eq!(
                servido.at_fina(xi, yi).to_bits(),
                inteiro.at_fina(xi, yi).to_bits(),
                "({x}, {y})"
            );
            assert_eq!(
                servido.at_sombra(xi, yi).to_bits(),
                inteiro.at_sombra(xi, yi).to_bits()
            );
        }
    }
    // As linhas que só a revisão 1 montou não passam para a 2.
    assert!(CacheCampo::serve(cache.as_ref(), &chave(&g2), 2, &o, (80, 90)).is_none());
}
