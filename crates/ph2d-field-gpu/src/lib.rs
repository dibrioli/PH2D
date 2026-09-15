//! ⭐⭐⭐ **O CAMPO IMPLÍCITO NO DISPOSITIVO.**
//!
//! # A medição que escolheu esta rota (`docs/Render3d/05` §32–§33)
//!
//! A mesma peça, a mesma lei de marcha, o mesmo orçamento de passos:
//!
//! | `1920×1080` | CPU | **GPU** | ganho |
//! |---|---:|---:|---:|
//! | só o traçado | `29,18 ms` | `0,67 ms` | `43,6×` |
//! | traçado + oclusão inteira | `1 998 ms` | **`5,00 ms`** | **`399,5×`** |
//!
//! ⇒ o que na CPU custava **dois segundos em dezasseis etapas visíveis** cabe em `5 ms` de um
//! quadro de `16,7`.
//!
//! # ⭐ O catálogo inteiro por VINTE E OITO opcodes
//!
//! O gerador não conhece nenhuma das `62` primitivas: ele traduz a **fita** que o documento já
//! produz ([`ph2d_field_eval::wgsl`]), onde uma rosca e um filete já são `min`, `max` e `sqrt`.
//!
//! # ⚠️ Um pipeline por ESTRUTURA, e as constantes num buffer
//!
//! Arrastar um slider muda um número e não a árvore. Se a constante fosse escrita no shader, cada
//! quadro do arrasto recompilaria — medido, `6` a `49 ms`. ⇒ a chave do cache é o **texto**, que
//! não muda, e os números viajam num buffer que se reescreve de graça.

// ⚠️ **`BTreeMap` e não `HashMap`** — HR-5/ADR-0022. Aqui ele também é o certo por outra razão:
// a chave é o TEXTO do shader, e uma ordem estável faz um censo de pipelines compilados ser
// reprodutível entre corridas.
use ph2d_field::{FieldDoc, NodeKind};
use std::collections::BTreeMap;

/// ⭐⭐⭐ **ESTE DOCUMENTO PODE IR PARA O DISPOSITIVO?**
///
/// # ⚠️⚠️ A resposta MUDOU em 2026-09-15, e a redacção antiga fica aqui como aviso
///
/// Ela era **`não` para toda peça com ESCULTURA** ([`ph2d_field::NodeKind::Sampled`]), e o motivo
/// era real: uma escultura não é uma expressão, o compilador da fita traduzia-a para
/// `Tree::constant(ABSENT)` — **espaço vazio** — e sem esta porta ligar a GPU faria a escultura
/// **desaparecer da peça, em silêncio e com o resto dela perfeito**.
///
/// ⛔ E o gate de paridade não o veria: ele compara a fita com a fita, e as duas concordam que ali
/// não há nada. *Foi a cena da ponte a ler `0,000` de desvio que mostrou o buraco — um zero de
/// «igual» e um de «nenhum dos dois sabe» são o mesmo byte.*
///
/// ⭐ Hoje a escultura **atravessa** ([`crate::sculpt`]): ela entra na árvore como uma variável e o
/// shader amostra a grade. ⇒ o que sobra desta porta é uma pergunta mais estreita e verdadeira:
/// ***a folha amostrada sabe entregar a grade?*** Uma que não saiba fica na CPU, que sabe desenhá-la.
///
/// ⚠️ **Um nome que o registo não conhece PASSA**, e não é um furo: ele lê como espaço vazio nos
/// dois motores — é o que o `ABSENT` do [`ph2d_field_eval::hybrid`] significa.
#[must_use]
pub fn supports(doc: &FieldDoc, reg: &ph2d_field_eval::hybrid::Registry) -> bool {
    doc.nodes().iter().all(|n| match &n.kind {
        NodeKind::Sampled { key } => reg.get(key).is_none_or(|f| f.grid().is_some()),
        _ => true,
    })
}

