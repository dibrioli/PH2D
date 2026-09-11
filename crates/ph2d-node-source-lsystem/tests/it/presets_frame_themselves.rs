//! ⭐⭐⭐ **CADA MOLDE ENQUADRA-SE COMO OS IRMÃOS** — o portão que faltava aos oito.
//!
//! # Por que este ficheiro existe
//!
//! Report do Enio, 2026-08-29: *"o modo tree funciona aparentemente bem. os demais tem
//! resultado questionável."* A auditoria multiagêntica do mesmo dia mediu-o: a razão
//! `maior_dimensão / step` ia de **2,7** (Tree) a **2 581,8** (Koch) — **963×** entre dois
//! itens do mesmo selector, com uma coluna da cena `=108` a ter ~4 unidades de mundo.
//!
//! ⚠️⚠️ **E o único gate por-molde que existia contava ELEMENTOS**
//! (`every_preset_is_a_grammar_that_actually_draws`: `s.count() > 3`). A Koch passava com
//! 3 126 elementos a medir 1 291 unidades; o Sprig passava com 16 a medir largura **exactamente
//! 0,00**. *Uma contagem é a única grandeza que SOBE com este defeito* — ela não podia
//! reprovar, em molde nenhum, por construção.
//!
//! ⚠️ **A bancada que media isto já existia** (`examples/preset_report.rs`, seis réguas) e
//! **nenhum portão a corria**. Este ficheiro é essa bancada promovida: *uma ferramenta que
//! nenhum passo escrito chama pelo nome morre.*
//!
//! # A régua, e por que é esta
//!
//! `maior_dimensão_da_caixa / step` — **derivada do próprio param do nó**. ⛔ Ela NÃO pode ser
//! emprestada da cena `=108`: aquela cena tem tabela própria (`PLANTS`) e **nunca instancia um
//! `PRESET`**, então uma barra tirada de lá perderia a densidade a que foi medida.

use ph2d_node_source_lsystem as ls;
use ph2d_nodegraph::attr::{Column, Stream};

/// Os defaults do manifesto — lidos do manifesto, nunca escritos aqui.
fn default_of(name: &str) -> f32 {
    ls::MANIFEST
        .params
        .iter()
        .find(|p| p.name == name)
        .map_or(0.0, |p| p.default)
}

/// Coze um molde com o enquadramento que ele PRÓPRIO declara.
fn shoot(p: &ls::Preset, over: &[(&str, f32)]) -> Stream {
    let mut o: Vec<(&str, f32)> = vec![
        (ls::param::MODE, ls::MODE_GRAMMAR as f32),
        (ls::param::ANGLE, p.angle),
        (ls::param::STEP, p.step),
        (ls::param::WIDTH, p.width),
        (ls::param::SEED, default_of(ls::param::SEED)),
    ];
    o.extend_from_slice(over);
    ls::probe_build(p.axiom, p.rules, p.generations, &o)
}

/// Uma coluna escalar do esqueleto, ou vazia.
fn scalar(s: &Stream, name: &str) -> Vec<f32> {
    match s.get(name) {
        Some(Column::Scalar(v)) => v.clone(),
        _ => Vec::new(),
    }
}

/// `(largura, altura)` da caixa.
fn bbox(s: &Stream) -> (f32, f32) {
    match s.get("P") {
        Some(Column::Vec2(v)) if !v.is_empty() => {
            let x0 = v.iter().map(|q| q[0]).fold(f32::MAX, f32::min);
            let x1 = v.iter().map(|q| q[0]).fold(f32::MIN, f32::max);
            let y0 = v.iter().map(|q| q[1]).fold(f32::MAX, f32::min);
            let y1 = v.iter().map(|q| q[1]).fold(f32::MIN, f32::max);
            (x1 - x0, y1 - y0)
        }
        _ => (0.0, 0.0),
    }
}

