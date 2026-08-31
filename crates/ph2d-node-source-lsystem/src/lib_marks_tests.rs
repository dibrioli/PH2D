//! **AS TRÊS LETRAS QUE PLANTAM UM OBJECTO** — `J` · `K` · `M`: que elas existem no alfabeto
//! da tartaruga, e QUANTO cada marca já abriu.
//!
//! ⚠️ **Este arquivo existe por um TETO DE LOC** (HR-18, 700 no default da workspace), e o
//! corte é por RESPONSABILIDADE: o irmão [`super::tests`] mede o nó inteiro (defaults,
//! gerações fraccionárias, a costura com o painel), e este mede **as marcas**.

use super::*;
use ph2d_nodegraph::attr::Column;

/// A coluna escalar `name` do esqueleto. ⚠️ Gémea da do irmão `lib_tests`: ela é três linhas
/// e viaja com quem a usa — partilhá-la obrigaria a um módulo de teste comum entre dois
/// `#[path]`, que é mais encanamento do que a duplicação que evita.
fn scal(s: &ph2d_nodegraph::attr::Stream, name: &str) -> Vec<f32> {
    match s.get(name) {
        Some(Column::Scalar(v)) => v.clone(),
        _ => panic!("coluna {name}"),
    }
}

/// ⭐⭐ **AS TRÊS LETRAS SÃO MESMO ÂNCORAS** — o contrato entre [`LEAF_PARAMS`] e o alfabeto.
///
/// ⚠️ **O emparelhamento por índice é compile-enforced** (dois arrays de `3`), mas isso não diz
/// que as letras EXISTEM: um `LEAF_SYMBOLS` com um `b'Q'` compilaria, o painel ofereceria o
/// campo, e o artista escreveria um nome que nunca planta nada. *Uma lista bem-formada não é
/// uma lista verdadeira.*
///
/// A régua é o produto: uma gramática que emite a letra tem de devolver um elemento com aquele
/// `sym`, e ele NÃO pode ter osso (uma âncora é uma marca, não um segmento).
#[test]
fn every_leaf_letter_is_an_anchor_the_turtle_actually_emits() {
    for (i, letter) in LEAF_SYMBOLS.iter().enumerate() {
        let src = format!("F[{}]", *letter as char);
        let s = probe_build(&src, "F -> F", 1.0, &[]);
        let syms = scal(&s, "sym");
        let lens = scal(&s, "len");
        let at = syms
            .iter()
            .position(|v| *v as i32 as u8 == *letter)
            .unwrap_or_else(|| {
                panic!(
                    "a letra {} ({}) do param `{}` não produz elemento nenhum: {syms:?}",
                    *letter as char, letter, LEAF_PARAMS[i]
                )
            });
        assert_eq!(
            lens[at], 0.0,
            "a marca {} tem de ser uma ÂNCORA (comprimento zero), e veio com osso",
            *letter as char
        );
    }
}

