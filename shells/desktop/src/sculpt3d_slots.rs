//! **A TABELA DE SLOTS DO DEVICE** — que peça mora em cada índice do
//! renderizador, e o que cada slot precisa receber.
//!
//! Filho (`#[path]`) de [`super`] para alcançar os campos privados; o corte é
//! entre *o que a forma DOA* (a [`super::donation`], de onde este código veio) e
//! *o que o DEVICE tem*. Ele nasceu com o isolamento: enquanto toda peça era
//! desenhada, a `k`-ésima da lista era o `k`-ésimo slot e a pergunta não
//! existia.
//!
//! # A pergunta certa é *o device tem ESTA peça?*, não *eu já a mandei?*
//!
//! ⚠️ **E a versão anterior fazia a segunda, com um bug VIVO.** O `uploaded` é
//! memória da CENA (*a CPU já mandou esta malha?*) e os slots são do DEVICE —
//! duas coisas diferentes, que coincidiam por acidente enquanto o índice da
//! lista era o do slot. Apagar a peça 0 de três desloca as sobreviventes para os
//! slots 0 e 1 **com `uploaded == true`**, e a guarda antiga (`object_count() <=
//! i`) não pega: os slots EXISTEM, só descrevem outra peça. O resultado é a
//! sobrevivente desenhada com a geometria da peça morta, na pose dela própria —
//! sem erro, sem warning, e invisível numa cena de primitivas iguais, que é
//! exatamente a cena que o smoke monta.
//!
//! A cura é a **testemunha**: o slot carrega o [`ObjectId`] de quem mora nele, e
//! a sincronização pergunta a ele. Isso mata a classe inteira de uma vez — o
//! delete que desloca, o isolamento que compacta, a fusão que troca N peças por
//! uma, o undo que recoloca no fim — em vez de enumerar os gestos que precisam
//! invalidar, que é a lista que nasce incompleta no dia seguinte.

use super::{ObjectId, Sculpt3dScene};

/// O que o slot `k` precisa receber neste frame.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) enum SlotJob {
    /// **A malha inteira** — o slot não tem esta peça (ou não existe ainda), ou
    /// a CPU a reconstruiu.
    Full,
    /// **Só os vértices que se moveram** — o caminho de um dab.
    Region,
    /// **Nada**: o device já tem esta peça, e ela não mudou.
    Skip,
}

/// O que a decisão precisa saber sobre uma peça à vista. Quatro fatos, e nenhum
/// deles é uma malha: é isso que a torna gateável sem device.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) struct PieceState {
    /// O índice na lista de objetos da cena.
    pub(super) piece: usize,
    pub(super) id: ObjectId,
    /// A CPU já mandou esta malha alguma vez?
    pub(super) uploaded: bool,
    /// Há vértices esperando o upload incremental?
    pub(super) dirty: bool,
}

/// Uma linha do plano: que peça da CENA mora no slot, e o que ele recebe.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) struct SlotPlan {
    pub(super) piece: usize,
    pub(super) job: SlotJob,
}

/// **O QUE O DEVICE PRECISA RECEBER** — a decisão, sem device nenhum.
///
/// ⚠️ **Função livre e pura de propósito**, o precedente do `place` do import e
/// do `swap_window` do desfazer: uma [`Sculpt3dScene`] não nasce sem um
/// `wgpu::Device`, então uma decisão que morasse dentro dela só seria alcançável
/// por um gate de GPU — e o defeito que ela previne (um slot descrevendo outra
/// peça) **não levanta erro**: ele desenha. Aqui o gate dirige a pergunta e lê a
/// resposta.
///
/// A `k`-ésima peça à vista mora no slot `k`; `slots` é o que o device tem hoje.
pub(super) fn plan_slots(slots: &[ObjectId], visible: &[PieceState]) -> Vec<SlotPlan> {
    visible
        .iter()
        .enumerate()
        .map(|(k, p)| {
            // A TESTEMUNHA: o slot `k` diz de quem ele é. Se ele não é desta
            // peça, o que está lá é a geometria de outra — e nenhuma pergunta
            // sobre o que a CPU mandou responde isso.
            let holds = slots.get(k) == Some(&p.id);
            let job = if !holds || !p.uploaded {
                SlotJob::Full
            } else if p.dirty {
                SlotJob::Region
            } else {
                SlotJob::Skip
            };
            SlotPlan {
                piece: p.piece,
                job,
            }
        })
        .collect()
}

impl Sculpt3dScene {
    /// O plano deste frame — os fatos da cena, pela porta acima.
    ///
    /// ⚠️ A alocação é uma por FRAME, no laço de desenho — não no de dab, que é
    /// onde este módulo mede custo (`sculpt3d.rs::sculpt_at`).
    pub(super) fn slot_plan(&self) -> Vec<SlotPlan> {
        let visible: Vec<PieceState> = self
            .visible_pieces()
            .map(|piece| {
                let o = &self.objects[piece];
                PieceState {
                    piece,
                    id: o.id,
                    uploaded: o.uploaded,
                    dirty: !o.dirty.is_empty(),
                }
            })
            .collect();
        plan_slots(&self.slots, &visible)
    }

