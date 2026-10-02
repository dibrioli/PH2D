//! ⭐⭐⭐⭐ **UMA FORMA NOVA E UMA COR NOVA NÃO RECOMPILAM O PINTOR** — os gates do report do dono de
//! 2026-10-01: *«ao acrescentar um box ele demora uns 5 segundos para aparecer e a resolução cai ao
//! rotacionar a view. tb 5 seg para mudar de cor»*.
//!
//! Três curas, e cada uma é invisível a toda régua de VALOR (a imagem sai a mesma), logo cada uma
//! afirma a CONTA — que pipelines nasceram — com o CONTROLO que a impede de ficar verde a medir
//! nada:
//!
//! 1. **a lei do dono é INTERPRETADA** ([`ph2d_field_eval::owners::wgsl::texto_interpretado`]): o
//!    texto do pintor deixou de levar uma fita por folha;
//! 2. **a fita vai por ENTRADA** (`ph2d_field_gpu::paint_entradas`): só o `assa_sondas`, que marcha
//!    a peça, recompila com uma estrutura nova;
//! 3. **o quadro diz quanto foi COMPILAÇÃO** ([`ph2d_field_gpu::trace::Pintado::compilado_ms`]): o
//!    laço da resolução desconta-o, e um quadro que compilou deixa de se ler como uma peça cara.
//!
//! ⚠️ **O traçador é UM por processo** ([`crate::gpu_frame::shared`]) e estes gates correm com
//! `--test-threads=1` ao lado de outros ⇒ tudo o que se afirma é uma DIFERENÇA medida no próprio
//! gate, e as peças usam uma família (toros) que nenhum outro gate desta crate monta em união.

use super::*;

/// Uma peça de `n` toros em união dura — e as folhas POSTAS, que é o que a lei do dono recebe.
fn toros(n: usize) -> (ph2d_field::FieldDoc, Vec<ph2d_field::FieldDoc>) {
    use ph2d_field::{FieldDoc, NodeId, Primitive, Xform};
    #[allow(clippy::cast_precision_loss)]
    let folhas: Vec<ph2d_field::Node> = (0..n)
        .map(|i| {
            ph2d_field_eval::leaf(
                Primitive::Torus {
                    major: 0.16,
                    minor: 0.06,
                },
                Xform::at(i as f32 * 0.3 - 0.15 * (n as f32 - 1.0), 0.0, 0.0),
            )
        })
        .collect();
    let mut nos = folhas.clone();
    let mut raiz = NodeId(0);
    #[allow(clippy::cast_possible_truncation)]
    for i in 1..n {
        nos.push(ph2d_field::Node {
            xform: Xform::IDENTITY,
            kind: ph2d_field::NodeKind::Combine {
                op: ph2d_field::Op::Union(ph2d_field::Blend::Sharp),
                children: vec![raiz, NodeId(i as u32)],
            },
            mods: Vec::new(),
            verb: None,
        });
        raiz = NodeId((nos.len() - 1) as u32);
    }
    let postas = folhas
        .into_iter()
        .map(|f| FieldDoc::new(vec![f], NodeId(0)).expect("a folha"))
        .collect();
    (FieldDoc::new(nos, raiz).expect("a peça"), postas)
}

/// Os materiais de `n` folhas: todos iguais, ou cada um com a sua cor.
fn materiais(n: usize, distintos: bool) -> Vec<ph2d_material::Surface> {
    #[allow(clippy::cast_precision_loss)]
    (0..n)
        .map(|i| {
            let mut m = ph2d_material::OpenPbr::default();
            if distintos {
                let f = i as f32 / n.max(1) as f32;
                m.base_color = [0.9 - 0.8 * f, 0.2 + 0.6 * f, 0.3];
            }
            m.prepare()
        })
        .collect()
}

/// Quantos pipelines desta ENTRADA o traçador já compilou.
fn conta(t: &crate::gpu_frame::SharedTracer, entrada: &str) -> usize {
    t.lock()
        .expect("o traçador")
        .entradas_compiladas()
        .iter()
        .filter(|e| e.as_str() == entrada)
        .count()
}

