//! ⭐⭐⭐ **Um APELIDO não tem valor: ele tem PAI.**
//!
//! Os 16 slots `timeline-*` resolvem exactamente para um slot geral. Até 2026-09-07 isso era uma
//! **coincidência medida**: 57 valores escritos à mão no `tokens.json` (16 no `forge`, 16 no
//! `sunstone`, 16 no `blueprint`, 9 no `workshop`) que este gate comparava par a par, 64 pares,
//! 0 divergências. Hoje é **construção** — [`ColorToken::alias_parent`] declara o pai e a fábrica
//! resolve através dele, logo não há valor a escrever nem valor que possa divergir por engano.
//!
//! # ⛔⛔ A ordem era «fundir os apelidos», e a medição partiu-a em duas metades
//!
//! O dono mandou fundir (2026-09-07) sobre uma nota de 30/08 que prometia *«83 → 67 slots por
//! tema, zero pixels»*. Medido antes de executar:
//!
//! - **os VALORES** eram duplicação a sério — 57 cópias mantidas à mão. **Apagadas**, e a prova de
//!   que foi zero-pixel é contra o ficheiro ANTIGO: 64 pares, 0 divergências, cada valor apagado
//!   byte a byte igual ao do pai;
//! - **os NOMES** não eram duplicação: cada um é uma linha **regulável** no painel *Tokens*
//!   (`ColorToken::ALL`) e um destino de vínculo no editor vetorial. Apagá-los tirava ao artista
//!   16 controlos que ele tem hoje.
//!
//! ⭐ **E o estado da arte, que o dono mandou consultar, diz o mesmo.** O manual do Blender
//! (CC-BY-SA): *«The colors for each editor can be set separately by simply selecting the editor
//! you wish to change»* — uma secção de cor por editor. O `theme_modern.cpp` do Godot (MIT)
//! escreve cor por CONTROLO (`font_color` de `Tree`, de `Button`, …). *As duas referências mantêm
//! tokens de componente e expõem-nos; o que elas não fazem é escrever o valor deles à mão.*
//!
//! ⚠️ **E metade da ordem já estava feita sem ninguém saber:** a família moderna deriva tudo desde
//! a wave 1, logo ali os 16 nunca tiveram valor próprio — a nota de 30/08 descrevia um mundo que a
//! wave 1 já tinha mudado.
//!
//! # O que significa este gate ficar VERMELHO
//!
//! Que alguém deu valor próprio a um slot do Timeline **outra vez** — pondo-o de volta no
//! `tokens.json` — ou que a tabela de pais deixou de cobrir os 16. ⛔ Não afrouxe: o que se mede é
//! uma igualdade de bytes, e uma igualdade aproximada não afirma nada.

use ph2d_tokens::{ColorToken as C, Theme};

/// ⭐⭐ **A população são os APELIDOS, e ela sai do PRODUTO** (`alias_parent`), nunca de um prefixo
/// de nome.
///
/// ⛔⛔ **Ela era `key().starts_with("timeline-")`** — e um apelido novo fora daquela família (o
/// `bone-handle`, 2026-09-16) entrava **sem gate nenhum**, que é o censo definido por prefixo que o
/// `CLAUDE.md` §5.0 nomeia: *a varredura continua verde a medir o que já media*. O piso abaixo
/// mantém a metade que ela tinha.
fn aliases() -> Vec<C> {
    C::ALL
        .iter()
        .copied()
        .filter(|t| t.alias_parent().is_some())
        .collect()
}

/// ⭐ **Todo `timeline-*` declara um pai, e resolve para ele em TODO tema.**
#[test]
fn every_timeline_slot_resolves_to_its_declared_parent() {
    let all = [
        Theme::Forge,
        Theme::Workshop,
        Theme::Sunstone,
        Theme::Blueprint,
        Theme::Dark,
        Theme::Gray,
        Theme::Light,
        Theme::Oled,
    ];
    let mut pares = 0;
    for t in all {
        for a in aliases() {
            let parent = a
                .alias_parent()
                .unwrap_or_else(|| panic!("`{}` nao declara pai", a.key()));
            let (x, y) = (a.factory(t), parent.factory(t));
            assert_eq!(
                (x.r, x.g, x.b, x.a),
                (y.r, y.g, y.b, y.a),
                "`{}` nao resolve para `{}` no tema {t:?}",
                a.key(),
                parent.key()
            );
            pares += 1;
        }
    }
    assert_eq!(
        pares,
        17 * 8,
        "esperava 17 apelidos x 8 temas e comparei {pares}: ou a familia mudou de tamanho, ou o \
         filtro deixou de achar os apelidos"
    );
}

/// ⛔⛔ **Nenhum apelido tem valor escrito à mão no `tokens.json`.**
///
/// Esta é a metade que a fusão comprou: enquanto os 57 valores existiam, cada tema novo obrigava
/// a escrevê-los outra vez, e um deles podia divergir sem ninguém ver. *Um valor que não existe
/// não pode divergir.*
#[test]
fn no_alias_carries_a_hand_written_value() {
    let json = std::fs::read_to_string(
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../docs/design/tokens.json"),
    )
    .expect("tokens.json");
    // ⚠️ **A varredura sai da população de apelidos**, e não do prefixo `timeline-`: um apelido
    // novo noutra família (o `bone-handle`) escaparia a um prefixo escrito à mão, e o gate ficaria
    // verde a medir o que já media.
    let chaves: Vec<String> = aliases()
        .iter()
        .map(|a| format!("\"{}\"", a.key()))
        .collect();
    let strays: Vec<&str> = json
        .lines()
        .filter(|l| {
            let t = l.trim_start();
            chaves.iter().any(|k| t.starts_with(k.as_str()))
        })
        .collect();
    assert!(
        strays.is_empty(),
        "estes apelidos voltaram a ter valor proprio no `tokens.json` — um apelido tem PAI, nao \
         valor:\n  {}",
        strays.join("\n  ")
    );
}

