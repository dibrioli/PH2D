//! **O ARM da topologia dinâmica** — o interruptor, o detalhe e o que um dab
//! faz com eles.
//!
//! Filho (`#[path]`) de [`super`]. O motor mora na `ph2d-mesh`
//! ([`ph2d_mesh::refine_in_sphere`]); aqui fica o que é do EDITOR: quando o
//! refino roda, o que ele custa ao traço em voo e como o desfazer o carrega.
//!
//! # Ele é opt-in, e a razão é dos autores do Blender
//!
//! *"o dyntopo foi sempre explicado como uma otimização quando na verdade é bem
//! mais lento e come muito mais memória"* — por isso o default é DESLIGADO, e o
//! padrão desta casa continua sendo `remesh sob comando + multires`
//! ([`docs/3D/04-Ferramentas/04.3-Topologia.md`]).
//!
//! # As DUAS metades, e a segunda chegou depois
//!
//! Armado, um dab **refina** onde a aresta é longa demais e **COLAPSA** onde ela
//! é curta demais. O limiar do colapso é DERIVADO do alvo do refino (ver
//! [`collapse_target`]) e não é um segundo slider: a relação entre os dois é a
//! histerese que impede o par de moer, e ela é propriedade do par, não
//! preferência. É o default do Blender (*Subdivide Collapse*); o SculptGL expõe
//! os dois e ship o de colapso em ZERO.
//!
//! # As três consequências de ligar, e nenhuma é escondida
//!
//! 1. **A peça é TRIANGULADA** ([`ph2d_mesh::Mesh::triangulate`]). Não é gosto:
//!    partir a aresta de um quad devolve um triângulo e um pentágono.
//! 2. **O desfazer de um traço passa a ser a MALHA INTEIRA.** A janela por
//!    vértice (`StrokeUndo::Stroke`) descreve *quais índices se moveram*, e um
//!    traço que muda a contagem não tem essa janela — os índices de depois não
//!    descrevem os de antes. É o mesmo argumento do remesh, e a mesma entrada
//!    (`Remeshed`), que já é uma TROCA simétrica: desfazer devolve a malha de
//!    antes, refazer devolve a de depois.
//! 3. **A pilha de multiresolução recusa.** Refinar a base sob níveis que são
//!    subdivisão dela deixaria cada nível descrevendo outra malha — o mesmo
//!    motivo pelo qual o remesh recusa, e a recusa é NOMEADA no log em vez de
//!    achatar a pilha em silêncio.

use ph2d_mesh::{
    Collapse, Refine, collapse_in_sphere, collapse_target, edge_target, refine_in_sphere,
};

use super::Sculpt3dScene;

/// O estado autorado do modo.
///
/// ⚠️ **`detail` é uma FRAÇÃO, não um comprimento** — quem o transforma em
/// alvo de aresta é o [`edge_target`], contra o raio do pincel. Guardar um
/// comprimento aqui daria ao artista um número que muda de significado quando
/// ele troca de pincel.
#[derive(Clone, Copy, PartialEq, Debug)]
pub(super) struct Dyntopo {
    pub(super) armed: bool,
    pub(super) detail: f32,
}

impl Default for Dyntopo {
    fn default() -> Self {
        // ⚠️ O default é DESARMADO (ver o cabeçalho) e o detalhe nasce no MEIO:
        // ligado no extremo fino, o primeiro traço num modelo grosso multiplica
        // a contagem de faces antes de o artista ter visto o que a tecla faz.
        Self {
            armed: false,
            detail: 0.5,
        }
    }
}

/// Os três degraus que a tecla percorre, e os nomes que o log usa.
///
/// ⚠️ **Três e não um slider contínuo**, porque a UI aqui é o teclado (a aba
/// Topologia é wave de UI, `04.3`): um passo por toque com nome próprio é
/// legível num log, e `0,5 → 0,53 → 0,56` não é.
pub(super) const DETAIL_STEPS: [(f32, &str); 3] = [(0.15, "grosso"), (0.5, "medio"), (1.0, "fino")];

impl Sculpt3dScene {
    /// Liga/desliga. Devolve `(ligado, faces trianguladas)` — o segundo é zero
    /// quando a malha já era de triângulos, e é o número que o log mostra
    /// porque **triangular muda a malha** e uma mudança calada é a que o artista
    /// descobre no save.
    pub(super) fn toggle_dyntopo(&mut self) -> (bool, usize) {
        let on = !self.dyntopo.armed;
        self.dyntopo.armed = on;
        if !on {
            return (false, 0);
        }
        // ⚠️ **Triangula ao LIGAR, não no primeiro dab.** No primeiro dab a
        // mudança chegaria junto com o barro e o artista não teria como separar
        // *"a ferramenta mudou minha malha"* de *"a ferramenta esculpiu"*.
        let Some(o) = self.obj_mut() else {
            return (true, 0);
        };
        let added = o.stack.mesh_mut().triangulate();
        if added > 0 {
            self.mesh_rebuilt();
        }
        (true, added)
    }

