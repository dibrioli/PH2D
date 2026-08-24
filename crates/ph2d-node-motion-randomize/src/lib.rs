#![forbid(unsafe_code)]
//! `motion.randomize` — **cada elemento um pouco diferente do vizinho**, no canal que o
//! artista escolhe (Enio, 2026-08-24: *«no random das partículas faltou rot, opacity, hue,
//! contraste, brilho, etc»*).
//!
//! ## ⚠️ A capacidade JÁ EXISTIA — o que faltava era alcançá-la
//!
//! Medido antes de escrever uma linha: a cadeia `value.instance_field(Random) →
//! motion.drive(<canal>)` **já entrega** variância por-elemento em Rotation, Opacity, Hue,
//! Saturation e Value, estável entre tiques (21 partículas, 21 valores distintos, 21/21
//! estáveis). ⇒ *isto não é capacidade nova; é a mesma resposta a um gesto.* Dizê-lo é o
//! ponto: um nó novo que duplicasse capacidade seria trabalho refeito, e este não duplica —
//! ele apaga três armadilhas que a composição tinha.
//!
//! **A armadilha 1 — o nome.** Quem procura «random» no palette não acha nada: a
//! aleatoriedade mora num modo do `value.instance_field`, cujo nome não a menciona.
//!
//! **A armadilha 2 — a identidade é OPT-IN.** O `key_by` daquele nó nasce em `Index`, e o
//! índice é *posição na lista*: num emissor a janela viva desliza, então todo valor troca
//! de dono quando a mais velha morre — a variação **cintila**. Aqui não há knob: a chave é
//! a coluna `id` quando ela existe, e o índice quando não existe.
//!
//! **A armadilha 3 — e é a que morde em silêncio — o MODO errado não faz nada.** Medido:
//! `Add` numa opacidade que já está saturada satura (21 partículas, **1** valor distinto);
//! `Multiply` numa rotação que vale zero fica zero (idem). O artista vê *«não está
//! funcionando»* e não tem como saber qual das duas escolhas era a certa.
//!
//! ## ⭐ A lei deste nó: uma variância é uma dispersão em torno do que já está lá, e o que
//! **«em torno»** significa depende do canal
//!
//! - Um **ÂNGULO** e uma **POSIÇÃO** dispersam-se **somando**: eles não têm zero natural de
//!   escala, e multiplicar um zero devolve zero para sempre.
//! - Uma **MAGNITUDE SEM TETO** (tamanho, brilho) dispersa-se **multiplicando**, simétrica
//!   em torno de `1`.
//! - ⭐ Uma **MAGNITUDE COM TETO** (opacidade, saturação) dispersa-se **só para BAIXO** — e
//!   esta metade da lei veio de uma MEDIÇÃO, não do gosto. Uma dispersão simétrica sobre uma
//!   alfa que já vale `1` atira metade dos sorteios contra o clamp: medido, **19** valores
//!   distintos em 40 elementos, metade colados no topo. *Não há «mais que totalmente
//!   opaco»*, e é por isso que a referência chama a este knob um `Opacity Random %` — uma
//!   REDUÇÃO. O gate `every_channel_actually_spreads_from_its_own_neutral` é quem o disse.
//! - O **MATIZ** é um ângulo que dá a volta, então soma — e ao contrário dos outros ele
//!   **não** precisa de clamp. ⚠️ **Mas a unidade dele NÃO é o grau:** o `rgb_to_hsv` da
//!   casa devolve o matiz em `[0,1)`, então uma volta inteira vale `1`. Multiplicar por
//!   `360` ali faria `amount = 0,01` rodar **três voltas e meia**, e o knob inteiro seria
//!   ruído — foi o que o gate mediu antes de isto shipar.
//!
//! É esta tabela que o nó sabe e que um `motion.drive` genérico não pode saber: o `drive`
//! recebe o modo do artista, e a escolha errada é indistinguível de um defeito.
//!
//! ⛔ **O «contraste» que o pedido nomeia NÃO entra, e o motivo é que ele não existe neste
//! nível.** Contraste é uma relação entre valores DENTRO de uma imagem; uma instância
//! carrega **uma** cor (`tint`), e o contraste de uma cor só é indefinido. O vizinho que
//! responde à mesma intenção é a **Saturation**, que shipa. Um contraste por-elemento pede
//! uma operação por-pixel na sprite, que é outra família (`fx.*`).
//!
//! `amount = 0` devolve a entrada **verbatim**, por ramo — o nó recém-largado não mexe em
//! nada. `Effect::Pure`, sem relógio: a mesma cena dá a mesma dispersão em todo scrub.

