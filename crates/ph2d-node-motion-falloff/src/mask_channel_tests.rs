//! Os gates do [`super::MASK_CHANNEL`] — o canal que a máscara escreve (doc 89, folha 05).

use super::*;
use ph2d_nodegraph::cook::{Cook, OpResolver};
use ph2d_nodegraph::graph::{Edge, Graph};

/// Uma fileira de cinco pontos em X, de `−4` a `4` — atravessa um campo Linear de
/// raio 4 de ponta a ponta, então os cinco pesos são todos diferentes.
static SRC_MAN: NodeManifest = NodeManifest {
    id: NodeTypeId::of("motion.falloff.test.row"),
    name: "motion.falloff.test.row",
    inputs: &[],
    outputs: &[PortSpec {
        name: "out",
        ty: INST_VEC2,
    }],
    effect: Effect::Pure,
    clock: Clock::Frame,
    params: &[],
    lowerings: &[LoweringKind::Cpu],
};

struct Row;
impl NodeOp for Row {
    fn manifest(&self) -> &'static NodeManifest {
        &SRC_MAN
    }
    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        let p: Vec<[f32; 2]> = (0..5).map(|i| [i as f32 * 2.0 - 4.0, 0.0]).collect();
        ctx.emit(Stream::new(5).with("P", Column::Vec2(p)));
    }
}

struct Ops;
impl OpResolver for Ops {
    fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
        match ty {
            t if t == SRC_MAN.id => Some(&Row),
            t if t == MANIFEST.id => Some(&MotionFalloff),
            _ => None,
        }
    }
}

/// Coze uma cadeia `row → falloff(canal[0]) → falloff(canal[1]) → …` e devolve as
/// duas colunas de máscara da saída.
fn masks(channels: &[i32]) -> (Option<Vec<f32>>, Option<Vec<f32>>) {
    let mut g = Graph::new();
    let mut head = g.add_node("motion.falloff.test.row");
    for ch in channels {
        let f = g.add_node("motion.falloff");
        g.connect(Edge {
            from: (head, 0),
            to: (f, 0),
            delayed: false,
        })
        .unwrap();
        // Um campo Linear: uma rampa de 0 a 1 ao longo de X, sem simetria que
        // pudesse fazer dois pontos distintos darem o mesmo peso.
        g.set_param(f, "shape", 2.0);
        g.set_param(f, "curve", 0.0);
        g.set_param(f, "radius", 4.0);
        g.set_param(f, MASK_CHANNEL, *ch as f32);
        head = f;
    }
    let mut cook = Cook::new();
    let out = cook.cook(&g, &Ops, head, 0.0).unwrap();
    let st = out[0].as_stream();
    let col = |n: &str| match st.get(n) {
        Some(Column::Scalar(v)) => Some(v.clone()),
        _ => None,
    };
    (col("falloff"), col(MASK_CHANNEL_Y_COLUMN))
}

/// **O DEFAULT ESCREVE A COLUNA DE SEMPRE, e não cunha a outra.**
#[test]
fn the_default_channel_writes_falloff_and_mints_nothing_else() {
    let (f, fy) = masks(&[0]);
    assert_eq!(
        f,
        Some(vec![0.0, 0.25, 0.5, 0.75, 1.0]),
        "a rampa linear de sempre"
    );
    assert_eq!(fy, None, "o canal Y não pode nascer sozinho");
}

/// **O CANAL Y ESCREVE O SEU E NÃO TOCA NO OUTRO** — a lei que faz dois campos
/// serem dois campos.
///
/// ⚠️ O oráculo mede as DUAS colunas. Um nó que escrevesse `falloff_y` **e**
/// deixasse um `falloff` de `1.0` atrás passaria por *"o canal Y funciona"* e
/// quebraria todo modificador a jusante que já tivesse uma máscara.
#[test]
fn the_y_channel_writes_its_own_column_and_leaves_falloff_untouched() {
    let (f, fy) = masks(&[1]);
    assert_eq!(f, None, "a coluna `falloff` não pode ser criada aqui");
    assert_eq!(fy, Some(vec![0.0, 0.25, 0.5, 0.75, 1.0]));
}

/// **OS DOIS CANAIS SÃO O MESMO CAMPO** — só o destino muda.
#[test]
fn the_two_channels_compute_the_same_field() {
    let (f, _) = masks(&[0]);
    let (_, fy) = masks(&[1]);
    assert_eq!(f, fy, "mudar de canal não pode mudar a lei do campo");
}

/// **DOIS CAMPOS NO MESMO CANAL COMPÕEM; EM CANAIS DIFERENTES, NÃO.**
///
/// ⚠️ É o gate que separa *"o canal existe"* de *"o canal é independente"*. Dois
/// Lineares iguais no mesmo canal dão o quadrado da rampa; um em cada canal deixa
/// as duas rampas intactas, lado a lado.
#[test]
fn fields_multiply_inside_a_channel_and_never_across_them() {
    let (f2, _) = masks(&[0, 0]);
    let squared: Vec<f32> = [0.0f32, 0.25, 0.5, 0.75, 1.0]
        .iter()
        .map(|v| v * v)
        .collect();
    assert_eq!(f2, Some(squared), "no mesmo canal os campos multiplicam");

    let (f, fy) = masks(&[0, 1]);
    let ramp = Some(vec![0.0, 0.25, 0.5, 0.75, 1.0]);
    assert_eq!(f, ramp, "…e em canais diferentes cada um fica inteiro");
    assert_eq!(fy, ramp);
}

/// **O KNOB ESTÁ PINTADO e o device tem uma VARIANTE por canal.**
#[test]
fn the_knob_is_painted_and_the_device_has_one_variant_per_channel() {
    let h = PARAM_HINTS
        .iter()
        .find(|h| h.param == MASK_CHANNEL)
        .expect("o Channel tem de estar pintado");
    match h.widget {
        ParamWidget::Enum { labels } => assert_eq!(labels, &["Falloff", "Falloff Y"]),
        _ => panic!("o Channel é um Enum"),
    }
    let base = GPU_KERNEL.resolve(&|_| 0.0);
    let y = GPU_KERNEL.resolve(&|n| {
        if n == MASK_CHANNEL {
            MASK_CHANNEL_Y as f32
        } else {
            0.0
        }
    });
    let writes = |k: &GpuKernel| {
        k.bindings
            .iter()
            .find(|b| matches!(b.access, ColumnAccess::ReadWrite))
            .map(|b| b.column)
    };
    assert_eq!(writes(base), Some("falloff"));
    assert_eq!(writes(y), Some(MASK_CHANNEL_Y_COLUMN));
    // ⚠️ E o CAMPO é o mesmo nos dois: a única diferença permitida entre os corpos
    // é o nome do acessor. Se alguém reescrever a lei numa das variantes, esta
    // substituição deixa de as igualar.
    assert_eq!(
        base.wgsl.replace("_falloff(", "_CH("),
        y.wgsl.replace("_falloff_y(", "_CH("),
        "as duas variantes têm de ser o MESMO campo"
    );
}
