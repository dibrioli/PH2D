//! **`ph2d-app-registry-init` — o ponto de extensão *append-only* por onde uma família entra na
//! shell** (W2, precedente: `ph2d-panel-registry-init`).
//!
//! # O problema que ele resolve, e que é de CALENDÁRIO antes de ser técnico
//!
//! Seis linhas saem da shell ao mesmo tempo. As famílias são disjuntas em ficheiros — é por isso
//! que a obra paraleliza — mas todas aterram nas **mesmas** costuras partilhadas: a lista de `mod`
//! do `main.rs`, os campos de `App`, e o `Cargo.toml` da shell. Seis diffs no mesmo `[dependencies]`
//! é colisão textual garantida, e é o caso em que a DIRETRIZ §1.5.5 manda a linha **parar**.
//!
//! ⇒ A dependência de cada família entra **aqui**, num bloco **gerado de uma varredura** de
//! `crates/ph2d-app-*`. A shell depende de *uma* crate — esta — e nunca de seis. Uma linha nova
//! não edita ficheiro central nenhum: ela cria `crates/ph2d-app-<fam>/`, corre
//! `cargo run -p ph2d-app-sync`, e o bloco regenera-se em ordem determinística.
//!
//! # ⚠️ O que este registo NÃO faz (e a razão é medida)
//!
//! ⛔ **Ele não abstrai o laço de quadro.** O `render_loop` chama **48 símbolos** do piloto, em
//! ordem e heterogéneos — desenhar, drenar nove pedidos, sincronizar o ECS, exportar, importar,
//! pintar quatro gizmos. Transformar isso numa lista de ganchos genéricos é redesenhar o laço a
//! partir de **uma** família, que é exactamente o que o briefing proíbe (*«medir antes de
//! generalizar»*). A shell continua a chamar `ph2d_app_field3d::…` pelo nome — e pode, porque a
//! seta aponta na direcção certa: **a shell depende da família, a família nunca depende da shell.**
//!
//! ⛔ **Ele não arma a família.** O piloto pergunta-se a si próprio se está armado
//! (`armed_scene()` lê a env ou o pill) e todo gancho dele é **inerte** quando a resposta é não.
//! Um registo que «ligasse» famílias seria um segundo dono dessa decisão.
//!
//! # O que ele carrega: a DECLARAÇÃO de cada família
//!
//! O que uma família declara é o que atravessa a fronteira dela e pode **colidir com outra**: os
//! roteadores de smoke (`PH2D_*_SMOKE`). Hoje nada no repo pergunta se duas famílias reclamam a
//! mesma variável de ambiente — e com seis a sair da shell ao mesmo tempo, essa é a colisão que
//! passaria **muda** (a segunda família a ler a mesma env simplesmente ganharia ou perderia,
//! conforme a ordem). O gate `no_two_families_claim_the_same_router` fecha-a.

#![forbid(unsafe_code)]

pub use ph2d_app_host::{AppFamily, AppFamilyRegistry, SmokeRouter};

/// **Monta o registo das famílias activas.**
///
/// O bloco é regenerado por `cargo run -p ph2d-app-sync` de uma varredura de `crates/ph2d-app-*`.
/// ⛔ Não edite entre os marcadores à mão — o gate de staleness apanha a deriva.
#[must_use]
pub fn register_all_app_families() -> AppFamilyRegistry {
    #[allow(unused_mut)]
    let mut reg = AppFamilyRegistry::new_empty();
    // <ph2d-app-sync:begin>
    #[cfg(feature = "app-field3d")]
    reg.push(ph2d_app_field3d::FAMILY);
// <ph2d-app-sync:end>
    reg
}

#[cfg(test)]
mod tests {
    use super::*;

    /// ⛔⛔ **DUAS famílias não podem reclamar a mesma `PH2D_*_SMOKE`** — e até hoje nada no repo
    /// perguntava isto.
    ///
    /// ⚠️ **A colisão passaria MUDA.** Duas famílias a ler a mesma variável não dão erro nenhum:
    /// as duas armam, as duas desenham, e o que o dono vê depende da ordem em que o laço as chama.
    /// É a mesma forma do *«número que soma entre linhas se CONTA, nunca se escolhe»* (CLAUDE.md
    /// §5.0), um nível acima: aqui o que colide é um NOME, e o git não sabe o que ele significa.
    #[test]
    fn no_two_families_claim_the_same_router() {
        let reg = register_all_app_families();
        let mut seen: Vec<(&str, &str)> = Vec::new();
        for (key, r) in reg.routers() {
            if let Some((dono, _)) = seen.iter().find(|(_, env)| *env == r.env) {
                panic!(
                    "a família `{key}` reclama `{}`, que já é da `{dono}` — duas famílias na mesma \
                     variável de ambiente armam as duas, sem erro nenhum, e quem ganha é a ordem \
                     do laço",
                    r.env
                );
            }
            seen.push((key, r.env));
        }
    }

    /// ⚠️ **Uma família registada tem de declarar pelo menos um roteador, e todo roteador tem de ter
    /// nível.** Sem esta metade, uma família que se registasse com `routers: &[]` passaria no gate
    /// acima **por vacuidade** — a armadilha do censo que mede zero e se lê como aprovado.
    #[test]
    fn every_registered_family_declares_a_reachable_router() {
        let reg = register_all_app_families();
        for f in reg.families() {
            assert!(
                !f.routers.is_empty(),
                "a família `{}` regista-se e não declara roteador nenhum — ela é inalcançável pelo \
                 smoke do dono, e o gate de colisão passa sobre ela por vacuidade",
                f.key
            );
            for r in f.routers {
                assert!(
                    r.env.starts_with("PH2D_") && r.env.ends_with("_SMOKE"),
                    "o roteador `{}` da família `{}` não segue a forma `PH2D_*_SMOKE`",
                    r.env,
                    f.key
                );
                assert!(
                    r.max_level >= 1,
                    "o roteador `{}` da família `{}` diz responder por `0` cenas",
                    r.env,
                    f.key
                );
            }
        }
    }

    /// ⛔ **E o registo não pode estar VAZIO nesta build.**
    ///
    /// ⚠️ É a lição que o `ph2d-panel-registry-init` pagou por escrito: lá o espelho da contagem é
    /// **escrito à mão**, e o painel de modelagem 3D esteve *«REGISTADO e NÃO CONTADO»* de 19/08
    /// até ao fecho da linha. Aqui não há espelho para derivar — o que se afirma é que o `default`
    /// **de facto liga alguém**, que é o que um `#[cfg]` mal escrito apaga em silêncio.
    #[test]
    fn the_default_build_registers_at_least_one_family() {
        assert!(
            !register_all_app_families().families().is_empty(),
            "nenhuma família no registo: ou o bloco gerado está vazio, ou o `default` do \
             Cargo.toml deixou de ligar as features `app-*`"
        );
    }
}