use ph2d_node_registry::{NodeRegistry, ParamUiHint, ParamWidget, RegistryError};
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::cook::EvalCtx;
use ph2d_nodegraph::effect::Effect;
use ph2d_nodegraph::node::{LoweringKind, NodeManifest, NodeOp, NodeTypeId, ParamSpec, PortSpec};
use ph2d_nodegraph::port::{Clock, Dim, Domain, PortType};

mod hash;
use hash::rand01;

const INST_VEC2: PortType = PortType::new(Domain::Instances, Dim::Vec2, Clock::Frame);

/// Os canais, na ordem em que o número os indexa.
///
/// ⚠️ **Apendar é a única operação legal** — um documento guarda o NÚMERO, e reordenar
/// trocaria o canal de toda cena já autorada, em silêncio.
pub const CHANNEL_LABELS: &[&str] = &[
    "Rotation",
    "Opacity",
    "Hue",
    "Saturation",
    "Brightness",
    "Size",
    "Position",
];

const CH_ROTATION: i32 = 0;
const CH_OPACITY: i32 = 1;
const CH_HUE: i32 = 2;
const CH_SATURATION: i32 = 3;
const CH_SIZE: i32 = 5;
const CH_POSITION: i32 = 6;

/// A volta inteira, em graus — a excursão que `amount = 1` compra na ROTAÇÃO.
const TURN: f32 = 360.0;

/// A volta inteira do MATIZ. ⚠️ **`1`, não `360`:** o `ph2d_color::rgb_to_hsv` devolve o
/// matiz em `[0,1)`, e o `hsv_to_rgba` envolve-o (`rem_euclid`) — quem soma aqui soma em
/// voltas, e é o próprio par de conversão que fixa a unidade.
const HUE_TURN: f32 = 1.0;

/// Pistas de hash. ⚠️ **Uma por EIXO nos canais de duas lanes**, senão a posição dispersa-se
/// na diagonal e o tamanho nunca deixa de ser quadrado.
const LANE_A: u32 = 0;
const LANE_B: u32 = 1;

/// The static contract of this node type (ADR-0031).
pub const MANIFEST: NodeManifest = NodeManifest {
    id: NodeTypeId::of("motion.randomize"),
    name: "motion.randomize",
    inputs: &[PortSpec {
        name: "in",
        ty: INST_VEC2,
    }],
    outputs: &[PortSpec {
        name: "out",
        ty: INST_VEC2,
    }],
    effect: Effect::Pure,
    clock: Clock::Frame,
    params: &[
        ParamSpec {
            name: "channel",
            default: 0.0,
        },
        // `0` = o nó não mexe em nada, por RAMO.
        ParamSpec {
            name: "amount",
            default: 0.0,
        },
        ParamSpec {
            name: "seed",
            default: 1.0,
        },
    ],
    lowerings: &[LoweringKind::Cpu],
};

