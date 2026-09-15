//! ⭐⭐⭐ **O VOCABULÁRIO DO PROJÉCTIL** (TOP-20 #14) — o instantâneo e a edição, num módulo abaixo
//! do [`crate::action_bus`] e do [`crate::screens`].
//!
//! # ⛔ Por que ele não está no `screens::hero`, com os irmãos
//!
//! Pela mesma razão do [`crate::tags_edits`], do [`crate::factory_edits`] e do
//! [`crate::topdown_edits`], e com o mesmo número atrás: a catraca do DAG tolera a aresta
//! `action_bus → screens` num **tecto** e escreve a cura ao lado dela.
//!
//! *É o quarto degrau da mesma migração, e cada um torna o resto mais barato.*
//!
//! # ⭐⭐ O que o painel DIZ que os campos sozinhos não diriam
//!
//! | aviso | o que se passa | como se cura |
//! |---|---|---|
//! | `no body` | sem `RigidBody` não há o que mover nem em que bater | anexar o corpo (a paleta já o pede) |
//! | `the body must be Kinematic` | um corpo dinâmico é do **solver**, e o projéctil não tem pose para escrever | trocar o *Body Type* |
//! | `only moves while the clock plays` | a ponte corre no passo fixo | carregar no play |
//! | `the flight is over` | o alcance ou os ricochetes acabaram | rebobinar |
//!
//! ⚠️ **O último é o desta wave**, e sem ele um projéctil que fez exactamente o que devia lê-se
//! como partido: ele está parado no ar, com todos os números certos no ecrã.
//!
//! # ⚠️ E o ALVO é um NOME
//!
//! O campo do painel é texto; o que o componente guarda é o `stable_name_id` dele. ⛔ Os bits de
//! uma entidade não sobrevivem a um `Ctrl+Z` — o undo respawna o mundo inteiro com bits novos.

/// Snapshot da secção PROJECTILE MOTION da entidade selecionada.
#[derive(Clone, Debug, PartialEq)]
pub struct InspectorProjectileInfo {
    pub entity_bits: u64,
    pub initial_speed: f32,
    pub acceleration: f32,
    pub max_speed: f32,
    pub gravity: f32,
    pub bounciness: f32,
    pub max_bounces: u32,
    pub range: f32,
    pub face_velocity: bool,
    pub homing_accel: f32,
    /// O NOME do alvo — vazio = ninguém. ⚠️ O componente guarda o `stable_name_id`; o que o painel
    /// mostra é o nome, e quem traduz é a shell, que tem o mundo.
    pub homing_target: String,
    /// ⚠️ O alvo está escrito e **não existe na cena**? — um nome que ninguém tem lê-se como um
    /// homing partido, e o painel tem de o dizer.
    pub homing_target_missing: bool,
    /// O corpo é **cinemático**? `false` ⇒ o aviso `the body must be Kinematic`.
    pub body_is_kinematic: bool,
    /// O objecto tem `RigidBody`?
    pub has_body: bool,
    /// O relógio está a andar?
    pub clock_playing: bool,
    /// ⭐ **O voo já acabou** — ver o cabeçalho.
    pub flight_over: bool,
    pub selected_count: usize,
}

/// Uma edição de um campo da secção PROJECTILE MOTION.
#[derive(Clone, Debug, PartialEq)]
pub enum ProjectileFieldEdit {
    InitialSpeed(f32),
    Acceleration(f32),
    MaxSpeed(f32),
    Gravity(f32),
    Bounciness(f32),
    MaxBounces(u32),
    Range(f32),
    FaceVelocity(bool),
    HomingAccel(f32),
    /// ⚠️ O NOME, cru. Quem o converte em `stable_name_id` é a shell.
    HomingTarget(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    /// ⚠️ **Todo campo do componente tem uma edição** — um campo sem variante é um knob que o
    /// painel mostra e que ninguém pode mexer, e ele lê-se exactamente como um controlo morto.
    ///
    /// ⛔ A lista é escrita à mão de propósito: ela é a **segunda leitura** do componente, e é a
    /// discordância entre as duas que acusa o esquecimento.
    #[test]
    fn todo_campo_do_componente_tem_uma_edicao() {
        let variantes = [
            ProjectileFieldEdit::InitialSpeed(0.0),
            ProjectileFieldEdit::Acceleration(0.0),
            ProjectileFieldEdit::MaxSpeed(0.0),
            ProjectileFieldEdit::Gravity(0.0),
            ProjectileFieldEdit::Bounciness(0.0),
            ProjectileFieldEdit::MaxBounces(0),
            ProjectileFieldEdit::Range(0.0),
            ProjectileFieldEdit::FaceVelocity(false),
            ProjectileFieldEdit::HomingAccel(0.0),
            ProjectileFieldEdit::HomingTarget(String::new()),
        ];
        assert_eq!(
            variantes.len(),
            10,
            "o `ProjectileMotion` tem DEZ campos autoráveis — se um nasceu, ele precisa de uma \
             variante aqui e de uma row no painel"
        );
    }

    /// ⚠️ **O alvo vazio e o alvo ausente são coisas DIFERENTES** — e o painel diz coisas
    /// diferentes sobre eles.
    #[test]
    fn o_alvo_vazio_e_o_alvo_ausente_nao_sao_a_mesma_coisa() {
        let base = InspectorProjectileInfo {
            entity_bits: 1,
            initial_speed: 12.0,
            acceleration: 0.0,
            max_speed: 0.0,
            gravity: 0.0,
            bounciness: 1.0,
            max_bounces: 0,
            range: 0.0,
            face_velocity: true,
            homing_accel: 0.0,
            homing_target: String::new(),
            homing_target_missing: false,
            body_is_kinematic: true,
            has_body: true,
            clock_playing: true,
            flight_over: false,
            selected_count: 1,
        };
        assert!(
            !base.homing_target_missing,
            "sem alvo escrito nao ha' falta"
        );
        let perdido = InspectorProjectileInfo {
            homing_target: "Alguem".into(),
            homing_target_missing: true,
            ..base
        };
        assert!(perdido.homing_target_missing);
    }
}
