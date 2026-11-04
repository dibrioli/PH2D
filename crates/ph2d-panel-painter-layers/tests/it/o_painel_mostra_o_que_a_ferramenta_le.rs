//! ⭐⭐ **O painel do pincel mostra o que a ferramenta na mão LÊ** (dono 23/10). Os dois lados, com o MESMO pincel
//! em todas as ferramentas (o Sync) e uma ponta de imagem (uma rotação ou um espelho só se vê numa ponta que não
//! é redonda), sobre riscas de tinta:
//!
//! 1. cada linha que o painel mostra à ferramenta muda o traço dela — ou está esmaecida com o porquê, ou também
//!    não age no Pincel (a inércia de fábrica é do `o_censo_so_encolhe`, não desta tabela);
//! 2. cada linha que a tabela (`ph2d_tool_painter::NAO_LE`) esconde, posta com o Pincel na mão, não muda nada na
//!    ferramenta — e o controlo: no Pincel ela muda o traço (senão o gate mediria o vazio).

use super::censo_dos_controlos::{
    Bancada, Gesto, diferenca, em_paralelo, gestos_da, nome, nomes, risca, risca_ou_nada,
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
    ensaio_ou_nada(modo, no_pincel, na_ferramenta).expect("os traços sujaram a pré-visualização")
}

/// O [`ensaio`] que devolve `None` quando os traços não sujaram a tela ([`risca_ou_nada`]).
fn ensaio_ou_nada(modo: &str, no_pincel: &Gesto, na_ferramenta: &Gesto) -> Option<Vec<u8>> {
    let mut b = bancada_com(modo, &ARMAR, no_pincel);
    b.aplica(
        na_ferramenta,
        &mut super::censo_dos_controlos::Entrega::default(),
    );
    risca_ou_nada(&mut b.tool)
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
        LinhaDaFerramenta::TracoComoCamada => &[PAINTER_TAPER_END],
        LinhaDaFerramenta::Continuo => &[PAINTER_BRUSH_CONTINUOUS_SPACING],
        LinhaDaFerramenta::Flow => &[PAINTER_BRUSH_FLOW_SLIDER],
        // A Cor abre o seletor (o gesto não a muda — ver `a_cor_escondida_nao_muda_a_ferramenta`); o Composite só
        // existe com o modo Paint (`composite_active`), e a troca de ferramenta desarma-o por construção.
        LinhaDaFerramenta::Cor | LinhaDaFerramenta::Composite => &[],
        // O preview não muda o traço: o gate dele é `a_preview_so_aparece_onde_a_ferramenta_a_le`.
        LinhaDaFerramenta::Preview => &[],
        LinhaDaFerramenta::Aguada => &AGUADA,
        LinhaDaFerramenta::DepositoMolhado => &DEPOSITO_MOLHADO,
        LinhaDaFerramenta::AguaDoVolume => &[PAINTER_WETPAINT_WATER],
        LinhaDaFerramenta::Forma => &FORMA,
        LinhaDaFerramenta::Orientacao => &[PAINTER_SHAPE_FOLLOW, PAINTER_PEN_ROTATION],
        LinhaDaFerramenta::OpacidadePelaVelocidade => &[PAINTER_PEN_SPEED_OPACITY],
        LinhaDaFerramenta::AguaViva => &[PAINTER_WETPAINT_LIVE_WATER],
        LinhaDaFerramenta::GraoDoPapel => &[PAINTER_BRUSH_GRAIN_SAME_AS_PAPER],
        LinhaDaFerramenta::MisturaDoPigmento => &[PAINTER_WETPAINT_PIGMENT_MIXING],
    }
}

/// Os ids que a ferramenta `f` esconde e que nenhum gesto de [`ids_da`] alcança (a amostra da Cor, o Composite): a pergunta
/// sobre eles é só se o painel os pinta.
fn so_nao_se_pintam(f: FerramentaDoPincel) -> Vec<NodeId> {
    NAO_LE
        .iter()
        .filter(|(g, _)| *g == f)
        .flat_map(|(_, ls)| ls.iter())
        .filter_map(|l| match l {
            LinhaDaFerramenta::Cor => Some(ph2d_editor_core::ids::PAINTER_COLOR_THUMB),
            LinhaDaFerramenta::Composite => Some(PAINTER_BRUSH_COMPOSITE_ENABLE),
            _ => None,
        })
        .collect()
}

