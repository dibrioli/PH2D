//! ⭐⭐ **O painel do pincel mostra o que a ferramenta na mão LÊ** (dono 23/10). Os dois lados, com o MESMO pincel
//! em todas as ferramentas (o Sync) e uma ponta de imagem (uma rotação ou um espelho só se vê numa ponta que não
//! é redonda), sobre riscas de tinta:
//!
//! 1. cada linha que o painel mostra à ferramenta muda o traço dela — ou está esmaecida com o porquê, ou também
//!    não age no Pincel (a inércia de fábrica é do `o_censo_so_encolhe`, não desta tabela);
//! 2. cada linha que a tabela (`ph2d_tool_painter::NAO_LE`) esconde, posta com o Pincel na mão, não muda nada na
//!    ferramenta — e o controlo: no Pincel ela muda o traço (senão o gate mediria o vazio).

use super::censo_dos_controlos::{
    Bancada, Gesto, diferenca, em_paralelo, gestos_da, nome, nomes, risca,
};
use super::censo_ferramenta_x_linha::bancada_com;
use ph2d_a11y::NodeId;
use ph2d_tool_painter::ids::*;
use ph2d_tool_painter::{FerramentaDoPincel, LinhaDaFerramenta, NAO_LE, PaintMedia};
use std::collections::BTreeMap;

/// O piso de |Δ| de um canal para o PINCEL «agir»: abaixo dele a mudança é arredondamento de uma troca de rota
/// interna (o dab que ganha um `tilt` sai da cache para a rota por dab — medido 23/10: Pincel |Δ| ≤ 4, Borracha ≤ 6,
/// Blur ≤ 2). Na ferramenta qualquer texel conta: a dúvida fica a favor da linha à vista.
const PISO: u8 = 8;

/// `(ferramenta, controlo, motivo)`: à vista, inerte no estado do censo e com razão para ficar à vista.
const INERTES_COM_MOTIVO: [(&str, NodeId, &str); 1] = [(
    "eraser",
    PAINTER_SHAPE_RAMP_ENABLE,
    "na ferramenta sem cor o Shape Tone é tom de COBERTURA e a rampa de fábrica (preto→branco) é a identidade: age \
     ao mexer na rampa (`dual_ferramenta_tests`, o tom na Borracha). Não esmaece: a linha já é inerte noutros estados \
     do Pincel (Grain armado, ferramentas do Wet Paint) que a lei teria de cobrir todos",
)];

const ARMAR: [Gesto; 2] = [Gesto::Clique(PAINTER_BRUSH_SYNC), Gesto::PontaColorida];

/// As ferramentas do trilho com o corpo do pincel, pelo fio do trilho.
const FERRAMENTAS: [(&str, FerramentaDoPincel); 5] = [
    ("eraser", FerramentaDoPincel::Borracha),
    ("blur", FerramentaDoPincel::Blur),
    ("smear", FerramentaDoPincel::Smear),
    ("clone", FerramentaDoPincel::Clone),
    ("mask", FerramentaDoPincel::Mascara),
];

fn ensaio(modo: &str, no_pincel: &Gesto, na_ferramenta: &Gesto) -> Vec<u8> {
    let mut b = bancada_com(modo, &ARMAR, no_pincel);
    b.aplica(
        na_ferramenta,
        &mut super::censo_dos_controlos::Entrega::default(),
    );
    risca(&mut b.tool)
}

/// O id que identifica o gesto na tabela: o do MENU, numa opção.
fn chave(g: &Gesto) -> Option<NodeId> {
    match g {
        Gesto::Opcao(menu, ..) => Some(*menu),
        _ => g.id(),
    }
}

