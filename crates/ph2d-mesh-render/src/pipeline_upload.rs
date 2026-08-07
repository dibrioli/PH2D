//! **COMO A MALHA SOBE PARA O DEVICE** — filho do [`super`], não irmão, porque
//! escreve os buffers PRIVADOS do slot.
//!
//! ⚠️ **O corte é por RESPONSABILIDADE:** o pai diz *como a forma é DESENHADA*
//! (os passes, a câmera, a profundidade, o G-buffer); aqui fica *como ela CHEGA
//! lá* — as três rotas de upload e os dois canais cuja ausência precisa ser
//! resolvida antes de virar bytes.
//!
//! As três rotas não são variações de uma: a **inteira** reconstrói quando a
//! capacidade não cabe, a **que reusa** reescreve tudo quando a topologia mudou
//! mas o tamanho serve, e a **parcial** toca só a janela de um dab. Confundi-las
//! é como um canal passa a viajar de carona numa mudança que não é dele.

use ph2d_mesh::{Mesh, Pose};
use wgpu::util::DeviceExt as _;

use crate::upload;

use super::{MeshGpu, MeshRenderer, ObjectRaw, Slot};

/// A máscara da malha, ou **zeros** para uma que ninguém mascarou.
///
/// ⚠️ O plano é `Option` na malha (uma esfera intocada não paga 4 B/vértice), e
/// o device quer um buffer sempre presente: um pipeline por presença de máscara
/// seriam DUAS respostas a *"como esta malha é desenhada"*, divergindo no único
/// lugar onde ninguém lê um número de volta.
fn masks_of<'a>(mesh: &'a Mesh, scratch: &'a mut Vec<f32>) -> &'a [f32] {
    match mesh.masks() {
        Some(m) => m,
        None => {
            scratch.clear();
            scratch.resize(mesh.vert_count(), ph2d_mesh::DEFAULT_MASK);
            scratch
        }
    }
}

/// O AO por vértice, com a ausência resolvida — o gêmeo exato do
/// [`masks_of`], e pelo mesmo motivo: o device não tem `Option`.
///
/// ⚠️ **A ausência sobe como `DEFAULT_AO` (céu aberto) e não como zero.** Uma
/// malha que ninguém assou tem de renderizar EXATAMENTE como antes de este
/// canal existir; se a ausência subisse escurecendo, toda peça nasceria suja e
/// o artista iria procurar a sujeira no shader.
fn ao_of<'a>(mesh: &'a Mesh, scratch: &'a mut Vec<f32>) -> &'a [f32] {
    match mesh.ao() {
        Some(a) => a,
        None => {
            scratch.clear();
            scratch.resize(mesh.vert_count(), ph2d_mesh::DEFAULT_AO);
            scratch
        }
    }
}

