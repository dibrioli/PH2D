//! A ponte entre o traço de escultura e a lei do pincel de POSE.
//!
//! ⚠️⚠️ **Este verbo desvia antes do `dab_core` e antes do espelho, e as duas
//! coisas têm razões DIFERENTES:**
//!
//! - **antes do `dab_core`**, porque não há núcleo por-vértice nenhum: a lei não
//!   tem atenuação radial, e um vértice a dez raios do cursor pode mover-se por
//!   inteiro porque a região cresce pela **ligação** da malha;
//! - **antes do espelho**, porque a lei **já resolve os oito octantes numa
//!   passagem só** (a simetria vive nos mapas afins). Passar pela expansão
//!   genérica aplicá-la-ia **duas** vezes.
//!
//! ⭐⭐ **E é aqui que ganhamos ao alvo sem tocar na lei:** a cadeia constrói-se
//! **uma vez, no pen-down**, e é reutilizada até o traço acabar. O alvo
//! reconstrói tudo a cada movimento do rato — mesmo **sem traço nenhum**, só
//! para desenhar o indicador sob o cursor —, e é essa a causa registada em
//! quatro relatos públicos de o editor engasgar em malha densa.

use ph2d_mesh::{Face, Mesh};

use crate::{Brush, Dab, Symmetry};

/// O que sobrevive de um evento para o outro dentro de um traço de pose.
#[derive(Clone, Debug)]
pub(super) struct PoseSessao {
    pose: ph2d_pose::Pose,
    /// As posições da malha **no início do traço**.
    ///
    /// ⚠️ **A malha INTEIRA, e não a pegada.** Os outros verbos congelam só os
    /// vértices tocados porque o raio limita quem pode mover-se; aqui não há
    /// esse limite (espec §13: *«nenhuma parte da malha é excluída pelo
    /// raio»*), e a lei mede todo deslocamento a partir daqui.
    p0: Vec<[f32; 3]>,
}

impl crate::SculptStroke {
    /// ⭐ **A cadeia VIVA do traço em curso** — a porta por onde o indicador
    /// ([`crate::pose_previa`]) lê o osso já dobrado pelo arrasto, em vez de
    /// construir uma cadeia sua.
    ///
    /// ⚠️ `None` fora de um traço de pose, que é **exactamente** a condição em
    /// que o indicador tem de construir a sua.
    pub(crate) fn pose_sessao(&self) -> Option<&ph2d_pose::Pose> {
        self.pose.as_ref().map(|s| &s.pose)
    }