/// ⭐⭐⭐ **OS OITO SAEM DO MESMO TAMANHO** — a cura do report.
///
/// A barra é uma RAZÃO entre os moldes, nunca um literal: o alvo é a mediana e cada um tem de
/// ficar dentro de `[1/k, k]` dela.
///
/// # ⛔⛔ A barra APERTOU de `1,6` para `1,10` em 2026-08-31, e o motivo é o que ela deixou passar
///
/// A 1.ª redacção tirava `k` da **dispersão que os quatro paramétricos já tinham** — *«`2,7 .. 3,9`
/// ⇒ `1,44×` entre o menor e o maior»*. ⚠️⚠️ **Isso é a dispersão ANTES da cura**, ou seja a
/// medida da doença que este gate existe para curar: ela admitia ±60 %, e em 2026-08-30 deixou
/// passar o `Wild` a **−15,6 %** sem se mexer (auditoria de seis lentes, doc 96 §1.1).
/// *Uma barra tirada da doença que se está a curar tolera a doença a voltar.*
///
/// # De onde sai o `1,10`
///
/// Da dispersão MEDIDA depois da cura, com a mediana em `1,777` unidades de mundo:
///
/// | molde | Tree | Fern | Bush | Weed | Wild | Koch | Dragon | Sprig |
/// |---|---|---|---|---|---|---|---|---|
/// | × mediana | 0,99 | 0,99 | 1,00 | 0,99 | **1,00** | 1,00 | 1,02 | 1,03 |
///
/// ⚠️ **E o que impede de apertar mais é MECÂNICO, não gosto:** o `step` de cada molde é gravado
/// com **três casas decimais**, e no menor deles (`Dragon`, `0,019`) isso é ±`2,6 %` de erro de
/// arredondamento sozinho. O `1,10` é ~3× o pior desvio observado e ~4× esse chão — apertado o
/// bastante para apanhar a classe de regressão que o `Wild` foi, largo o bastante para não
/// reprovar sobre a casa decimal.
#[test]
fn every_preset_frames_itself_like_its_siblings() {
    // ⚠️ **A régua aqui é a CAIXA (`max(w, h)`), e continua certa** — desde 2026-08-30 a lei do
    // crescimento usa outra (a largura média de Cauchy), mas a pergunta deste gate é o
    // ENQUADRAMENTO: quanto da tela o molde ocupa, que é a caixa. Ele corre em gerações
    // INTEIRAS, onde a lei do crescimento é inerte.
    const K: f32 = 1.10;
    let sizes: Vec<(&str, f32)> = ls::PRESETS
        .iter()
        .map(|p| {
            let (w, h) = bbox(&shoot(p, &[]));
            (p.label, w.max(h))
        })
        .collect();
    let mut sorted: Vec<f32> = sizes.iter().map(|(_, s)| *s).collect();
    sorted.sort_by(f32::total_cmp);
    let median = (sorted[3] + sorted[4]) * 0.5;
    assert!(median > 0.0, "nenhum molde desenhou: {sizes:?}");
    for (label, size) in &sizes {
        let ratio = size / median;
        assert!(
            (1.0 / K..=K).contains(&ratio),
            "o molde {label} sai {ratio:.2}x a mediana ({size:.2} contra {median:.2}) — \
             uma coluna da cena tem ~4 unidades de mundo, e foi isto que o dono do produto \
             viu em 2026-08-29. Todos: {sizes:?}"
        );
    }
}

/// ⭐⭐ **NENHUM MOLDE DESENHA UMA LINHA** — o gate que apanha o Sprig.
///
/// ⚠️ A largura dele era **exactamente 0,00**: a gramática `[+J][-J]` põe duas marcas, e uma
/// marca lê o osso do PAI e não o rumo da tartaruga (`turtle.rs`, comportamento declarado e
/// gateado), então as duas folhas nasciam no mesmo ponto e a viragem era deitada fora. O molde
/// saía um pau. *Uma bbox com um lado zero é a assinatura de um grau de liberdade morto.*
#[test]
fn every_preset_draws_in_two_dimensions() {
    for p in ls::PRESETS {
        let (w, h) = bbox(&shoot(p, &[]));
        assert!(
            w > 1e-3 && h > 1e-3,
            "o molde {} desenha {w:.4} x {h:.4} — um dos eixos e' uma linha",
            p.label
        );
    }
}

