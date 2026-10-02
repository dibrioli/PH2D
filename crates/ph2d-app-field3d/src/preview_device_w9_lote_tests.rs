//! ⭐⭐⭐⭐ **UMA FORMA NOVA COMPILA NUM LOTE SÓ, E A MÃO A MEXER NÃO ASSA AS SONDAS** — os gates do
//! report do dono de 2026-10-01: *«melhor mas ainda com delay de 1 ou 2 segundos»*.
//!
//! Medido na cena dele (`diag_o_preco_de_uma_forma_nova_ao_lado_do_no`, os quatro nós de toro):
//! uma caixa nova compilava a marcha (`309 ms`), DEPOIS o céu no tempo (`476`) e, no assente, as
//! sondas (`418`) — `~1,4 s`; e cada quadro de ARRASTO re-assava as sondas (`129 ms` de placa num
//! quadro de `171`). Duas leis, e as duas são invisíveis a toda régua de VALOR do quadro assente,
//! logo cada gate afirma a CONTA:
//!
//! 1. **o lote do quadro leva os OUTROS passes que levam a fita** (`ph2d_field_gpu::trace_marcha_com`):
//!    o quadro de movimento de uma estrutura nova compila também o céu e as sondas, e o assente
//!    seguinte NÃO compila nada;
//! 2. **um quadro que não pode esperar não assa as sondas** (`FieldPipelines::sondas_a_mexer`): lê
//!    as guardadas se a grade delas se deslocou menos do que a tolerância, e vai sem ricochete se
//!    não; o assente re-assa.

use super::*;

/// A MESMA peça de toros do irmão, com o do MEIO deslocado em `y` — um arrasto.
fn toros_com_o_do_meio_em(n: usize, dy: f32) -> ph2d_field::FieldDoc {
    use ph2d_field::{FieldDoc, NodeId, Primitive, Xform};
    let meio = n / 2;
    #[allow(clippy::cast_precision_loss)]
    let mut nos: Vec<ph2d_field::Node> = (0..n)
        .map(|i| {
            ph2d_field_eval::leaf(
                Primitive::Torus {
                    major: 0.16,
                    minor: 0.06,
                },
                Xform::at(
                    i as f32 * 0.3 - 0.15 * (n as f32 - 1.0),
                    if i == meio { dy } else { 0.0 },
                    0.0,
                ),
            )
        })
        .collect();
    #[allow(clippy::cast_possible_truncation)]
    nos.push(ph2d_field::Node {
        xform: Xform::IDENTITY,
        kind: ph2d_field::NodeKind::Combine {
            op: ph2d_field::Op::Union(ph2d_field::Blend::Sharp),
            children: (0..n as u32).map(NodeId).collect(),
        },
        mods: Vec::new(),
        verb: None,
    });
    #[allow(clippy::cast_possible_truncation)]
    FieldDoc::new(nos, NodeId(n as u32)).expect("a peça")
}

/// ⭐⭐⭐⭐ **A FORMA NOVA COMPILA NO 1.º QUADRO, E O ASSENTE SEGUINTE NÃO COMPILA NADA.**
///
/// ⛔ **O CONTROLO é o próprio quadro de movimento:** ele TEM de ter compilado (a estrutura é nova),
/// senão a peça coincidia com uma já vista e o «assente não compila» leria zero a medir nada.
#[test]
#[ignore = "precisa de GPU"]
fn uma_forma_nova_compila_num_lote_so() {
    let Some(t) = crate::gpu_frame::shared() else {
        println!("sem adaptador — saltado");
        return;
    };
    let mats = materiais(9, false);
    let surfaces = ph2d_field_render::Surfaces {
        all: &mats,
        owners: None,
    };
    // Aquece a peça de 8 (o pintor, as sondas e o céu da família).
    let (oito, _) = toros(8);
    let _ = pinta(t, &oito, &surfaces, false);
    let _ = pinta(t, &oito, &surfaces, true);
    let (nove, _) = toros(9);
    let antes = t.lock().expect("o traçador").compiled();
    let sondas_antes = conta(t, "assa_sondas");
    let ceu_antes = conta(t, "ceu_tempo_grava");
    let rodadas_antes = t.lock().expect("o traçador").rodadas_de_compilacao();
    let mexe = pinta(t, &nove, &surfaces, false);
    let depois_do_movimento = t.lock().expect("o traçador").compiled();
    assert!(
        depois_do_movimento > antes && mexe.compilado_ms > 0.0,
        "CONTROLO: o quadro de movimento de uma estrutura NOVA tem de compilar — a peça de 9 toros \
         já estava no cache, e o gate mediria nada"
    );
    assert_eq!(
        (conta(t, "assa_sondas"), conta(t, "ceu_tempo_grava")),
        (sondas_antes + 1, ceu_antes + 1),
        "o lote do 1.º quadro não levou as sondas e o céu da peça nova — eles compilariam em fila, \
         no quadro seguinte"
    );
    // ⭐ **UMA rodada** — a marcha, o céu e as sondas no MESMO lote. ⚠️ Sem esta metade o gate
    // ficava verde com o céu fora do lote: ele compilava na mesma, numa 2.ª rodada, e as contas de
    // pipelines acima saíam iguais (a mutação L7 sobreviveu por isso).
    assert_eq!(
        t.lock().expect("o traçador").rodadas_de_compilacao() - rodadas_antes,
        1,
        "o quadro da forma nova compilou em mais de UMA rodada — os passes compilam em fila"
    );
    let assente = pinta(t, &nove, &surfaces, true);
    assert_eq!(
        t.lock().expect("o traçador").compiled(),
        depois_do_movimento,
        "o assente a seguir à forma nova compilou — o lote do 1.º quadro esqueceu um passe"
    );
    assert!(
        assente.compilado_ms.abs() < f64::EPSILON,
        "o assente diz {:.1} ms de compilação",
        assente.compilado_ms
    );
}

