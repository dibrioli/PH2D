//! ⭐⭐⭐ **A TINTA FINA NO DEVICE** — os buffers que o gémeo em WGSL lê.
//!
//! ⚠️ **Módulo próprio e não mais um bloco no [`crate::pipeline_upload`]:** o
//! `pipeline_build` estava a **duas linhas** do tecto de LOC quando esta wave
//! chegou, e as entradas de layout não cabem lá. *O corte por
//! responsabilidade é mais barato do que a isenção que o evitaria* — e aqui
//! ele é o certo de qualquer maneira: um sítio só sabe a disposição de todas.
//!
//! ⚠️⚠️ **Os buffers existem SEMPRE, mesmo sem plano armado**, e isso não é
//! desperdício: o bind group por objecto é criado quando o slot nasce, e um
//! binding que aparece e desaparece obrigaria a reconstruir o layout — não o
//! bind, o LAYOUT, que é do pipeline. ⇒ sem plano eles são de um elemento e o
//! `armado` da configuração vale `0`, que é o que faz o shader devolver o
//! `in.vcolor` de sempre, ao bit.

use ph2d_mesh::Mesh;
use ph2d_mesh_colors::{Inclinacoes, Tinta};
use wgpu::util::DeviceExt as _;

use crate::MeshRenderer;

/// Os oito buffers, e o que cabe em cada um hoje.
pub(super) struct TintaGpu {
    pub(super) amostras: wgpu::Buffer,
    topo: wgpu::Buffer,
    origem: wgpu::Buffer,
    idx: wgpu::Buffer,
    pos: wgpu::Buffer,
    cfg: wgpu::Buffer,
    /// ⭐ **O RELEVO** (`docs/3D/29`) — um par `[altura, corpo]` por amostra,
    /// na MESMA ordem das amostras. Um dummy de `16` B quando o plano não tem
    /// relevo, e o bit [`RELEVO`] da configuração desligado.
    alturas: wgpu::Buffer,
    /// ⭐ **A INCLINAÇÃO de cada amostra** (`docs/3D/29` §9) — três `f32` por
    /// amostra, o gradiente da altura no objecto. Dummy e bit iguais aos das
    /// alturas.
    inclinacoes: wgpu::Buffer,
    /// Quem mantém as inclinações em dia na CPU, por pedaços durante um traço.
    /// `None` sem relevo.
    inc: Option<Inclinacoes>,
    /// ⭐ **A foto do que as inclinações descrevem** — o registo das faces, as
    /// posições e as alturas da última vez. A subida inteira corre em todo
    /// quadro em que a peça muda de FORMA, e refazer todas custava `80 ms` a
    /// `64x` (doc 29 §9.3): com a foto ela refaz só o que MUDOU, sem pedir ao
    /// chamador uma lista em que teria de acertar sempre.
    inc_foto: FotoDasInclinacoes,
    cap_amostras: usize,
    cap_topo: usize,
    cap_tri: usize,
    /// ⛔⛔ **A capacidade do buffer de ÍNDICES, e ela é um CAMPO** — não
    /// `cap_tri * 3`, que foi como nasceu e é um defeito com erro de
    /// validação do `wgpu` atrás.
    ///
    /// *Uma capacidade derivada da de outro buffer é uma segunda resposta à
    /// pergunta «quanto cabe AQUI?»*, e as duas divergem exactamente no
    /// instante em que importam: o `origem` realoca primeiro e actualiza o
    /// `cap_tri`, logo `cap_tri * 3` já descreve o tamanho NOVO enquanto o
    /// buffer de índices ainda é o de antes ⇒ a escrita passa pelo caminho
    /// rápido e despeja milhares de bytes num buffer de `16`.
    cap_idx: usize,
    cap_pos: usize,
    cap_alturas: usize,
    cap_inclinacoes: usize,
    /// O espelho do bit [`RELEVO`] já escrito no device — o incremental recusa
    /// quando o plano ganhou (ou perdeu) relevo desde a última subida inteira.
    relevo: bool,
    /// ⭐ **Há plano ligado?** — o espelho do `armado` que já foi escrito no
    /// device, para o `upload_tinta_at` não reescrever a configuração por
    /// quadro quando nada mudou.
    pub(super) armado: bool,
    /// ⭐⭐⭐ **Quantas amostras o device tem** — a testemunha que o upload
    /// INCREMENTAL exige antes de escrever por índice.
    ///
    /// ⚠️ *Escrever a amostra `i` num buffer que descreve outra topologia é a
    /// armadilha muda deste subsistema uma camada abaixo*, e a contagem é a
    /// única coisa que o device sabe de si mesmo. Ela é escrita **ao lado** da
    /// escrita que testemunha.
    pub(super) n_amostras: usize,
}