/// ⭐⭐ **O SLIDER `Angle` MEXE EM TODOS OS OITO.**
///
/// ⚠️ É o gate irmão do de cima, pelo outro lado: o Sprig era o único molde em que o `Angle`
/// era **byte-inerte** (bbox idêntica a 15° e a 60°), e a contagem de elementos não o via.
/// *Um controle pintado que não move a peça é um knob morto, e um molde pode matá-lo sozinho.*
#[test]
fn every_preset_answers_the_angle_slider() {
    for p in ls::PRESETS {
        let a = bbox(&shoot(p, &[(ls::param::ANGLE, p.angle * 0.5)]));
        let b = bbox(&shoot(p, &[(ls::param::ANGLE, p.angle * 1.5)]));
        assert!(
            (a.0 - b.0).abs() > 1e-4 || (a.1 - b.1).abs() > 1e-4,
            "o molde {} desenha o mesmo a {:.1} e a {:.1} graus — o Angle esta' morto nele",
            p.label,
            p.angle * 0.5,
            p.angle * 1.5
        );
    }
}

/// ⭐ **O que cada molde LÊ é DERIVADO do texto dele** — nunca declarado duas vezes.
#[test]
fn what_each_preset_reads_is_derived_from_its_text() {
    for p in ls::PRESETS {
        assert_eq!(
            p.reads,
            ls::Reads::of(p.rules),
            "o molde {} declara ler outra coisa do que a gramatica dele contem",
            p.label
        );
    }
    // ⚠️ E o CONTROLE das duas direcções — sem ele, um `Reads` que devolvesse sempre
    // `{true,true}` (ou sempre `false`) passaria a igualdade acima em todos os oito.
    assert!(ls::Reads::of("F(s)!A").width_scale);
    assert!(!ls::Reads::of("F -> FF").width_scale);
    assert!(ls::Reads::of("F\"F").length_scale);
    assert!(!ls::Reads::of("F -> FF").length_scale);
}

/// ⭐⭐⭐ **AS LISTAS DE GATE BATEM COM O QUE CADA GRAMÁTICA CONTÉM.**
///
/// ⚠️ `ParamGate::values` é um `&'static [i32]` e uma `const` não pode iterar a tabela — logo
/// as duas listas são escritas à mão. **É este gate que impede as duas respostas de
/// divergirem** no dia em que alguém acrescentar um molde ou meter um `!` numa regra.
/// *Uma lista à mão ao lado de um derivador é duas respostas à mesma pergunta, e a que o
/// artista vê é a que envelhece.*
#[test]
fn the_read_gates_agree_with_what_each_grammar_contains() {
    let mut reg = ph2d_node_registry::NodeRegistry::new();
    ls::register(&mut reg).expect("regista");
    let gates = reg.param_gates(ls::MANIFEST.id).expect("ha' gates");

    for (param, pick) in [
        (ls::param::WIDTH_SCALE, true),
        (ls::param::LENGTH_SCALE, false),
    ] {
        let g = gates
            .iter()
            .find(|g| g.param == param)
            .unwrap_or_else(|| panic!("o `{param}` tem de ser gateado pelo molde"));
        assert_eq!(
            g.when,
            ls::param::PRESET,
            "o sujeito e' o MOLDE, nao o modo"
        );
        let mut want: Vec<i32> = ls::PRESETS
            .iter()
            .enumerate()
            .filter(|(_, p)| {
                if pick {
                    p.reads.width_scale
                } else {
                    p.reads.length_scale
                }
            })
            .map(|(k, _)| k as i32)
            .collect();
        // ⚠️ O `Custom` entra SEMPRE: é onde o modo guiado e a gramática assada aterram, e
        // nos dois o knob está vivo (medido: a peça vai de `0,05` a `10,60` no guiado).
        want.push(ls::PRESET_CUSTOM as i32);
        let mut got = g.values.to_vec();
        got.sort_unstable();
        want.sort_unstable();
        assert_eq!(got, want, "a lista de `{param}` divergiu das gramaticas");
    }
}

