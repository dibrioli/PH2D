#![forbid(unsafe_code)]
//! `rig.bones` — **os OSSOS de uma corrente**: uma corrente de JUNTAS entra, uma lista de
//! OSSOS sai, cada um pousado na junta de que ele PENDE.
//!
//! ## Porque ele existe (report do dono, 2026-09-19)
//!
//! *«Funciona como desejado se coloque pivot offset x em -2 mas o pivot fica na ponta dos ossos.
//! assim skeleton:bend coloca a base de um osso na ponta do outro. Contudo o mais correto seria
//! se tivesse o mesmo resultado colocando na base do osso.»*
//!
//! ⛔⛔⛔ **A causa não estava na forma: estava no que a corrente ENTREGA.** A lei do
//! `fk::resolve` (o leito de toda a família `rig.*`) é
//!
//! ```text
//! wrot[i] = wrot[pai] + rot[i]
//! P[i]    = P[pai] + len[i] · (cos wrot[i], sin wrot[i])
//! ```
//!
//! ⇒ o `len[i]` e o `rot[i]` que o elemento `i` carrega são os do osso que **CHEGA** a ele, e o
//! `P[i]` dele é a **PONTA** desse osso. *Um elemento é uma JUNTA, e o osso que ele carrega vive
//! ATRÁS dele.* Carimbar uma forma de osso em cada junta desenha-a, portanto, uma junta à frente
//! — medido na cadeia do dono (5 juntas, `length = 0,6`, `Bend = 30°`, `Root Angle = 0`):
//!
//! | elemento | junta `P[i]` | onde a forma acaba (pivô na cabeça) | a junta seguinte |
//! |---|---|---|---|
//! | 0 | `(0,000, 0,000)` | `(0,600, 0,000)` | `(0,520, 0,300)` |
//! | 1 | `(0,520, 0,300)` | `(1,039, 0,600)` | `(0,820, 0,820)` |
//! | 2 | `(0,820, 0,820)` | `(1,119, 1,339)` | `(0,820, 1,420)` |
//!
//! *A cadeia não ladrilha: cada peça sai pelo lado de fora do arco, e a última fica pendurada
//! para lá da corrente.* Com `Pivot Offset X = −2` ela ladrilha ao milésimo — e é por isso que o
//! dono viu o desenho certo com o pivô no sítio errado.
//!
//! ⭐⭐⭐ **A cura é dar-lhe o osso e não a junta.** Este nó devolve, por osso, o quadro em que
//! ele de facto vive: a **CABEÇA** (a posição do pai, que é o ponto em torno do qual o `rot`
//! daquele osso o roda) e o ângulo de MUNDO daquele osso. Com isso o pivô natural da
//! `Shape:Bone` — a cabeça, `[0, 2s]`, o valor de fábrica — cai exactamente onde o dono o quer, e
//! `Pivot Offset X` volta a ser o que o nome diz: um **desvio**, com `0` no sítio certo.
//!
//! ## Três decisões, cada uma com o defeito que ela evita
//!
//! 1. ⛔ **As raízes SAEM.** Uma corrente de `n` juntas tem `n − r` ossos (`r` = raízes), e a
//!    raiz é a única junta sem osso a chegar. Era ela a peça pendurada para fora da corrente nas
//!    fotos do dono. *Emitir um osso por junta obriga sempre uma peça a mentir.*
//! 2. ⛔⛔ **`parent`, `lrot` e `wrot` NÃO viajam.** A saída já não é uma corrente — é uma lista
//!    de carimbos —, e deixá-las passar punha um `rig.fk` a jusante a re-resolver uma árvore cujos
//!    índices já não existem. ⚠️ O `lrot` é o pior dos três: com ele presente e `rot == wrot`, o
//!    degrau 3 da escada do [`fk::local`] devolveria o ângulo **LOCAL** e o `resolve` reescrevia
//!    `rot` com ele — *a corrente ficava desenhada com os ângulos relativos, calada*. Sem as três,
//!    um `rig.fk` a jusante é a identidade exacta (sem `parent` ⇒ tudo raiz ⇒ `P` sobrevive, e
//!    sem `lrot` o `local` devolve o próprio `rot`).
//! 3. ⚠️ **`Index`/`Count` são RE-CONTADOS**, e só se já existiam. A população mudou de *juntas*
//!    para *ossos*: um `Count` a dizer `n` sobre `n − 1` linhas faria toda a normalização a
//!    jusante (falloff por índice, `motion.stagger`) endereçar o elemento errado. ⛔ Não é um
//!    knob como o `reindex` do `motion.cull`: ali o artista escolhe entre manter a cor de cada
//!    peça e re-numerar; aqui a mudança de população é **estrutural** e nunca opcional.
//!
//! ⚠️ **Numa corrente que NÃO é um rig ele é a identidade** (doc 39): sem coluna `parent` não há
//! árvore, e o stream sai como entrou — ⛔ e não vazio, que é o que uma leitura ingénua de
//! *«sem pais ⇒ sem ossos»* entregaria a quem largar este nó sobre um `motion.grid`.
//!
//! `Effect::Pure`, sem params (o quadro está todo nas colunas — um botão aqui seria uma segunda
//! fonte de verdade para o mesmo ângulo). Sem transcendentais: ele não calcula pose nenhuma,
//! só **escolhe** pontos que o `resolve` já calculou.