const N: usize = 8;

/// ⭐ **O bit do RELEVO no `armado`** (`docs/3D/29`): `1` = há plano, `2` = e
/// ele tem relevo. ⚠️ O `0` continua a querer dizer *nenhum plano*, e é por
/// isso que o relevo é um BIT e não um valor novo: o `fs_main_tinta` e todo
/// leitor que já compara `armado == 0` continuam certos.
pub const RELEVO: u32 = 2;
/// O primeiro binding da tinta no grupo POR OBJECTO (o `0` é a `obj.model`).
const B0: u32 = 1;

fn buffer_de_armazenamento(binding: u32) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        // ⚠️ Só o FRAGMENTO: as baricêntricas resolvem-se por pixel, e o
        //    vértice não sabe em que triângulo está.
        visibility: wgpu::ShaderStages::FRAGMENT,
        ty: wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Storage { read_only: true },
            has_dynamic_offset: false,
            min_binding_size: None,
        },
        count: None,
    }
}

/// As sete entradas que o layout do grupo POR OBJECTO ganha.
pub(super) fn entradas_do_layout() -> [wgpu::BindGroupLayoutEntry; N] {
    [
        buffer_de_armazenamento(B0),
        buffer_de_armazenamento(B0 + 1),
        buffer_de_armazenamento(B0 + 2),
        buffer_de_armazenamento(B0 + 3),
        buffer_de_armazenamento(B0 + 4),
        wgpu::BindGroupLayoutEntry {
            binding: B0 + 5,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        },
        buffer_de_armazenamento(B0 + 6),
        buffer_de_armazenamento(B0 + 7),
    ]
}

/// A configuração: `lado`, `verts`, `arestas`, `armado`.
///
/// ⛔⛔ **Um plano GRADUADO (um nível por face — a P2) desarma-se aqui, e isso
/// é deliberado.** O registo que o shader lê descreve a retícula com UM `lado`,
/// e o bloco das arestas com `id × (lado − 1)`; com níveis por face as duas
/// contas passam a precisar de um **offset por aresta**.
///
/// ⚠️⚠️ **Esse offset JÁ EXISTIU e SAIU por ordem do dono (2026-09-24).** Entre
/// 23/09 e 24/09 o registo tinha `19` palavras por face e a placa desenhava um
/// plano graduado; quando o `Even Detail` — o único que CRIAVA planos
/// graduados — foi retirado, as nove palavras ficaram a ser pagas em toda peça
/// (`7,6 MB` contra `4,0` a `100 k` faces) para ler um plano que só um ficheiro
/// antigo traz. Hoje o carregador CONVERTE esse plano em uniforme ao abrir
/// (`Tinta::uniformizada`), logo **nenhum plano graduado chega aqui pelo
/// produto** — e esta guarda fica, porque é a resposta certa se algum chegar.
///
/// ⭐ **Desarmar entrega a cor por VÉRTICE, que é o caso base desta família e
/// está certo** — e a alternativa (assumir o lado da face `0`) desenharia a
/// tinta de umas faces no sítio das outras **sem nada no ecrã a acusar**.
/// *A resposta errada com a confiança da certa é o que esta guarda recusa.*
///
/// ⚠️ **Ela é PÚBLICA por causa do arnês e não do produto** (re-exportada como
/// `ph2d_mesh_render::tinta_cfg`): um arnês de paridade que CONSTRÓI o
/// uniforme em vez de o PEDIR mede outro programa — a cura de 23/09, que
/// sobrevive à volta do registo.
#[must_use]
pub fn cfg_de(t: Option<&Tinta>) -> [u32; 4] {
    // ⚠️ O `lado` mínimo é `1` mesmo desarmado: um `lado = 0` faria o shader
    //    dividir a retícula por zero se alguém o lesse por engano.
    let Some(t) = t else { return [1, 0, 0, 0] };
    let Some(lado) = t.lado_uniforme() else {
        return [1, 0, 0, 0];
    };
    [
        lado,
        t.topologia().verts() as u32,
        t.topologia().arestas() as u32,
        if t.tem_relevo() { 1 | RELEVO } else { 1 },
    ]
}