/// **Os rótulos batem com a tabela, e o último é o `Custom`.**
#[test]
fn the_labels_match_the_table_and_end_in_custom() {
    assert_eq!(ls::PRESET_LABELS.len(), ls::PRESETS.len() + 1);
    for (k, p) in ls::PRESETS.iter().enumerate() {
        assert_eq!(ls::PRESET_LABELS[k], p.label, "o rotulo {k} divergiu");
    }
    assert_eq!(ls::PRESET_LABELS[ls::PRESET_CUSTOM], "Custom");
}

/// ⭐⭐ **O `Generations` de cada molde ainda MEXE a planta** — o achado do crítico de
/// completude: em Bush e Koch a geração 8 era byte-a-byte a 7, e metade do slider não fazia
/// nada, porque a derivação larga a geração INTEIRA que não coube no orçamento.
///
/// A barra é o próprio enquadramento: no valor que o molde declara, e a **meio** dele, a
/// planta tem de ser outra.
#[test]
fn every_preset_still_grows_at_the_generation_it_declares() {
    for p in ls::PRESETS {
        let full = shoot(p, &[]);
        let half = ls::probe_build(
            p.axiom,
            p.rules,
            (p.generations - 1.0).max(1.0),
            &[
                (ls::param::MODE, ls::MODE_GRAMMAR as f32),
                (ls::param::ANGLE, p.angle),
                (ls::param::STEP, p.step),
                (ls::param::WIDTH, p.width),
            ],
        );
        assert!(
            full.count() > half.count(),
            "o molde {} tem o mesmo numero de elementos na geracao {} e na {} ({}) — a \
             derivacao saturou antes do que o molde declara",
            p.label,
            p.generations,
            p.generations - 1.0,
            full.count()
        );
    }
}

/// ⭐⭐⭐ **O MODO GUIADO DE FÁBRICA EXPRIME A GRAMÁTICA DE FÁBRICA, AO BIT.**
///
/// ⚠️ Esta afirmação vivia na cena `=108` e **mudou de casa** em 2026-08-29: enquanto a coluna
/// guiada de lá usava os defaults, ela provava isto de graça — e era exactamente por isso que
/// o smoke não demonstrava a feature. Aqui ela é uma lei do NÓ, e a cena voltou a ser um
/// sítio onde se vê uma coisa acontecer.
///
/// O guiado emite `A(s*length_scale)`; a gramática de fábrica traz o literal `0.7`. Com o
/// slider em `0,7` as duas expressões são a mesma, e a derivação e a tartaruga fazem as mesmas
/// contas na mesma ordem. ⛔ **Ao BIT** — uma barra frouxa aceitaria outra associação de
/// multiplicações, e foi assim que o gate do `rig.fk` apanhou 1 ULP nesta crate.
#[test]
fn the_factory_guided_mode_expresses_the_factory_grammar_bit_for_bit() {
    const L: f32 = 0.7;
    let shared: &[(&str, f32)] = &[(ls::param::LENGTH_SCALE, L)];
    let mut guided: Vec<(&str, f32)> = vec![(ls::param::MODE, ls::MODE_GUIDED as f32)];
    guided.extend_from_slice(shared);
    let mut authored: Vec<(&str, f32)> = vec![(ls::param::MODE, ls::MODE_GRAMMAR as f32)];
    authored.extend_from_slice(shared);

    let a = ls::probe_build(ls::DEFAULT_AXIOM, ls::DEFAULT_RULES, 5.0, &guided);
    let b = ls::probe_build(ls::DEFAULT_AXIOM, ls::DEFAULT_RULES, 5.0, &authored);
    assert_eq!(a.count(), b.count(), "as contagens tem de bater");
    let p = |s: &Stream| match s.get("P") {
        Some(Column::Vec2(v)) => v.clone(),
        _ => panic!("P"),
    };
    for (i, (u, w)) in p(&a).iter().zip(p(&b).iter()).enumerate() {
        assert_eq!(
            u.map(f32::to_bits),
            w.map(f32::to_bits),
            "o elemento {i} difere: {u:?} contra {w:?} — os sliders deixaram de exprimir a \
             gramatica de fabrica"
        );
    }
    // ⚠️ E o CONTROLE: com o `length_scale` NOUTRO valor as duas TÊM de divergir, senão este
    // gate estaria a medir dois caminhos que ignoram o slider.
    let c = ls::probe_build(
        ls::DEFAULT_AXIOM,
        ls::DEFAULT_RULES,
        5.0,
        &[
            (ls::param::MODE, ls::MODE_GUIDED as f32),
            (ls::param::LENGTH_SCALE, 1.2),
        ],
    );
    assert_ne!(
        p(&c)[a.count() - 1],
        p(&a)[a.count() - 1],
        "o `length_scale` nao mexe no guiado — a identidade acima seria um acidente"
    );
}

