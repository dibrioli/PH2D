//! ⭐⭐⭐ **`source.table` — uma linha do ficheiro é um ELEMENTO na tela.**
//!
//! É a metade do padrão-ouro que o *Import CSV* do Blender (4.3+) e o *Table Import* do Houdini
//! entregam: cada linha vira um ponto, cada coluna numérica vira um atributo com o nome do
//! cabeçalho. 300 linhas ⇒ 300 barras.
//!
//! # ⚠️ A OUTRA metade é outro nó, e a razão é da indústria
//!
//! A Adobe **não conseguiu** animar a partir de CSV/JSON e inventou o `.mgjson` porque
//! *"JSON, CSV e TSV só podem conter valores estáticos"*. O que ela descobriu é que uma tabela
//! responde a **duas perguntas incompatíveis**: *cada linha é um ELEMENTO* (aqui) ou *cada linha
//! é um MOMENTO* (o [`ph2d_node_value_table`](../../ph2d-node-value-table), que amostra no
//! playhead). Quem só faz a primeira não anima telemetria; quem só faz a segunda não desenha um
//! gráfico de barras.
//!
//! ⛔ **E são dois NÓS, não um interruptor.** A taxonomia deste motor já responde à pergunta —
//! os `source.*` entregam elementos e os `value.*` entregam um número. Um enum faria a
//! CARDINALIDADE da saída mudar por baixo do consumidor, e nenhum nó a jusante pode ser escrito
//! para os dois.
//!
//! # ⛔ O que este nó deliberadamente NÃO faz
//!
//! O Houdini deixa um atributo abranger **N colunas** (é assim que `x,y,z` viram a posição `P`).
//! Aqui isso seria construir o que a composição já exprime: o `value.attribute` lê **qualquer
//! coluna pelo nome** — é o desenho declarado dele, *"a named attribute and not an enum"* — e o
//! `motion.drive` leva-a a qualquer canal. Duas ligações fazem o que lá é um param.

//! # ⛔ NÃO HÁ TECTO DE LINHAS, e isso foi MEDIDO (§0.0)
//!
//! Antes de escrever um `MAX_ROWS` mediu-se o que ele limitaria
//! ([`examples/probe_frame_cost.rs`](../examples/probe_frame_cost.rs) e
//! [`ph2d-table/examples/probe_cost.rs`](../../ph2d-table/examples/probe_cost.rs)):
//!
//! | tabela | LER (uma vez, por ficheiro) | COPIAR (todo quadro) |
//! |---|---|---|
//! | 10 000 × 16 | `7,9 ms` | `0,06 ms` — **`0,4 %`** de um quadro |
//! | 100 000 × 16 | `68,8 ms` | `0,67 ms` — `4,0 %` |
//! | 1 000 000 × 16 | `620 ms` | `6,90 ms` — `41,3 %` |
//!
//! ⇒ **um milhão de linhas ainda cabe num quadro.** A leitura é uma vez por ficheiro (o cache
//! da shell), não por quadro, então os `620 ms` são um soluço ao ESCOLHER o ficheiro, não uma
//! taxa. Um tecto aqui seria um palpite a limitar um recurso que não está apertado.
//!
//! ⚠️ **O que aperta primeiro é o RENDERER** — desenhar um milhão de instâncias —, e esse
//! limite é dele. §0.0: *nunca deixe o caminho mais lento definir o teto do outro.*

use ph2d_node_registry::{NodeRegistry, ParamUiHint, ParamWidget, RegistryError};
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::cook::EvalCtx;
use ph2d_nodegraph::effect::Effect;
use ph2d_nodegraph::node::{NodeManifest, NodeOp, NodeTypeId, ParamSpec, PortSpec};
use ph2d_nodegraph::port::{Clock, Dim, Domain, PortType};

const INST_VEC2: PortType = PortType::new(Domain::Instances, Dim::Vec2, Clock::Frame);