use ph2d_node_registry::{NodeRegistry, RegistryError};
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::cook::EvalCtx;
use ph2d_nodegraph::effect::Effect;
use ph2d_nodegraph::node::{LoweringKind, NodeManifest, NodeOp, NodeTypeId, PortSpec};
use ph2d_nodegraph::port::{Clock, Dim, Domain, PortType};

const INST_VEC2: PortType = PortType::new(Domain::Instances, Dim::Vec2, Clock::Frame);

/// The static contract of this node type (ADR-0031).
pub const MANIFEST: NodeManifest = NodeManifest {
    id: NodeTypeId::of("rig.bones"),
    name: "rig.bones",
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
    // Sem params — ver o cabeçalho.
    params: &[],
    lowerings: &[LoweringKind::Cpu],
};

/// As colunas do contrato do rig que este nó lê ou retira. ⚠️ **Os nomes são os do
/// `fk.rs` da família** (que vive copiado em cinco crates, por decisão registada lá): repeti-los
/// aqui é a mesma cópia de folha, e uma dependência nova entre nós de rig não se paga por três
/// literais.
const PARENT: &str = "parent";
/// Ver a decisão (2) do cabeçalho — é esta que faz um `rig.fk` a jusante reescrever `rot`.
const LROT: &str = "lrot";
const WROT: &str = "wrot";
const INDEX: &str = "Index";
const COUNT: &str = "Count";

/// **O quadro de cada osso.** Ver o cabeçalho do módulo para a lei e para a medição.
#[must_use]
pub fn bones(input: &Stream) -> Stream {
    let n = input.count();
    let Some(Column::Scalar(parent)) = input.get(PARENT) else {
        // Não é um rig: a identidade (doc 39).
        return input.clone();
    };
    // ⚠️ **O predicado é o do [`fk::resolve`], letra por letra:** um índice finito, não negativo
    // e que aponta para TRÁS é um pai; tudo o resto é raiz. *Duas leituras de «quem é raiz» que
    // divirjam põem um osso a nascer num sítio que a pose nunca visitou.*
    let mut ossos: Vec<(usize, usize)> = Vec::new();
    for (i, &pi) in parent.iter().enumerate().take(n) {
        #[expect(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            reason = "o indice do pai viaja num f32, como em toda a familia rig"
        )]
        let j = pi as usize;
        if pi >= 0.0 && pi.is_finite() && j < i {
            ossos.push((i, j));
        }
    }
    let pos = positions(input, n);
    let m = ossos.len();
    let mut out = Stream::new(m);
    for (name, col) in input.columns() {
        if name == PARENT || name == LROT || name == WROT {
            continue;
        }
        out.set(name.clone(), colhe(col, &ossos));
    }
    // ⭐ **A CABEÇA**: o osso `i` pende da junta `j`, e é ali que ele roda.
    out.set(
        "P",
        Column::Vec2(ossos.iter().map(|&(_, j)| pos[j]).collect()),
    );
    // ⭐⭐⭐ **O QUADRO DO OSSO, quando a corrente não o traz** (ordem do dono, 2026-09-21).
    //
    // Uma corrente RESOLVIDA carrega o `rot` e o `len` que o `fk::resolve` usou — e a lei dele é
    // `P[i] = P[pai] + len[i] · (cos rot[i], sin rot[i])`, logo o segmento `P[i] − P[pai]` **É**
    // esse quadro, ao arredondamento. Uma corrente que nunca passou por um solver — uma CORDA,
    // que declara `parent` porque a peça `i` está presa à `i − 1` — não traz nem um nem outro, e
    // sem eles toda peça carimbada sai sem rodar: *uma corda desenhada como um rosário de contas
    // em vez de um cordão*, que é o report do dono à letra.
    //
    // ⚠️⚠️ **DERIVA-SE só o que FALTA, e a razão não é timidez:** num rig o número já existe e
    // foi o solver que o calculou; re-derivá-lo por `atan2` trocaria um valor exacto por um com
    // erro de vírgula flutuante em toda a família, para não mudar resposta nenhuma. *Onde a
    // corrente responde, a resposta dela ganha.*
    derive_frame(&mut out, input, &ossos, &pos);
    // Ver a decisão (3) do cabeçalho.
    #[expect(
        clippy::cast_precision_loss,
        reason = "Index/Count sao f32 em todo o modulo"
    )]
    if input.get(INDEX).is_some() {
        out.set(
            INDEX,
            Column::Scalar((0..m).map(|i| i as f32).collect::<Vec<_>>()),
        );
    }
    #[expect(
        clippy::cast_precision_loss,
        reason = "Index/Count sao f32 em todo o modulo"
    )]
    if input.get(COUNT).is_some() {
        out.set(COUNT, Column::Scalar(vec![m as f32; m]));
    }
    out
}