    /// O rótulo do degrau atual — **a mesma tabela que a tecla percorre**, e é
    /// isso que impede o log de dizer "médio" enquanto o motor usa outro número.
    pub(super) fn detail_label(&self) -> &'static str {
        DETAIL_STEPS
            .iter()
            .find(|(d, _)| (d - self.dyntopo.detail).abs() < 1e-6)
            .map_or("custom", |(_, l)| l)
    }

    /// O degrau seguinte do detalhe. Devolve o rótulo para o log.
    pub(super) fn cycle_detail(&mut self) -> &'static str {
        let at = DETAIL_STEPS
            .iter()
            .position(|(d, _)| (d - self.dyntopo.detail).abs() < 1e-6)
            .unwrap_or(0);
        let (d, label) = DETAIL_STEPS[(at + 1) % DETAIL_STEPS.len()];
        self.dyntopo.detail = d;
        label
    }

    /// **Refina onde o dab vai cair.** Chamado por [`Sculpt3dScene::sculpt_at`]
    /// ANTES do carimbo, e devolve `true` se a topologia mudou.
    ///
    /// ⚠️ **A ordem é a wave inteira: refinar e DEPOIS carimbar.** O contrário
    /// deposita o barro na malha grossa e adensa em cima — o detalhe nasce um
    /// dab atrasado, e o traço fica com a silhueta do que a malha era, não do
    /// que ela é.
    /// ⭐⭐ **O PINCEL DIZ PORQUE NÃO FEZ NADA** — uma vez por traço.
    ///
    /// ⚠️ **Report do dono, 2026-09-14: *«não vejo efeito com density»*.** O
    /// pincel estava certo e a queixa também: um verbo cujo efeito inteiro é
    /// sobre a topologia **parece partido** sempre que o passe não corre, e há
    /// **três** razões diferentes que o artista vê **iguais** — nada acontece.
    ///
    /// ⛔ Só para quem não tem outra forma de se mostrar
    /// ([`ph2d_sculpt3d::Verb::sem_lei_por_vertice`]): num `Draw` um passe que
    /// não parte nada é o caso **normal** (a malha já tem a densidade pedida
    /// ali), e uma linha de log por dab seria um log que ninguém lê.
    fn queixa_do_passe(&mut self, verbo: ph2d_sculpt3d::Verb, motivo: &str) {
        if !verbo.sem_lei_por_vertice() || self.dyn_queixa_dita {
            return;
        }
        self.dyn_queixa_dita = true;
        eprintln!("[sculpt3d] {} nao mudou a malha: {motivo}", verbo.label());
    }

    pub(super) fn refine_for_dab(
        &mut self,
        verbo: ph2d_sculpt3d::Verb,
        centre: [f32; 3],
        radius: f32,
    ) -> bool {
        if !self.dyntopo.armed {
            self.queixa_do_passe(
                verbo,
                "a topologia dinamica esta' DESLIGADA -- aperte P para ligar",
            );
            return false;
        }
        // ⭐⭐⭐ **A PERGUNTA É AO VERBO, e até 2026-09-14 ela não era feita.**
        //
        // Esta porta tem **um** chamador de produto — o braço do carimbo —, logo
        // a pergunta que o produto respondia era *«este gesto passou pelo
        // caminho do carimbo?»* e não *«este verbo cria superfície nova?»*.
        // ⚠️ É a mesma família de defeito que este módulo já pagou três vezes ao
        // contrário: *inferir uma propriedade do VERBO a partir do CAMINHO que o
        // gesto tomou.*
        //
        // ⛔ **Report do dono:** *«algumas tools que não deveriam fazer a
        // subdivisão … estão fazendo (como smooth) enquanto algumas que deveriam
        // não estão»*. A tabela inteira é pergunta de ORÁCULO
        // (`docs/3D/22_plano_quem_subdivide_no_dyntopo.md`) — **uma** célula
        // dela não é, e é a que esta linha cura: a **MÁSCARA** não move um
        // vértice, e medido nesta cena ela levava a peça de `830` para `1 331`
        // vértices.
        if !verbo.refina_no_dyntopo() && !verbo.colapsa_no_dyntopo() {
            return false;
        }
        // ⚠️ **Recusa com a pilha montada** (ver o cabeçalho). Silenciosa aqui
        // de propósito: a mensagem sai no ARM, uma vez, em vez de por dab.
        if self.level_count() > 1 {
            self.queixa_do_passe(
                verbo,
                "ha' uma pilha de multiresolucao montada -- J reverte-a",
            );
            return false;
        }
        let target = edge_target(radius, self.dyntopo.detail);
        // A peça ativa — a MESMA que o `sculpt_at` acabou de escolher pelo
        // `pick_active`, e é por isso que o índice basta aqui.
        //
        // ⚠️ O buffer de nascimentos é SCRATCH da cena, não estado do modo: o
        // `Dyntopo` guarda o que o artista autorou (o interruptor e o detalhe) e
        // segue `Copy`. Reusá-lo entre dabs é o que mantém o refino sem alocação
        // no caminho quente.
        let mut births = std::mem::take(&mut self.dyn_births);
        let mut remap = std::mem::take(&mut self.dyn_remap);
        let mesh = self.objects[self.active].stack.mesh_mut();
        let mut region = std::mem::take(&mut self.dyn_region);
        // ⚠️ **O COLAPSO PRIMEIRO, e a ordem é a do canal.** As duas metades
        // falam com o traço em voo por canais diferentes — o colapso por uma
        // RENUMERAÇÃO, o refino por uma lista de NASCIMENTOS —, e o segundo
        // afirma que a malha cresceu exactamente o que ele partiu. Refinar antes
        // faria a renumeração chegar depois e descrever índices que já não são os
        // que o `grow_with` acabou de instalar.
        // ⚠️ **As duas colunas são lidas em separado**, e não porque algum verbo
        // hoje as separe: elas são **leis independentes** que por acaso vivem na
        // mesma porta (um verbo pode querer relaxar densidade sem criar
        // detalhe), e o estudo pode separá-las. *Uma porta que lê uma coluna só
        // obriga quem a preenche a escolher pelas duas.*
        // ⚠️⚠️ **As duas colunas são lidas em SEPARADO, e o `&&` curto-circuita** —
        // um verbo que diga `false` não chega a chamar o motor. Elas são **leis
        // independentes** que por acaso vivem na mesma porta (um verbo pode
        // querer relaxar densidade sem criar detalhe), e o estudo pode separá-las.
        // *Uma porta que lê uma coluna só obriga quem a preenche a escolher
        // pelas duas.*
        //
        // ⛔ E o veredito é lido como **booleano** e não por um estado novo nos
        // enums da `ph2d-mesh`: `Collapse::Enough` significa *«nenhuma aresta
        // está sob o limiar»*, que é um facto sobre a MALHA — usá-lo para dizer
        // *«o verbo não pediu»* poria duas coisas diferentes no mesmo byte, que
        // é o defeito que este módulo acabou de pagar noutro sítio.
        let cut = verbo.colapsa_no_dyntopo()
            && matches!(
                collapse_in_sphere(
                    mesh,
                    centre,
                    radius,
                    collapse_target(target),
                    &mut remap,
                    &mut region,
                ),
                Collapse::Done { .. }
            );
        let done = verbo.refina_no_dyntopo()
            && matches!(
                refine_in_sphere(mesh, centre, radius, target, &mut births, &mut region),
                Refine::Done { .. }
            );
        self.dyn_region = region;
        if cut {
            // ⚠️ **Antes do `grow_with`, sempre.** Ele afirma que a malha cresceu
            // exactamente o número de nascimentos que chegaram, e a conta é
            // contra `slot.len()` — que ainda descreve a malha de antes do
            // colapso enquanto ninguém aplicar o remap.
            self.stroke.shrink_with(&remap);
        }
        self.dyn_remap = remap;
        if done {
            // O traço em voo sobrevive: os índices antigos não se moveram, e cada
            // vértice novo HERDA o `pre` do par que o gerou. Ver
            // `SculptStroke::grow_with` — chamar `begin` aqui jogaria fora o
            // `pre`, e tratar o novo como nunca-visto conta o deslocamento do
            // traço duas vezes (a agulha).
            let mesh = self.objects[self.active].stack.mesh();
            self.stroke.grow_with(mesh, &births);
        }
        self.dyn_births = births;
        if !done && !cut {
            // ⚠️ **A razão mais provável, e a mais difícil de adivinhar de
            // fora:** o alvo de aresta é `raio × f(detalhe)`, logo um detalhe
            // FINO pede arestas curtas e não há nada a colapsar. Medido no
            // percurso do dono: com o detalhe em `grosso` a peça vai de `822`
            // para `399` vértices em dez toques; em `medio`, ela lê
            // `396 -> 396` — **zero**, e o artista vê a mesma coisa que veria
            // com o pincel partido.
            self.queixa_do_passe(
                verbo,
                "nao ha' aresta fora da faixa aqui -- baixe o detalhe com U \
                 (ou aumente o pincel com ]) e passe de novo",
            );
            return false;
        }
        // A malha tem faces novas: o upload incremental não as descreve.
        self.mesh_rebuilt();
        true
    }
}

/// **Quem muda a topologia em dyntopo** — ver [`tests`].
#[cfg(test)]
#[path = "dyntopo_tests.rs"]
mod tests;
