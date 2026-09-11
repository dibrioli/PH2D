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
    #[cfg(feature = "app-flip")]
    reg.push(ph2d_app_flip::FAMILY);
    #[cfg(feature = "app-physics")]
    reg.push(ph2d_app_physics::FAMILY);
    #[cfg(feature = "app-vec")]
    reg.push(ph2d_app_vec::FAMILY);
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

    /// **As famílias cuja extracção está a MEIO: a crate existe, os roteadores ainda não saíram.**
    ///
    /// ⚠️ **Isto é uma catraca, e ela só ENCOLHE** — cada nome sai daqui no dia em que a Fase B
    /// daquela família levar o roteador de cenas para a crate. A lista chega a **vazia** e então
    /// este bloco e a metade `if` do gate abaixo desaparecem com ela.
    ///
    /// ⛔⛔ **Por que ela tem de existir, em vez de o gate simplesmente aceitar `routers: &[]`:**
    /// *«ainda não mudou»* e *«alguém esqueceu»* leem-se **exactamente igual** numa lista vazia —
    /// é a mesma forma do `id órfão` contra o `controlo morto` (CLAUDE.md §5), cuja cura é oposta.
    /// Escrever o nome aqui torna a ausência **declarada**: quem lê sabe que é dívida conhecida, e
    /// quem acrescentar uma família nova sem roteador **reprova**, que é o que a L0 desenhou.
    ///
    /// ⚠️ **E a catraca traz o censo de obsolescência** (CLAUDE.md §5.0: *«uma catraca sem censo de
    /// obsolescência não desce: ela vira LICENÇA»*) — a segunda metade do gate reprova um nome que
    /// já não descreve nada, seja porque a família passou a declarar roteador, seja porque ela
    /// deixou de existir.
    ///
    /// ⭐ **Medido em 2026-09-11, na integração das seis linhas da W2:** das cinco famílias da
    /// Fase A, só a `flip` lê as próprias `PH2D_*_SMOKE` dentro da crate (15 delas); `vec`,
    /// `motion`, `physics` e `sculpt3d` extraíram código e **não** o roteador — o `match` de cenas
    /// toca a `App`, que é precisamente o que a Fase B ainda deve.
    const FAMILIAS_COM_O_ROTEADOR_AINDA_NA_SHELL: &[&str] = &["physics", "vec"];

    /// ⚠️ **Uma família registada tem de declarar pelo menos um roteador, e todo roteador tem de ter
    /// nível.** Sem esta metade, uma família que se registasse com `routers: &[]` passaria no gate
    /// acima **por vacuidade** — a armadilha do censo que mede zero e se lê como aprovado.
    ///
    /// A única excepção é **declarada, uma a uma**, na
    /// [`FAMILIAS_COM_O_ROTEADOR_AINDA_NA_SHELL`] — e a excepção é gateada nos **dois** sentidos.
    #[test]
    fn every_registered_family_declares_a_reachable_router() {
        let reg = register_all_app_families();
        for f in reg.families() {
            let a_meio = FAMILIAS_COM_O_ROTEADOR_AINDA_NA_SHELL.contains(&f.key);
            assert!(
                !f.routers.is_empty() || a_meio,
                "a família `{}` regista-se e não declara roteador nenhum — ela é inalcançável pelo \
                 smoke do dono, e o gate de colisão passa sobre ela por vacuidade. Se a extracção \
                 dela está a MEIO (a crate saiu, o roteador de cenas ficou na shell), escreva o \
                 nome em `FAMILIAS_COM_O_ROTEADOR_AINDA_NA_SHELL` — uma ausência declarada é \
                 dívida; uma ausência muda é um defeito",
                f.key
            );
            assert!(
                !(a_meio && !f.routers.is_empty()),
                "a família `{}` está em `FAMILIAS_COM_O_ROTEADOR_AINDA_NA_SHELL` e JÁ declara \
                 roteador — a entrada está obsoleta: apague-a (a catraca só encolhe)",
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
        // ⛔ A OUTRA metade do censo de obsolescência: um nome na catraca que já não corresponde a
        // família nenhuma desta build. Sem ela, uma família apagada (ou renomeada) deixaria a
        // entrada para trás e a lista pararia de encolher sem ninguém ver — que é literalmente a
        // «catraca que vira licença».
        for orfao in FAMILIAS_COM_O_ROTEADOR_AINDA_NA_SHELL {
            assert!(
                reg.families().iter().any(|f| f.key == *orfao),
                "`{orfao}` está em `FAMILIAS_COM_O_ROTEADOR_AINDA_NA_SHELL` e não é família \
                 nenhuma desta build — a entrada está obsoleta, apague-a"
            );
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