/// ⭐⭐⭐ **QUANTOS VALORES VIVOS UMA FITA PODE TER ANTES DE A PLACA DEIXAR DE COMPENSAR.**
///
/// # ⛔⛔ O recurso é o FICHEIRO DE REGISTOS, e foi a curva que o nomeou
///
/// O `vivos` do [`ph2d_field_eval::point_tape::TapeShape`] é o scratch **por thread**. Numa placa
/// ele decide a **ocupação**: quantos fios cabem num multiprocessador ao mesmo tempo. Medido a
/// `1920×1080` com um perfil extrudado de `N` arestas (a família que o `+ Extrude` do artista
/// produz), o custo **por aresta** — que é plano enquanto a fita cabe nos registos:
///
/// | arestas | instruções | vivos | quadro | ms/aresta |
/// |---:|---:|---:|---:|---:|
/// | `32` | `1 043` | `163` | `20,7 ms` | `0,647` |
/// | `64` | `2 063` | `320` | `43,9 ms` | `0,686` |
/// | `128` | `4 083` | `623` | `129,5 ms` | `1,012` |
/// | `144` | `4 589` | `700` | `160,4 ms` | `1,114` |
/// | **`152`** | `4 846` | **`743`** | `191,0 ms` | **`1,257`** ⬅ o último deste lado |
/// | `160` | `5 100` | `779` | `273,5 ms` | **`1,709`** ⬅ `+36 %` por `+5 %` de arestas |
/// | `256` | `8 124` | `1 230` | `1 255,7 ms` | `4,905` |
///
/// ⭐ **O salto é DISCRETO e não gradual** — `+36 %` de custo por `+5 %` de trabalho —, que é a
/// assinatura de a ocupação cair um degrau, e não de mais aritmética.
///
/// ⛔⛔ **E acima dele o dispositivo é MAIS LENTO que a CPU que ele substituiu:** a `256` arestas
/// mediu-se `0,20×` — cinco vezes pior. *Esta linha pôs o quadro na placa e teria feito a peça
/// desenhada do artista ficar mais lenta do que era, no topo da faixa que o slider dele alcança.*
///
/// ⚠️ **Este número é do EIXO CERTO e não de qualquer um**: a cena da superfórmula tem `766`
/// instruções e apenas `34` vivos, e é lenta por outra razão (o minorante do campo). *Um tecto sobre
/// as instruções mandaria essa peça para a CPU sem curar nada.*
///
/// ⏳ **Ele tem de ser re-medido numa máquina calma e noutra placa** — a coluna do relógio foi
/// tirada a `load 80–110` (outra linha a correr a suíte dela), e o ficheiro de registos é da placa.
/// O que NÃO depende de nenhuma das duas é a forma da curva, e é ela que escolheu o eixo.
pub const MAX_VIVOS: usize = 743;

pub mod material_parity;
pub mod owners_parity;
pub mod paint;
pub mod parity;
pub mod probe;
pub mod sculpt;
pub mod trace;
mod trace_grupo;
mod trace_leitura;
mod trace_to_cpu;
mod trace_uniforme;
mod trace_wgsl;

/// O molde do shader: a fita do documento, mais o que o chamador quiser à volta.
///
/// ⚠️ `{FIELD}` é substituído pelo corpo gerado e `{CONSTS}` pelo binding das constantes — e é por
/// isso que um consumidor (o traçado, a oclusão, a malha) escreve só a **sua** parte.
pub const FIELD_SLOT: &str = "{FIELD}";

/// Um cache de pipelines **por estrutura**, com as constantes de fora — mais as **grades** das
/// esculturas, que são grandes e não cabem num buffer por quadro.
pub struct FieldPipelines {
    por_texto: BTreeMap<String, wgpu::ComputePipeline>,
    /// ⭐⭐⭐ **A grade que já está na placa**, com a identidade que a produziu e as referências
    /// FORTES que impedem o alocador de reciclar os endereços dela — ver [`crate::sculpt::identity`].
    grades: Option<GradesNaPlaca>,
    /// Quantas vezes uma grade subiu — ver [`FieldPipelines::grades_enviadas`].
    envios: usize,
}

/// ⭐⭐⭐ **A grade residente** — ver [`FieldPipelines::grades`].
struct GradesNaPlaca {
    /// Os ponteiros das esculturas que produziram este buffer, na ordem delas.
    chave: Vec<usize>,
    /// ⛔⛔ **As referências FORTES, e elas não são lastro:** sem elas a escultura podia morrer, o
    /// alocador devolver o mesmo endereço a outra, e o cache servir a grade errada **sem erro
    /// nenhum**. *Um cache que compara endereços tem de impedir que eles sejam reciclados.*
    ///
    /// ⚠️ **Ninguém a LÊ, e é essa exactamente a função dela** — ela existe para que os endereços
    /// da [`Self::chave`] não possam ser reciclados enquanto este buffer viver. O `dead_code` diz a
    /// verdade sobre a leitura e a mentira sobre o propósito.
    #[allow(dead_code)]
    vivas: Vec<std::sync::Arc<dyn ph2d_field_eval::hybrid::Sampled>>,
    buffer: wgpu::Buffer,
}

impl Default for FieldPipelines {
    fn default() -> Self {
        Self::new()
    }
}

impl FieldPipelines {
    #[must_use]
    pub fn new() -> Self {
        Self {
            por_texto: BTreeMap::new(),
            grades: None,
            envios: 0,
        }
    }

