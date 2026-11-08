//! ⭐ **As regiões da aba em cartões** (dono 08/11, «tudo fica mais legal dentro de molduras»): a
//! lista, o código e a faixa de baixo são três cartões dentro da aba, com a margem e o vão da aba
//! Collision Layers; e o separador «Problems (1)» cabe inteiro.

use crate::{AREA, Mesa};
use ph2d_editor_core::text_elide::elisao;
use ph2d_editor_core::zones::Rect;
use ph2d_tokens::Spacing;

const ERRO: &str = "function update(self, dt)\n  local x = (\nend\n";

fn dentro(fora: Rect, r: Rect, m: f32) -> bool {
    r.x >= fora.x + m
        && r.y >= fora.y + m
        && r.x + r.w <= fora.x + fora.w - m
        && r.y + r.h <= fora.y + fora.h - m
}

fn separados(a: Rect, b: Rect) -> bool {
    a.x + a.w < b.x || b.x + b.w < a.x || a.y + a.h < b.y || b.y + b.h < a.y
}

#[test]
fn os_tres_cartoes_ficam_dentro_da_margem_e_nao_se_tocam() {
    let (mut m, ids) = Mesa::new(&[("a.luau", ERRO)]);
    let id = ids[0];
    m.pede(id);
    m.pinta(id);
    let margem = Spacing::Md.px();
    assert!(margem > 0.0);
    let [lista, codigo, faixa] = m.docs().card_rects(id).expect("aberto");
    for (nome, r) in [("lista", lista), ("código", codigo), ("faixa", faixa)] {
        assert!(r.w > 0.0 && r.h > 0.0, "{nome}: {r:?}");
        assert!(
            dentro(AREA, r, margem),
            "{nome} {r:?} fora da margem de {AREA:?}"
        );
    }
    for (a, b) in [(lista, codigo), (lista, faixa), (codigo, faixa)] {
        assert!(separados(a, b), "os cartões tocam-se: {a:?} {b:?}");
    }
    // Controlo: a faixa e os seus separadores pintam-se DENTRO do cartão dela.
    let band = m.docs().band_of(id).expect("faixa");
    for (_, t) in &band.tabs {
        assert!(dentro(faixa, *t, 0.0), "{t:?} fora de {faixa:?}");
    }
}

/// O separador «Problems (1)» pinta-se inteiro na largura da cena 11 (a elisão cortava-o em
/// «Problems (…»: o texto ia num orçamento `(w + 2·pad) − 2·pad`, um ULP abaixo da medida).
#[test]
fn o_separador_problems_cabe_inteiro() {
    let (mut m, ids) = Mesa::new(&[("a.luau", ERRO)]);
    let id = ids[0];
    m.pede(id);
    m.pinta(id);
    assert!(
        !m.docs().problems_of(id).is_empty(),
        "controlo: há problema"
    );
    let (_, medidos) = elisao::medindo(|| m.pinta(id));
    let tab: Vec<_> = medidos
        .iter()
        .filter(|x| x.texto.starts_with("Problems ("))
        .collect();
    assert!(!tab.is_empty(), "controlo: o separador foi medido");
    for x in tab {
        assert!(x.coube(), "cortado: {x:?}");
    }
}
