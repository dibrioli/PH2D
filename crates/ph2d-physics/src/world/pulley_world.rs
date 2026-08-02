//! **O QUE O MUNDO SABE DIZER SOBRE UMA POLIA** — instalar a tabela, perguntar o
//! comprimento, a velocidade, o recolhido (ADR-0131, W-Pulley).
//!
//! Módulo FILHO de [`super`] pelo teto de LOC, e o corte é por responsabilidade:
//! lá mora a **dinâmica** (o impulso, a massa efetiva, o limitador, a ruptura);
//! aqui, a superfície que o [`super::super::PhysicsWorld`] expõe sobre ela — a
//! porta de instalação que a ponte usa todo dispatch e as consultas de que o
//! desenho, o readout e as sondas vivem.
//!
//! Ele é `impl super::super::PhysicsWorld` e não um tipo próprio: quem pergunta
//! *"qual é o vão desta corda?"* tem o mundo na mão, e um segundo objeto entre os
//! dois seria uma indireção sem fato novo.

use super::rope_route::{self, RopeWheel};
use super::{PulleyDesc, end};
use rapier2d::na::Vector2;

impl crate::PhysicsWorld {
    /// **Install the pulley table**, replacing whatever was there.
    ///
    /// Wholesale rather than per-pulley because a pulley is not owned by a body:
    /// the bridge re-derives the whole set from the authored components every
    /// dispatch (the same shape the joint reconcile has), so an incremental API
    /// would need a removal door whose only caller would be a diff nobody keeps.
    pub fn set_pulleys(&mut self, pulleys: Vec<PulleyDesc>, wheels: Vec<RopeWheel>) {
        self.pulleys = pulleys;
        self.pulley_wheels = wheels;
    }

    /// Sweep door for the table on [`PULLEY_BIAS`] — see the field's own note.
    pub fn set_pulley_bias(&mut self, bias: f32) {
        self.pulley_bias = bias;
    }

    /// A porta de varredura de [`PULLEY_CORRECTION_LAG`], irmã da de cima e pelo
    /// mesmo motivo: a tabela do teto é medida contra o PRODUTO.
    pub fn set_pulley_correction_lag(&mut self, lag: f32) {
        self.pulley_lag = lag;
    }

    /// **Trocar a tabela pela do chamador**, devolvendo-lhe a anterior.
    ///
    /// É por aqui que a ponte instala as polias todo dispatch: ela reconstrói a
    /// lista num scratch próprio e troca, então o caso comum não aloca nada — o
    /// que o gate de zero-alloc do caminho quente exige. `set_pulleys` fica para
    /// fixtures, onde a alocação não importa e a leitura é mais direta.
    pub fn swap_pulleys(&mut self, other: &mut Vec<PulleyDesc>, wheels: &mut Vec<RopeWheel>) {
        std::mem::swap(&mut self.pulleys, other);
        std::mem::swap(&mut self.pulley_wheels, wheels);
    }

    /// **Pôr os eixos MONTADOS onde os corpos deles estão AGORA** (W-Pulley W3).
    ///
    /// A [`mount::refresh_mounts`] tem DOIS chamadores e eles pedem coisas
    /// diferentes, então nenhum dos dois é redundante:
    ///
    /// - o `step` a roda por SUB-PASSO, para o SOLVER: geometria de um sub-passo
    ///   atrás puxa numa direção que já não é a da corda;
    /// - a PONTE a roda uma vez no fim de todo dispatch, para o DESENHO — e essa
    ///   é a metade que faltava. A arena é reinstalada a cada dispatch com o
    ///   centro derivado da pose de REPOUSO (é o que a colheita do ECS sabe), e
    ///   um quadro mais rápido que o tique não dá passo nenhum ⇒ ele desenhava a
    ///   roldana **onde ela foi autorada**. Medido num bloco que viaja: salto de
    ///   **1,27 m** entre um quadro e o seguinte, com a simulação correta o tempo
    ///   todo — o tremor que o smoke da talha reportou.
    ///
    /// No-op para toda roldana pregada no cenário, que é o que toda roldana era
    /// antes do W3.
    pub fn refresh_mounted_wheels(&mut self) {
        super::refresh_mounts(&self.bodies, &mut self.pulley_wheels);
    }

    /// The live pulleys — what the overlay draws the rope from.
    #[must_use]
    pub fn pulleys(&self) -> &[PulleyDesc] {
        &self.pulleys
    }

