//! ⭐⭐⭐ **A ROTA de um sink que mistura em grupo, percorrida no PUMP** (doc 118 W3) — as
//! instâncias que o renderer recebe, e em que lista.
//!
//! ⚠️ Pela mesma razão do irmão `passe_no_pump_tests`: *um gate que chama a função em vez de
//! percorrer a rota afirma que a lei existe, nunca que o produto a usa.* O que decide a lista é um
//! `if` dentro do laço de sinks, e só o pump o alcança.

use super::*;
use ph2d_nodegraph::cook::EvalCtx;
use ph2d_nodegraph::effect::Effect;
use ph2d_nodegraph::node::{LoweringKind, NodeManifest, NodeOp, NodeTypeId, ParamSpec, PortSpec};
use ph2d_nodegraph::port::{Clock, Dim, Domain, PortType};

const INST_VEC2: PortType = PortType::new(Domain::Instances, Dim::Vec2, Clock::Frame);

/// Três linhas por ordem: uma IMAGEM, uma FORMA (`geometry_id 5`) e outra IMAGEM — a ordem é o que
/// a rota nova tem de preservar.
static SRC_MAN: NodeManifest = NodeManifest {
    id: NodeTypeId::of("motion.test.imagem_forma_imagem"),
    name: "motion.test.imagem_forma_imagem",
    inputs: &[],
    outputs: &[PortSpec {
        name: "out",
        ty: INST_VEC2,
    }],
    effect: Effect::Pure,
    clock: Clock::Frame,
    // O nó é ele próprio o sink — declara os dois params que o pump lê.
    params: &[
        ParamSpec {
            name: SINK_BLEND_PARAM,
            default: 0.0,
        },
        ParamSpec {
            name: SINK_BLEND_WITH_PARAM,
            default: 0.0,
        },
    ],
    lowerings: &[LoweringKind::Cpu],
};

struct Src;
impl NodeOp for Src {
    fn manifest(&self) -> &'static NodeManifest {
        &SRC_MAN
    }
    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        ctx.emit(
            Stream::new(3)
                .with("P", Column::Vec2(vec![[0.0, 0.0], [1.0, 0.0], [2.0, 0.0]]))
                .with("geometry_id", Column::Scalar(vec![0.0, 5.0, 0.0]))
                .with("texture_id", Column::Scalar(vec![3.0, 0.0, 4.0]))
                // Só a 1.ª imagem escolhe um PEDAÇO (o quadrante de cima-direita do recorte dela); a
                // 3.ª leva a identidade — as duas metades da composição.
                .with(
                    "uv_cell",
                    Column::Vec4(vec![
                        [0.5, 0.5, 0.5, 0.0],
                        [1.0, 1.0, 0.0, 0.0],
                        [1.0, 1.0, 0.0, 0.0],
                    ]),
                )
                .with(
                    "uv_rect",
                    Column::Vec4(vec![
                        [0.0, 0.0, 0.5, 0.5],
                        [0.0, 0.0, 1.0, 1.0],
                        [0.5, 0.5, 1.0, 1.0],
                    ]),
                ),
        );
    }
}

struct Ops;
impl OpResolver for Ops {
    fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
        (ty == SRC_MAN.id).then_some(&Src as &dyn NodeOp)
    }
}

/// Corre um quadro com o `Blend` em `tag` e devolve o pump.
fn um_quadro(tag: f32, com: f32) -> MotionCookPump {
    let mut g = Graph::new();
    let sink = g.add_node(SRC_MAN.name);
    g.set_param(sink, SINK_BLEND_PARAM, tag);
    g.set_param(sink, SINK_BLEND_WITH_PARAM, com);
    let mut pump = MotionCookPump::new();
    pump.define_a_lei(false);
    assert!(pump.pump(&g, &Ops, &[sink], 0, 0.0, [0.0, 0.0, 1.0, 1.0], [1.0, 1.0]));
    pump
}

/// ⛔ **O CONTROLO: sem mistura em grupo, a rota é a de sempre** — as imagens no passe de sprites,
/// a forma no Vello. `Normal` e `Subtract` (que o Vello não tem) ficam os dois aqui.
#[test]
fn sem_camada_as_imagens_ficam_no_passe_de_sprites() {
    for tag in [0.0, 2.0] {
        let p = um_quadro(tag, 0.0);
        assert_eq!(
            p.instances.len(),
            2,
            "tag {tag}: as duas imagens sao sprites"
        );
        assert_eq!(
            p.vector_instances.len(),
            1,
            "tag {tag}: so' a forma vai ao Vello"
        );
        assert!(!p.vector_instances[0].mistura.tem_camada());
    }
}

/// ⭐⭐⭐ **Com mistura em grupo, o sink INTEIRO vai ao Vello, pela ORDEM das linhas** — a imagem, a
/// forma e a imagem, cada uma carimbada com o grupo dela.
#[test]
fn com_camada_o_sink_inteiro_vai_ao_vello_pela_ordem() {
    for tag in [1.0, 3.0, 4.0] {
        let p = um_quadro(tag, 2.0);
        assert!(
            p.instances.is_empty(),
            "tag {tag}: nenhuma imagem pode ficar nas sprites"
        );
        let v = &p.vector_instances;
        assert_eq!(v.len(), 3, "tag {tag}: as tres linhas no Vello");
        // A ORDEM: imagem (textura 3) · forma (geometria 5) · imagem (textura 4).
        assert_eq!((v[0].geometry_id, v[0].texture_id), (0, 3));
        assert_eq!((v[1].geometry_id, v[1].texture_id), (5, 0));
        assert_eq!((v[2].geometry_id, v[2].texture_id), (0, 4));
        // O PEDAÇO compõe-se no recorte da 1.ª, e a 3.ª (identidade) chega inteira, ao bit.
        assert_eq!(v[0].atlas_uv, [0.25, 0.0, 0.5, 0.25]);
        assert_eq!(v[2].atlas_uv, [0.5, 0.5, 1.0, 1.0]);
        for vi in v {
            assert!(
                vi.mistura.tem_camada(),
                "tag {tag}: o grupo tem de ser carimbado"
            );
            assert_eq!(vi.mistura.com, BlendWith::Scene);
        }
    }
}

/// **O pedaço de um `uv_cell` compõe-se DENTRO do recorte** — a MESMA conta do `sprite.wgsl`.
#[test]
fn o_pedaco_compoe_dentro_do_recorte() {
    // A metade de cima-direita de um recorte que é o quadrante `[0,5 .. 1]²` do átlas.
    let r = crate::uv_do_pedaco([0.5, 0.5, 1.0, 1.0], [0.5, 0.5, 0.5, 0.0]);
    assert_eq!(r, [0.75, 0.5, 1.0, 0.75]);
    // A identidade devolve o rectângulo ao bit — mesmo quando `u0 + (u1 − u0)` erraria.
    let u = [0.1, 0.2, 0.7, 0.9];
    assert_eq!(
        crate::uv_do_pedaco(u, ph2d_render::RenderInstance::IDENTITY_UV_XFORM),
        u
    );
}
