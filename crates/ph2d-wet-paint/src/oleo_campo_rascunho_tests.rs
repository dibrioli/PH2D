//! Gate do rascunho do campo (filho de [`super`]): B14.A1 — os planos `f64` do campo não se realocam a cada imagem.

use super::*;

const LADO: usize = 96;

/// Uma mancha de tinta que muda com `n` (a imagem seguinte pinta outra coisa).
fn grade(n: usize) -> Grid {
    let mut g = Grid::new(LADO, LADO);
    for y in 1..=LADO {
        for x in 1..=LADO {
            let (dx, dy) = (x as f64 - 48.0, y as f64 - 40.0 - n as f64);
            if dx * dx + dy * dy < (22.0 + 5.0 * n as f64).powi(2) {
                g.sett[x + y * g.s] = 1800.0 + 41.0 * ((x * 5 + y * 11 + n) % 13) as f32;
            }
        }
    }
    g.sett_rgb.fill([180.0, 120.0, 60.0]);
    g
}

fn ponteiros(r: &RascunhoCampo) -> Vec<(*const f64, usize)> {
    let mut p: Vec<_> = r
        .livres
        .iter()
        .map(|v| (v.as_ptr(), v.capacity()))
        .collect();
    p.sort_unstable();
    p
}

fn bits(v: &[f32]) -> Vec<u32> {
    v.iter().map(|x| x.to_bits()).collect()
}

/// ⭐ B14.A1 (AU-12-3): o mesmo rascunho na imagem seguinte REUSA os planos (os mesmos ponteiros e capacidades), e
/// o campo que sai dele — já sujo de uma imagem com outra tinta e outra janela — é o do campo fresco, ao bit.
#[test]
fn o_rascunho_do_campo_nao_realoca_e_da_o_mesmo_campo() {
    let o = OleoReal::default();
    let (g0, g1) = (grade(0), grade(2));
    let mut ras = RascunhoCampo::default();
    let _ = CampoAltura::de_com_em(
        &mut ras,
        &EmSerie,
        &[(&g0, true)],
        &o,
        1,
        (1, 1, LADO, LADO),
    );
    let antes = ponteiros(&ras);
    assert!(
        antes.len() >= 4,
        "o rascunho guardou {} planos",
        antes.len()
    );
    let janela = (20, 25, 70, 80);
    let c = CampoAltura::de_com_em(&mut ras, &EmSerie, &[(&g1, true)], &o, 1, janela);
    assert_eq!(
        ponteiros(&ras),
        antes,
        "a imagem seguinte realocou os planos do rascunho"
    );
    let f = CampoAltura::de_com(&EmSerie, &[(&g1, true)], &o, 1, janela);
    assert_eq!((c.x0, c.y0, c.w, c.h), (f.x0, f.y0, f.w, f.h));
    for (nome, a, b) in [
        ("v", &c.v, &f.v),
        ("detalhe", &c.detalhe, &f.detalhe),
        ("sombra", &c.sombra, &f.sombra),
        ("borda", &c.borda, &f.borda),
        ("brilho", &c.brilho, &f.brilho),
        ("oleo", &c.oleo, &f.oleo),
    ] {
        assert_eq!(bits(a), bits(b), "plano {nome}");
    }
    let albedo = |c: &CampoAltura| {
        c.albedo
            .iter()
            .flatten()
            .map(|x| x.to_bits())
            .collect::<Vec<_>>()
    };
    assert_eq!(albedo(&c), albedo(&f), "albedo");
}