impl TintaGpu {
    /// Os sete buffers de um elemento — o estado de quem não tem plano.
    pub(super) fn vazia(device: &wgpu::Device) -> Self {
        let st = wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST;
        let um = |rotulo: &str, dados: &[u8], uso: wgpu::BufferUsages| {
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(rotulo),
                contents: dados,
                usage: uso,
            })
        };
        let zero4 = bytemuck::cast_slice(&[0.0f32; 4]);
        let zero4u = bytemuck::cast_slice(&[0u32; 4]);
        Self {
            // `COPY_SRC`: o plano ACHATADO na placa (`tinta_achata`) lê-se de volta nos gates.
            amostras: um(
                "ph2d-mesh tinta amostras",
                zero4,
                st | wgpu::BufferUsages::COPY_SRC,
            ),
            topo: um("ph2d-mesh tinta topo", zero4u, st),
            origem: um("ph2d-mesh tinta origem", zero4u, st),
            idx: um("ph2d-mesh tinta idx", zero4u, st),
            pos: um("ph2d-mesh tinta pos", zero4, st),
            // `COPY_SRC`: os gates leem-nos de volta (`le_relevo_at`).
            alturas: um(
                "ph2d-mesh tinta alturas",
                zero4,
                st | wgpu::BufferUsages::COPY_SRC,
            ),
            inclinacoes: um(
                "ph2d-mesh tinta inclinacoes",
                zero4,
                st | wgpu::BufferUsages::COPY_SRC,
            ),
            inc: None,
            inc_foto: FotoDasInclinacoes::default(),
            cfg: um(
                "ph2d-mesh tinta cfg",
                bytemuck::cast_slice(&cfg_de(None)),
                wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            ),
            // ⚠️⚠️ **As capacidades são o tamanho REAL do dummy (`16` B), e não
            //   um `4` conservador.** A redacção anterior declarava `4` sobre um
            //   buffer de `16` — conservador por ACIDENTE, e foi esse acidente
            //   que escondeu o defeito do `cap_idx` durante a wave inteira:
            //   *uma capacidade que mente para baixo só desperdiça, e uma que
            //   mente para cima escreve fora do buffer.*
            cap_amostras: 16,
            cap_topo: 16,
            cap_tri: 16,
            cap_idx: 16,
            cap_pos: 16,
            cap_alturas: 16,
            cap_inclinacoes: 16,
            relevo: false,
            armado: false,
            n_amostras: 0,
        }
    }

    /// As sete entradas do bind group por objecto.
    pub(super) fn entradas(&self) -> [wgpu::BindGroupEntry<'_>; N] {
        fn r(binding: u32, b: &wgpu::Buffer) -> wgpu::BindGroupEntry<'_> {
            wgpu::BindGroupEntry {
                binding,
                resource: b.as_entire_binding(),
            }
        }
        [
            r(B0, &self.amostras),
            r(B0 + 1, &self.topo),
            r(B0 + 2, &self.origem),
            r(B0 + 3, &self.idx),
            r(B0 + 4, &self.pos),
            r(B0 + 5, &self.cfg),
            r(B0 + 6, &self.alturas),
            r(B0 + 7, &self.inclinacoes),
        ]
    }
}

