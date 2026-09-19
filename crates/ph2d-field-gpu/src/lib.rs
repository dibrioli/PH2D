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

// ⛔⛔⛔ **AQUI VIVIA UM TECTO, E ELE SAIU PORQUE A GRANDEZA DELE NÃO ORDENA OS RESULTADOS**
// (2026-09-15). `MAX_VIVOS = 743` → `358` → `MAX_GUARDADOS = 3 463` → `13 789` → **nada**.
//
// As três primeiras saíram de uma sonda com DOIS defeitos que se somavam: ela media o **quadro
// pintado inteiro** do lado da placa e **só o traçado** do lado da CPU, com o traçador e o material
// a compilar a `opt-0` contra um WGSL optimizado pelo driver. ⇒ *o número não foi afinado três
// vezes; a régua é que estava partida.*
//
// A quarta veio de estender a varredura ao topo do slider do artista — e foi ela que matou a ideia
// de tecto. Medido a `1920×1080`, CPU a `95`–`99 %` ociosa:
//
// | arestas | guardados | placa | CPU | razão |
// |---:|---:|---:|---:|---:|
// | `32` | `879` | `23,7 ms` | `181,4` | `7,65×` |
// | `96` | `2 602` | `87,1` | `463,6` | `5,32×` |
// | `128` | `3 462` | `181,0` | `609,7` | `3,37×` |
// | `192` | `5 189` | `919,2` | `884,1` | `0,96×` ⬅ penhasco |
// | `256` | `6 903` | `631,0` / `1 283,6` | `1 243,9` / `1 150,0` | `1,97×` / `0,90×` |
// | `384` | `10 351` | `1 309,1` | `1 669,1` | `1,28×` |
// | `512` | `13 789` | `2 066,3` / `787,2` | `2 366,8` / `2 334,6` | `1,15×` / `2,97×` |
// | `768` | `20 675` | `6 575,7` / `6 429,9` | `3 423,1` / `3 366,1` | `0,52×` / `0,52×` ⬅ penhasco |
// | `1024` | `27 563` | `3 508,7` | `4 546,9` | `1,30×` |
//
// ⛔⛔ **O `768` perde de forma REPRODUTÍVEL e os dois vizinhos ganham.** Um tecto em `13 789`
// apanharia o `768` **e excluiria o `1024`, que a placa ganha** — ⇒ *a grandeza não ordena os
// resultados, logo nenhum corte sobre ela é melhor do que outro.* E a dispersão fecha a porta: a
// MESMA peça de `512` leu `787` e `2 066 ms` (`2,6×`) em corridas do mesmo código.
//
// ⭐ O que protege a faixa onde toda peça REAL vive (a pior das 17 cenas mede `2 663` guardados) é
// o gate `na_faixa_do_produto_a_placa_ganha_com_margem`, que exige `≥ 2×` entre `64` e `128`
// arestas — medido `3,4×`–`12,7×` entre `1 %` e `99 %` de CPU ociosa. E nas próprias cenas do
// produto a placa ganha `2,58×`–`98×`.
//
// ⏳ **A cura dos penhascos não é uma constante: é um LAÇO FECHADO** — comparar o quadro que o
// dispositivo de facto entregou com o que a CPU entregou, como o divisor da pré-visualização já faz
// com o orçamento. Está nomeada em `docs/Render3d/05` §43.10 e **ninguém a mediu**.

/// ⭐⭐⭐ **O BRILHO no dispositivo** — ver o módulo.
pub mod brilho;
pub mod material_parity;
pub mod owners_parity;
pub mod paint;
/// ⭐ **O corpo do shader do pintor** — irmão por responsabilidade do [`paint`]: ali monta-se, aqui
/// compila-se. ⛔ Corte por tecto de LOC, nunca isenção (`CLAUDE.md` §5.0).
///
/// ⚠️ **Ele é `pub` por UMA coisa só: a [`paint_wgsl::CURVATURA`]**, que tem um segundo leitor — o
/// instrumento que mede a curvatura nos dois motores. ⛔ O resto do módulo continua `pub(crate)`
/// (`PINTOR` e `PINTOR_SONDAS` são marcas por preencher, e um texto com `{…}` lá fora é uma forma
/// de alguém esquecer uma). *Uma lei com dois leitores exporta-se; um corpo de shader por montar,
/// não.*
/// ⭐ **Os bytes do uniforme do pintor** — ver o módulo.
mod paint_uniforme;
pub mod paint_wgsl;
/// ⭐ **A segunda metade do shader do pintor** — o hemisfério que ele integra.
mod paint_wgsl_sondas;
pub mod parity;
pub mod probe;
pub mod sculpt;
pub mod trace;
mod trace_grupo;
mod trace_lampadas;
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