/// ⭐⭐⭐⭐ **ARRASTAR NÃO ASSA AS SONDAS, E O RICOCHETE CONTINUA LÁ — o assente re-assa.**
///
/// Três metades, e cada uma reprova uma mutação diferente:
/// 1. o quadro de MOVIMENTO de uma edição não assa (a conta não sobe);
/// 2. ele LÊ as guardadas: a imagem difere da do mesmo quadro sem sondas guardadas (que vai sem
///    ricochete) — sem isto, «não assa» e «não tem ricochete» leem-se iguais;
/// 3. o ASSENTE seguinte re-assa (a conta sobe UMA vez) — a chave guardada não foi reescrita.
///
/// ⛔ **O CONTROLO da fixtura:** a edição desloca a grade MENOS do que a tolerância — é a condição
/// de as guardadas servirem. ⚠️ Ela não pode ser «a bola não muda»: num arrasto ela muda SEMPRE.
#[test]
#[ignore = "precisa de GPU"]
fn arrastar_nao_assa_as_sondas_e_o_assente_re_assa() {
    let Some(t) = crate::gpu_frame::shared() else {
        println!("sem adaptador — saltado");
        return;
    };
    let reg = crate::smoke::sampled_registry();
    let (a, b) = (
        toros_com_o_do_meio_em(5, 0.0),
        toros_com_o_do_meio_em(5, 0.03),
    );
    let celulas = deslocamento_em_celulas(&reg, &a, &b);
    assert!(
        celulas > 0.0 && celulas < ph2d_field_gpu::sondas_na_placa::TOLERANCIA_EM_CELULAS,
        "CONTROLO: a fixtura deslocou a grade {celulas:.3} células — fora de (0, tolerância), ela não \
         exercita as sondas guardadas de uma peça DIFERENTE"
    );
    let mats = materiais(5, false);
    let surfaces = ph2d_field_render::Surfaces {
        all: &mats,
        owners: None,
    };
    let _ = pinta(t, &a, &surfaces, false);
    let _ = pinta(t, &a, &surfaces, true);
    let assadas = t.lock().expect("o traçador").sondas_assadas();
    // ⚠️ As duas imagens comparadas partem do MESMO céu (sem histórico): o 2.º quadro de `b`
    // herdaria o do 1.º, e a diferença seria do céu e não das sondas — a 1.ª redacção deste gate
    // ficou VERDE por isso, a medir nada.
    t.lock().expect("o traçador").esquece_o_ceu();
    let com_guardadas = pinta(t, &b, &surfaces, false).rgba;
    assert_eq!(
        t.lock().expect("o traçador").sondas_assadas(),
        assadas,
        "o quadro de movimento de um arrasto assou as sondas (`~130 ms` na cena do dono)"
    );
    t.lock().expect("o traçador").esquece_as_sondas();
    t.lock().expect("o traçador").esquece_o_ceu();
    let sem_guardadas = pinta(t, &b, &surfaces, false).rgba;
    assert!(
        com_guardadas != sem_guardadas,
        "o quadro de movimento com sondas guardadas saiu igual ao sem elas — ele não leu as \
         guardadas, foi sem ricochete"
    );
    // Repõe as guardadas da pose A e confirma que o assente da B as re-assa.
    let _ = pinta(t, &a, &surfaces, true);
    let antes = t.lock().expect("o traçador").sondas_assadas();
    let _ = pinta(t, &b, &surfaces, false);
    let _ = pinta(t, &b, &surfaces, true);
    assert_eq!(
        t.lock().expect("o traçador").sondas_assadas(),
        antes + 1,
        "o assente depois do arrasto não re-assou as sondas — a chave guardada foi reescrita por um \
         quadro que não as assou"
    );
}

