//! **O CENSO DOS PARAMS DE FORMA** — a espécie inteira que a caça aos knobs mortos não
//! podia ver.
//!
//! ⚠️ **A sonda do [doc 90](../../../../docs/Motion%20Nodes/90_caca_aos_knobs_mortos.md)
//! varre `660 params declarados`, e ela lê o `MANIFEST`.** Uma curva, uma rampa, uma
//! paleta e uma expressão **não são `ParamSpec`** — são *text params*, um canal ao lado,
//! por decisão registada (*«uma curva não é um número»*). Logo nenhum deles foi alguma vez
//! perguntado se alguém o lê. São **dezassete**, em dezasseis nós.
//!
//! ⛔ **E o defeito que isto deixou passar chegou por foto (Enio, 2026-08-24):** o
//! `motion.oscillator` oferece o editor **Custom Wave** em toda onda, e a `waveform` só o
//! lê no braço `Custom`. O artista abre o nó, desenha uma forma, e não acontece nada —
//! *«Wave curve dos osciladores não está funcionando»*. O motor estava certo o tempo todo
//! (a cozedura com `wave = Custom` segue a curva ao valor); o que estava errado é que o
//! controle era oferecido num modo que não o lê. É a doença que a própria tabela deste nó
//! já curava **quatro vezes** para `frequency`/`bpm`/`amplitude`/`min`/`max`.
//!
//! ## A pergunta que este censo faz
//!
//! Para todo param de FORMA (widget `Curve`/`Gradient`/`Palette`/`Text`): ou ele é lido em
//! **todo** modo do nó, ou ele tem de estar **gateado** ao modo que o lê. Não há terceira
//! resposta — a que não existe é *«ele é oferecido sempre e lido às vezes»*.
//!
//! ⚠️ **A tabela [`ALWAYS_READ`] é a metade que uma máquina não deriva:** *este nó lê esta
//! forma em todo modo?* é uma pergunta sobre o `eval` dele. O que o censo garante é que a
//! resposta **existe e está escrita** — um param de forma NOVO sem decisão reprova, em vez
//! de nascer mudo como o do oscilador nasceu.

use ph2d_node_registry::{NodeRegistry, ParamWidget};

fn registry() -> NodeRegistry {
    let mut reg = NodeRegistry::new();
    ph2d_node_registry_init::register_all_nodes(&mut reg).expect("os nos registram");
    reg
}

/// **Os params de forma que o nó lê em TODO modo** — `(nó, param, porquê)`.
///
/// ⚠️ O terceiro campo não é decoração: ele é a diferença entre *«conferi o `eval` e ele lê
/// sempre»* e *«ninguém olhou»*, e é a única coisa que a próxima pessoa consegue conferir.
const ALWAYS_READ: &[(&str, &str, &str)] = &[
    (
        "value.curve",
        "curve",
        "o nó É a curva -- não há modo em que ele não a leia",
    ),
    (
        "motion.color_ramp",
        "ramp",
        "a rampa é a saída do nó; não há 'preset vs custom', os presets escrevem esta string",
    ),
    (
        "motion.color_array",
        "palette",
        "a paleta é a fonte das cores em todos os modos",
    ),
    (
        "motion.strobe",
        "curve",
        "a curva molda o brilho em todo flash -- lida sem condição no `eval`",
    ),
    (
        "fx.glow",
        "ramp",
        "a rampa de cor do halo é lida em toda `operation` e todo `source`",
    ),
    (
        "value.pattern",
        "table",
        "a tabela É o padrão que o nó publica",
    ),
    (
        "motion.expression",
        "expr",
        "a expressão É o que o nó avalia",
    ),
    (
        "motion.drive",
        "column",
        "o nome da coluna a conduzir, lido em todo canal e todo modo",
    ),
    ("source.text", "text", "o texto É o que a fonte desenha"),
    (
        "source.text",
        "font",
        "a face escolhida vale para todo o texto",
    ),
    (
        "audio.bands",
        "file",
        "o ficheiro de áudio É a fonte que o nó analisa, em toda configuração de banda",
    ),
    (
        "pulse.signal",
        "name",
        "o nome do sinal emitido, lido em todo disparo",
    ),
    (
        "motion.time_remap",
        "curve",
        "o remapeamento É a curva; sem ela o nó é a identidade e não tem outro modo",
    ),
    (
        "rig.skeleton",
        "branches",
        "a lista de ramos É o esqueleto que o nó constrói",
    ),
];