    /// A arena de roldanas que as faixas de [`PulleyDesc`] indexam.
    #[must_use]
    pub fn pulley_wheels(&self) -> &[RopeWheel] {
        &self.pulley_wheels
    }

    /// **Quanto de corda os tambores desta corda já recolheram**, em metros —
    /// zero para uma corda sem motor, que é o estado de toda corda que ninguém
    /// dirigiu.
    ///
    /// É o número que o readout mostra e o que os gates comparam contra `ω·r·t`;
    /// o comprimento que a restrição de fato segura é
    /// `total_length − pulley_reeled`.
    ///
    /// ⚠️ **O mapa NÃO é podado quando uma corda sai da tabela**, e isso é
    /// deliberado: uma corda que pisca fora por um dispatch (um `active`
    /// desmarcado, um rename a caminho) mantém o guincho onde ele estava, em vez
    /// de o rebobinar em silêncio. O preço, nomeado: apagar uma corda e criar
    /// outra **com o mesmo nome** no MESMO run herda o recolhido — a mesma
    /// exposição de toda ligação por nome deste editor, e o Reset a cura, porque
    /// ele constrói um mundo novo.
    #[must_use]
    pub fn pulley_reeled(&self, desc: &PulleyDesc) -> f32 {
        self.pulley_payout.get(&desc.id).copied().unwrap_or(0.0)
    }

    /// A massa efetiva de cada ponta — diagnóstico para as tabelas de
    /// `measure_pulley.rs`, que precisam distinguir *quanto* cada lado absorve
    /// do *onde* ele está.
    #[must_use]
    pub fn pulley_branch_k(&self, desc: &PulleyDesc) -> Option<(f32, f32)> {
        let a = end(&self.bodies, desc.body_a, desc.local_a)?;
        let b = end(&self.bodies, desc.body_b, desc.local_b)?;
        let mut scratch = Vec::new();
        let r = rope_route::route(
            [a.point.x, a.point.y],
            [b.point.x, b.point.y],
            desc.wheels(&self.pulley_wheels),
            &mut scratch,
        )?;
        Some((
            a.k(Vector2::new(r.dir_a[0], r.dir_a[1])),
            b.k(Vector2::new(r.dir_b[0], r.dir_b[1])),
        ))
    }

    /// **A que velocidade a corda CORRE**, em m/s, positiva no sentido A → B.
    ///
    /// É o que faz uma roldana girar (`ω = s·lado/r`), e é UM número por corda e
    /// não um por roda: a corda é inextensível, então ela passa por todas as
    /// roldanas na mesma taxa. Derivar por roda seria N respostas para um fato.
    ///
    /// Sai do ramo A: se ele se ALONGA, a corda está sendo puxada de B para A,
    /// logo ela corre no sentido B → A — daí o sinal trocado.
    #[must_use]
    pub fn pulley_rope_speed(&self, desc: &PulleyDesc) -> Option<f32> {
        let a = end(&self.bodies, desc.body_a, desc.local_a)?;
        let b = end(&self.bodies, desc.body_b, desc.local_b)?;
        let mut scratch = Vec::new();
        let r = rope_route::route(
            [a.point.x, a.point.y],
            [b.point.x, b.point.y],
            desc.wheels(&self.pulley_wheels),
            &mut scratch,
        )?;
        Some(-a.rate(Vector2::new(r.dir_a[0], r.dir_a[1])))
    }

    /// O comprimento de rota de uma polia **como ela está agora**, para o
    /// chamador semear `total_length` da pose de repouso em vez de pedir ao
    /// artista que meça uma corda com régua.
    ///
    /// `None` quando a rota é degenerada — a mesma recusa que o `apply` faz,
    /// perguntada pela mesma porta, para que um semeio nunca nomeie um
    /// comprimento que o passe depois se recuse a segurar.
    #[must_use]
    pub fn pulley_span(&self, desc: &PulleyDesc) -> Option<f32> {
        let a = end(&self.bodies, desc.body_a, desc.local_a)?;
        let b = end(&self.bodies, desc.body_b, desc.local_b)?;
        let mut scratch = Vec::new();
        Some(
            rope_route::route(
                [a.point.x, a.point.y],
                [b.point.x, b.point.y],
                desc.wheels(&self.pulley_wheels),
                &mut scratch,
            )?
            .length,
        )
    }
}