/// ⭐⭐⭐⭐ **LONGE DE MAIS, O QUADRO DE MOVIMENTO VAI SEM RICOCHETE — e continua sem assar.**
///
/// Acima da [`ph2d_field_gpu::sondas_na_placa::TOLERANCIA_EM_CELULAS`] as velhas erram mais do que
/// ir sem ricochete (a tabela está no doc da constante) ⇒ o quadro sai IGUAL ao de sem sondas
/// guardadas, e a conta não sobe. ⛔ O CONTROLO é a fixtura estar de facto acima da tolerância.
#[test]
#[ignore = "precisa de GPU"]
fn longe_de_mais_o_movimento_vai_sem_ricochete() {
    let Some(t) = crate::gpu_frame::shared() else {
        println!("sem adaptador — saltado");
        return;
    };
    let reg = crate::smoke::sampled_registry();
    let (a, b) = (
        toros_com_o_do_meio_em(5, 0.0),
        toros_com_o_do_meio_em(5, 0.3),
    );
    let celulas = deslocamento_em_celulas(&reg, &a, &b);
    assert!(
        celulas > ph2d_field_gpu::sondas_na_placa::TOLERANCIA_EM_CELULAS,
        "CONTROLO: a fixtura deslocou a grade só {celulas:.3} células"
    );
    let mats = materiais(5, false);
    let surfaces = ph2d_field_render::Surfaces {
        all: &mats,
        owners: None,
    };
    let _ = pinta(t, &a, &surfaces, false);
    let _ = pinta(t, &a, &surfaces, true);
    let assadas = t.lock().expect("o traçador").sondas_assadas();
    // ⚠️ As duas imagens comparadas partem do MESMO céu (sem histórico): o 2.º quadro de `b`
    // herdaria o do 1.º, e a diferença seria do céu e não das sondas — a 1.ª redacção deste gate
    // ficou VERDE por isso, a medir nada.
    t.lock().expect("o traçador").esquece_o_ceu();
    let com_guardadas = pinta(t, &b, &surfaces, false).rgba;
    assert_eq!(
        t.lock().expect("o traçador").sondas_assadas(),
        assadas,
        "o quadro de movimento assou as sondas"
    );
    t.lock().expect("o traçador").esquece_as_sondas();
    t.lock().expect("o traçador").esquece_o_ceu();
    let sem_guardadas = pinta(t, &b, &surfaces, false).rgba;
    assert!(
        com_guardadas == sem_guardadas,
        "longe de mais, o quadro de movimento leu as sondas velhas em vez de ir sem ricochete"
    );
}

/// O deslocamento da grade das sondas entre duas peças, em CÉLULAS — a mesma conta do
/// `ChaveDasSondas::deslocamento_em_celulas`, refeita aqui a partir das bolas (é o CONTROLO das
/// fixturas, não a lei).
fn deslocamento_em_celulas(
    reg: &ph2d_field_eval::hybrid::Registry,
    a: &ph2d_field::FieldDoc,
    b: &ph2d_field::FieldDoc,
) -> f32 {
    let (ba, bb) = (
        ph2d_field_eval::bounds::bounding_ball(a, reg).expect("a bola"),
        ph2d_field_eval::bounds::bounding_ball(b, reg).expect("a bola"),
    );
    let margem = ph2d_field_render::probes::PROBE_MARGIN;
    #[allow(clippy::cast_precision_loss)]
    let celula = 2.0 * ba.radius.max(bb.radius) * margem
        / (ph2d_field_render::probes::PROBE_GRID - 1) as f32;
    let dc = (0..3)
        .map(|i| (ba.center[i] - bb.center[i]).abs())
        .fold(0.0f32, f32::max);
    (dc + (ba.radius - bb.radius).abs() * margem) / celula
}