/// ⭐⭐⭐ **O PESO DE UMA MARCA SÓ SOBE** — a lei que o 2.º smoke de 2026-08-30 impôs.
///
/// Enio: *"a cada segmento a folha cresce e diminui. bem bizarro"*. A 1.ª lei era um
/// cruza-fade (peso `f` à geração nova, `1 − f` à anterior), para que uma planta parada
/// mostrasse só as pontas — e o preço era cada folha **encolher até sumir** quando o ramo
/// seguinte brotava dela.
///
/// ⚠️ **A afirmação forte é a MONOTONIA**, não os casos: uma folha nascida em `b` nunca fica
/// mais pequena do que já esteve, por mais que a planta cresça. Um gate que só olhasse os
/// extremos passaria com um vale no meio — que é exactamente o que a lei antiga tinha.
#[test]
fn a_marks_weight_only_ever_grows() {
    // ⚠️ **A régua corre no PRODUTO, não numa cópia do plano de gerações.** A 1.ª redacção
    // fabricava a sequência `(youngest, fracção)` à mão — e fabricou-a ao contrário, andando
    // para trás no tempo. *Uma fixtura que reimplementa a lei que testa pode estar errada de
    // maneiras que a lei não está.*
    let p = &PRESETS[0];
    // ⚠️ **O nível mínimo sai do caminho**, senão este gate mediria DUAS leis ao mesmo tempo
    // e uma folha calada pelo nível leria como uma folha que nunca amadureceu.
    let over = [
        (param::ANGLE, p.angle),
        (param::STEP, p.step),
        (param::LEAF_FIRST_LEVEL, 1.0),
    ];
    let mut visto: std::collections::BTreeMap<i64, f32> = Default::default();
    let mut g = 0.5f32;
    while g <= 6.0 {
        let s = probe_build(p.axiom, p.rules, g, &over);
        let (sym, born) = (scal(&s, "sym"), scal(&s, "gen"));
        let grow = match s.get("mark_grow") {
            Some(Column::Scalar(v)) => v.clone(),
            _ => panic!("sem `mark_grow`"),
        };
        for i in 0..sym.len() {
            if !crate::LEAF_SYMBOLS.contains(&(sym[i] as i32 as u8)) {
                continue;
            }
            let e = visto.entry(born[i] as i64).or_insert(0.0);
            assert!(
                grow[i] >= *e - 1e-6,
                "a folha da geracao {} ENCOLHEU a g={g}: {} -> {}",
                born[i],
                *e,
                grow[i]
            );
            *e = grow[i];
        }
        g += 0.25;
    }
    // ⚠️ **O CONTROLE**: a varredura tem de ter visto várias gerações e todas maduras no fim —
    // um laço que não iterasse passaria as afirmações acima em silêncio.
    assert!(visto.len() >= 4, "a varredura viu {} geracoes", visto.len());
    assert!(
        visto.values().all(|w| (*w - 1.0).abs() < 1e-6),
        "no fim toda folha tem de estar madura: {visto:?}"
    );
    // A geração mais nova abre com a fracção; toda a mais velha está cheia.
    use crate::turtle::mark_grow;
    assert_eq!(mark_grow(b'J', 5, (5, 0.25)), 0.25);
    assert_eq!(mark_grow(b'J', 4, (5, 0.25)), 1.0);
    assert_eq!(mark_grow(b'J', 1, (5, 0.25)), 1.0);
    // ⚠️ **As TRÊS letras**, não só o `J` — uma lei escrita para uma letra deixaria as outras
    // duas sem crescimento nenhum.
    for sym in *crate::LEAF_SYMBOLS {
        assert_eq!(
            mark_grow(sym, 5, (5, 0.5)),
            0.5,
            "a letra {} nao abre",
            sym as char
        );
    }
    // ⛔ **E um OSSO está sempre cheio** — devolver-lhe o peso da marca apagaria a planta.
    for sym in *b"FGfg" {
        assert_eq!(
            mark_grow(sym, 5, (5, 0.5)),
            1.0,
            "o osso {} encolheu",
            sym as char
        );
    }
}