/// ⛔⛔⛔ **O PAI de cada apelido é uma DECISÃO, e depois da fusão ela deixa de ser mensurável.**
///
/// ⚠️⚠️ **Isto nasceu de uma mutação que SOBREVIVEU:** apontar o `timeline-ruler-tick` para
/// `Text2` em vez de `Text3` não acorda o teste de resolução — e não pode acordar, porque depois
/// da fusão o apelido resolve *através* do pai que se declarar. **A verdade que provava o par
/// estava nos 57 valores que esta wave apagou**, e apagá-los levou o oráculo com eles.
///
/// ⇒ o que fica gateável é outra coisa: que a **declaração** não mude em silêncio. Esta lista é a
/// decisão congelada, medida uma vez contra o `tokens.json` ANTIGO (64 pares, 0 divergências);
/// repontar um apelido passa a exigir editar esta linha, que é onde alguém repara.
///
/// *Quando uma fusão apaga o oráculo, o que sobra para gatear é a intenção — e ela gateia-se por
/// congelamento, não por medição.*
#[test]
fn the_parent_of_each_alias_is_the_one_the_measurement_found() {
    const FROZEN: &[(&str, &str)] = &[
        ("timeline-curve", "accent"),
        ("timeline-handle", "accent"),
        ("timeline-key-selected", "accent"),
        ("timeline-loop-brace", "accent"),
        ("timeline-playhead", "accent"),
        ("timeline-summary-ring", "accent"),
        ("timeline-handle-line", "accent-soft"),
        ("timeline-loop-region", "accent-soft"),
        ("timeline-row-alt", "bg-2"),
        ("timeline-ruler-bg", "bg-2"),
        ("timeline-marker", "warn"),
        ("timeline-summary-key", "warn"),
        ("timeline-key-active", "accent-press"),
        ("timeline-missing", "danger"),
        ("timeline-key", "text-1"),
        ("timeline-ruler-tick", "text-3"),
        // ⭐ **O `bone-handle` entra por OUTRA medição** (2026-09-16, report do dono: as alças de
        // curvatura vestiam a cor do osso): ali o oráculo não é o `tokens.json` antigo, é a
        // DISTÂNCIA perceptual ao corpo do osso nos 8 temas, e ela continua a poder ser medida —
        // o gate `the_bend_handle_never_wears_the_colour_of_the_bone` fá-lo a cada corrida.
        ("bone-handle", "success"),
    ];
    let mut seen = 0;
    for a in aliases() {
        let parent = a.alias_parent().expect("apelido sem pai");
        let esperado = FROZEN
            .iter()
            .find(|(k, _)| *k == a.key())
            .unwrap_or_else(|| panic!("`{}` nao esta' na lista congelada", a.key()))
            .1;
        assert_eq!(
            parent.key(),
            esperado,
            "`{}` foi repontado para `{}` — a medicao de 2026-08-30 achou `{esperado}`",
            a.key(),
            parent.key()
        );
        seen += 1;
    }
    assert_eq!(
        seen,
        FROZEN.len(),
        "a lista congelada e a familia divergiram"
    );
}

/// ⚠️ **A metade de OBSOLESCÊNCIA: a família de apelidos é NOMEADA, e cresce com a nota junto.**
///
/// O censo de cor de 2026-08-30 mediu que os 16 do Timeline eram **exactamente** os 16 apelidos de
/// todo o sistema — os 34 slots dos nós são valores distintos. ⭐ **O 17.º nasceu em 2026-09-16**:
/// o `bone-handle`, de um report do dono (as alças de curvatura vestiam a cor do osso), e o pai
/// dele foi MEDIDO nos 8 temas — ver a tabela em `ph2d_tokens::color_alias`.
///
/// ⇒ a contagem sobe **com** o nome ao lado, e um apelido que nasça sem entrar nesta lista reprova.
#[test]
fn the_alias_family_is_exactly_the_named_slots() {
    /// As famílias que declaram apelidos, e a contagem de cada uma.
    const FAMILIAS: &[(&str, usize)] = &[("timeline-", 16), ("bone-", 1)];
    let declared: Vec<&str> = C::ALL
        .iter()
        .filter(|t| t.alias_parent().is_some())
        .map(|t| t.key())
        .collect();
    let total: usize = FAMILIAS.iter().map(|(_, n)| n).sum();
    assert_eq!(
        declared.len(),
        total,
        "a familia de apelidos mudou: {declared:?}"
    );
    for (prefixo, n) in FAMILIAS {
        let quantos = declared.iter().filter(|k| k.starts_with(prefixo)).count();
        assert_eq!(
            quantos, *n,
            "a familia `{prefixo}*` tem {quantos} apelidos e a nota diz {n}: {declared:?}"
        );
    }
    // E um pai nunca é ele próprio um apelido — senão a resolução seria uma cadeia.
    for t in C::ALL {
        if let Some(p) = t.alias_parent() {
            assert!(
                p.alias_parent().is_none(),
                "`{}` aponta para `{}`, que TAMBEM e' apelido: a resolucao viraria cadeia",
                t.key(),
                p.key()
            );
        }
    }
}