    /// Põe no device o que a CPU mudou. **Porta única do upload** — o laço de
    /// desenho e a doação passam pela MESMA, senão as duas rotas discordariam
    /// sobre que malha o dispositivo tem.
    ///
    /// ⚠️ **Ela sobe as peças À VISTA, compactadas**: a `k`-ésima visível vira o
    /// slot `k`. É assim que o isolamento some com as outras sem o renderizador
    /// aprender o que é visibilidade — e é assim que a DOAÇÃO passa a doar só o
    /// que se vê, de graça, porque o G-buffer sai do mesmo renderizador.
    pub(super) fn sync_mesh(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) {
        let plan = self.slot_plan();
        self.renderer.truncate_objects(plan.len());
        self.slots.truncate(plan.len());
        for (k, line) in plan.into_iter().enumerate() {
            let i = line.piece;
            self.renderer.set_pose(k, self.objects[i].pose);
            // ⚠️ **O preview é posto em dia ANTES do upload, e com o MESMO
            // `dirty` que a posição vai subir** — é essa coincidência que o
            // mantém barato: o padrão é lido na posição do vértice, então os
            // vértices que se moveram são exatamente os que ele deixou de
            // descrever. Depois do upload a lista está limpa e a janela seria
            // vazia; o barro ficaria com o padrão de antes do traço.
            {
                let brush = self.brush.clone();
                let armed = self.alpha_preview;
                let obj = &mut self.objects[i];
                let mesh = obj.stack.mesh();
                let moved = std::mem::take(&mut obj.dirty);
                obj.preview.refresh(mesh, &brush, armed, &moved);
                obj.dirty = moved;
            }
            match line.job {
                SlotJob::Skip => {}
                SlotJob::Full => {
                    self.renderer.upload_at(
                        device,
                        queue,
                        k,
                        self.objects[i].stack.mesh(),
                        &self.objects[i].preview.values,
                    );
                    self.objects[i].uploaded = true;
                    self.objects[i].dirty.clear();
                    self.objects[i].preview.whole_dirty = false;
                }
                SlotJob::Region => {
                    // A região, e o cheio como fallback: `upload_region_at`
                    // recusa quando a topologia mudou, e recusar é a resposta
                    // certa — escrever a região sobre um buffer de outra
                    // topologia poria bytes válidos nos vértices errados.
                    let ok = self.renderer.upload_region_at(
                        queue,
                        k,
                        self.objects[i].stack.mesh(),
                        &self.objects[i].dirty,
                        &self.objects[i].preview.values,
                    );
                    if !ok {
                        self.renderer.upload_at(
                            device,
                            queue,
                            k,
                            self.objects[i].stack.mesh(),
                            &self.objects[i].preview.values,
                        );
                        self.objects[i].preview.whole_dirty = false;
                    }
                    self.objects[i].dirty.clear();
                }
            }
            // ⚠️ **FORA do `match`, e o defeito que isto conserta é MUDO.** A
            // janela do dab não cobre uma troca de padrão — girar o eixo não
            // move um vértice —, então o `dirty` fica VAZIO, o plano cai em
            // `Skip`, e dentro do braço `Region` esta linha nunca rodaria no
            // caso exato que ela existe para cobrir: o barro ficaria com o
            // padrão anterior enquanto o quadro do painel já mostra o novo. Os
            // dois discordando é o pior desfecho possível para um preview.
            if self.objects[i].preview.whole_dirty
                && self
                    .renderer
                    .upload_preview_at(queue, k, &self.objects[i].preview.values)
            {
                self.objects[i].preview.whole_dirty = false;
            }
            // ⚠️ **As ARESTAS, só com a malha armada.** A lista custa até 24 B por
            // vértice e a maioria esculpe sem ela; construí-la junto com a malha
            // faria todo artista pagar pela vista de um. A porta é idempotente,
            // então chamá-la por frame é um `is_some()` — e é ELA que reconstrói
            // depois de um remesh, porque o `upload_at` acima derruba a lista
            // exatamente quando a topologia pode ter mudado.
            if self.wireframe {
                self.renderer
                    .upload_wire_at(device, k, self.objects[i].stack.mesh());
            }
            // A testemunha é escrita ao lado da escrita que ela testemunha: uma
            // linha adiante, na mesma função. Duas funções para as duas metades
            // seriam duas verdades sobre o que o device tem.
            let id = self.objects[i].id;
            if self.slots.len() <= k {
                self.slots.push(id);
            } else {
                self.slots[k] = id;
            }
        }
    }
}

#[cfg(test)]
#[path = "sculpt3d_slots_tests.rs"]
mod tests;