/// Todo param de forma do registry, com o widget que o veste.
fn shape_params(reg: &NodeRegistry) -> Vec<(&'static str, &'static str)> {
    let mut out = Vec::new();
    for m in reg.manifests() {
        let Some(hints) = reg.param_ui(m.id) else {
            continue;
        };
        for h in hints {
            if matches!(
                h.widget,
                ParamWidget::Curve
                    | ParamWidget::Gradient
                    | ParamWidget::Palette
                    | ParamWidget::Text
            ) {
                out.push((m.name, h.param));
            }
        }
    }
    out
}

/// ⭐⭐ **TODO PARAM DE FORMA OU É LIDO SEMPRE, OU É GATEADO** — o censo que teria
/// apanhado o editor mudo do `motion.oscillator`.
#[test]
fn every_shape_param_is_either_always_read_or_gated_to_the_mode_that_reads_it() {
    let reg = registry();
    let all = shape_params(&reg);
    assert!(
        all.len() >= 14,
        "CONTROLE: o censo tem de achar os params de forma que existem, e achou {}",
        all.len()
    );
    let mut mute = Vec::new();
    for (ty, param) in &all {
        let id = reg
            .manifests()
            .find(|m| m.name == *ty)
            .expect("o tipo veio do registry")
            .id;
        let gated = reg
            .param_gates(id)
            .is_some_and(|gs| gs.iter().any(|g| g.param == *param));
        let declared = ALWAYS_READ.iter().any(|(t, p, _)| t == ty && p == param);
        if !gated && !declared {
            mute.push(format!("{ty}::{param}"));
        }
    }
    assert!(
        mute.is_empty(),
        "estes params de FORMA são oferecidos em todo modo e ninguém disse se todo modo os \
         LÊ — ou gateie ao modo que os lê, ou declare-os em `ALWAYS_READ` com o porquê: \
         {mute:?}"
    );
}

/// ⭐⭐⭐ **NENHUM GATE É INERTE** — um que liste TODOS os valores do modo não esconde
/// nada, e lê-se como cura no censo acima.
///
/// ⚠️ **Este gate nasceu de uma mutação SOBREVIVENTE.** Trocar o `values` da curva do
/// `motion.oscillator` de `[Custom]` para `[0,1,2,3,4,5]` passava no censo — porque ele
/// perguntava *«existe gate?»* e não *«o gate esconde alguma coisa?»*. É a mesma doença um
/// nível acima da que o report do Enio expôs: um controle oferecido num modo que não o lê,
/// e agora um gate que não retira modo nenhum.
///
/// A régua é derivada, não escrita: o param que o gate observa (`when`) tem um
/// `ParamWidget::Enum`, então a escada dele é conhecida — e a lista do gate tem de ser um
/// subconjunto **estrito** dela.
#[test]
fn no_gate_hides_nothing() {
    let reg = registry();
    let mut inert = Vec::new();
    for m in reg.manifests() {
        let Some(gates) = reg.param_gates(m.id) else {
            continue;
        };
        let Some(hints) = reg.param_ui(m.id) else {
            continue;
        };
        for g in gates {
            let Some(h) = hints.iter().find(|h| h.param == g.when) else {
                continue;
            };
            let ParamWidget::Enum { labels } = h.widget else {
                continue;
            };
            if g.values.len() >= labels.len() {
                inert.push(format!(
                    "{}::{} (gate por `{}`: {} valores de {} modos)",
                    m.name,
                    g.param,
                    g.when,
                    g.values.len(),
                    labels.len()
                ));
            }
        }
    }
    assert!(
        inert.is_empty(),
        "estes gates listam TODOS os modos do param que observam -- eles nao escondem nada, \
         e um censo que so' pergunta «existe gate?» le^-os como cura: {inert:?}"
    );
}

/// ⚠️ **A tabela não pode envelhecer para o outro lado.** Uma entrada que nomeia um nó ou
/// um param que já não existe é uma decisão sobre nada — e ela ficaria a esconder o
/// sucessor dela do censo acima.
#[test]
fn every_always_read_entry_names_a_shape_param_that_exists() {
    let reg = registry();
    let all = shape_params(&reg);
    for (ty, param, why) in ALWAYS_READ {
        assert!(
            all.iter().any(|(t, p)| t == ty && p == param),
            "`{ty}::{param}` está em ALWAYS_READ e não é um param de forma do registry"
        );
        assert!(
            why.len() > 20,
            "`{ty}::{param}` tem de dizer POR QUE é lido sempre, e diz {why:?}"
        );
    }
}