/// Os ids que o painel pinta para cada linha da tabela — os que um gesto alcança.
fn ids_da(l: LinhaDaFerramenta) -> &'static [NodeId] {
    match l {
        LinhaDaFerramenta::Blend => &[PAINTER_BRUSH_BLEND],
        LinhaDaFerramenta::Randomize => &[PAINTER_BRUSH_COLOR_JITTER_HUE],
        LinhaDaFerramenta::Simetria => &[PAINTER_BRUSH_SYMMETRY_USE],
        LinhaDaFerramenta::Solido => &[PAINTER_LINE_SOLID],
        LinhaDaFerramenta::CorPorCamada => &[PAINTER_SHAPE_PER_LAYER_COLOR],
        LinhaDaFerramenta::TracoComoCamada => &[
            PAINTER_TRACO_FIELDS[0],
            PAINTER_TRACO_FIELDS[1],
            PAINTER_TRACO_FIELDS[2],
            PAINTER_TAPER_END,
        ],
        LinhaDaFerramenta::Continuo => &[PAINTER_BRUSH_CONTINUOUS_SPACING],
        // A Cor abre o seletor (o gesto não a muda — ver `a_cor_escondida_nao_muda_a_ferramenta`); o Composite só
        // existe com o modo Paint (`composite_active`), e a troca de ferramenta desarma-o por construção.
        LinhaDaFerramenta::Cor | LinhaDaFerramenta::Composite => &[],
        // O preview não muda o traço: o gate dele é `a_preview_so_aparece_onde_a_ferramenta_a_le`.
        LinhaDaFerramenta::Preview => &[],
        LinhaDaFerramenta::Aguada => &AGUADA,
        LinhaDaFerramenta::DepositoMolhado => &DEPOSITO_MOLHADO,
    }
}

/// Os ids da aguada: a secção Watercolor (cabeça e linhas), o «Automatic» da Shape e o «Same as Paper» do Grain.
const AGUADA: [NodeId; 6] = [
    PAINTER_WATERCOLOR_SECTION,
    PAINTER_WATERCOLOR_FILL,
    PAINTER_WATERCOLOR_EDGE,
    PAINTER_WATERCOLOR_DRY_NOW,
    PAINTER_SHAPE_WATERCOLOR_AUTO,
    PAINTER_WATERCOLOR_GRAN_SAME,
];

/// Os ids do que o pincel do Wet Paint pousa.
const DEPOSITO_MOLHADO: [NodeId; 7] = [
    PAINTER_BRUSH_DAB_QUINA,
    PAINTER_WETPAINT_PAPER_VISUAL,
    PAINTER_WETPAINT_WATER,
    PAINTER_WETPAINT_VOL_OPACITY,
    PAINTER_WETPAINT_PIGMENT,
    PAINTER_WETPAINT_PICKUP,
    PAINTER_WETPAINT_DUST,
];

#[test]
fn cada_linha_visivel_age_na_ferramenta_ou_diz_porque_nao() {
    let nomes = nomes();
    let base_pincel = ensaio("brush", &Gesto::Nenhum, &Gesto::Nenhum);
    let mut falhas = Vec::new();
    let mut medidas_total = 0;
    for (modo, f) in FERRAMENTAS {
        let b: Bancada = bancada_com(modo, &ARMAR, &Gesto::Nenhum);
        assert_eq!(
            b.tool.brush_settings().ferramenta(),
            f,
            "{modo}: a ferramenta não ficou na mão"
        );
        let inercias = b.tool.brush_settings().inercias;
        let base = ensaio(modo, &Gesto::Nenhum, &Gesto::Nenhum);
        let gs: Vec<Gesto> = gestos_da(b)
            .into_iter()
            // Esmaecida pelo controlo (o MENU, numa opção) ou pela própria linha.
            .filter(|g| {
                [chave(g), g.id()]
                    .into_iter()
                    .flatten()
                    .all(|i| inercias.dica_de(i).is_none())
            })
            .collect();
        // Um ensaio que rebenta (o traço não sujou a tela) conta como falha com o nome, nunca some.
        let agem = em_paralelo(&gs, |g| {
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                diferenca(&base, &ensaio(modo, &Gesto::Nenhum, g)).0 > 0
            }))
        });
        // Por CONTROLO (as opções de um menu são um controlo só): age se algum gesto dele age.
        let mut por_controlo: BTreeMap<NodeId, (bool, bool, Vec<&Gesto>)> = BTreeMap::new();
        for (g, a) in gs.iter().zip(&agem) {
            let e = por_controlo
                .entry(chave(g).unwrap_or(NodeId(0)))
                .or_default();
            e.0 |= matches!(a, Ok(true));
            e.1 |= a.is_err();
            e.2.push(g);
        }
        for (id, (age, rebentou, gs_dele)) in por_controlo {
            if age {
                continue;
            }
            // O mesmo controlo no Pincel: age? rebenta também? (Uma linha que só a ferramenta tem — o cartão do
            // Clone, o da Máscara — não existe no Pincel: não é desta tabela.)
            let no_pincel: Vec<Option<bool>> = em_paralelo(&gs_dele, |g| {
                std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    diferenca(&base_pincel, &ensaio("brush", &Gesto::Nenhum, g)).2 > PISO
                }))
                .ok()
            });
            let n = nome(&nomes, id);
            if INERTES_COM_MOTIVO
                .iter()
                .any(|(m, i, _)| *m == modo && *i == id)
            {
                continue;
            }
            if no_pincel.contains(&Some(true)) {
                falhas.push(format!(
                    "{modo}: «{n}» está à vista, age no Pincel e não nesta ferramenta"
                ));
            } else if rebentou && no_pincel.iter().all(Option::is_some) {
                falhas.push(format!(
                    "{modo}: «{n}» rebenta o ensaio só nesta ferramenta"
                ));
            }
        }
        medidas_total += gs.len();
    }
    assert!(
        medidas_total > 5 * 100,
        "controlo: só {medidas_total} linhas medidas"
    );
    assert!(falhas.is_empty(), "{falhas:#?}");
}