impl MeshRenderer {
    /// ⭐⭐⭐ **Sobe o plano de tinta fina do objecto `index`** — ou desarma-o.
    ///
    /// Irmã da [`Self::upload_preview_at`] e da [`Self::upload_wire_at`]: uma
    /// porta à parte em vez de um argumento novo no [`Self::upload_at`], que é
    /// o caminho de TODA peça e tem dezenas de chamadores.
    ///
    /// ⚠️ **Ela é chamada DEPOIS do `upload_at`**, sempre: os índices e as
    /// posições que ela sobe têm de ser os da topologia que o device acabou de
    /// receber. *Um plano da malha de antes é tinta no vértice errado, e
    /// nenhuma contagem o vê.*
    ///
    /// ⛔ **E `None` não apaga os buffers, só escreve `armado = 0`.** Apagar
    /// faria o bind group ficar sem recurso e o layout é do PIPELINE — ver o
    /// cabeçalho deste módulo.
    pub fn upload_tinta_at(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        index: usize,
        mesh: &Mesh,
        tinta: Option<&Tinta>,
    ) {
        let Some(slot) = self.slots.get(index) else {
            return;
        };
        let Some(t) = tinta else {
            if slot.gpu.tinta.armado {
                queue.write_buffer(&slot.gpu.tinta.cfg, 0, bytemuck::cast_slice(&cfg_de(None)));
                self.slots[index].gpu.tinta.armado = false;
            }
            return;
        };

        // ⚠️ As faces são lidas do MESH e não da topologia da `Tinta`: o
        //    payload precisa dos cantos, e a `Topologia` não os guarda.
        let faces = || mesh.faces().iter().map(ph2d_mesh::Face::verts);
        let mut pay = Vec::new();
        // ⛔⛔⛔ **A CERCA, e ela vem do PÂNICO do dono de 2026-09-21** — ver
        // [`ph2d_mesh_colors::Topologia::descreve`]. O cabeçalho desta porta já
        // escrevia o perigo (*«um plano da malha de antes é tinta no vértice
        // errado, e nenhuma contagem o vê»*) e **nada o media**: no perfil
        // `smoke` o `debug_assert` do payload não existe, e o que o artista
        // recebia era `index out of bounds` a meio de uma pincelada.
        //
        // ⚠️ **Desarmar é a resposta CERTA e não um remendo:** o plano já não
        // descreve esta malha, logo não há tinta fina que se possa desenhar. O
        // device passa a mostrar a cor POR VÉRTICE, que é exactamente o que a
        // `tinta_da_peca::garante` vai reconstruir assim que o traço largar o
        // plano. *Meio quadro com a cor de baixa resolução é o que já ia
        // acontecer; um pânico é a sessão inteira.*
        // ⚠️⚠️ **As DUAS perguntas, e nenhuma cobre a outra.** A
        // [`ph2d_mesh_colors::Topologia::descreve`] é `O(1)` e é a única que vê
        // os **VÉRTICES** — o payload nunca os olha, porque um registo é feito
        // de faces. O payload é o veredito FORTE, face a face, e é a única que
        // separa duas malhas com as mesmas duas contagens. *Uma sozinha aprova
        // metade das mudanças de topologia.*
        //
        // ⭐ A `descreve` vem à frente pela ordem barata: ela corta sem
        // percorrer as faces todas, que é o caso comum durante um traço de
        // FORMA com o plano armado.
        if !t
            .topologia()
            .descreve(mesh.vert_count(), mesh.faces().len())
            || !t.topologia().payload(faces(), &mut pay)
        {
            if slot.gpu.tinta.armado {
                queue.write_buffer(&slot.gpu.tinta.cfg, 0, bytemuck::cast_slice(&cfg_de(None)));
                self.slots[index].gpu.tinta.armado = false;
            }
            return;
        }
        let mut tris = Vec::new();
        let mut origem = Vec::new();
        mesh.triangle_indices_com_origem(&mut tris, Some(&mut origem));

        // ⭐⭐⭐⭐ **NENHUM ACHATAMENTO, e a diferença é MEDIDA:** `[f32; 3]` e
        // `[u32; 3]` são contíguos, logo `&[[f32; 3]]` **já é** o bloco de bytes
        // que o device quer — as três linhas que aqui estavam construíam uma
        // cópia inteira do plano, das posições e dos índices **em todo quadro
        // com o plano emprestado**.
        //
        // ⛔ Medido em 2026-09-20, só o `collect` das amostras: `21,9 ms` no
        // degrau `8×` da peça de fábrica (`72 MB`) e `81,7 ms` no `16×`
        // (`288 MB`), contra um quadro de `16,7`. *Uma cópia que existe só para
        // mudar o TIPO do slice é a forma mais cara de não dizer nada.*
        let amostras: &[u8] = bytemuck::cast_slice(t.amostras());
        let idx: &[u8] = bytemuck::cast_slice(&tris);
        let pos: &[u8] = bytemuck::cast_slice(mesh.positions());

        let mut refez = false;
        let st = wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST;
        {
            let g = &mut self.slots[index].gpu.tinta;
            refez |= poe(
                device,
                queue,
                &mut g.amostras,
                &mut g.cap_amostras,
                amostras,
                "amostras",
                st | wgpu::BufferUsages::COPY_SRC,
            );
            refez |= poe(
                device,
                queue,
                &mut g.topo,
                &mut g.cap_topo,
                bytemuck::cast_slice(&pay),
                "topo",
                st,
            );
            refez |= poe(
                device,
                queue,
                &mut g.origem,
                &mut g.cap_tri,
                bytemuck::cast_slice(&origem),
                "origem",
                st,
            );
            refez |= poe(device, queue, &mut g.idx, &mut g.cap_idx, idx, "idx", st);
            refez |= poe(device, queue, &mut g.pos, &mut g.cap_pos, pos, "pos", st);
            // ⚠️ Sem relevo o buffer fica como está (o dummy, ou o relevo de
            //    uma subida anterior): o bit do `cfg` é quem diz ao shader que
            //    não o leia, logo não há bytes a apagar.
            if let Some(a) = t.relevo() {
                refez |= poe(
                    device,
                    queue,
                    &mut g.alturas,
                    &mut g.cap_alturas,
                    bytemuck::cast_slice(a),
                    "alturas",
                    st | wgpu::BufferUsages::COPY_SRC,
                );
                let cantos = |f: usize| mesh.faces()[f].verts();
                let mesma =
                    g.inc.as_ref().is_some_and(|i| i.serve(t)) && g.inc_foto.descreve(&pay, mesh);
                match g.inc.as_mut().filter(|_| mesma) {
                    // ⭐ A MESMA topologia: refaz só as amostras das faces cujas
                    //   posições ou alturas mudaram desde a foto.
                    Some(inc) => {
                        let sujas = g.inc_foto.o_que_mudou(mesh.positions(), a);
                        let mut mudadas = Vec::new();
                        inc.atualiza(t, &cantos, mesh.positions(), &sujas, &mut mudadas);
                        let mut corridas = Vec::new();
                        corridas_das_sujas(&mut mudadas, &mut corridas);
                        let gb: &[u8] = bytemuck::cast_slice(inc.por_amostra());
                        for &(de, ate) in &corridas {
                            queue.write_buffer(&g.inclinacoes, de as u64, &gb[de..ate]);
                        }
                    }
                    // A adjacência é refeita com o plano: esta porta é também a
                    // da topologia nova.
                    None => {
                        let inc = Inclinacoes::nova(t, cantos, mesh.positions());
                        refez |= poe(
                            device,
                            queue,
                            &mut g.inclinacoes,
                            &mut g.cap_inclinacoes,
                            bytemuck::cast_slice(inc.por_amostra()),
                            "inclinacoes",
                            st | wgpu::BufferUsages::COPY_SRC,
                        );
                        g.inc = Some(inc);
                        g.inc_foto.tira(&pay, mesh.positions(), a);
                    }
                }
            } else {
                g.inc = None;
                g.inc_foto = FotoDasInclinacoes::default();
            }
            queue.write_buffer(&g.cfg, 0, bytemuck::cast_slice(&cfg_de(Some(t))));
            g.relevo = t.tem_relevo();
            g.armado = true;
            g.n_amostras = t.amostras().len();
        }
        if refez {
            self.refaz_bind(device, index);
        }
    }
}