/// ⚠️ **E a coluna existe na CORRENTE**, não só na função — o consumidor é a membrana do shell,
/// e uma lei sem coluna não alcança ninguém.
#[test]
fn the_skeleton_publishes_the_mark_opening_weight() {
    let p = &PRESETS[0];
    // ⚠️ Idem: aqui mede-se a IDADE, e o nível tem gate próprio.
    let over = [
        (param::ANGLE, p.angle),
        (param::STEP, p.step),
        (param::LEAF_FIRST_LEVEL, 1.0),
    ];
    let pesos = |g: f32| -> Vec<f32> {
        let s = probe_build(p.axiom, p.rules, g, &over);
        let sym = scal(&s, "sym");
        let grow = match s.get("mark_grow") {
            Some(Column::Scalar(v)) => v.clone(),
            _ => panic!("a coluna `mark_grow` nao viaja na corrente"),
        };
        (0..sym.len())
            .filter(|&i| sym[i] as i32 as u8 == b'J')
            .map(|i| grow[i])
            .collect()
    };
    // Geração INTEIRA: toda folha está madura.
    let cheias = pesos(5.0);
    assert_eq!(cheias.len(), 31, "uma marca por segmento, a g=5");
    assert!(
        cheias.iter().all(|w| (*w - 1.0).abs() < 1e-6),
        "numa planta parada nenhuma folha esta' a meio: {cheias:?}"
    );
    // A MEIO: só a colheita nova está a abrir, e ela é a diferença entre as duas contagens.
    let meio = pesos(4.5);
    let novas = meio.iter().filter(|w| (**w - 0.5).abs() < 1e-6).count();
    let velhas = meio.iter().filter(|w| (**w - 1.0).abs() < 1e-6).count();
    assert_eq!(
        (novas, velhas),
        (16, 15),
        "a g=4,5 as 15 folhas velhas ficam cheias e as 16 novas abrem a meio: {meio:?}"
    );
}

/// ⛔⛔ **DUAS FOLHAS NÃO SE EMPILHAM NO MESMO PONTO** — a outra metade do report de
/// 2026-08-30 (*"elas aparecem em cada segmento"*), e esta era do MOLDE, não da lei.
///
/// A gramática de fábrica era `A(s) -> F(s)![+A(s*0.7)J][-A(s*0.7)J]`: o `J` vinha **depois**
/// da sub-árvore inteira, e a tartaruga, ao sair dela, está de volta ao fim do `F` — onde as
/// marcas de todas as gerações que a envolvem também caem. Medido: **62 marcas em 30 sítios**
/// (`2,07×`), folhas idênticas uma em cima da outra.
///
/// ⇒ o `J` passa a vir **logo a seguir ao segmento** (`F(s)[J]!…`): uma marca por segmento,
/// `1,00×`. *Uma contagem de marcas não vê um empilhamento; o que o vê é contar os SÍTIOS.*
#[test]
fn no_two_leaves_land_on_the_same_spot() {
    for p in PRESETS {
        let s = probe_build(
            p.axiom,
            p.rules,
            p.generations,
            &[(param::ANGLE, p.angle), (param::STEP, p.step)],
        );
        let sym = scal(&s, "sym");
        let pos = match s.get("P") {
            Some(Column::Vec2(v)) => v.clone(),
            _ => panic!("sem P"),
        };
        // ⛔⛔ **A CHAVE INCLUI O PAI, e a razão é a curva do DRAGÃO:** ela toca-se a si
        // própria (é isso que uma curva que ladrilha o plano faz), então duas marcas de ramos
        // DIFERENTES caem no mesmo ponto — `2048` marcas em `1324` sítios — sem que nada esteja
        // empilhado por culpa da colocação.
        //
        // ⇒ o que este gate acusa é o empilhamento QUE EU FAÇO: duas marcas do MESMO pai no
        // mesmo sítio, que foi o defeito de `A(s) -> F(s)![+A J][-A J]` (o `J` depois da
        // sub-árvore volta ao pé do pai, e as gerações que a envolvem caem todas ali).
        // *Uma régua que não separa a geometria da figura da colocação acusa a figura.*
        let parent = scal(&s, "parent");
        let mut sitios: Vec<(i64, i64, i64)> = (0..sym.len())
            .filter(|&i| crate::LEAF_SYMBOLS.contains(&(sym[i] as i32 as u8)))
            .map(|i| {
                (
                    parent[i] as i64,
                    (pos[i][0] * 1e4) as i64,
                    (pos[i][1] * 1e4) as i64,
                )
            })
            .collect();
        let marcas = sitios.len();
        sitios.sort_unstable();
        sitios.dedup();
        assert_eq!(
            marcas,
            sitios.len(),
            "{}: {marcas} marcas em {} sitios — folhas empilhadas",
            p.label,
            sitios.len()
        );
    }
}