#[test]
fn nenhuma_linha_escondida_mudaria_a_ferramenta() {
    let nomes = nomes();
    let pincel = gestos_da(bancada_com("brush", &ARMAR, &Gesto::Nenhum));
    let base_pincel = ensaio("brush", &Gesto::Nenhum, &Gesto::Nenhum);
    let mut falhas = Vec::new();
    let mut medidas = 0;
    for (modo, f) in FERRAMENTAS {
        let escondidas: Vec<NodeId> = NAO_LE
            .iter()
            .filter(|(g, _)| *g == f)
            .flat_map(|(_, ls)| ls.iter().flat_map(|l| ids_da(*l).iter().copied()))
            .collect();
        // A ferramenta não as pinta (o Composite também, que não tem gesto: ver `ids_da`).
        let b = bancada_com(modo, &ARMAR, &Gesto::Nenhum);
        let composite = NAO_LE
            .iter()
            .any(|(g, ls)| *g == f && ls.contains(&LinhaDaFerramenta::Composite))
            .then_some(PAINTER_BRUSH_COMPOSITE_ENABLE);
        for id in escondidas.iter().chain(composite.iter()) {
            if b.rects().iter().any(|(i, _)| i == id) {
                falhas.push(format!(
                    "{modo}: «{}» está escondida na tabela e o painel pinta-a",
                    nome(&nomes, *id)
                ));
            }
        }
        let base = ensaio(modo, &Gesto::Nenhum, &Gesto::Nenhum);
        let gs: Vec<&Gesto> = pincel
            .iter()
            .filter(|g| chave(g).is_some_and(|i| escondidas.contains(&i)))
            .collect();
        let r = em_paralelo(&gs, |g| {
            (
                diferenca(&base_pincel, &ensaio("brush", &Gesto::Nenhum, g)).0,
                diferenca(&base, &ensaio(modo, g, &Gesto::Nenhum)).0,
            )
        });
        // Por controlo: vivo no Pincel se ALGUM gesto dele (uma opção do menu) age lá.
        let mut vivo: std::collections::BTreeMap<NodeId, bool> = Default::default();
        for (g, (no_pincel, na_ferramenta)) in gs.iter().zip(r) {
            let id = chave(g).unwrap_or(NodeId(0));
            *vivo.entry(id).or_default() |= no_pincel > 0;
            if na_ferramenta > 0 {
                let n = nome(&nomes, id);
                falhas.push(format!(
                    "{modo}: «{n}» está escondida e muda {na_ferramenta} texels da ferramenta"
                ));
            }
            medidas += 1;
        }
        for (id, v) in vivo {
            if !v {
                let n = nome(&nomes, id);
                falhas.push(format!(
                    "{modo}: controlo — «{n}» não age nem no Pincel (o gate mediria o vazio)"
                ));
            }
        }
    }
    assert!(
        medidas >= 20,
        "controlo: só {medidas} gestos escondidos medidos"
    );
    assert!(falhas.is_empty(), "{falhas:#?}");
}

/// A Cor (que o gesto não alcança: a amostra abre o seletor) pela porta do tool: a cor do pincel não muda a
/// ferramenta que não deposita.
#[test]
fn a_cor_escondida_nao_muda_a_ferramenta() {
    for (modo, f) in FERRAMENTAS {
        if !NAO_LE
            .iter()
            .any(|(g, ls)| *g == f && ls.contains(&LinhaDaFerramenta::Cor))
        {
            continue;
        }
        let base = ensaio(modo, &Gesto::Nenhum, &Gesto::Nenhum);
        let mut b = bancada_com(modo, &ARMAR, &Gesto::Nenhum);
        b.tool.set_brush_color_srgb8([250, 20, 200]);
        assert_eq!(
            diferenca(&base, &risca(&mut b.tool)).0,
            0,
            "{modo}: a cor escondida mudou a ferramenta"
        );
    }
}