impl MeshRenderer {
    /// ⭐⭐⭐⭐ **SOBE SÓ AS AMOSTRAS QUE O TRAÇO ESCREVEU** — e é isto que tira
    /// o custo de ter tinta fina de cima de `O(plano)`.
    ///
    /// Devolve `false` quando não pode, e aí **quem chama sobe o plano
    /// inteiro**: o device tem de já ter este plano (mesma contagem de
    /// amostras, `armado`), senão escrever por índice põe bytes válidos no
    /// sítio errado — a armadilha muda desta fronteira.
    ///
    /// ⛔⛔ **A dívida que ela paga estava NOMEADA e por medir:** *«o upload é
    /// INTEIRO e por quadro durante um traço … a escrita é por AMOSTRA e não
    /// passa pelo `dirty`, que é uma janela de VÉRTICES»*. Medido em
    /// 2026-09-20, só o empacotamento: `21,9 ms` a `8×` e `81,7 ms` a `16×` na
    /// peça de fábrica, **por quadro**, contra um quadro de `16,7`.
    ///
    /// ⚠️ **As amostras são ORDENADAS e escritas em CORRIDAS**, e não uma a
    /// uma: um `write_buffer` por amostra é uma chamada de driver por `12`
    /// bytes, e a pegada de um dab tem milhares delas. A ordenação é sobre a
    /// janela do QUADRO (as escritas desde o último upload), não sobre o traço.
    ///
    /// ⚠️ **Só as AMOSTRAS.** O `topo`, o `origem`, o `idx` e o `pos` são
    /// função da TOPOLOGIA e das POSIÇÕES, e um verbo de cor não mexe em
    /// nenhuma das duas — quem chama afirma isso, e no dia em que deixar de ser
    /// verdade cai no caminho inteiro.
    pub fn upload_tinta_amostras_at(
        &mut self,
        queue: &wgpu::Queue,
        index: usize,
        mesh: &Mesh,
        tinta: &Tinta,
        sujas: &mut Vec<u32>,
    ) -> bool {
        let Some(slot) = self.slots.get_mut(index) else {
            return false;
        };
        let g = &mut slot.gpu.tinta;
        // ⚠️ O RELEVO também tem de ser o que o device tem: um plano que o
        //    ganhou a meio de um traço (o 1.º toque de impasto) precisa da
        //    subida inteira — o buffer das alturas ainda é o dummy de 16 B, e
        //    escrever nele por índice seria escrever fora dele.
        if !g.armado || g.n_amostras != tinta.amostras().len() || g.relevo != tinta.tem_relevo() {
            return false;
        }
        // ⚠️ As inclinações por pedaços precisam da adjacência DESTE plano.
        if g.relevo && !g.inc.as_ref().is_some_and(|i| i.serve(tinta)) {
            return false;
        }
        if sujas.is_empty() {
            return true;
        }
        let bytes: &[u8] = bytemuck::cast_slice(tinta.amostras());
        let mut corridas = Vec::new();
        corridas_das_sujas(sujas, &mut corridas);
        let alturas: &[u8] = tinta.relevo().map_or(&[], bytemuck::cast_slice);
        for &(de, ate) in &corridas {
            queue.write_buffer(&g.amostras, de as u64, &bytes[de..ate]);
            if !alturas.is_empty() {
                let (ha, hb) = em_alturas((de, ate));
                queue.write_buffer(&g.alturas, ha as u64, &alturas[ha..hb]);
            }
        }
        // ⭐ Uma altura mudada muda a inclinação das amostras das células
        //   dela — mais do que as sujas. Três `f32` por amostra, como a cor,
        //   logo as corridas em bytes são as mesmas contas.
        if let Some(inc) = g.inc.as_mut() {
            let mut mudadas = Vec::new();
            inc.atualiza(
                tinta,
                &|f| mesh.faces()[f].verts(),
                mesh.positions(),
                sujas,
                &mut mudadas,
            );
            g.inc_foto
                .anota_alturas(sujas, tinta.relevo().unwrap_or(&[]));
            corridas_das_sujas(&mut mudadas, &mut corridas);
            let gb: &[u8] = bytemuck::cast_slice(inc.por_amostra());
            for &(de, ate) in &corridas {
                queue.write_buffer(&g.inclinacoes, de as u64, &gb[de..ate]);
            }
        }
        true
    }

