//! ⭐⭐⭐ **A TINTA FINA NO DEVICE** — os buffers que o gémeo em WGSL lê.
//!
//! ⚠️ **Módulo próprio e não mais um bloco no [`crate::pipeline_upload`]:** o
//! `pipeline_build` estava a **duas linhas** do tecto de LOC quando esta wave
//! chegou, e seis entradas de layout não cabem lá. *O corte por
//! responsabilidade é mais barato do que a isenção que o evitaria* — e aqui
//! ele é o certo de qualquer maneira: um sítio só sabe a disposição dos seis.
//!
//! ⚠️⚠️ **Os buffers existem SEMPRE, mesmo sem plano armado**, e isso não é
//! desperdício: o bind group por objecto é criado quando o slot nasce, e um
//! binding que aparece e desaparece obrigaria a reconstruir o layout — não o
//! bind, o LAYOUT, que é do pipeline. ⇒ sem plano eles são de um elemento e o
//! `armado` da configuração vale `0`, que é o que faz o shader devolver o
//! `in.vcolor` de sempre, ao bit.

use ph2d_mesh::Mesh;
use ph2d_mesh_colors::Tinta;
use wgpu::util::DeviceExt as _;

use crate::MeshRenderer;

/// Os seis buffers, e o que cabe em cada um hoje.
pub(super) struct TintaGpu {
    amostras: wgpu::Buffer,
    topo: wgpu::Buffer,
    origem: wgpu::Buffer,
    idx: wgpu::Buffer,
    pos: wgpu::Buffer,
    cfg: wgpu::Buffer,
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
    /// ⭐ **Há plano ligado?** — o espelho do `armado` que já foi escrito no
    /// device, para o `upload_tinta_at` não reescrever a configuração por
    /// quadro quando nada mudou.
    pub(super) armado: bool,
}

const N: usize = 6;
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

/// As seis entradas que o layout do grupo POR OBJECTO ganha.
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
    ]
}

/// A configuração: `lado`, `verts`, `arestas`, `armado`.
fn cfg_de(t: Option<&Tinta>) -> [u32; 4] {
    match t {
        // ⚠️ O `lado` mínimo é `1` mesmo desarmado: um `lado = 0` faria o
        //    shader dividir a retícula por zero se alguém o lesse por engano.
        None => [1, 0, 0, 0],
        Some(t) => [
            t.lado(),
            t.topologia().verts() as u32,
            t.topologia().arestas() as u32,
            1,
        ],
    }
}

impl TintaGpu {
    /// Os seis buffers de um elemento — o estado de quem não tem plano.
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
            amostras: um("ph2d-mesh tinta amostras", zero4, st),
            topo: um("ph2d-mesh tinta topo", zero4u, st),
            origem: um("ph2d-mesh tinta origem", zero4u, st),
            idx: um("ph2d-mesh tinta idx", zero4u, st),
            pos: um("ph2d-mesh tinta pos", zero4, st),
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
            armado: false,
        }
    }

    /// As seis entradas do bind group por objecto.
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
        t.topologia().payload(faces(), &mut pay);
        let mut tris = Vec::new();
        let mut origem = Vec::new();
        mesh.triangle_indices_com_origem(&mut tris, Some(&mut origem));

        let amostras: Vec<f32> = t.amostras().iter().flat_map(|c| *c).collect();
        let idx: Vec<u32> = tris.iter().flat_map(|t| *t).collect();
        let pos: Vec<f32> = mesh.positions().iter().flat_map(|p| *p).collect();

        let mut refez = false;
        let st = wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST;
        {
            let g = &mut self.slots[index].gpu.tinta;
            refez |= poe(
                device,
                queue,
                &mut g.amostras,
                &mut g.cap_amostras,
                bytemuck::cast_slice(&amostras),
                "amostras",
                st,
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
            refez |= poe(
                device,
                queue,
                &mut g.idx,
                &mut g.cap_idx,
                bytemuck::cast_slice(&idx),
                "idx",
                st,
            );
            refez |= poe(
                device,
                queue,
                &mut g.pos,
                &mut g.cap_pos,
                bytemuck::cast_slice(&pos),
                "pos",
                st,
            );
            queue.write_buffer(&g.cfg, 0, bytemuck::cast_slice(&cfg_de(Some(t))));
            g.armado = true;
        }
        if refez {
            self.refaz_bind(device, index);
        }
    }
}

impl MeshRenderer {
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