#[test]
#[ignore = "sonda"]
fn probe_mark_depths_and_param_count() {
    println!("params do no': {}", MANIFEST.params.len());
    for p in PRESETS.iter().take(5) {
        let s = probe_build(
            p.axiom,
            p.rules,
            p.generations,
            &[(param::ANGLE, p.angle), (param::STEP, p.step)],
        );
        let sym = scal(&s, "sym");
        let depth = scal(&s, "depth");
        let mut hist: std::collections::BTreeMap<i64, usize> = Default::default();
        for i in 0..sym.len() {
            if crate::LEAF_SYMBOLS.contains(&(sym[i] as i32 as u8)) {
                *hist.entry(depth[i] as i64).or_default() += 1;
            }
        }
        println!("{:<6} marcas por profundidade: {hist:?}", p.label);
    }
}

/// ⭐⭐⭐ **NADA DE FOLHA NA RAIZ NEM NO CAULE** — report do Enio (2026-08-30, foto com duas
/// setas): *"ainda nascem folhas no fim de cada segmento mesmo se o segmento é a raiz ou o
/// caule"*.
///
/// ⚠️ **A afirmação é sobre a FAIXA, não sobre um número:** subir o `First Level` só pode
/// APAGAR folhas, e as que ficam são exactamente as dos níveis a partir dele. Um gate que só
/// contasse passaria com a faixa invertida.
#[test]
fn no_leaf_grows_on_the_root_or_the_trunk() {
    let p = &PRESETS[0];
    let abertas = |first: f32| -> Vec<u16> {
        let s = probe_build(
            p.axiom,
            p.rules,
            p.generations,
            &[
                (param::ANGLE, p.angle),
                (param::STEP, p.step),
                (param::LEAF_FIRST_LEVEL, first),
            ],
        );
        let (sym, depth) = (scal(&s, "sym"), scal(&s, "depth"));
        let grow = match s.get("mark_grow") {
            Some(Column::Scalar(v)) => v.clone(),
            _ => panic!("sem `mark_grow`"),
        };
        (0..sym.len())
            .filter(|&i| crate::LEAF_SYMBOLS.contains(&(sym[i] as i32 as u8)) && grow[i] > 0.0)
            .map(|i| depth[i] as u16)
            .collect()
    };
    // A árvore de fábrica tem marcas nos níveis 1..5 (contagens 1 · 2 · 4 · 8 · 16).
    let todas = abertas(1.0);
    assert_eq!(todas.len(), 31, "a fixtura tem de ter as 31 marcas");
    // ⛔ **O DEFAULT não deixa nada no caule**: as duas setas da foto são os níveis 1 e 2.
    let padrao = abertas(3.0);
    assert!(
        padrao.iter().all(|d| *d >= 3),
        "o default deixou folha no caule: {padrao:?}"
    );
    assert_eq!(padrao.len(), 28, "sobram as dos niveis 3, 4 e 5");
    // ⚠️ **Monótona na faixa**: subir o nível só apaga.
    let mut anterior = usize::MAX;
    for first in 1..=6u16 {
        let v = abertas(f32::from(first));
        assert!(
            v.len() <= anterior,
            "subir o First Level para {first} ACRESCENTOU folhas: {} -> {}",
            anterior,
            v.len()
        );
        assert!(
            v.iter().all(|d| *d >= first),
            "com First Level {first} sobrou uma folha mais rasa: {v:?}"
        );
        anterior = v.len();
    }
    assert_eq!(
        anterior, 0,
        "acima do nivel mais fundo nao sobra folha nenhuma"
    );
}

