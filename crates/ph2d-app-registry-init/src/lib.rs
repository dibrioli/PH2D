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
//! ⇒ **O que este registo É, medido** (auditoria de arquitectura 2026-09-12, A7): um **catálogo de
//! declarações** — a `const FAMILY` de cada família num bloco **gerado de uma varredura** de
//! `crates/ph2d-app-*` (`cargo run -p ph2d-app-sync`). Uma linha nova não edita ficheiro central
//! nenhum aqui: cria `crates/ph2d-app-<fam>/`, corre o sync, e o bloco regenera-se em ordem
//! determinística.
//!
//! ⛔ **Ele NÃO é o funil de dependências da shell, e não pode ser.** Este cabeçalho prometia *«a shell
//! depende de UMA crate — esta — e nunca de seis»*; a promessa era impossível por desenho, porque o
//! `render_loop` chama os corpos de cada família PELO NOME e em ordem (HOWTO §4), e o `cargo machete`
//! acusou a dependência da shell neste registo como morta. A promessa saiu.
//!
//! ⭐ **A invariante que de facto importa tem gate:** a shell liga EXACTAMENTE as famílias que este
//! registo regista (`tests/it/the_shell_links_exactly_the_registered_families.rs`). Uma família
//! ligada e não registada escapa ao gate de colisão de roteadores abaixo; uma registada e não ligada
//! é uma declaração morta.
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
    #[cfg(feature = "app-audio")]
    reg.push(ph2d_app_audio::FAMILY);
    #[cfg(feature = "app-components")]
    reg.push(ph2d_app_components::FAMILY);
    #[cfg(feature = "app-field3d")]
    reg.push(ph2d_app_field3d::FAMILY);
    #[cfg(feature = "app-flip")]
    reg.push(ph2d_app_flip::FAMILY);
    #[cfg(feature = "app-motion")]
    reg.push(ph2d_app_motion::FAMILY);
    #[cfg(feature = "app-painter")]
    reg.push(ph2d_app_painter::FAMILY);
    #[cfg(feature = "app-physics")]
    reg.push(ph2d_app_physics::FAMILY);
    #[cfg(feature = "app-sculpt3d")]
    reg.push(ph2d_app_sculpt3d::FAMILY);
    #[cfg(feature = "app-skeleton")]
    reg.push(ph2d_app_skeleton::FAMILY);
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
    /// metade `if` do gate abaixo desaparecem com ela»*. Desapareceram.
    ///
    /// ⚠️⚠️ **E no MESMO dia entraram SETE famílias, não seis** — a `line/app-vec` criou a
    /// `skeleton` na mesma rodada. Seis declaram roteador; a sétima não declara nenhum e **isso está
    /// certo**, por um motivo que a catraca morta não sabia exprimir: a cena dela vive numa crate
    /// IRMÃ. Ver [`FAMILIAS_CUJA_CENA_VIVE_NUMA_CRATE_IRMA`]. *A lista que morreu e a que nasceu
    /// leem-se igual e querem dizer o oposto — foi o integrador que as separou, porque nenhum dos
    /// dois lados do merge tinha a resposta inteira.*
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

    /// ⭐⭐⭐ **A OUTRA ausência legítima: a família cuja CENA vive numa crate IRMÃ.**
    ///
    /// ⛔⛔ **Ela não é a catraca das famílias com outro nome, e pô-la lá seria MENTIRA
    /// PERMANENTE.** A `FAMILIAS_COM_O_ROTEADOR_AINDA_NA_SHELL` — que morreu em 12/09, no mesmo dia
    /// em que esta nasceu — descrevia uma extracção **a meio** — a crate saiu, o
    /// `match` de cenas ficou na shell — e o censo de obsolescência dela dispara no dia em que a
    /// família passa a declarar roteador. Uma família que nunca vai declarar nenhum ficaria lá para
    /// sempre, a descrever uma shell onde o roteador não está: *uma entrada que nada pode apagar é
    /// exactamente a catraca que vira LICENÇA* (CLAUDE.md §5.0).
    ///
    /// ⭐ **O caso real, e é o primeiro:** a `skeleton` (W2 Fase C) é família própria desde o
    /// ADR-0169, mas a única cena que a exercita é a `PH2D_VEC_BONE_SMOKE` — e essa cena monta um
    /// **braço vectorial** e prende-o, logo ela é da `vec` tanto quanto é do esqueleto. *Quem possui
    /// a cena é quem a constrói.*
    ///
    /// ⚠️ **E declará-la nas DUAS famílias seria o defeito oposto**: o
    /// [`no_two_families_claim_the_same_router`] existe precisamente para impedir que duas famílias
    /// respondam pela mesma variável de ambiente.
    ///
    /// ⭐⭐ **A entrada é VERIFICADA, não tolerada** — é isso que a separa de uma folga. Cada linha
    /// diz *quem* não tem roteador **e** *quem o tem em seu lugar*, e o gate confirma que essa
    /// segunda família existe e declara mesmo aquela env. ⇒ apagar o roteador do lado de lá reprova
    /// aqui, e dar um roteador próprio ao esqueleto também.
    const FAMILIAS_CUJA_CENA_VIVE_NUMA_CRATE_IRMA: &[(&str, &str)] =
        &[("skeleton", "PH2D_VEC_BONE_SMOKE")];

    /// ⚠️ **Uma família registada tem de declarar pelo menos um roteador, e todo roteador tem de ter
    /// nível.** Sem esta metade, uma família que se registasse com `routers: &[]` passaria no gate
    /// acima **por vacuidade** — a armadilha do censo que mede zero e se lê como aprovado.
    ///
    /// ⭐ **Desde 12/09 a catraca `FAMILIAS_COM_O_ROTEADOR_AINDA_NA_SHELL` morreu** — a `motion`
    /// era a última e a lista esvaziou-se. ⚠️ **Mas «morreu a catraca» não é «não há excepção»**, e
    /// a redacção anterior dizia isso: sobram **duas** listas, e nenhuma é uma folga.
    /// 1. [`ROTEADORES_FORA_DA_FORMA`] — sobre o **NOME** de uma env, não sobre a ausência de um
    ///    roteador;
    /// 2. [`FAMILIAS_CUJA_CENA_VIVE_NUMA_CRATE_IRMA`] — a única ausência que esta metade aceita, e
    ///    ela é **VERIFICADA**: a entrada nomeia quem declara o roteador em lugar da família, e o
    ///    gate confirma que essa outra família existe e declara mesmo aquela env.
    ///
    /// As duas são gateadas nos **dois** sentidos, como a catraca morta era.
    #[test]
    fn every_registered_family_declares_a_reachable_router() {
        let reg = register_all_app_families();
        for f in reg.families() {
            // ⚠️⚠️ **Esta variável foi escrita à parte do `a_meio` DE PROPÓSITO pela `line/app-vec`,
            //    e a previsão dela cumpriu-se no MESMO dia.** O comentário original dizia: *«a
            //    `line/app-motion` vai apagar a lista de cima INTEIRA quando a `motion` sair, e uma
            //    condição que partilhasse a variável iria embora com ela — deixando a `skeleton` a
            //    reprovar por uma razão que não é a dela.»* Foi exactamente isso: as duas linhas
            //    correram em paralelo, a `motion` apagou a `FAMILIAS_COM_O_ROTEADOR_AINDA_NA_SHELL`
            //    (ela esvaziou-se ao a `motion` sair) e a `vec` criou esta.
            //    ⭐⭐ **Duas listas com o mesmo ASPECTO e significados OPOSTOS**, e o merge não ficou
            //    com nenhum dos dois lados: ficou com a segunda, porque a primeira deixou de
            //    descrever alguém. *É a lei do «número que soma entre linhas se CONTA, nunca se
            //    escolhe» (CLAUDE.md §5.0) um nível acima — aqui o que colide é uma LISTA, e o git
            //    não sabe o que ela significa.* ⇒ resolvido pelo integrador, 2026-09-12.
            let irma = FAMILIAS_CUJA_CENA_VIVE_NUMA_CRATE_IRMA
                .iter()
                .find(|(k, _)| *k == f.key);
            assert!(
                !f.routers.is_empty() || irma.is_some(),
                "a família `{}` regista-se e não declara roteador nenhum — ela é inalcançável pelo \
                 smoke do dono, e o gate de colisão passa sobre ela por vacuidade. \
                 ⛔ A catraca `FAMILIAS_COM_O_ROTEADOR_AINDA_NA_SHELL` MORREU em 2026-09-12 (a \
                 `motion` era a última): não há mais dívida de extracção-a-meio a que se juntar — \
                 leve o roteador para a crate da família. A única ausência que este gate ainda \
                 aceita é a da família cuja CENA vive numa crate IRMÃ \
                 (`FAMILIAS_CUJA_CENA_VIVE_NUMA_CRATE_IRMA`), e ela é VERIFICADA e não tolerada: a \
                 entrada nomeia quem declara o roteador em seu lugar",
                f.key
            );
            // ⭐ **O censo de obsolescência da lista que SOBRA, e ele é uma VERIFICAÇÃO:** a família
            //    nomeada como dona tem de existir e tem de declarar mesmo aquela env.
            if let Some((_, env)) = irma {
                assert!(
                    f.routers.is_empty(),
                    "a família `{}` está em `FAMILIAS_CUJA_CENA_VIVE_NUMA_CRATE_IRMA` e JÁ declara \
                     roteador próprio — a entrada está obsoleta: apague-a (a catraca só encolhe)",
                    f.key
                );
                assert!(
                    reg.families()
                        .iter()
                        .any(|o| o.key != f.key && o.routers.iter().any(|r| r.env == *env)),
                    "a família `{}` diz que a cena dela é a `{env}` de outra família, e NENHUMA a \
                     declara — ou a cena morreu, ou a dona deixou de a registar. Em qualquer dos \
                     casos esta família ficou inalcançável pelo smoke do dono, em silêncio",
                    f.key
                );
            }
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