    /// Um evento de pose. Devolve quantos vértices se moveram.
    pub(super) fn pose_dab(
        &mut self,
        mesh: &mut Mesh,
        brush: &Brush,
        dab: &Dab,
        sym: Symmetry,
    ) -> usize {
        let mut ctrl = brush.pose.lei(brush);
        ctrl.simetria = [sym.x, sym.y, sym.z];

        let mut sessao = match self.pose.take() {
            Some(s) => s,
            None => match Self::pose_comecar(mesh, dab, &ctrl) {
                Some(s) => {
                    self.pose_construcoes += 1;
                    s
                }
                // A malha não tem vértices, ou não há nada ao alcance: o traço
                // inteiro não move nada. ⚠️ O alvo cala-se aqui; nós ao menos
                // não fingimos ter começado.
                None => return 0,
            },
        };

        let evento = ph2d_pose::Evento {
            // ⭐ `Grip::Hold` promete que o `pull` é o deslocamento **TOTAL**
            // desde o pen-down — que é exactamente o `G` da lei (`T = C + G·s`).
            arrasto: dab.pull,
            dx_pixels: brush.pose.arrasto_x_pixels,
        };
        // A curva é a do PINCEL, avaliada pela crate que é dona dela.
        let curva = |p: f32| brush.falloff.weight(p);
        sessao.pose.evento(&ctrl, &evento, &curva);

        let mut saida = std::mem::take(&mut self.pose_saida);
        sessao.pose.posicoes(
            &ctrl,
            &sessao.p0,
            ph2d_pose::Fatores {
                // §9 — a máscara escala o **deslocamento**; não muda os pesos
                // nem o pivô. A lei do factor (`1 − máscara`) é a mesma que o
                // `mask_ops::free_weight` desta crate já aplica aos outros
                // verbos.
                mascara: mesh.masks(),
                ..Default::default()
            },
            &mut saida,
        );

        // ⛔⛔ **A ESCRITA É EM TRÊS PASSOS, E O DO MEIO É O UNDO** (report do
        // dono, 2026-09-14: *«undo/redo não funciona para esse pincel»*).
        //
        // O `close_stroke` da cena grava a janela `touched()` + `base_positions()`
        // e **devolve cedo** quando ela está vazia; quem a enche é o `capture`,
        // que vive no laço por-vértice do `dab_core` — e este verbo desvia antes
        // dele ([`crate::Verb::resolve_a_propria_regiao`]). ⇒ o traço movia a
        // malha e não deixava rasto nenhum para o `Ctrl+Z`. *Uma janela vazia e
        // um gesto que não fez nada são o mesmo byte para quem grava.*
        //
        // ⚠️ **O `capture` tem de correr ANTES da escrita**, e é isso que o
        // parte em dois laços: ele lê `mesh.positions()` para congelar o `pre`, e
        // depois de escrever o `pre` seria a pose deste evento. Ele é
        // **idempotente** (carimbo por época), então um vértice que entre na
        // janela no 3.º evento traz na mesma a posição do pen-down — a malha só
        // é escrita a partir do `p0`, logo quem ainda não se moveu está onde
        // nasceu.
        //
        // ⛔ **É o mesmo desvio que o TECIDO já fazia** (`stroke_cloth`, que
        // chama o `capture` à mão sobre a região dele). A pose é que não o fazia.
        let mut movidos = std::mem::take(&mut self.moved);
        movidos.clear();
        for (i, (destino, actual)) in saida.iter().zip(mesh.positions()).enumerate() {
            if actual != destino {
                movidos.push(u32::try_from(i).unwrap_or(u32::MAX));
            }
        }
        for &v in &movidos {
            self.capture(mesh, v);
        }
        let posicoes = mesh.positions_mut();
        for &v in &movidos {
            posicoes[v as usize] = saida[v as usize];
        }
        self.moved = movidos;
        self.pose_saida = saida;
        self.pose = Some(sessao);

        // ⚠️ **ESCRITA, e não herdada** — a mesma linha que o tecido paga: o
        // `last_gpu_dirty` escolhe a janela de upload por esta bandeira e o
        // `begin` não a reinicia. Sem ela, um traço de pose logo a seguir a um
        // de MÁSCARA subiria a janela errada, e o defeito seria *«a malha mudou
        // e a tela não»* com todos os gates de CPU verdes.
        self.last_paints_mask = false;
        if self.moved.is_empty() {
            return 0;
        }
        mesh.refresh_region(&self.moved, &mut self.region);
        self.moved.len()
    }

    /// O pen-down: constrói a cadeia inteira, **uma vez** (espec §10).
    fn pose_comecar(mesh: &Mesh, dab: &Dab, ctrl: &ph2d_pose::Controlos) -> Option<PoseSessao> {
        let p0 = mesh.positions().to_vec();
        // ⛔ **A ocultação de vértices não está ligada, e é uma ausência
        // DECLARADA:** a lei aceita-a (`escondido`), o corpus do oráculo nunca
        // esconde nada, e esta malha não expõe a bandeira por vértice. *Escrito
        // aqui para que quem a ligar saiba onde ela entra — e não a descubra
        // por um vértice que se recusa a mexer.*
        let escondido = vec![false; p0.len()];
        let mut viz = ph2d_pose::Vizinhanca::construir(
            p0.len(),
            mesh.faces().iter().map(Face::verts),
            &escondido,
        );
        if !ctrl.so_conectado {
            // ⛔ `O(V²)`, declarado. No alvo ele é repetido a cada movimento de
            // câmara; aqui é pago **uma vez por traço**.
            viz.ligar_pecas(&p0, ctrl.distancia_max_entre_pecas);
        }
        let eleito = ph2d_pose::cadeia::mais_proximo_global(&p0, &escondido, dab.center)?;
        let pose = ph2d_pose::Pose::comecar(&viz, &p0, &escondido, eleito, dab.center, ctrl);
        Some(PoseSessao { pose, p0 })
    }
}