/// **O ficheiro é um TEXT PARAM** — o padrão que deixa um param não-`f32` existir sem tocar o
/// contrato congelado (§6), e o mesmo do `audio.bands`.
///
/// ⚠️ **É um CAMINHO, e nunca o CONTEÚDO.** O texto do grafo é lido por LINHA
/// (`MotionDoc::from_text` parte em `\n`), então colar uma tabela aqui **corromperia o
/// documento** — o projeto deixaria de abrir. O caminho é estado de documento; o preço (mover o
/// ficheiro quebra o vínculo) é o *missing footage* que todo DCC tem e sabe nomear.
pub const FILE_KEY: &str = "file";

pub mod param {
    /// Quanto espaço horizontal separa duas linhas.
    pub const SPACING: &str = "spacing";
}

pub const MANIFEST: NodeManifest = NodeManifest {
    id: NodeTypeId::of("source.table"),
    name: "source.table",
    inputs: &[],
    outputs: &[PortSpec {
        name: "out",
        ty: INST_VEC2,
    }],
    // A tabela não é função do playhead: o mesmo ficheiro dá o mesmo desenho em todo instante.
    effect: Effect::Pure,
    clock: Clock::Frame,
    params: &[ParamSpec {
        name: param::SPACING,
        default: 0.25,
    }],
    // ⛔ CPU-only: o dado vem de um canal externo, e não há caminho de device que o leia.
    lowerings: &[ph2d_nodegraph::node::LoweringKind::Cpu],
};

struct SourceTable;

impl NodeOp for SourceTable {
    fn manifest(&self) -> &'static NodeManifest {
        &MANIFEST
    }

    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        let file = ctx.text_param(FILE_KEY).unwrap_or("").to_string();
        let spacing = ctx.param(param::SPACING);
        let table = ctx.external(&ph2d_node_registry::table_external_key(&file));
        let n = table.count();
        // ⚠️ **Sem ficheiro, o nó desenha NADA — e não um ponto na origem.** Um elemento
        // solitário lê-se como *"a tabela carregou e tem uma linha"*, que é a mentira mais cara
        // que este nó pode contar. Zero elementos é a identidade honesta de *ainda não há dado*.
        if n == 0 {
            ctx.emit(Stream::new(0).with("P", Column::Vec2(Vec::new())));
            return;
        }
        // As linhas assentam numa fileira, centrada — é o que torna a tabela VISÍVEL no
        // instante em que carrega. ⛔ Quem quiser a posição vinda dos dados liga
        // `value.attribute → motion.drive(Position)`: a composição já o exprime.
        let mid = (n as f32 - 1.0) * 0.5;
        let p: Vec<[f32; 2]> = (0..n).map(|i| [(i as f32 - mid) * spacing, 0.0]).collect();
        let mut out = Stream::new(n).with("P", Column::Vec2(p));
        for (name, col) in table.columns() {
            out.set(name.clone(), col.clone());
        }
        out.set(
            "Index",
            Column::Scalar((0..n).map(|i| i as f32).collect::<Vec<_>>()),
        );
        out.set("Count", Column::Scalar(vec![n as f32; n]));
        ctx.emit(out);
    }
}

static PARAM_HINTS: &[ParamUiHint] = &[
    ParamUiHint {
        param: FILE_KEY,
        label: "Table File",
        min: 0.0,
        max: 0.0,
        step: 0.0,
        // ⚠️ **Uma ESPÉCIE, nunca uma lista de extensões**: esta crate não depende do leitor,
        // logo não pode saber o que este build lê. Quem sabe é a shell.
        widget: ParamWidget::File {
            kind: ph2d_node_registry::FileKind::Table,
        },
    },
    ParamUiHint {
        param: param::SPACING,
        label: "Spacing",
        min: 0.0,
        max: 2.0,
        step: 0.01,
        widget: ParamWidget::Slider,
    },
];

/// Register this node with the runtime registry.
pub fn register(reg: &mut NodeRegistry) -> Result<(), RegistryError> {
    reg.register(Box::new(SourceTable))?;
    reg.register_ui(
        MANIFEST.id,
        ph2d_node_registry::NodeUiManifest {
            display_name: "Table",
            category: ph2d_node_registry::NodeUiCategory::Source,
            silhouette: ph2d_node_registry::NodeSilhouette::TrapezoidDown,
        },
    );
    reg.register_param_ui(MANIFEST.id, PARAM_HINTS);
    Ok(())
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