/// **QUEM é cada elemento** — a coluna `id` quando ela existe, o índice quando não.
///
/// ⚠️ **Sem knob, de propósito.** O `value.instance_field` oferece a escolha e nasce em
/// `Index`; aqui não há caso legítimo para a posição na lista: onde há `id` ele é a
/// identidade, e onde não há o índice **é** a identidade. Um knob aqui seria um botão cuja
/// única posição errada cintila.
fn identity_at(input: &Stream, i: usize) -> u32 {
    match input.get("id") {
        Some(Column::Scalar(v)) => v.get(i).map_or(i as u32, |x| *x as u32),
        _ => i as u32,
    }
}

/// O sorteio bipolar de `[-1, 1)` na pista `lane`.
fn jitter(seed: u32, id: u32, lane: u32) -> f32 {
    rand01(seed, id, lane).mul_add(2.0, -1.0)
}

/// O fator MULTIPLICATIVO de uma magnitude SEM TETO — simétrico em torno de `1`, nunca
/// negativo.
///
/// ⚠️ **O piso é `0` e não um epsilon:** um tamanho zero é uma instância invisível, que é
/// uma coisa que o artista pode querer; um tamanho negativo é uma inversão de winding, que
/// não é.
fn factor(seed: u32, id: u32, lane: u32, amount: f32) -> f32 {
    jitter(seed, id, lane).mul_add(amount, 1.0).max(0.0)
}

/// O fator de uma magnitude COM TETO — uma redução, em `[1 − amount, 1]`.
///
/// ⚠️ **Unipolar de propósito** — ver a lei no doc do módulo: a alfa e a saturação nascem
/// no topo da faixa, e um sorteio simétrico gastaria metade do curso contra o clamp.
fn fade(seed: u32, id: u32, lane: u32, amount: f32) -> f32 {
    rand01(seed, id, lane).mul_add(-amount, 1.0).max(0.0)
}

struct MotionRandomize;