impl MeshRenderer {
    /// Sobe a malha do objeto `index`. Reusa os buffers quando cabem — um dab
    /// que só move vértices não realoca nada.
    pub fn upload_at(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        index: usize,
        mesh: &Mesh,
    ) {
        mesh.triangle_indices(&mut self.scratch_indices);
        let verts = mesh.vert_count();
        let idx = self.scratch_indices.len();
        let index_count = (idx * 3) as u32;

        let fits = self
            .slots
            .get(index)
            .is_some_and(|s| s.gpu.vert_capacity >= verts && s.gpu.index_capacity >= idx);

        if fits {
            let g = &mut self
                .slots
                .get_mut(index)
                .expect("acabou de ser conferido")
                .gpu;
            queue.write_buffer(&g.positions, 0, bytemuck::cast_slice(mesh.positions()));
            queue.write_buffer(&g.normals, 0, bytemuck::cast_slice(mesh.normals()));
            queue.write_buffer(
                &g.masks,
                0,
                bytemuck::cast_slice(masks_of(mesh, &mut self.scratch_masks)),
            );
            queue.write_buffer(&g.curvatures, 0, bytemuck::cast_slice(mesh.curvatures()));
            queue.write_buffer(&g.curv_world, 0, bytemuck::cast_slice(mesh.curv_world()));
            queue.write_buffer(
                &g.ao,
                0,
                bytemuck::cast_slice(ao_of(mesh, &mut self.scratch_ao)),
            );
            queue.write_buffer(&g.indices, 0, bytemuck::cast_slice(&self.scratch_indices));
            g.index_count = index_count;
            // ⚠️ **A topologia pode ter mudado, então as arestas morrem aqui.**
            // Esta rota é a do buffer que CABE — e caber é sobre tamanho, não
            // sobre as faces serem as mesmas: um remesh que devolva a mesma
            // contagem reusa os buffers e reescreve os índices. Uma lista de
            // arestas sobrevivente descreveria a malha anterior, e o wireframe
            // desenharia um grafo que a superfície não tem.
            g.wire = None;
            g.wire_count = 0;
            return;
        }

        let vb = |label, data: &[u8]| {
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(label),
                contents: data,
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            })
        };
        let gpu = MeshGpu {
            positions: vb("ph2d-mesh pos", bytemuck::cast_slice(mesh.positions())),
            normals: vb("ph2d-mesh nrm", bytemuck::cast_slice(mesh.normals())),
            masks: vb(
                "ph2d-mesh mask",
                bytemuck::cast_slice(masks_of(mesh, &mut self.scratch_masks)),
            ),
            curvatures: vb("ph2d-mesh curv", bytemuck::cast_slice(mesh.curvatures())),
            curv_world: vb("ph2d-mesh curvw", bytemuck::cast_slice(mesh.curv_world())),
            ao: vb(
                "ph2d-mesh ao",
                bytemuck::cast_slice(ao_of(mesh, &mut self.scratch_ao)),
            ),
            indices: device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("ph2d-mesh idx"),
                contents: bytemuck::cast_slice(&self.scratch_indices),
                usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
            }),
            index_count,
            vert_capacity: verts,
            index_capacity: idx,
            wire: None,
            wire_count: 0,
        };
        if let Some(slot) = self.slots.get_mut(index) {
            // O bind group e o buffer de pose sobrevivem: só a geometria mudou.
            slot.gpu = gpu;
            return;
        }
        // Um objeto novo. ⚠️ Só o PRÓXIMO da lista pode nascer aqui — um índice
        // salteado deixaria um buraco que o desenho percorreria como se fosse
        // um objeto, e a pose de todos depois dele apontaria para a malha
        // errada. A cena constrói a lista em ordem; se este `if` disparar, é
        // porque alguém a construiu de outro jeito.
        if index != self.slots.len() {
            return;
        }
        let model = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("ph2d-mesh model"),
            size: size_of::<ObjectRaw>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let bind = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("ph2d-mesh object bind"),
            layout: &self.obj_bgl,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: model.as_entire_binding(),
            }],
        });
        self.slots.push(Slot { gpu, model, bind });
        if self.poses.len() < self.slots.len() {
            self.poses.resize(self.slots.len(), Pose::IDENTITY);
        }
    }

    /// Sobe **só os vértices que um dab moveu** — posições e normais.
    ///
    /// Devolve `false` quando não há com que reconciliar (nada subiu ainda, ou a
    /// contagem de vértices mudou), e aí o chamador cai no [`Self::upload`]
    /// cheio. Recusar é a resposta certa: subir uma região sobre um buffer de
    /// outra topologia escreveria bytes válidos nos vértices errados, e o
    /// sintoma seria geometria puxada para lugares que ninguém tocou.
    ///
    /// ⚠️ **Os ÍNDICES não são reenviados**, e é isso que torna o caminho barato:
    /// um dab move vértices e não muda a topologia. Quem mudar a topologia
    /// (remesh, subdivisão, dyntopo — a W4) passa pelo `upload` cheio.
    ///
    /// ⚠️ **A normal viaja junto com a posição, sempre.** São dois buffers, mas
    /// um fato só: o `refresh_region` já recomputou as normais de todo vértice
    /// que este `moved` contém, e subir metade deixaria a malha iluminada por
    /// normais de antes do dab — um erro que só aparece na tela, nunca num
    /// número.
    pub fn upload_region_at(
        &mut self,
        queue: &wgpu::Queue,
        index: usize,
        mesh: &Mesh,
        moved: &[u32],
    ) -> bool {
        let Some(slot) = self.slots.get(index) else {
            return false;
        };
        if slot.gpu.vert_capacity != mesh.vert_count() {
            return false;
        }
        if moved.is_empty() {
            return true;
        }
        self.scratch_moved.clear();
        self.scratch_moved.extend_from_slice(moved);
        self.scratch_moved.sort_unstable();
        self.scratch_moved.dedup();
        upload::plan_runs(&self.scratch_moved, &mut self.scratch_runs);
        let masks = masks_of(mesh, &mut self.scratch_masks);
        let g = &self.slots.get(index).expect("conferido acima").gpu;

        let stride = size_of::<[f32; 3]>() as u64;
        for &(a, b) in &self.scratch_runs {
            let (a, b) = (a as usize, (b as usize).min(mesh.vert_count()));
            if a >= b {
                continue;
            }
            let at = a as u64 * stride;
            queue.write_buffer(
                &g.positions,
                at,
                bytemuck::cast_slice(&mesh.positions()[a..b]),
            );
            queue.write_buffer(&g.normals, at, bytemuck::cast_slice(&mesh.normals()[a..b]));
            // ⚠️ **A máscara viaja na MESMA janela, sempre.** Um dab é de
            // geometria ou de máscara — nunca dos dois — então uma das duas
            // cópias reescreve bytes idênticos, e a alternativa (perguntar de
            // qual canal é este dab) seria o chamador conhecendo a regra que o
            // `last_gpu_dirty` existe para responder.
            queue.write_buffer(&g.masks, a as u64 * 4, bytemuck::cast_slice(&masks[a..b]));
            // **E a CURVATURA também**, pela mesma janela e pelo mesmo motivo.
            // ⚠️ Aqui ela é ainda mais obrigatória que a máscara: a lista que
            // chega é o `refreshed()` do `RegionScratch`, que é exatamente o
            // conjunto cuja curvatura foi recomputada. Deixá-la de fora daria uma
            // superfície cujo RELEVO é novo e cuja LEITURA de cavidade é a de
            // antes do traço — a fresta desenhada onde ela estava, no lugar exato
            // em que o artista está olhando.
            queue.write_buffer(
                &g.curvatures,
                a as u64 * 4,
                bytemuck::cast_slice(&mesh.curvatures()[a..b]),
            );
            // **E a de MUNDO na mesma janela** — ela sai do MESMO gather que a
            // irmã acima, então a lista que a torna válida é a mesma. Deixá-la
            // de fora daria um espalhamento medido contra a curvatura de antes
            // do traço, e o sintoma seria o SSS desenhando a forma anterior.
            queue.write_buffer(
                &g.curv_world,
                a as u64 * 4,
                bytemuck::cast_slice(&mesh.curv_world()[a..b]),
            );
            // ⚠️ **E o AO NÃO entra aqui, de propósito.** Ele é a exceção porque
            // é a exceção na malha: um dab RECOMPUTA normal e curvatura, e só
            // ENVELHECE o AO — os valores por vértice ficam exatamente onde
            // estavam, então esta janela empurraria bytes idênticos. O canal só
            // muda quando um bake novo o instala ou quando a TOPOLOGIA mexe nele,
            // e as duas rotas passam pelo upload inteiro logo acima.
        }
        true
    }

    /// **Garante a lista de arestas do objeto `index`** — o buffer que o passe
    /// de wireframe desenha. No-op se ela já existe.
    ///
    /// ⚠️ **Porta SEPARADA do [`Self::upload_at`], e a razão é quem tem o quê.**
    /// A lista sai da adjacência da `Mesh`, que o device não guarda; e ela custa
    /// até 24 B por vértice, que ninguém deve pagar com o wireframe desligado.
    /// Então quem chama é a cena, no frame em que o artista arma a malha — e o
    /// custo aparece uma vez, ali, em vez de em todo `upload_at` de todo dab que
    /// muda topologia.
    ///
    /// Devolve `false` quando não há slot (a cena ainda não subiu o objeto).
    pub fn upload_wire_at(&mut self, device: &wgpu::Device, index: usize, mesh: &Mesh) -> bool {
        let Some(slot) = self.slots.get(index) else {
            return false;
        };
        if slot.gpu.wire.is_some() {
            return true;
        }
        crate::wire_indices(mesh, &mut self.scratch_indices_flat);
        let count = u32::try_from(self.scratch_indices_flat.len()).unwrap_or(u32::MAX);
        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("ph2d-mesh wire"),
            contents: bytemuck::cast_slice(&self.scratch_indices_flat),
            usage: wgpu::BufferUsages::INDEX,
        });
        let g = &mut self.slots.get_mut(index).expect("conferido acima").gpu;
        g.wire = Some(buffer);
        g.wire_count = count;
        true
    }
}