/// ⭐⭐ **A FOLHA TEM VIRAGEM PRÓPRIA** — report do Enio (2026-08-30): *"nem as rotações das
/// folhas"*.
///
/// ⚠️ **Três afirmações, e a primeira é a que protege o resto:** com os dois em `0` o ângulo é
/// o do ramo **ao bit** (uma feature nova não pode mexer no que já shipou); o `Leaf Angle` soma
/// o mesmo a todas; e o `Leaf Spread` abre-as **umas em relação às outras**, dentro da faixa.
#[test]
fn a_leaf_has_a_turn_of_its_own() {
    let p = &PRESETS[0];
    let angs = |angle: f32, spread: f32| -> Vec<f32> {
        let s = probe_build(
            p.axiom,
            p.rules,
            p.generations,
            &[
                (param::ANGLE, p.angle),
                (param::STEP, p.step),
                (param::LEAF_FIRST_LEVEL, 1.0),
                (param::LEAF_ANGLE, angle),
                (param::LEAF_SPREAD, spread),
            ],
        );
        let (sym, rot) = (scal(&s, "sym"), scal(&s, "rot"));
        (0..sym.len())
            .filter(|&i| crate::LEAF_SYMBOLS.contains(&(sym[i] as i32 as u8)))
            .map(|i| rot[i])
            .collect()
    };
    let base = angs(0.0, 0.0);
    assert!(base.len() > 8, "so' {} folhas", base.len());
    // 1. ⛔ **Identidade ao BIT** com os dois em zero.
    for (a, b) in base.iter().zip(angs(0.0, 0.0)) {
        assert_eq!(a.to_bits(), b.to_bits(), "o zero tem de ser o no-op exacto");
    }
    // 2. O `Leaf Angle` soma o MESMO a todas.
    for (a, b) in base.iter().zip(angs(30.0, 0.0)) {
        assert!(
            (b - a - 30.0).abs() < 1e-3,
            "o Leaf Angle tem de somar 30 a esta folha: {a} -> {b}"
        );
    }
    // 3. O `Leaf Spread` abre-as UMAS EM RELAÇÃO ÀS OUTRAS, e dentro da faixa.
    let abertas = angs(0.0, 60.0);
    let desvios: Vec<f32> = base.iter().zip(&abertas).map(|(a, b)| b - a).collect();
    assert!(
        desvios.iter().all(|d| d.abs() <= 30.0 + 1e-3),
        "o sorteio saiu da faixa +-30: {desvios:?}"
    );
    let (mn, mx) = desvios
        .iter()
        .fold((f32::MAX, f32::MIN), |(a, b), d| (a.min(*d), b.max(*d)));
    assert!(
        mx - mn > 30.0,
        "as folhas tem de abrir UMAS EM RELACAO AS OUTRAS, e nao em bloco: {mn}..{mx}"
    );
    // ⚠️ **E é DETERMINÍSTICO** — duas derivações iguais dão os mesmos bits.
    for (a, b) in abertas.iter().zip(angs(0.0, 60.0)) {
        assert_eq!(a.to_bits(), b.to_bits(), "o sorteio tem de reproduzir");
    }
}

#[test]
#[ignore = "sonda"]
fn probe_demo_fern_depths() {
    let casos = [
        (
            "demo FERN",
            "A(s)",
            "A(s) -> F(s)[+B(s*0.55)]!A(s*0.87) ; B(s) -> F(s)[J][-B(s*0.72)]B(s*0.8)",
        ),
        ("preset Fern", PRESETS[1].axiom, PRESETS[1].rules),
        ("preset Tree", PRESETS[0].axiom, PRESETS[0].rules),
        ("preset Wild", PRESETS[4].axiom, PRESETS[4].rules),
        ("preset Sprig", PRESETS[7].axiom, PRESETS[7].rules),
    ];
    for (nome, axiom, rules) in casos {
        let s = probe_build(axiom, rules, 5.0, &[(param::LEAF_FIRST_LEVEL, 1.0)]);
        let (sym, depth) = (scal(&s, "sym"), scal(&s, "depth"));
        let mut hist: std::collections::BTreeMap<i64, usize> = Default::default();
        for i in 0..sym.len() {
            if crate::LEAF_SYMBOLS.contains(&(sym[i] as i32 as u8)) {
                *hist.entry(depth[i] as i64).or_default() += 1;
            }
        }
        let visiveis: usize = hist.iter().filter(|(d, _)| **d >= 3).map(|(_, n)| n).sum();
        let total: usize = hist.values().sum();
        println!(
            "{nome:<12} por profundidade {hist:?} — com First Level=3 sobram {visiveis} de {total}"
        );
    }
}