    /// ⭐ **Refaz o bind group por objecto** — a única saída quando um buffer
    /// da tinta foi REALOCADO, porque um bind group guarda o recurso e não o
    /// nome dele.
    fn refaz_bind(&mut self, device: &wgpu::Device, index: usize) {
        let bind = {
            let slot = &self.slots[index];
            let mut e = vec![wgpu::BindGroupEntry {
                binding: 0,
                resource: slot.model.as_entire_binding(),
            }];
            e.extend(slot.gpu.tinta.entradas());
            device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("ph2d-mesh object bind"),
                layout: &self.obj_bgl,
                entries: &e,
            })
        };
        self.slots[index].bind = bind;
    }
}

/// ⭐⭐⭐ **AS CORRIDAS CONTÍGUAS de uma lista de amostras sujas**, já em BYTES
/// — a aritmética que pode pôr bytes válidos no sítio errado, isolada para
/// poder ser medida **sem device**.
///
/// ⚠️ **Uma escrita por amostra é uma chamada de driver por `12` bytes**, e a
/// pegada de um dab tem milhares delas; as amostras de uma face são contíguas
/// no plano, logo agrupá-las dá poucas corridas longas.
///
/// ⚠️ **A lista chega por ORDEM DE TOQUE e com repetidos** (uma amostra
/// re-escrita por dois dabs do mesmo quadro entra duas vezes) — ordenar e
/// deduplicar é obrigatório, e é por isso que ela recebe `&mut`.
///
/// Devolve pares `(início, fim)` em **bytes**, prontos para o `write_buffer`.
pub(super) fn corridas_das_sujas(sujas: &mut Vec<u32>, out: &mut Vec<(usize, usize)>) {
    out.clear();
    sujas.sort_unstable();
    sujas.dedup();
    let mut i = 0usize;
    while i < sujas.len() {
        let inicio = sujas[i];
        let mut fim = inicio;
        while i + 1 < sujas.len() && sujas[i + 1] == fim + 1 {
            i += 1;
            fim = sujas[i];
        }
        out.push((inicio as usize * 12, (fim as usize + 1) * 12));
        i += 1;
    }
}