/// Pinta a peça pelo caminho do produto e devolve o que o quadro devolveu.
fn pinta(
    t: &crate::gpu_frame::SharedTracer,
    doc: &ph2d_field::FieldDoc,
    surfaces: &ph2d_field_render::Surfaces<'_>,
    assente: bool,
) -> ph2d_field_gpu::trace::Pintado {
    let cam = ph2d_field_render::Orbit::default();
    crate::gpu_frame::paint(
        t,
        doc,
        &crate::smoke::sampled_registry(),
        &cam,
        &[crate::gpu_frame::tests_lampada(&cam)],
        surfaces,
        &ph2d_field_render::Presentation::of(ph2d_view_transform::Look::default()),
        [0, 0, 0, 0],
        None,
        LW,
        LH,
        assente,
    )
    .expect("o dispositivo pinta esta peça")
}

fn donos(postas: &[ph2d_field::FieldDoc]) -> ph2d_field_eval::owners::Owners {
    let cam = ph2d_field_render::Orbit::default();
    ph2d_field_eval::owners::Owners::new(
        postas,
        &crate::smoke::sampled_registry(),
        ph2d_field_render::hit_tolerance(
            cam.half_extent,
            f32::from(u16::try_from(LH).unwrap_or(u16::MAX)),
        ),
    )
}

/// ⭐⭐⭐⭐ **ACRESCENTAR UMA FORMA RECOMPILA AS SONDAS E MAIS NENHUMA ENTRADA DO PINTOR.**
///
/// Medido antes da cura (`diag_o_preco_de_uma_forma_nova`, `PH2D_PIPELINE_LOG`): a forma nova
/// recompilava o `pinta` (`~0,5`–`0,9 s`) e o `pinta_bordas` (`1,3`–`3,9 s`) — duas vezes, porque a
/// lei do dono mudava o texto dos dois e a fita da peça também.
///
/// ⛔ **O CONTROLO é o `assa_sondas`:** ele MARCHA a peça, logo uma estrutura nova TEM de o
/// recompilar. Sem ele, um gate cuja peça nova coincidisse com uma já compilada leria zero em toda a
/// linha e passaria com a cura apagada — a forma exacta do vácuo que o
/// `a_fita_inerte_faz_o_cache_acertar_na_peca_seguinte` pagou.
#[test]
#[ignore = "precisa de GPU"]
fn uma_forma_nova_recompila_as_sondas_e_nao_o_pintor() {
    let Some(t) = crate::gpu_frame::shared() else {
        println!("sem adaptador — saltado");
        return;
    };
    let (doc5, postas5) = toros(5);
    let mats5 = materiais(5, true);
    let donos5 = donos(&postas5);
    let _ = pinta(
        t,
        &doc5,
        &ph2d_field_render::Surfaces {
            all: &mats5,
            owners: Some(&donos5),
        },
        true,
    );
    const PINTOR: [&str; 4] = [
        "pinta",
        "pinta_bordas",
        "pinta_ricochete",
        "borra_ricochete",
    ];
    let antes: Vec<usize> = PINTOR.iter().map(|e| conta(t, e)).collect();
    let sondas_antes = conta(t, "assa_sondas");

    let (doc6, postas6) = toros(6);
    let mats6 = materiais(6, true);
    let donos6 = donos(&postas6);
    let _ = pinta(
        t,
        &doc6,
        &ph2d_field_render::Surfaces {
            all: &mats6,
            owners: Some(&donos6),
        },
        true,
    );
    assert_eq!(
        conta(t, "assa_sondas"),
        sondas_antes + 1,
        "CONTROLO: a forma nova não recompilou as sondas — a peça de 6 já estava no cache, ou o \
         ricochete não correu, e o resto deste gate mediria nada"
    );
    for (e, a) in PINTOR.iter().zip(&antes) {
        assert_eq!(
            conta(t, e),
            *a,
            "a forma nova RECOMPILOU o `{e}` — o texto dele voltou a depender da peça (a lei do \
             dono compilada por folha, ou a fita real numa entrada que não a lê). Era isto que \
             fazia uma caixa nova demorar segundos a aparecer"
        );
    }
}