impl NodeOp for MotionRandomize {
    fn manifest(&self) -> &'static NodeManifest {
        &MANIFEST
    }

    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        let channel = ctx.param("channel").round() as i32;
        let amount = ctx.param("amount");
        let seed = ctx.param("seed").max(0.0).round() as u32;
        let input = ctx.input(0).clone();
        // ⚠️ **`0` sai por RAMO** — o nó recém-largado devolve a entrada byte a byte, e não
        // um `× 1,0` que em `f32` não é a identidade para todo valor.
        // `is_nan() ||` e não uma comparação negada: sobre um tipo parcialmente ordenado a
        // negação esconde o caso `NaN`, e um `amount` dirigido por fio pode sê-lo.
        if amount.is_nan() || amount <= 0.0 || input.count() == 0 {
            ctx.emit(input);
            return;
        }
        let n = input.count();
        let ids: Vec<u32> = (0..n).map(|i| identity_at(&input, i)).collect();
        let mut out = input.clone();
        match channel {
            CH_ROTATION => {
                let base = scalar_or(&input, "rot", n, 0.0);
                // SOMA, em graus: um ângulo não tem zero de escala.
                let v = base
                    .iter()
                    .zip(&ids)
                    .map(|(r, id)| jitter(seed, *id, LANE_A).mul_add(amount * TURN, *r))
                    .collect();
                out.set("rot", Column::Scalar(v));
            }
            CH_POSITION => {
                if let Some(Column::Vec2(p)) = input.get("P") {
                    // SOMA, em unidades de mundo, com uma pista por eixo.
                    let v = p
                        .iter()
                        .zip(&ids)
                        .map(|(q, id)| {
                            [
                                jitter(seed, *id, LANE_A).mul_add(amount, q[0]),
                                jitter(seed, *id, LANE_B).mul_add(amount, q[1]),
                            ]
                        })
                        .collect();
                    out.set("P", Column::Vec2(v));
                }
            }
            CH_SIZE => {
                // MULTIPLICA — um tamanho é uma magnitude. Uma pista por eixo, senão a
                // dispersão preserva a proporção e nada deixa de ser quadrado.
                let base = match input.get("size") {
                    Some(Column::Vec2(s)) => s.clone(),
                    _ => vec![[1.0, 1.0]; n],
                };
                let v = base
                    .iter()
                    .zip(&ids)
                    .map(|(s, id)| {
                        [
                            s[0] * factor(seed, *id, LANE_A, amount),
                            s[1] * factor(seed, *id, LANE_B, amount),
                        ]
                    })
                    .collect();
                out.set("size", Column::Vec2(v));
            }
            _ => {
                let mut t = match input.get("tint") {
                    Some(Column::Vec4(c)) => c.clone(),
                    _ => vec![[1.0, 1.0, 1.0, 1.0]; n],
                };
                for (ti, id) in t.iter_mut().zip(&ids) {
                    match channel {
                        CH_OPACITY => {
                            // REDUZ — ver a lei: não há «mais que totalmente opaco».
                            ti[3] = (ti[3] * fade(seed, *id, LANE_A, amount)).clamp(0.0, 1.0);
                        }
                        CH_HUE => {
                            // SOMA, em graus, e **sem clamp** — um matiz dá a volta.
                            let (h, s, v) = ph2d_color::rgb_to_hsv(*ti);
                            let h = jitter(seed, *id, LANE_A).mul_add(amount * HUE_TURN, h);
                            *ti = ph2d_color::hsv_to_rgba(h, s, v, ti[3]);
                        }
                        CH_SATURATION => {
                            let (h, s, v) = ph2d_color::rgb_to_hsv(*ti);
                            // REDUZ, pela mesma razão da alfa: a saturação tem topo.
                            let s = (s * fade(seed, *id, LANE_A, amount)).clamp(0.0, 1.0);
                            *ti = ph2d_color::hsv_to_rgba(h, s, v, ti[3]);
                        }
                        // **Brightness** (`4`) e todo número fora da escada: o `value` do
                        // HSV. ⚠️ Sem constante própria de propósito — ela seria um nome
                        // que nenhum ramo consulta, e o `_` é quem de facto o serve.
                        _ => {
                            let (h, s, v) = ph2d_color::rgb_to_hsv(*ti);
                            let v = v * factor(seed, *id, LANE_A, amount);
                            *ti = ph2d_color::hsv_to_rgba(h, s, v, ti[3]);
                        }
                    }
                }
                out.set("tint", Column::Vec4(t));
            }
        }
        ctx.emit(out);
    }
}

/// A coluna escalar `name`, ou `n` cópias de `fallback` quando ela não existe.
fn scalar_or(input: &Stream, name: &str, n: usize, fallback: f32) -> Vec<f32> {
    match input.get(name) {
        Some(Column::Scalar(v)) if v.len() == n => v.clone(),
        _ => vec![fallback; n],
    }
}

/// Register this node with the runtime registry.
pub fn register(reg: &mut NodeRegistry) -> Result<(), RegistryError> {
    reg.register(Box::new(MotionRandomize))?;
    reg.register_ui(
        MANIFEST.id,
        ph2d_node_registry::NodeUiManifest {
            display_name: "Randomize",
            category: ph2d_node_registry::NodeUiCategory::Transform,
            silhouette: ph2d_node_registry::NodeSilhouette::Rect,
        },
    );
    reg.register_param_ui(MANIFEST.id, PARAM_HINTS);
    Ok(())
}

static PARAM_HINTS: &[ParamUiHint] = &[
    ParamUiHint {
        param: "channel",
        label: "Channel",
        min: 0.0,
        max: 6.0,
        step: 1.0,
        widget: ParamWidget::Enum {
            labels: CHANNEL_LABELS,
        },
    },
    ParamUiHint {
        param: "amount",
        label: "Amount",
        min: 0.0,
        max: 1.0,
        step: 0.01,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "seed",
        label: "Seed",
        min: 0.0,
        max: 9999.0,
        step: 1.0,
        widget: ParamWidget::Seed,
    },
];

#[cfg(test)]
mod tests;
