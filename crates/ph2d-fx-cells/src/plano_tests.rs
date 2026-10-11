use super::*;

fn caixa(x: f32, y: f32, w: f32, h: f32) -> Caixa {
    Caixa {
        lo: [x, y],
        hi: [x + w, y + h],
        ancora: [x + w * 0.5, y + h * 0.5],
        _pad: [0.0; 2],
    }
}

/// O rectângulo do átlas que a célula ocupa (a saída transladada).
fn no_atlas(c: &Celula) -> [i32; 4] {
    [c.saida[0] + c.t[0], c.saida[1] + c.t[1], c.saida[2] + c.t[0], c.saida[3] + c.t[1]]
}

/// ⭐ Entre os CONTEÚDOS de duas células da mesma página há pelo menos `2M + 1` pixels — a pilha,
/// que lê no máximo `M` para cada lado, nunca alcança a vizinha. E tudo cabe na página.
#[test]
fn two_cells_never_come_within_twice_the_reach() {
    let m = 7u32;
    // Uma grelha de discos a TOCAR-SE no ecrã: a camada inteira os veria como uma mancha.
    let caixas: Vec<Caixa> = (0..64)
        .map(|i| caixa(100.0 + (i % 8) as f32 * 14.0, 80.0 + (i / 8) as f32 * 14.0, 14.0, 14.0))
        .collect();
    let paginas = planeia(&caixas, [1600, 900], m, 128);
    assert!(paginas.len() > 1, "a fixtura exerce mais de uma página");
    let total: usize = paginas.iter().map(|p| p.celulas.len()).sum();
    assert_eq!(total, 64);
    for p in &paginas {
        for (a, ca) in p.celulas.iter().enumerate() {
            let ra = no_atlas(ca);
            assert!(ra[0] >= 0 && ra[1] >= 0, "dentro da página");
            assert!(ra[2] as u32 <= p.tamanho[0] && ra[3] as u32 <= p.tamanho[1], "dentro da página");
            for cb in &p.celulas[a + 1..] {
                let rb = no_atlas(cb);
                // O conteúdo é a saída sem a margem; a distância entre conteúdos ≥ 2M + 1.
                let m = m as i32;
                let gx = (rb[0] + m - (ra[2] - m)).max(ra[0] + m - (rb[2] - m));
                let gy = (rb[1] + m - (ra[3] - m)).max(ra[1] + m - (rb[3] - m));
                assert!(gx.max(gy) >= 2 * m + 1, "células {a} e {} a {} px", cb.instancia, gx.max(gy));
            }
        }
    }
}

/// As páginas são trechos CONTÍGUOS e na ordem — a volta, página a página, preserva a ordem de z.
#[test]
fn pages_keep_the_z_order() {
    let caixas: Vec<Caixa> = (0..40).map(|i| caixa(10.0 * i as f32, 20.0, 30.0, 30.0)).collect();
    let paginas = planeia(&caixas, [1600, 900], 10, 200);
    let ordem: Vec<u32> = paginas.iter().flat_map(|p| p.celulas.iter().map(|c| c.instancia)).collect();
    assert_eq!(ordem, (0..40).collect::<Vec<u32>>());
}

/// A translação é INTEIRA e a âncora anda com ela: a fração sub-pixel não muda na ida.
#[test]
fn the_anchor_travels_with_the_cell() {
    let mut c = caixa(300.25, 200.75, 20.0, 10.0);
    c.ancora = [310.375, 205.5];
    let p = &planeia(&[c], [1600, 900], 5, 512)[0];
    let cel = p.celulas[0];
    assert_eq!(cel.org[0] - cel.t[0] as f32, 310.375);
    assert_eq!(cel.org[1] - cel.t[1] as f32, 205.5);
}

/// Fora da janela (mais a margem) não há célula; uma caixa vazia (sem geometria) também não.
#[test]
fn what_cannot_reach_the_window_gets_no_cell() {
    let fora = caixa(-500.0, -500.0, 10.0, 10.0);
    let a_tocar = caixa(-14.0, 10.0, 10.0, 10.0); // a margem 6 alcança o pixel 0
    let vazia = Caixa {
        lo: [1.0e30; 2],
        hi: [-1.0e30; 2],
        ..Caixa::default()
    };
    let p = planeia(&[fora, a_tocar, vazia], [800, 600], 6, 512);
    let vivas: Vec<u32> = p.iter().flat_map(|p| p.celulas.iter().map(|c| c.instancia)).collect();
    assert_eq!(vivas, vec![1]);
}

/// Uma instância maior que a página corre SOZINHA, numa página do tamanho da janela, sem translação.
#[test]
fn a_cell_larger_than_a_page_runs_alone_untranslated() {
    let grande = caixa(-200.0, 50.0, 3000.0, 400.0);
    let pequenas = [caixa(10.0, 10.0, 8.0, 8.0), caixa(40.0, 10.0, 8.0, 8.0)];
    let caixas = [pequenas[0], grande, pequenas[1]];
    let p = planeia(&caixas, [1600, 900], 4, 1024);
    assert_eq!(p.len(), 3, "antes · ela · depois");
    assert_eq!(p[1].tamanho, [1600, 900]);
    assert_eq!(p[1].celulas[0].t, [0, 0]);
    assert_eq!(p[1].celulas[0].saida, [0, 45, 1600, 455]);
}