/// ⛔⛔⛔ **UM MOLDE NÃO PODE ESVAZIAR-SE COM O PRÓPRIO `First Level`.**
///
/// Report do Enio (2026-08-30, depois de eu shipar o knob): *"as folhas não aparecem"*. Um
/// default único de `3` — o que a árvore de fábrica pede — deixava o `Sprig` com **zero** de
/// `10` marcas, porque ali o `J` vive num ramo lateral de primeiro nível enquanto no `Tree` ele
/// vive no eixo. *A profundidade de encaixe significa coisas diferentes em gramáticas
/// diferentes.*
///
/// ⚠️ **E a mutação que repunha o `3` no `Sprig` SOBREVIVEU a toda a suíte** — não havia nada a
/// dizer que um molde com marcas tem de mostrar pelo menos uma. É esse o buraco que este gate
/// fecha, e ele mede o molde **com o número que o molde carrega**.
#[test]
fn no_preset_silences_its_own_leaves() {
    let mut com_marcas = 0;
    for p in PRESETS {
        let s = probe_build(
            p.axiom,
            p.rules,
            p.generations,
            &[
                (param::ANGLE, p.angle),
                (param::STEP, p.step),
                (param::LEAF_FIRST_LEVEL, p.leaf_first_level),
            ],
        );
        let sym = scal(&s, "sym");
        let grow = match s.get("mark_grow") {
            Some(Column::Scalar(v)) => v.clone(),
            _ => panic!("sem `mark_grow`"),
        };
        let marcas: Vec<usize> = (0..sym.len())
            .filter(|&i| crate::LEAF_SYMBOLS.contains(&(sym[i] as i32 as u8)))
            .collect();
        // ⛔⛔ **TODO molde emite marcas** — report do Enio (2026-08-30): *"vários dos
        // presets não produzem folhas"*. Eram QUATRO de oito (`Bush`, `Weed`, `Koch`,
        // `Dragon`), por uma decisão minha que a medição desmentiu: com o `[J]` no sítio certo
        // o `Bush` e o `Weed` dão `121` marcas bem distribuídas por `1..5`, e são plantas.
        //
        // ⚠️ **As curvas (`Koch`, `Dragon`) também a levam**, e o `First Level` delas é `1`
        // porque as marcas estão TODAS na profundidade `1`: uma curva não tem tronco, logo o
        // nível não tem por onde discriminar. *Quem escreve um nome numa curva quer decoração,
        // e a resposta honesta a «não produz folhas» não é explicar porquê — é produzir.*
        assert!(
            !marcas.is_empty(),
            "{}: o molde nao emite marca nenhuma, entao escrever um nome em «Leaf (J)» nao \
             planta nada e nada na tela o diz",
            p.label
        );
        com_marcas += 1;
        let vivas = marcas.iter().filter(|&&i| grow[i] > 0.0).count();
        assert!(
            vivas > 0,
            "{}: tem {} marca(s) e o proprio First Level ({}) apaga TODAS",
            p.label,
            marcas.len(),
            p.leaf_first_level
        );
    }
    // ⚠️ **O CONTROLE**: o laço tem de ter visto TODOS os moldes.
    assert_eq!(
        com_marcas,
        PRESETS.len(),
        "o gate tem de cobrir os {} moldes",
        PRESETS.len()
    );
}