/// `(controlo, motivo)`: à vista da Borracha num meio dela, inerte nas riscas secas E molhadas, e com razão.
const INERTES_DO_MEIO: [(NodeId, &str); 4] = [
    (PAINTER_WETPAINT_SECTION, "a cabeça da secção dobra-a"),
    (
        PAINTER_WETPAINT_RESET,
        "repõe os knobs, que a bancada deixa no de fábrica",
    ),
    (
        PAINTER_WETPAINT_TUNING,
        "abre o painel lateral dos knobs do motor",
    ),
    (
        PAINTER_WETPAINT_TOOL_IDS[1],
        "o Erase do rádio é a própria Borracha (já escolhido)",
    ),
];

/// A Borracha no `meio`, sobre as riscas secas ou (`molhado`) re-pintadas no Wet Paint.
fn bancada_do_meio(meio: PaintMedia, molhado: bool) -> Bancada {
    let mut b = bancada_com("eraser", &ARMAR, &Gesto::Nenhum);
    b.tool.set_paint_media(meio);
    b.no_modo("eraser");
    assert_eq!(
        b.tool.paint_media(),
        meio,
        "a Borracha não ficou em {meio:?}"
    );
    if molhado {
        super::censo_ferramenta_x_linha::molha(&mut b, "eraser");
    }
    b
}

fn ensaio_do_meio(meio: PaintMedia, molhado: bool, g: &Gesto) -> Vec<u8> {
    let mut b = bancada_do_meio(meio, molhado);
    b.aplica(g, &mut super::censo_dos_controlos::Entrega::default());
    risca(&mut b.tool)
}

/// ⭐ **A Borracha em cada meio dela só vê o que esse meio lhe muda** (24/10): cada linha que o meio ACRESCENTA ao
/// painel da Borracha (o que ela não vê no Digital) muda o apagado — nas riscas secas ou nas molhadas — ou diz porque
/// não; e as linhas que a tabela lhe esconde (a aguada, o depósito molhado) não estão pintadas. Vermelho antes: na
/// Watercolor a secção inteira (Body, Edge, Dry…) à vista e inerte; no Wet Paint o Water, a Opacity, o Pigment, o
/// Pickup e o Brush dust.
#[test]
fn a_borracha_em_cada_meio_so_ve_o_que_o_meio_lhe_muda() {
    let nomes = nomes();
    let digital = bancada_do_meio(PaintMedia::Digital, false).ids();
    let mut falhas = Vec::new();
    let mut medidas = 0;
    for (meio, estados) in [
        (PaintMedia::Watercolor, &[false][..]),
        (PaintMedia::WetPaint, &[false, true][..]),
    ] {
        let b = bancada_do_meio(meio, false);
        let ids = b.ids();
        for l in [
            LinhaDaFerramenta::Aguada,
            LinhaDaFerramenta::DepositoMolhado,
        ] {
            for id in ids_da(l) {
                if ids.contains(id) {
                    falhas.push(format!(
                        "{meio:?}: «{}» está escondida e pintada",
                        nome(&nomes, *id)
                    ));
                }
            }
        }
        let gs: Vec<Gesto> = gestos_da(b)
            .into_iter()
            .filter(|g| chave(g).is_some_and(|i| !digital.contains(&i)))
            .collect();
        let mut age: BTreeMap<NodeId, bool> = BTreeMap::new();
        for &molhado in estados {
            let base = ensaio_do_meio(meio, molhado, &Gesto::Nenhum);
            let r = em_paralelo(&gs, |g| {
                diferenca(&base, &ensaio_do_meio(meio, molhado, g)).0 > 0
            });
            for (g, a) in gs.iter().zip(r) {
                *age.entry(chave(g).unwrap_or(NodeId(0))).or_default() |= a;
                medidas += 1;
            }
        }
        for (id, a) in age {
            if !a && !INERTES_DO_MEIO.iter().any(|(i, _)| *i == id) {
                falhas.push(format!(
                    "{meio:?}: «{}» à vista da Borracha e inerte",
                    nome(&nomes, id)
                ));
            }
        }
    }
    assert!(
        medidas >= 20,
        "controlo: só {medidas} gestos do meio medidos"
    );
    assert!(falhas.is_empty(), "{falhas:#?}");
}