/// ⭐⭐⭐⭐ **A PRIMEIRA COR DIFERENTE NÃO COMPILA NADA** — *«5 seg para mudar de cor»*.
///
/// Com todos os materiais iguais a peça não pede lei do dono ([`crate::materials`]); a primeira cor
/// diferente pede-a. Antes da cura essa passagem trocava o stub de uma folha pela lei compilada, e
/// o texto novo recompilava o pintor inteiro. Hoje a lei interpretada tem o MESMO texto com zero
/// folhas e com `n`.
///
/// ⛔ **O CONTROLO é a IMAGEM:** com donos e cores distintas o quadro TEM de mudar — senão o
/// segundo quadro podia estar a ignorar a lei, e «não compilou nada» seria verdade por a lei não
/// correr.
#[test]
#[ignore = "precisa de GPU"]
fn a_primeira_cor_diferente_nao_compila_nada() {
    let Some(t) = crate::gpu_frame::shared() else {
        println!("sem adaptador — saltado");
        return;
    };
    let (doc, postas) = toros(4);
    let iguais = materiais(4, false);
    let antes = pinta(
        t,
        &doc,
        &ph2d_field_render::Surfaces {
            all: &iguais,
            owners: None,
        },
        true,
    );
    let compilados = t.lock().expect("o traçador").compiled();
    let distintos = materiais(4, true);
    let lei = donos(&postas);
    let depois = pinta(
        t,
        &doc,
        &ph2d_field_render::Surfaces {
            all: &distintos,
            owners: Some(&lei),
        },
        true,
    );
    assert_ne!(
        antes.rgba, depois.rgba,
        "CONTROLO: as cores distintas não mudaram a imagem — a lei do dono não correu"
    );
    let agora = t.lock().expect("o traçador").compiled();
    assert_eq!(
        agora,
        compilados,
        "a primeira cor diferente compilou {} pipeline(s) — o texto do pintor voltou a mudar quando \
         a peça ganha donos",
        agora - compilados
    );
}

/// ⭐⭐⭐ **O QUADRO DIZ QUANTO FOI COMPILAÇÃO, e um quadro que não compilou diz zero.**
///
/// É o número que o laço da resolução desconta (`smoke_draw_thread`): sem ele um quadro que
/// compilou media `2`–`5 s` e mandava o movimento seguinte para o tamanho mais grosso.
///
/// ⛔ As DUAS metades: o quadro que compila reporta mais do que zero (senão o desconto é inerte e
/// o laço volta a ler a compilação), e o mesmo quadro repetido reporta EXACTAMENTE zero (senão o
/// desconto comeria custo de desenho a sério).
#[test]
#[ignore = "precisa de GPU"]
fn o_quadro_diz_quanto_foi_compilacao() {
    let Some(t) = crate::gpu_frame::shared() else {
        println!("sem adaptador — saltado");
        return;
    };
    let (doc, postas) = toros(7);
    let mats = materiais(7, true);
    let lei = donos(&postas);
    let surfaces = ph2d_field_render::Surfaces {
        all: &mats,
        owners: Some(&lei),
    };
    let primeiro = pinta(t, &doc, &surfaces, false);
    assert!(
        primeiro.compilado_ms > 0.0,
        "a peça de 7 toros compilou a marcha e o quadro reportou {} ms de compilação",
        primeiro.compilado_ms
    );
    let segundo = pinta(t, &doc, &surfaces, false);
    assert!(
        segundo.compilado_ms == 0.0,
        "o MESMO quadro repetido reportou {} ms de compilação — o desconto comeria custo de desenho",
        segundo.compilado_ms
    );
}

/// ⭐ **O DESCONTO está nos DOIS quadros da placa** — o pintor de material e o matcap. Censo de
/// texto porque a thread de desenho pede uma janela e um dispositivo; a lei que ele protege é a do
/// gate de cima.
#[test]
fn o_laco_da_resolucao_desconta_a_compilacao() {
    let fonte = include_str!("smoke_draw_thread.rs");
    let n = fonte
        .matches("t0.elapsed().as_secs_f64() * 1000.0 - pintura.compilado_ms")
        .count();
    assert_eq!(
        n, 2,
        "o desconto da compilação devia estar nos DOIS quadros da placa (material e matcap) e está \
         em {n}"
    );
}