/// ⛔⛔⛔ **O CENSO DAS MARCAS DE CADA MOLDE — DERIVADO, e não escrito ao lado do valor.**
///
/// # Por que este gate existe
///
/// Auditoria de seis lentes, doc 96 §5.2. Cada `leaf_first_level` da tabela levava um comentário
/// com números contados à mão, e **cinco de oito não se reproduziam**:
///
/// | molde | o comentário dizia | medido |
/// |---|---|---|
/// | **Bush** | `121` marcas, `1..5`, sobram `96` | **`156`**, `1..4`, sobram **`48`** ← *os números do Weed* |
/// | **Dragon** | `512` marcas | **`2 048`** (4×) |
/// | **Fern** | `2..5`, `16 de 26` | `1..5`, `16` de **`31`** |
///
/// Consequência prática: quem lê o comentário do `Bush` espera que `First Level = 3` mostre
/// **96 de 121** folhas (79 %); o produto mostra **48 de 156** (31 %).
///
/// ⚠️⚠️ **O gate que existia não podia acusar isto:** `no_preset_silences_its_own_leaves` afirma
/// só `!marcas.is_empty()` e `vivas > 0`, e **uma contagem 4× errada não move nenhum dos dois
/// predicados**. É a mesma cegueira que o doc-comment DELE acusa no gate anterior (*«uma
/// contagem é a única grandeza que SOBE com este defeito»*), reaparecida um nível acima.
///
/// # A cura é a FORMA, não os números
///
/// ⛔ Reescrever os comentários com os valores certos compra um dia. O que impede a próxima
/// deriva é a **propriedade**, e ela é a razão de o campo existir: *o `First Level` de cada
/// molde tem de deixar folhas vivas **e** calar as do tronco*. Ambos os lados, medidos, para os
/// oito. Os números exactos saem da sonda irmã (`preset_report`), que ninguém copia para um
/// comentário.
#[test]
fn every_presets_first_level_keeps_leaves_alive_and_silences_the_trunk() {
    for p in ls::PRESETS {
        let s = shoot(p, &[]);
        let sym = scalar(&s, "sym");
        let depth = scalar(&s, "depth");
        assert_eq!(
            sym.len(),
            depth.len(),
            "`{}`: colunas desalinhadas",
            p.label
        );

        let marcas: Vec<u16> = sym
            .iter()
            .zip(&depth)
            .filter(|(c, _)| ls::LEAF_SYMBOLS.contains(&(**c as i32 as u8)))
            .map(|(_, d)| *d as u16)
            .collect();
        assert!(
            !marcas.is_empty(),
            "`{}` não emite marca nenhuma — o campo `leaf_first_level` dele não tem sujeito",
            p.label
        );
        let first = p.leaf_first_level as u16;
        let vivas = marcas.iter().filter(|d| **d >= first).count();
        let caladas = marcas.len() - vivas;
        let (dmin, dmax) = (
            marcas.iter().copied().min().unwrap_or(0),
            marcas.iter().copied().max().unwrap_or(0),
        );

        // 1. O molde tem de ficar com folhas — foi isto que esvaziou o `Sprig` uma vez.
        assert!(
            vivas > 0,
            "`{}`: `First Level = {first}` apaga TODAS as {} marcas (profundidades {dmin}..{dmax})",
            p.label,
            marcas.len()
        );
        // 2. E o `first` tem de descrever a planta: ou ele cala alguma coisa, ou ele é `1` —
        //    que é a resposta honesta para uma figura sem tronco (as curvas têm as marcas todas
        //    na profundidade 1, e ali não há por onde discriminar).
        assert!(
            caladas > 0 || first <= dmin,
            "`{}`: `First Level = {first}` não cala nada e não é o mínimo ({dmin}) — ele descreve \
             uma planta que este molde não é",
            p.label
        );
    }
}

