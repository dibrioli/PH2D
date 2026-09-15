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
        // ⛔⛔ **A INVERSÃO É A PONTE, e sem ela o modo de torção é INERTE.**
        //
        // As duas casas escrevem a curva do pincel com argumentos **opostos**, e
        // cada uma está certa em casa: para a [`ph2d_pose::Curva`] o argumento é
        // *quanto FALTA* (`1` no segmento mais perto do cursor, §5.2, e a lei
        // dela amostra em `1 − i/n`), e para o [`crate::Falloff::weight`] é
        // *quanto já se ANDOU* (`1` na borda do carimbo, onde o peso é zero).
        //
        // ⚠️ Ligadas sem a inversão, o 1.º segmento recebia `weight(1,0)`, que é
        // **`0,0` em TODAS as doze curvas** — e com o valor de fábrica (`1`
        // segmento) o pincel inteiro não rodava um vértice. Medido: `2,98e-8`
        // (ruído de `f32`) a `1` segmento e `5,96e-8` a `2` e a `4`, com o
        // arrasto de `40 px` a chegar intacto. Com a inversão: `2,94e-1`
        // (`Constant`) contra `1,84e-2` (`Sharper`) a `2` segmentos.
        //
        // ⛔⛔ **É a MESMA ponte que o pincel de CONTORNO pagou um dia antes**
        // ([`crate::stroke_boundary`], que escreve `weight(1.0 - p)` pela mesma
        // razão) — e **o corpus de paridade não pode apanhá-la nas duas**: a
        // bancada corre a `ph2d-pose` **directamente**, com a convenção dela
        // (`ph2d_pose::suave`), logo *uma paridade medida a montante de uma
        // conversão não afirma nada sobre a conversão*. Quem a apanhou foi o
        // [`censo dos knobs`](ph2d_app_sculpt3d), a medir o barro pela porta do
        // produto.
        let curva = |p: f32| brush.falloff.weight(1.0 - p);
        sessao.pose.evento(&ctrl, &evento, &curva);

        let mut saida = std::mem::take(&mut self.pose_saida);
        let fatores = ph2d_pose::Fatores {
            // §9 — a máscara escala o **deslocamento**; não muda os pesos
            // nem o pivô. A lei do factor (`1 − máscara`) é a mesma que o
            // `mask_ops::free_weight` desta crate já aplica aos outros
            // verbos.
            mascara: mesh.masks(),
            ..Default::default()
        };
        sessao.pose.posicoes(&ctrl, &sessao.p0, fatores, &mut saida);
        self.alisa_a_pose(mesh, brush, &sessao, fatores, &mut saida);

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

    /// ⭐⭐⭐ **A AUTO-SUAVIZAÇÃO DESTE PINCEL SEGUE OS PESOS, E NÃO O RAIO.**
    ///
    /// # Porque ela existe
    ///
    /// O painel oferece o *Auto smooth* com este verbo na mão desde que ele
    /// nasceu, e ele **desvia antes do laço por-vértice** onde o passe genérico
    /// corre ([`crate::SculptStroke::dab`] chama-o como um segundo
    /// [`crate::SculptStroke::dab_core`] sobre a mesma pegada) ⇒ *o artista
    /// arrastava o controlo e o barro não sentia nada*. Foi o censo dos knobs
    /// que o mediu (`0,000e0` entre as duas pontas da faixa).
    ///
    /// # Porque ela NÃO é o passe genérico emprestado
    ///
    /// ⛔ O passe genérico alisa **dentro do raio do carimbo**, e a deformação
    /// deste verbo alcança muito mais longe — a região dele cresce pela
    /// **ligação** da malha, não pelo raio. Emprestá-lo reproduziria à letra um
    /// defeito que os próprios autores do alvo registam em público
    /// (`#133792`: *«o efeito dela desaparece longe do cursor»*), e a espec
    /// deste pincel manda o contrário com todas as letras — §15: *«**Não
    /// copiar**: se oferecermos auto-suavização aqui, ela segue **os pesos**,
    /// não o raio»*, repetido no item 18 da lista de verificação dela.
    ///
    /// ⇒ o peso de cada vértice é o [`ph2d_pose::cadeia::Cadeia::peso_total`]
    /// — a mesma grandeza que decide **quanto** ele acompanha a cadeia —, vezes
    /// o factor do §9 pela porta que já o possui ([`ph2d_pose::Fatores::de`]):
    /// *um vértice mascarado não se mexe, e também não se alisa.*
    ///
    /// # O orçamento é o da casa
    ///
    /// As passadas vêm de [`crate::auto_smooth::iteration_strengths`], que é a
    /// mesma lei que todo outro verbo usa — ⛔ **não** um lerp com o número do
    /// slider, que reproduz a referência só abaixo de `0,24`. E a pergunta *«ele
    /// está armado?»* é feita à porta única ([`crate::Brush::auto_smooth_brush`]),
    /// que é a MESMA que o painel consulta para pintar a fileira: duas cópias
    /// divergiriam num knob que aparece e não faz nada — que é precisamente o
    /// defeito que isto cura.
    ///
    /// # O que ela NÃO muda
    ///
    /// ⚠️ **No ponto neutro (`auto_smooth = 0`) o caminho é byte-idêntico**, por
    /// construção: a porta devolve `None` e nem a adjacência é consultada. É o
    /// que mantém os `69` traços do oráculo intocados — ⛔ e eles não poderiam
    /// medir isto de qualquer forma: a bancada corre a `ph2d-pose`
    /// **directamente**, e esta lei é nossa.
    ///
    /// ⚠️ **E ela NÃO quebra o rebase do §7.2:** o resultado continua a ser uma
    /// função pura de `p0` e do arrasto TOTAL — ela corre sobre a `saida` deste
    /// evento, nunca sobre a malha já escrita, logo não acumula com a contagem
    /// de eventos que o ponteiro entregou.
    fn alisa_a_pose(
        &mut self,
        mesh: &Mesh,
        brush: &Brush,
        sessao: &PoseSessao,
        fatores: ph2d_pose::Fatores<'_>,
        saida: &mut Vec<[f32; 3]>,
    ) {
        if brush.auto_smooth_brush().is_none() {
            return;
        }
        let cadeia = sessao.pose.cadeia();
        let adj = mesh.adjacency();
        let mut outro = std::mem::take(&mut self.pose_alisado);
        for passe in crate::auto_smooth::iteration_strengths(brush.auto_smooth).as_slice() {
            outro.clear();
            outro.extend_from_slice(saida);
            for v in 0..saida.len() {
                let w = passe.weight * cadeia.peso_total(v) * fatores.de(v);
                if w <= 0.0 {
                    continue;
                }
                let base = saida[v];
                let media = ph2d_mesh::ring_average(adj, v as u32, base, |nb| saida[nb as usize]);
                let m = 1.0 - w;
                outro[v] = [
                    base[0] * m + media[0] * w,
                    base[1] * m + media[1] * w,
                    base[2] * m + media[2] * w,
                ];
            }
            std::mem::swap(saida, &mut outro);
        }
        self.pose_alisado = outro;
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
