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

use ph2d_mesh::{Refine, edge_target, refine_in_sphere};

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
    pub(super) fn refine_for_dab(&mut self, centre: [f32; 3], radius: f32) -> bool {
        if !self.dyntopo.armed {
            return false;
        }
        // ⚠️ **Recusa com a pilha montada** (ver o cabeçalho). Silenciosa aqui
        // de propósito: a mensagem sai no ARM, uma vez, em vez de por dab.
        if self.level_count() > 1 {
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
        let mesh = self.objects[self.active].stack.mesh_mut();
        let mut region = std::mem::take(&mut self.dyn_region);
        let out = refine_in_sphere(mesh, centre, radius, target, &mut births, &mut region);
        self.dyn_region = region;
        let done = matches!(out, Refine::Done { .. });
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
        if !done {
            return false;
        }
        // A malha tem faces novas: o upload incremental não as descreve.
        self.mesh_rebuilt();
        true
    }
}