/// ⛔⛔⛔ **A SEMENTE ENTRA NO PRODUTO POR UMA PORTA SÓ** — e o gate mede a CONSEQUÊNCIA, não
/// a chamada.
///
/// # O defeito
///
/// Auditoria de seis lentes, doc 96 §B2. A semente era lida de duas maneiras — `abs() as u32`
/// para a escolha de regras e `to_bits()` para a folha — e o resultado não era
/// não-determinismo, era **duas identidades sob um nome**: arrastá-la mudava as folhas
/// continuamente e a estrutura em degraus, e `−1` dava a mesma planta que `+1`.
///
/// # A régua
///
/// ⚠️ **Não «as duas chamam `seed_bits`»** — isso seria um gate textual, e um `to_bits` novo
/// escrito à mão passaria por ele. A régua é o que a lei COMPRA: sementes que diferem só na
/// fracção, ou só no sinal, têm de dar plantas **diferentes**. Com a truncagem, não davam.
#[test]
fn the_seed_has_one_law_and_every_distinct_value_is_a_distinct_plant() {
    // O `Wild` é o único molde ESTOCÁSTICO — os outros sete ignoram a semente na estrutura.
    let w = ls::PRESETS
        .iter()
        .find(|p| p.label == "Wild")
        .expect("o molde estocástico");
    let figura = |seed: f32| {
        let s = shoot(w, &[(ls::param::SEED, seed)]);
        let (a, b) = bbox(&s);
        (s.count(), a.to_bits(), b.to_bits())
    };
    // 1. ⭐ A FRACÇÃO conta — a truncagem comia-a, e quatro sementes davam a mesma planta.
    let base = figura(1.0);
    let mut distintas = std::collections::BTreeSet::new();
    for k in 0..8u8 {
        distintas.insert(figura(1.0 + f32::from(k) * 0.125));
    }
    assert!(
        distintas.len() >= 6,
        "oito sementes entre `1,0` e `1,875` deram só {} plantas distintas — a lei está a \
         deitar fora a fracção",
        distintas.len()
    );
    // 2. ⭐ E o SINAL conta — o `abs` fazia `−1` e `+1` a mesma planta.
    assert_ne!(
        figura(-1.0),
        base,
        "`−1` e `+1` dão a MESMA planta — a lei está a deitar fora o sinal"
    );
    // 3. ⚠️ A metade oposta: a mesma semente reproduz, senão «distintas» seria satisfeito por
    //    não-determinismo, que é o defeito contrário e igualmente mau.
    assert_eq!(figura(1.0), base, "a mesma semente tem de reproduzir");
    // 4. ⛔ E um não-finito não escolhe regra nenhuma — ele cai no `0`, não num lixo.
    assert_eq!(
        figura(f32::NAN),
        figura(f32::INFINITY),
        "`NaN` e `∞` têm de cair na MESMA semente neutra"
    );
}
