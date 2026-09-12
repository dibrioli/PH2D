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
    #[cfg(feature = "app-motion")]
    reg.push(ph2d_app_motion::FAMILY);
    #[cfg(feature = "app-physics")]
    reg.push(ph2d_app_physics::FAMILY);
    #[cfg(feature = "app-sculpt3d")]
    reg.push(ph2d_app_sculpt3d::FAMILY);
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

    /// **A forma de um roteador é `PH2D_*_SMOKE` — e a excepção é NOMEADA, nunca tolerada.**
    ///
    /// ⭐⭐⭐ **A catraca `FAMILIAS_COM_O_ROTEADOR_AINDA_NA_SHELL` MORREU em 2026-09-12**: a
    /// `motion` era a última, e o doc dela dizia *«a lista chega a vazia e então este bloco e a
    /// metade `if` do gate abaixo desaparecem com ela»*. Desapareceram. **As seis famílias
    /// declaram roteador.**
    ///
    /// ⚠️⚠️ **Mas o corte da `motion` trouxe uma pergunta que a forma não previa:** o roteador
    /// PRINCIPAL dela — as `114` cenas que o dono smoka — chama-se **`PH2D_GPU_COOK_DEMO`**, e
    /// acaba em `_DEMO`. Ele é anterior a esta convenção e o nome dele é o **endereço público**
    /// que o dono escreve (`CLAUDE.md §5` e dezenas de docs dizem `PH2D_GPU_COOK_DEMO=<n>`).
    ///
    /// ⛔ **As três saídas, e porque esta:**
    /// 1. *não o declarar* — a família maior do repo registaria `37` roteadores e esconderia o
    ///    que de facto abre as cenas. O registo passaria a MENTIR, que é o oposto do que a
    ///    catraca acima existia para impedir;
    /// 2. *renomear a env* — parte todo passo de smoke já escrito. O nome é a superfície;
    /// 3. **declarar a excepção, com censo** — é o mesmo molde da catraca que morreu:
    ///    *uma excepção declarada é dívida; uma excepção muda é um defeito.*
    ///
    /// ⚠️ ⛔ **Ela NÃO é `PH2D_GPU_COOK`** (sem `_DEMO`), que é diagnóstico (`=0` volta à CPU) e
    /// **não** entra em `FAMILY` nenhuma. Duas envs, um prefixo, papéis opostos.
    const ROTEADORES_FORA_DA_FORMA: &[&str] = &["PH2D_GPU_COOK_DEMO"];

    /// ⚠️ **Uma família registada tem de declarar pelo menos um roteador, e todo roteador tem de ter
    /// nível.** Sem esta metade, uma família que se registasse com `routers: &[]` passaria no gate
    /// acima **por vacuidade** — a armadilha do censo que mede zero e se lê como aprovado.
    ///
    /// ⭐ **Desde 12/09 não há excepção nenhuma a esta metade** — a catraca
    /// `FAMILIAS_COM_O_ROTEADOR_AINDA_NA_SHELL` esvaziou-se e morreu. A única lista de tolerância
    /// que sobra é a [`ROTEADORES_FORA_DA_FORMA`], que é sobre o NOME de uma env, não sobre a
    /// ausência de um roteador — e ela é gateada nos **dois** sentidos, como a outra era.
    #[test]
    fn every_registered_family_declares_a_reachable_router() {
        let reg = register_all_app_families();
        for f in reg.families() {
            assert!(
                !f.routers.is_empty(),
                "a família `{}` regista-se e não declara roteador nenhum — ela é inalcançável \
                 pelo smoke do dono, e o gate de colisão passa sobre ela por vacuidade. \
                 ⛔ A catraca `FAMILIAS_COM_O_ROTEADOR_AINDA_NA_SHELL` MORREU em 2026-09-12 \
                 (a `motion` era a última): não há mais dívida declarada a que se juntar — \
                 leve o roteador para a crate da família",
                f.key
            );
            for r in f.routers {
                assert!(
                    r.env.starts_with("PH2D_")
                        && (r.env.ends_with("_SMOKE") || ROTEADORES_FORA_DA_FORMA.contains(&r.env)),
                    "o roteador `{}` da família `{}` não segue a forma `PH2D_*_SMOKE`. Se ele é \
                     anterior à convenção e o nome já é endereço público, escreva-o em \
                     `ROTEADORES_FORA_DA_FORMA` — com a razão",
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
        // ⛔⛔ **O CENSO DE OBSOLESCÊNCIA muda de sujeito, mas não morre com a catraca.** Ele
        // media nomes de FAMÍLIA; agora mede nomes de ENV. A lei é a mesma e é a razão de existir
        // das duas listas (`CLAUDE.md` §5.0): *uma tolerância sem censo não desce — vira LICENÇA.*
        // Uma env renomeada, ou uma que passe a acabar em `_SMOKE`, deixa a entrada para trás e
        // ninguém vê.
        let declarados: Vec<&str> = reg
            .families()
            .iter()
            .flat_map(|f| f.routers.iter().map(|r| r.env))
            .collect();
        for orfao in ROTEADORES_FORA_DA_FORMA {
            assert!(
                declarados.contains(orfao),
                "`{orfao}` está em `ROTEADORES_FORA_DA_FORMA` e não é roteador de família \
                 nenhuma desta build — a entrada está obsoleta, apague-a"
            );
            assert!(
                !orfao.ends_with("_SMOKE"),
                "`{orfao}` está em `ROTEADORES_FORA_DA_FORMA` e JÁ segue a forma `PH2D_*_SMOKE` \
                 — a excepção deixou de descrever alguma coisa: apague-a (a lista só encolhe)"
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