/// ⭐ **A foto do que as [`Inclinacoes`] descrevem** — ver `TintaGpu::inc_foto`.
#[derive(Default)]
struct FotoDasInclinacoes {
    pay: Vec<u32>,
    pos: Vec<[f32; 3]>,
    alturas: Vec<f32>,
}

impl FotoDasInclinacoes {
    fn tira(&mut self, pay: &[u32], pos: &[[f32; 3]], relevo: &[[f32; 2]]) {
        self.pay.clear();
        self.pay.extend_from_slice(pay);
        self.pos.clear();
        self.pos.extend_from_slice(pos);
        self.alturas.clear();
        self.alturas.extend(relevo.iter().map(|r| r[0]));
    }

    /// A mesma topologia da foto — o registo face a face e a contagem de
    /// vértices (o registo não vê um vértice órfão).
    fn descreve(&self, pay: &[u32], mesh: &Mesh) -> bool {
        self.pay == pay && self.pos.len() == mesh.positions().len()
    }

    /// As amostras sujas desde a foto — os VÉRTICES que se moveram (o índice
    /// da amostra de um vértice é o dele) e as amostras de altura mudada —, e
    /// a foto passa a ser a de agora.
    fn o_que_mudou(&mut self, pos: &[[f32; 3]], relevo: &[[f32; 2]]) -> Vec<u32> {
        let mut sujas = Vec::new();
        for (v, (foto, agora)) in self.pos.iter_mut().zip(pos).enumerate() {
            if foto != agora {
                *foto = *agora;
                sujas.push(v as u32);
            }
        }
        for (i, (foto, agora)) in self.alturas.iter_mut().zip(relevo).enumerate() {
            if foto.to_bits() != agora[0].to_bits() {
                *foto = agora[0];
                sujas.push(i as u32);
            }
        }
        sujas
    }

    /// O incremental do traço já refez estas amostras: a foto acompanha.
    fn anota_alturas(&mut self, sujas: &[u32], relevo: &[[f32; 2]]) {
        for &i in sujas {
            if let (Some(f), Some(r)) = (self.alturas.get_mut(i as usize), relevo.get(i as usize)) {
                *f = r[0];
            }
        }
    }
}

/// A corrida de BYTES do relevo que corresponde a uma corrida de BYTES das
/// amostras: a mesma faixa de amostras, a `8` bytes cada (`[altura, corpo]`)
/// em vez de `12`.
///
/// ⚠️ Escrita à parte, e com gate, pela mesma razão da [`corridas_das_sujas`]:
/// é aritmética que põe bytes VÁLIDOS no sítio errado quando erra.
pub(super) fn em_alturas((de, ate): (usize, usize)) -> (usize, usize) {
    (de / 12 * 8, ate / 12 * 8)
}

#[path = "tinta_gpu_relevo.rs"]
mod relevo;

#[cfg(test)]
#[path = "tinta_gpu_tests.rs"]
mod tests;

/// Escreve `dados` no buffer, realocando quando não cabem. Devolve `true`
/// quando realocou — e aí o bind group tem de ser refeito.
fn poe(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    buf: &mut wgpu::Buffer,
    cap: &mut usize,
    dados: &[u8],
    rotulo: &str,
    uso: wgpu::BufferUsages,
) -> bool {
    // ⚠️ O `wgpu` recusa uma escrita que não seja múltipla de 4 bytes e recusa
    //    um buffer de tamanho zero — os dois casos existem (uma malha sem
    //    amostras interiores, um plano vazio), e por isso o piso é `4`.
    let n = dados.len().max(4);
    if n <= *cap {
        if !dados.is_empty() {
            queue.write_buffer(buf, 0, dados);
        }
        return false;
    }
    *buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some(rotulo),
        contents: dados,
        usage: uso,
    });
    *cap = n;
    true
}