/// Os ids da silhueta do carimbo: o Falloff, a Hardness, a textura da Shape e o que dela depende, o Shape Tone, e as
/// fileiras de giro e de espelho do Jitter.
const FORMA: [NodeId; 21] = [
    PAINTER_BRUSH_FALLOFF,
    PAINTER_BRUSH_HARDNESS,
    PAINTER_SHAPE_KIND,
    PAINTER_SHAPE_SIZE_X,
    PAINTER_SHAPE_SIZE_Y,
    PAINTER_SHAPE_ANGLE,
    PAINTER_SHAPE_OFFSET_X,
    PAINTER_SHAPE_OFFSET_Y,
    PAINTER_SHAPE_ALPHA_FROM_IMAGE,
    PAINTER_SHAPE_PER_LAYER_COLOR,
    PAINTER_SHAPE_RAMP_ENABLE,
    PAINTER_BRUSH_JITTER_ROTATE,
    PAINTER_BRUSH_JITTER_ROTATE_CHIP,
    PAINTER_MARCA_FIELDS[2],
    PAINTER_MARCA_FIELDS[3],
    PAINTER_MARCA_FIELDS[4],
    PAINTER_MARCA_FIELDS[5],
    PAINTER_MARCA_CHIPS[2],
    PAINTER_MARCA_CHIPS[3],
    PAINTER_MARCA_CHIPS[4],
    PAINTER_MARCA_CHIPS[5],
];

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
const DEPOSITO_MOLHADO: [NodeId; 22] = [
    PAINTER_BRUSH_DAB_QUINA,
    // A ponta que gira com inércia (L3).
    PAINTER_BRUSH_DAB_ROTATION_LAG,
    // As pontas rectas (J6): o corte do traço que o pincel pousa.
    PAINTER_BRUSH_PONTAS_RECTAS,
    PAINTER_BRUSH_PONTAS_ALEM,
    // As pontas que esbatem (J6t): o depósito ao comprido.
    PAINTER_BRUSH_PONTAS_ESBATE,
    PAINTER_WETPAINT_PAPER_VISUAL,
    // A caixa Real Oil é do TRAÇO (dono 28/10): pousa na célula.
    PAINTER_WETPAINT_OLEO_REAL,
    PAINTER_WETPAINT_VOL_OPACITY,
    // Os controlos de lei da E10 (F3 §9): o Pigment saiu, fundido na Dilution. A Hardness não é só do pouso: o Blend
    // da água também lê o `paperGate` (3048 texels, 01/11) e a borracha da água também (`apply_erase`, E10h) — fica
    // fora desta linha, à vista no Pincel, no Blend e na Borracha do Wet Paint.
    PAINTER_WETPAINT_DILUTION,
    PAINTER_WETPAINT_MEDIUM,
    PAINTER_WETPAINT_BODY,
    PAINTER_WETPAINT_PICKUP,
    PAINTER_WETPAINT_DUST,
    PAINTER_WETPAINT_WET_BLOOM,
    PAINTER_WETPAINT_TIP_PICKUP,
    PAINTER_WETPAINT_REWET,
    PAINTER_WETPAINT_REWET_DRAG,
    PAINTER_WETPAINT_SOLUBILITY,
    // A tinta que filtra, tapa e amacia no molhado (dono 30/10): lidas no pouso.
    PAINTER_WETPAINT_GLAZE,
    PAINTER_WETPAINT_COVER,
    PAINTER_WETPAINT_DRY_COVER,
    PAINTER_WETPAINT_WET_SOFTEN,
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
        // Um ensaio que rebenta conta como falha com o nome, nunca some. Os traços que NÃO sujam a tela não são um
        // rebentamento: a base sujou-a, o gesto deixou-a como estava — o controlo AGE ([`risca_ou_nada`]).
        let agem = em_paralelo(&gs, |g| {
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                ensaio_ou_nada(modo, &Gesto::Nenhum, g).is_none_or(|px| diferenca(&base, &px).0 > 0)
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
        // A ferramenta não as pinta (nem as que não têm gesto: ver `ids_da`).
        let b = bancada_com(modo, &ARMAR, &Gesto::Nenhum);
        for id in escondidas.iter().chain(&so_nao_se_pintam(f)) {
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
const INERTES_DO_MEIO: [(NodeId, &str); 3] = [
    (PAINTER_WETPAINT_SECTION, "a cabeça da secção dobra-a"),
    (
        PAINTER_WETPAINT_RESET,
        "repõe os knobs, que a bancada deixa no de fábrica",
    ),
    (
        PAINTER_WETPAINT_TUNING,
        "abre o painel lateral dos knobs do motor",
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
            LinhaDaFerramenta::AguaDoVolume,
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

/// As ferramentas da água, pelo fio do trilho (dono 25/10: moravam no Inspector).
const DA_AGUA: [(&str, FerramentaDoPincel); 5] = [
    ("wet_smear", FerramentaDoPincel::SmearMolhado),
    ("wet_blend", FerramentaDoPincel::BlendMolhado),
    ("wet_water", FerramentaDoPincel::Molhar),
    ("wet_dry", FerramentaDoPincel::Secar),
    ("wet_blow", FerramentaDoPincel::Soprar),
];

/// O Pincel da água na mão, as riscas MOLHADAS com ele, `no_pincel` ainda com ele — DEPOIS das riscas: mexe nos
/// ajustes, não na tinta que a ferramenta `fio` vai encontrar — e então a ferramenta. `bake`: a sessão encerrada
/// antes da ferramenta (sair do Wet Paint é o bake), a base de um gesto que a encerra por lei (o Blend com tinta
/// viva, `set_blend`).
fn bancada_da_agua(fio: &str, no_pincel: &Gesto, bake: bool) -> Bancada {
    bancada_da_agua_com(&ARMAR, fio, no_pincel, bake)
}

/// A ponta REDONDA (o Sync sem a ponta de imagem do [`ARMAR`]): a de `o_carimbo_das_ferramentas_da_agua`.
const PONTA_REDONDA: [Gesto; 1] = [Gesto::Clique(PAINTER_BRUSH_SYNC)];

fn bancada_da_agua_com(armar: &[Gesto], fio: &str, no_pincel: &Gesto, bake: bool) -> Bancada {
    let mut b = bancada_com("eraser", armar, &Gesto::Nenhum);
    b.tool.set_paint_media(PaintMedia::WetPaint);
    b.no_modo("wet_paint");
    super::censo_ferramenta_x_linha::riscas(&mut b);
    assert!(
        b.tool.wet_pigmento().is_some(),
        "{fio}: as riscas não ficaram molhadas"
    );
    b.aplica(
        no_pincel,
        &mut super::censo_dos_controlos::Entrega::default(),
    );
    if bake {
        b.no_modo("blur");
        b.no_modo("wet_paint");
    }
    b.no_modo(fio);
    b
}

/// À vista das ferramentas da água, inerte nesta bancada (o carimbo de fábrica não é achatado) e com razão: o giro da
/// pega e o Reset da Shape (que repõe o Flatten e o giro) agem nelas com o carimbo achatado
/// (`o_carimbo_das_ferramentas_da_agua`).
fn inerte_da_agua_com_motivo(id: NodeId) -> bool {
    id == PAINTER_SHAPE_RESET || id == painter_brush_dab_handle_id(1)
}

fn ensaio_da_agua(fio: &str, no_pincel: &Gesto, na_ferramenta: &Gesto, bake: bool) -> Vec<u8> {
    ensaio_da_agua_com(&ARMAR, fio, no_pincel, na_ferramenta, bake)
}

fn ensaio_da_agua_com(
    armar: &[Gesto],
    fio: &str,
    no_pincel: &Gesto,
    na_ferramenta: &Gesto,
    bake: bool,
) -> Vec<u8> {
    let mut b = bancada_da_agua_com(armar, fio, no_pincel, bake);
    b.aplica(
        na_ferramenta,
        &mut super::censo_dos_controlos::Entrega::default(),
    );
    risca(&mut b.tool)
}

/// ⭐⭐ **Cada ferramenta da água só vê o que ela lê** (dono 25/10: *«é necessário configurar o inspector para cada
/// tipo de tool, escondendo o que não faz parte do uso»*), sobre as riscas MOLHADAS:
///
/// 1. cada controlo à vista muda o traço dela, ou também não age no Pincel da água (o chrome: secções, Reset, Sync)
///    — o que só ela mostra tem de agir;
/// 2. cada linha que a tabela lhe esconde não está pintada e, mexida com o Pincel da água na mão, não muda o traço
///    dela — e o controlo: muda o do Pincel (senão o gate mediria o vazio).
#[test]
fn cada_ferramenta_da_agua_so_ve_o_que_le_smear() {
    cada_ferramenta_da_agua_so_ve_o_que_le("wet_smear");
}

#[test]
fn cada_ferramenta_da_agua_so_ve_o_que_le_blend() {
    cada_ferramenta_da_agua_so_ve_o_que_le("wet_blend");
}

#[test]
fn cada_ferramenta_da_agua_so_ve_o_que_le_water() {
    cada_ferramenta_da_agua_so_ve_o_que_le("wet_water");
}

#[test]
fn cada_ferramenta_da_agua_so_ve_o_que_le_dry() {
    cada_ferramenta_da_agua_so_ve_o_que_le("wet_dry");
}

#[test]
fn cada_ferramenta_da_agua_so_ve_o_que_le_blow() {
    cada_ferramenta_da_agua_so_ve_o_que_le("wet_blow");
}

/// O corpo dos cinco irmãos acima, um por ferramenta do [`DA_AGUA`] (partido em 02/11: as cinco num só teste passavam o
/// prazo de 600 s sozinhas).
fn cada_ferramenta_da_agua_so_ve_o_que_le(so: &str) {
    let da_agua: Vec<(&str, FerramentaDoPincel)> = DA_AGUA
        .iter()
        .copied()
        .filter(|(fio, _)| *fio == so)
        .collect();
    assert_eq!(da_agua.len(), 1, "{so}: não é uma ferramenta da água");
    // ⚠️ O trabalho é ACHATADO em (ferramenta, gesto), em três filas da largura da máquina: o laço em série pelas 5
    // ferramentas, com um `em_paralelo` por controlo inerte, passou o prazo de 600 s sozinho (02/11; a bancada da água
    // custa ~2,5 s, o ensaio ~3 s). As perguntas são as do laço; o que só depende do gesto do Pincel (o controlo «age no
    // Pincel», o bake) mede-se UMA vez, e a base com bake uma vez por ferramenta, em vez de uma por gesto escondido.
    let nomes = nomes();
    let pincel = gestos_da(bancada_da_agua("wet_paint", &Gesto::Nenhum, false));
    let base_pincel = ensaio_da_agua("wet_paint", &Gesto::Nenhum, &Gesto::Nenhum, false);
    let age_no_pincel = |g: &Gesto| {
        diferenca(
            &base_pincel,
            &ensaio_da_agua("wet_paint", &Gesto::Nenhum, g, false),
        )
        .2 > PISO
    };
    struct DaFerramenta {
        escondidas: Vec<NodeId>,
        pintadas: Vec<NodeId>,
        base: Vec<u8>,
        gs: Vec<Gesto>,
    }
    let fs: Vec<DaFerramenta> = em_paralelo(&da_agua, |(fio, f)| {
        let b = bancada_da_agua(fio, &Gesto::Nenhum, false);
        assert_eq!(
            b.tool.brush_settings().ferramenta(),
            *f,
            "{fio}: a ferramenta não ficou na mão"
        );
        let escondidas: Vec<NodeId> = NAO_LE
            .iter()
            .filter(|(g, _)| g == f)
            .flat_map(|(_, ls)| ls.iter().flat_map(|l| ids_da(*l).iter().copied()))
            .collect();
        let pintadas = escondidas
            .iter()
            .copied()
            .chain(so_nao_se_pintam(*f))
            .filter(|id| b.rects().iter().any(|(i, _)| i == id))
            .collect();
        let inercias = b.tool.brush_settings().inercias;
        let base = ensaio_da_agua(fio, &Gesto::Nenhum, &Gesto::Nenhum, false);
        let gs: Vec<Gesto> = gestos_da(b)
            .into_iter()
            .filter(|g| {
                [chave(g), g.id()]
                    .into_iter()
                    .flatten()
                    .all(|i| inercias.dica_de(i).is_none())
            })
            .collect();
        DaFerramenta {
            escondidas,
            pintadas,
            base,
            gs,
        }
    });
    // Os gestos do Pincel que alguma ferramenta esconde.
    let escondidos_do_pincel: Vec<usize> = (0..pincel.len())
        .filter(|&p| {
            chave(&pincel[p]).is_some_and(|i| fs.iter().any(|d| d.escondidas.contains(&i)))
        })
        .collect();

    // Fila 1: cada gesto à vista de cada ferramenta; as bases (ponta redonda; sessão encerrada) de cada uma; e, por
    // gesto escondido do Pincel, se ele age no Pincel e se encerra a sessão (o bake da base).
    enum Um<'a> {
        Age(usize, &'a Gesto),
        BaseRedonda(usize),
        BaseBake(usize),
        DoPincel(usize),
    }
    enum UmSai {
        Age(bool),
        Px(Vec<u8>),
        DoPincel(bool, bool),
    }
    let mut fila: Vec<Um> = Vec::new();
    for (fi, d) in fs.iter().enumerate() {
        fila.extend(d.gs.iter().map(|g| Um::Age(fi, g)));
        fila.push(Um::BaseRedonda(fi));
        fila.push(Um::BaseBake(fi));
    }
    fila.extend(escondidos_do_pincel.iter().map(|&p| Um::DoPincel(p)));
    let saidas = em_paralelo(&fila, |t| match t {
        Um::Age(fi, g) => {
            let fio = da_agua[*fi].0;
            UmSai::Age(
                diferenca(
                    &fs[*fi].base,
                    &ensaio_da_agua(fio, &Gesto::Nenhum, g, false),
                )
                .0 > 0,
            )
        }
        Um::BaseRedonda(fi) => UmSai::Px(ensaio_da_agua_com(
            &PONTA_REDONDA,
            da_agua[*fi].0,
            &Gesto::Nenhum,
            &Gesto::Nenhum,
            false,
        )),
        Um::BaseBake(fi) => UmSai::Px(ensaio_da_agua(
            da_agua[*fi].0,
            &Gesto::Nenhum,
            &Gesto::Nenhum,
            true,
        )),
        Um::DoPincel(p) => {
            let g = &pincel[*p];
            let bake = bancada_da_agua("wet_paint", g, false)
                .tool
                .wet_pigmento()
                .is_none();
            UmSai::DoPincel(age_no_pincel(g), bake)
        }
    });
    let mut agem: Vec<Vec<bool>> = fs.iter().map(|_| Vec::new()).collect();
    let mut base_redonda: Vec<Option<Vec<u8>>> = fs.iter().map(|_| None).collect();
    let mut base_bake: Vec<Option<Vec<u8>>> = fs.iter().map(|_| None).collect();
    let mut do_pincel: BTreeMap<usize, (bool, bool)> = BTreeMap::new();
    let mut k_pincel = escondidos_do_pincel.iter();
    for (t, s) in fila.iter().zip(saidas) {
        match (t, s) {
            (Um::Age(fi, _), UmSai::Age(a)) => agem[*fi].push(a),
            (Um::BaseRedonda(fi), UmSai::Px(px)) => base_redonda[*fi] = Some(px),
            (Um::BaseBake(fi), UmSai::Px(px)) => base_bake[*fi] = Some(px),
            (Um::DoPincel(_), UmSai::DoPincel(a, b)) => {
                do_pincel.insert(*k_pincel.next().expect("um por gesto"), (a, b));
            }
            _ => unreachable!("a saída é da tarefa"),
        }
    }
    let mut medidas = 0;
    // Por ferramenta, os controlos à vista que não agiram (fora os inertes com motivo), com os seus gestos.
    let mut inertes: Vec<Vec<(NodeId, Vec<&Gesto>)>> = Vec::new();
    for (fi, d) in fs.iter().enumerate() {
        let mut por_controlo: BTreeMap<NodeId, (bool, Vec<&Gesto>)> = BTreeMap::new();
        for (g, a) in d.gs.iter().zip(&agem[fi]) {
            let e = por_controlo
                .entry(chave(g).unwrap_or(NodeId(0)))
                .or_default();
            e.0 |= *a;
            e.1.push(g);
            medidas += 1;
        }
        inertes.push(
            por_controlo
                .into_iter()
                .filter(|(id, (age, _))| !age && !inerte_da_agua_com_motivo(*id))
                .map(|(id, (_, gs))| (id, gs))
                .collect(),
        );
    }

    // Fila 2: os inertes pela ponta REDONDA (a ponta de IMAGEM da bancada é uma régua parcial: o sopro do Blow é fraco
    // e, por ela, o Speed → Opacity não chega a um texel — pela redonda, 329; sonda 29/10; só o inerte nas DUAS é
    // inerte), e cada gesto escondido do Pincel na ferramenta.
    enum Dois<'a> {
        Redonda(usize, &'a Gesto),
        Escondido(usize, usize),
    }
    let mut fila: Vec<Dois> = Vec::new();
    for (fi, ins) in inertes.iter().enumerate() {
        for (_, gs) in ins {
            fila.extend(gs.iter().map(|g| Dois::Redonda(fi, g)));
        }
        fila.extend(
            escondidos_do_pincel
                .iter()
                .filter(|&&p| chave(&pincel[p]).is_some_and(|i| fs[fi].escondidas.contains(&i)))
                .map(|&p| Dois::Escondido(fi, p)),
        );
    }
    let saidas = em_paralelo(&fila, |t| match t {
        Dois::Redonda(fi, g) => {
            let base_r = base_redonda[*fi].as_ref().expect("medida na fila 1");
            diferenca(
                base_r,
                &ensaio_da_agua_com(&PONTA_REDONDA, da_agua[*fi].0, &Gesto::Nenhum, g, false),
            )
            .0
        }
        Dois::Escondido(fi, p) => {
            let base = if do_pincel[p].1 {
                base_bake[*fi].as_ref().expect("medida na fila 1")
            } else {
                &fs[*fi].base
            };
            diferenca(
                base,
                &ensaio_da_agua(da_agua[*fi].0, &pincel[*p], &Gesto::Nenhum, false),
            )
            .0
        }
    });
    let mut redonda: BTreeMap<(usize, *const Gesto), bool> = BTreeMap::new();
    let mut escondido: Vec<Vec<(usize, usize)>> = fs.iter().map(|_| Vec::new()).collect();
    for (t, n) in fila.iter().zip(saidas) {
        match t {
            Dois::Redonda(fi, g) => {
                redonda.insert((*fi, std::ptr::from_ref(*g)), n > 0);
            }
            Dois::Escondido(fi, p) => escondido[*fi].push((*p, n)),
        }
    }

    // Fila 3: o inerte nas duas pontas que o Pincel também mostra — age no Pincel?
    let mut fila: Vec<&Gesto> = Vec::new();
    let mut inertes_nas_duas: Vec<Vec<(NodeId, Vec<&Gesto>, bool)>> = Vec::new();
    for (fi, ins) in inertes.iter().enumerate() {
        let mut v = Vec::new();
        for (id, gs) in ins {
            if gs.iter().any(|g| redonda[&(fi, std::ptr::from_ref(*g))]) {
                continue;
            }
            let no_pincel = pincel.iter().any(|p| chave(p) == Some(*id));
            if no_pincel {
                fila.extend(gs.iter().copied());
            }
            v.push((*id, gs.clone(), no_pincel));
        }
        inertes_nas_duas.push(v);
    }
    let agem_no_pincel = em_paralelo(&fila, |g| age_no_pincel(g));
    let age_no_pincel_de: BTreeMap<*const Gesto, bool> = fila
        .iter()
        .map(|g| std::ptr::from_ref(*g))
        .zip(agem_no_pincel)
        .collect();

    let mut falhas = Vec::new();
    for (fi, (fio, f)) in da_agua.iter().enumerate() {
        for id in &fs[fi].pintadas {
            falhas.push(format!(
                "{fio}: «{}» está escondida e pintada",
                nome(&nomes, *id)
            ));
        }
        for (id, gs, no_pincel) in &inertes_nas_duas[fi] {
            let n = nome(&nomes, *id);
            if !no_pincel {
                falhas.push(format!("{fio}: «{n}» só ela mostra e é inerte"));
            } else if gs.iter().any(|g| age_no_pincel_de[&std::ptr::from_ref(*g)]) {
                falhas.push(format!(
                    "{fio}: «{n}» está à vista, age no Pincel da água e não nesta ferramenta"
                ));
            }
        }
        // O controlo é POR LINHA: viva se algum id dela age no Pincel (uma linha junta ids que só agem num estado
        // que a bancada não arma — o Square tip, o Per-Layer Color).
        let mut vivo: BTreeMap<String, bool> = BTreeMap::new();
        for &(p, na_ferramenta) in &escondido[fi] {
            let id = chave(&pincel[p]).unwrap_or(NodeId(0));
            let linhas = NAO_LE
                .iter()
                .filter(|(h, _)| h == f)
                .flat_map(|(_, ls)| ls.iter())
                .filter(|l| ids_da(**l).contains(&id));
            for l in linhas {
                *vivo.entry(format!("{l:?}")).or_default() |= do_pincel[&p].0;
            }
            if na_ferramenta > 0 {
                falhas.push(format!(
                    "{fio}: «{}» está escondida e muda {na_ferramenta} texels",
                    nome(&nomes, id)
                ));
            }
            medidas += 1;
        }
        for (l, v) in vivo {
            if !v {
                falhas.push(format!(
                    "{fio}: controlo — a linha {l} não age nem no Pincel da água"
                ));
            }
        }
    }
    assert!(
        medidas >= 20,
        "{so}: controlo — só {medidas} gestos medidos"
    );
    assert!(falhas.is_empty(), "{falhas:#?}");
}

/// ⭐ **O carimbo das ferramentas da água** (25/10): o que o censo acima não alcança — a bancada dele usa uma ponta de
/// imagem, que esconde o Falloff e a Hardness, e não achata o carimbo, onde girar um redondo não mostra nada. Aqui,
/// sem ponta de imagem e sobre as riscas molhadas, cada controlo age na ferramenta ⟺ a tabela diz que ela o lê; o
/// Flatten e o giro (com o carimbo achatado) agem em todas — por isso a Shape delas os mostra, com o Taper, e o
/// Reset da Shape fica (repõe os dois). O controlo: no Pincel da água cada um age.
#[test]
fn o_carimbo_das_ferramentas_da_agua() {
    type Mexe = fn(&mut ph2d_tool_painter::PainterTool);
    // `(nome, achatado, linha que o lê — `None` = todas —, gesto)`.
    let variantes: [(&str, bool, Option<LinhaDaFerramenta>, Mexe); 8] = [
        ("Falloff", false, Some(LinhaDaFerramenta::Forma), |t| {
            t.set_brush_falloff(3)
        }),
        ("Hardness", false, Some(LinhaDaFerramenta::Forma), |t| {
            t.set_brush_hardness(0.1);
        }),
        ("Flatten", false, None, |t| t.set_brush_dab_flatten(0.6)),
        ("o giro da pega", true, None, |t| {
            t.set_brush_dab_angle(60.0)
        }),
        ("Follow", true, Some(LinhaDaFerramenta::Orientacao), |t| {
            t.set_brush_shape_follow(1);
        }),
        (
            "Dab rotation",
            true,
            Some(LinhaDaFerramenta::Orientacao),
            |t| {
                t.set_mao(ph2d_tool_painter::LINHAS_DA_MAO[8], 1.0);
            },
        ),
        (
            "Speed → Opacity",
            false,
            Some(LinhaDaFerramenta::OpacidadePelaVelocidade),
            |t| {
                t.set_mao(ph2d_tool_painter::LINHAS_DA_MAO[17], 1.0);
            },
        ),
        (
            "Live Water",
            false,
            Some(LinhaDaFerramenta::AguaViva),
            |t| t.set_wet_agua_viva(true),
        ),
    ];
    let bancada = |fio: &str| {
        let mut b = bancada_com(
            "eraser",
            &[Gesto::Clique(PAINTER_BRUSH_SYNC)],
            &Gesto::Nenhum,
        );
        b.tool.set_paint_media(PaintMedia::WetPaint);
        b.no_modo("wet_paint");
        super::censo_ferramenta_x_linha::riscas(&mut b);
        b.no_modo(fio);
        b
    };
    let fios: Vec<&str> = std::iter::once("wet_paint")
        .chain(DA_AGUA.iter().map(|(f, _)| *f))
        .collect();
    // `(fio, None)` a base, `(fio, Some(true))` a base achatada, `(fio, Some(false))` a base de novo (o ruído).
    let mut casos: Vec<(&str, Result<usize, Option<bool>>)> = Vec::new();
    for fio in &fios {
        casos.extend((0..variantes.len()).map(|i| (*fio, Ok(i))));
        casos.extend([None, Some(true), Some(false)].map(|e| (*fio, Err(e))));
    }
    let r = em_paralelo(&casos, |(fio, caso)| {
        let mut b = bancada(fio);
        match caso {
            Ok(i) => {
                let (_, achata, _, f) = variantes[*i];
                if achata {
                    b.tool.set_brush_dab_flatten(0.6);
                }
                f(&mut b.tool);
            }
            Err(Some(true)) => b.tool.set_brush_dab_flatten(0.6),
            Err(_) => {}
        }
        risca(&mut b.tool)
    });
    let em = |fio: &str, caso: Result<usize, Option<bool>>| {
        &r[casos.iter().position(|c| *c == (fio, caso)).unwrap()]
    };
    let mut falhas = Vec::new();
    for fio in &fios {
        let lei = bancada(fio).tool.brush_settings();
        let ruido = diferenca(em(fio, Err(None)), em(fio, Err(Some(false)))).0;
        assert_eq!(ruido, 0, "{fio}: a base não se repete");
        for (i, (n, achata, linha, _)) in variantes.iter().enumerate() {
            let base = em(fio, Err(if *achata { Some(true) } else { None }));
            let age = diferenca(base, em(fio, Ok(i))).0 > 0;
            let le = linha.is_none_or(|l| lei.le(l));
            if age != le {
                falhas.push(format!(
                    "{fio}: «{n}» {} e a tabela diz que {}",
                    if age { "age" } else { "não age" },
                    if le { "a lê" } else { "não a lê" }
                ));
            }
        }
    }
    assert!(falhas.is_empty(), "{falhas:#?}");
}

/// Bug #57: o Fast dry encolhe a região da água; o passo seguinte não pode ler o rascunho do solver de um passo velho,
/// que a cópia de um instantâneo não tem. Controlo: o Fast dry muda o traço (senão o teste mediria o vazio).
#[test]
fn o_fast_dry_de_raiz_e_o_do_instantaneo() {
    let corre = |copia: bool, gesto: &Gesto| {
        let mut b = bancada_da_agua("wet_water", &Gesto::Nenhum, false);
        if copia {
            b.tool = b.tool.instantaneo().expect("copia-se");
        }
        b.aplica(gesto, &mut super::censo_dos_controlos::Entrega::default());
        risca(&mut b.tool)
    };
    let seca = Gesto::Clique(PAINTER_WETPAINT_FASTDRY);
    let raiz = corre(false, &seca);
    assert_eq!(diferenca(&raiz, &corre(true, &seca)), (0, 0, 0));
    assert!(
        diferenca(&raiz, &corre(false, &Gesto::Nenhum)).0 > 0,
        "o Fast dry não mudou o traço"
    );
}