    /// ⭐⭐⭐ **O buffer das grades desta peça**, subido só quando a identidade delas muda.
    ///
    /// ⚠️ **Sem escultura devolve um buffer MÍNIMO**, que o layout exige e o shader nunca lê: um
    /// `BindGroup` recusa uma entrada em falta, e um buffer de zero bytes também.
    ///
    /// ⚠️⚠️ **A chave é a IDENTIDADE e não o conteúdo.** Comparar `8 MB` para decidir se se enviam
    /// `8 MB` é pagar o preço duas vezes — e a escultura só muda quando quem a gerou a substitui,
    /// que é exactamente o que o ponteiro do `Arc` diz.
    pub fn grades(
        &mut self,
        device: &wgpu::Device,
        sculpts: &[ph2d_field_eval::device::DeviceSculpt],
    ) -> &wgpu::Buffer {
        use wgpu::util::DeviceExt;
        let chave = crate::sculpt::identity(sculpts);
        if self.grades.as_ref().is_none_or(|g| g.chave != chave) {
            let valores = crate::sculpt::grid_values(sculpts).unwrap_or_default();
            let bytes: Vec<u8> = if valores.is_empty() {
                vec![0u8; 16]
            } else {
                valores.iter().flat_map(|f| f.to_le_bytes()).collect()
            };
            self.envios += 1;
            self.grades = Some(GradesNaPlaca {
                chave,
                vivas: sculpts
                    .iter()
                    .map(|s| std::sync::Arc::clone(&s.field))
                    .collect(),
                buffer: device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("grades"),
                    contents: &bytes,
                    usage: wgpu::BufferUsages::STORAGE,
                }),
            });
        }
        &self.grades.as_ref().expect("acabou de se preencher").buffer
    }

    /// ⭐⭐ **Quantas vezes as grades SUBIRAM à placa** — o número que o gate de *«um arrasto não
    /// reenvia a escultura»* observa.
    ///
    /// ⚠️ **Contagem e não presença:** `is_some()` responde `1` tanto a uma subida como a mil, e é
    /// exactamente a diferença entre as duas que este número existe para dizer.
    #[must_use]
    pub fn grades_enviadas(&self) -> usize {
        self.envios
    }

    /// Quantos pipelines estão compilados — o número que um gate de *«um arrasto não recompila»*
    /// observa.
    #[must_use]
    pub fn compiled(&self) -> usize {
        self.por_texto.len()
    }

    /// ⭐ **O pipeline desta estrutura**, compilando-o na primeira vez que ela aparece.
    ///
    /// `molde` é o WGSL do consumidor com [`FIELD_SLOT`] algures dentro.
    pub fn get(
        &mut self,
        device: &wgpu::Device,
        molde: &str,
        field: &ph2d_field_eval::wgsl::TapeWgsl,
    ) -> &wgpu::ComputePipeline {
        self.entry(device, molde, field, "main")
    }

    /// ⭐ **O mesmo, nomeando a ENTRADA** — um molde com duas passagens compila **um** módulo e
    /// dois pipelines. ⚠️ A chave inclui a entrada: dois pipelines do mesmo texto são coisas
    /// diferentes.
    pub fn entry(
        &mut self,
        device: &wgpu::Device,
        molde: &str,
        field: &ph2d_field_eval::wgsl::TapeWgsl,
        entrada: &str,
    ) -> &wgpu::ComputePipeline {
        self.entry_with_layout(device, molde, field, entrada, None)
    }

    /// ⭐⭐ **O mesmo, com o layout de propósito** — e ele é obrigatório quando o molde tem DUAS
    /// entradas que usam bindings diferentes.
    ///
    /// ⛔⛔ O layout AUTO-DERIVADO só declara os bindings que **aquela entrada usa**: a passagem do
    /// centro não toca na lista de bordas, logo o layout dela tem `4` entradas e o `BindGroup` de
    /// `6` é recusado. *Um layout derivado por entrada não é o layout do módulo* — e o erro só
    /// aparece em tempo de execução, quando o grupo é criado.
    pub fn entry_with_layout(
        &mut self,
        device: &wgpu::Device,
        molde: &str,
        field: &ph2d_field_eval::wgsl::TapeWgsl,
        entrada: &str,
        layout: Option<&wgpu::PipelineLayout>,
    ) -> &wgpu::ComputePipeline {
        let src = molde.replace(FIELD_SLOT, &field.source);
        let chave = format!("{entrada}\u{0}{src}");
        self.por_texto.entry(chave).or_insert_with(|| {
            let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("campo"),
                source: wgpu::ShaderSource::Wgsl(src.as_str().into()),
            });
            device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some("campo"),
                layout,
                module: &module,
                entry_point: Some(entrada),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                cache: None,
            })
        })
    }
}