/// O ângulo de MUNDO de cada osso, na convenção do [`fk::resolve`] da família.
const ROT: &str = "rot";
/// O comprimento de cada osso, na mesma convenção.
const LEN: &str = "len";

/// **O quadro que a corrente não trouxe** — ver a chamada, que é onde a lei está escrita.
///
/// ⚠️ **As duas metades são independentes de propósito:** uma corrente pode trazer o comprimento
/// e não o ângulo (ou o contrário), e escrever as duas em bloco faria a presença de uma decidir
/// pela outra.
fn derive_frame(out: &mut Stream, input: &Stream, ossos: &[(usize, usize)], pos: &[[f32; 2]]) {
    let delta = |&(i, j): &(usize, usize)| [pos[i][0] - pos[j][0], pos[i][1] - pos[j][1]];
    if input.get(ROT).is_none() {
        out.set(
            ROT,
            Column::Scalar(
                ossos
                    .iter()
                    .map(|b| {
                        let d = delta(b);
                        // ⚠️ Duas juntas no MESMO ponto não têm direcção; `atan2(0, 0)` devolve
                        // `0`, que é o valor que uma peça de comprimento nulo desenha na mesma.
                        //
                        // ⛔⛔⛔ **E O RESULTADO VAI EM GRAUS, que é a unidade de ÂNGULO desta
                        // casa** — report do dono (2026-09-21, foto): *«a rot não acontece»*.
                        // A 1.ª redacção desta lei devolvia o `atan2` CRU, e o desenho lê o `rot`
                        // em graus e converte na borda (`ph2d_eval_motion::lower`: *«the `rot`
                        // column is in **degrees** — the app's authored-angle unit … radians live
                        // nowhere in the Motion authoring surface»*), nas DUAS rotas. Medido na
                        // corda do dono: o 1.º segmento saía `−1,999` e desenhava-se a **−2°** —
                        // vinte peças praticamente deitadas, que é a foto à letra.
                        //
                        // ⚠️⚠️ **E as fixturas DESTA crate já o diziam:** a `corrente` constrói a
                        // cadeia com `dir(graus)`, e o contrato do rig (`fk.rs`) chama ao `rot`
                        // *«o ângulo de MUNDO do osso»*. A lei nova era a única coisa da família a
                        // falar noutra unidade — e o gate que a cobria nasceu na MESMA wave e
                        // herdou o engano (ele exigia `FRAC_PI_2`).
                        d[1].atan2(d[0]).to_degrees()
                    })
                    .collect::<Vec<_>>(),
            ),
        );
    }
    if input.get(LEN).is_none() {
        out.set(
            LEN,
            Column::Scalar(
                ossos
                    .iter()
                    .map(|b| {
                        let d = delta(b);
                        d[0].hypot(d[1])
                    })
                    .collect::<Vec<_>>(),
            ),
        );
    }
}

/// As posições do stream (ausentes → a origem), do tamanho declarado.
fn positions(s: &Stream, n: usize) -> Vec<[f32; 2]> {
    match s.get("P") {
        Some(Column::Vec2(v)) if v.len() == n => v.clone(),
        _ => vec![[0.0, 0.0]; n],
    }
}

/// Uma coluna re-amostrada pelo ELEMENTO de cada osso (o filho, que é quem carrega o `rot`, o
/// `len` e tudo o que o artista pendurou nele) — irmã do `gather_col` do `motion.cull`.
fn colhe(col: &Column, ossos: &[(usize, usize)]) -> Column {
    fn take<T: Clone>(v: &[T], ossos: &[(usize, usize)]) -> Vec<T> {
        ossos.iter().map(|&(i, _)| v[i].clone()).collect()
    }
    match col {
        Column::Scalar(v) => Column::Scalar(take(v, ossos)),
        Column::Vec2(v) => Column::Vec2(take(v, ossos)),
        Column::Vec3(v) => Column::Vec3(take(v, ossos)),
        Column::Vec4(v) => Column::Vec4(take(v, ossos)),
    }
}

struct RigBones;

impl NodeOp for RigBones {
    fn manifest(&self) -> &'static NodeManifest {
        &MANIFEST
    }

    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        let out = bones(ctx.input(0));
        ctx.emit(out);
    }
}

/// Register this node with the runtime registry. Called (via codegen) from
/// `ph2d-node-registry-init::register_all_nodes`.
pub fn register(reg: &mut NodeRegistry) -> Result<(), RegistryError> {
    reg.register(Box::new(RigBones))?;
    reg.register_ui(
        MANIFEST.id,
        ph2d_node_registry::NodeUiManifest {
            display_key: "node.rig.bones.name",
            category: ph2d_node_registry::NodeUiCategory::Transform,
            silhouette: ph2d_node_registry::NodeSilhouette::Rect,
        },
    );
    Ok(())
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
