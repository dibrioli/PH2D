//! ⭐⭐⭐ **O IMPACTO está FIADO no quadro** (plano 28, W5): a pausa no golpe é pedida DEPOIS do
//! passo da física, consumida ANTES do acumulador do passo fixo, e os números são pintados pela
//! banda da cena.
//!
//! # ⚠️ Porque é um gate de TEXTO
//!
//! A lei (`ph2d_app_components::impacto`) tem os gates dela — a pausa medida em tiques a `30`, `60`
//! e `144` fps —, e todos entram **abaixo** da shell: as três chamadas moram em fases que pedem a
//! `App` inteira e não são alcançáveis de um teste. *Um motor com a lei certa e a shell a não o
//! ligar lê-se como um motor sem a lei.* ⇒ a régua lê as três fases, com `include_str!` para que
//! mudar um ficheiro de sítio NÃO compile.

const RELOGIOS: &str = include_str!("../../src/render_loop/fase_fixed_step_clocks.rs");
const FISICA: &str = include_str!("../../src/render_loop/fase_physics_step.rs");
const SOBREPOSICAO: &str = include_str!("../../src/render_loop/fase_physics_overlay.rs");

/// ⭐⭐⭐ **A pausa é tempo de parede retido ANTES do acumulador** — é isso que congela o jogo
/// inteiro (física, relógios, vida) e que faz a pausa comer o mesmo tempo a qualquer cadência.
/// ⚠️ Depois do `advance` ela não reteria nada: os tiques do quadro já teriam corrido.
///
/// **Mutações que devem sangrar:** apagar o `retem` · movê-lo para depois do `advance` · dar ao
/// `advance` o `wall_dt` de antes da retenção.
#[test]
fn a_pausa_e_retida_antes_do_acumulador() {
    let retem = RELOGIOS
        .find("health_bars.impacto.retem(wall_dt, a_correr)")
        .expect("o relógio já não retém a pausa no golpe");
    let avanca = RELOGIOS
        .find("self.fixed_step.advance(wall_dt)")
        .expect("o acumulador mudou de forma");
    assert!(
        retem < avanca,
        "a pausa é retida DEPOIS do acumulador — os tiques do quadro já correram"
    );
    assert!(
        RELOGIOS[retem - 40..retem].contains("let wall_dt ="),
        "a retenção já não substitui o `wall_dt` que o acumulador recebe"
    );
    assert!(
        RELOGIOS.contains("let a_correr = self.playhead.is_playing();"),
        "a pausa deixou de perguntar ao relógio se o jogo está a correr"
    );
}

/// ⭐⭐ **Os golpes são OUVIDOS depois do passo que os produz** — antes dele o canal ainda tem os
/// do quadro anterior (ou nenhuns).
///
/// **Mutação que deve sangrar:** apagar o `ouve` · pô-lo antes do `dispatch`.
#[test]
fn os_golpes_sao_ouvidos_depois_do_passo() {
    let passo = FISICA
        .find("ph2d_app_physics::bridge::dispatch::dispatch(")
        .expect("o passo da física mudou de forma");
    let ouve = FISICA
        .find("health_bars.impacto.ouve(sim, physics.health_events())")
        .expect("ninguém ouve os golpes — a pausa e os números nunca nascem");
    assert!(
        ouve > passo,
        "os golpes são ouvidos ANTES do passo que os produz"
    );
}

/// ⭐⭐ **Os números são PINTADOS pela janela da CENA** — a banda, nunca a janela inteira (a lei
/// do mapeamento mundo↔tela que esta casa já pagou cinco vezes).
///
/// **Mutação que deve sangrar:** apagar o `pinta` · dar-lhe outra janela.
#[test]
fn os_numeros_sao_pintados_pela_banda_da_cena() {
    let pinta = SOBREPOSICAO
        .find(".pinta(camera, janela_da_cena, vector_scene, paint_ctx.text)")
        .expect("os números de dano não são pintados");
    assert!(
        SOBREPOSICAO[..pinta].contains("health_bars"),
        "a pintura não lê o estado do impacto"
    );
}
