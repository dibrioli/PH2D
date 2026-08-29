//! Gates do **nó inteiro** — os defaults, o crescimento fraccionário, e a costura entre a
//! gramática e o painel.

use super::*;
use ph2d_nodegraph::attr::Column;

fn scal(s: &ph2d_nodegraph::attr::Stream, name: &str) -> Vec<f32> {
    match s.get(name) {
        Some(Column::Scalar(v)) => v.clone(),
        _ => panic!("coluna {name}"),
    }
}

fn height(s: &ph2d_nodegraph::attr::Stream) -> f32 {
    match s.get("P") {
        Some(Column::Vec2(v)) => v.iter().map(|p| p[1]).fold(f32::MIN, f32::max),
        _ => 0.0,
    }
}

fn default_tree(generations: f32) -> ph2d_nodegraph::attr::Stream {
    probe_build(DEFAULT_AXIOM, DEFAULT_RULES, generations, &[])
}

/// **O nó dropado da paleta desenha uma árvore** — não um ponto, não nada.
///
/// ⚠️ Um gerador cujo default é vazio treina o artista a achar que o nó está partido, e ele
/// nunca chega a escrever a primeira regra.
#[test]
fn the_factory_defaults_draw_a_tree_that_branches() {
    let s = default_tree(5.0);
    assert!(s.count() > 30, "so {} elementos", s.count());
    assert!(height(&s) > 1.0, "altura {}", height(&s));
    // Ramifica: há mais do que um elemento a pendurar-se no mesmo pai.
    let parent = scal(&s, "parent");
    let mut sorted = parent.clone();
    sorted.sort_by(f32::total_cmp);
    sorted.dedup();
    assert!(
        sorted.len() < parent.len(),
        "sem pais repetidos nao ha ramo nenhum"
    );
    // E as três colunas que só um L-System sabe estão lá.
    for c in ["depth", "gen", "sym"] {
        assert!(s.get(c).is_some(), "falta a coluna {c}");
    }
}

/// ⚠️ **O slider `Step` do painel está VIVO no axioma de fábrica.**
///
/// O default é `A(step)`, não `A(0.5)`: uma expressão vê os params do nó pelo nome. Um
/// literal ali deixaria o slider inerte no estado em que o artista o encontra — o knob morto
/// que o doc 90 caça, no primeiro segundo de vida do nó.
#[test]
fn the_step_slider_is_alive_in_the_factory_axiom() {
    let a = height(&default_tree(5.0));
    let b = height(&probe_build(
        DEFAULT_AXIOM,
        DEFAULT_RULES,
        5.0,
        &[(param::STEP, 1.0)],
    ));
    assert!(
        b > a * 1.8,
        "dobrar o Step tem de dobrar a planta: {a} contra {b}"
    );
}

/// ⭐⭐ **GERAÇÕES FRACCIONÁRIAS: a planta cresce CONTINUAMENTE, e não aos saltos.**
///
/// ⚠️⚠️ **A 1.ª redacção deste gate era VAZIA, e uma mutação provou-o.** Ela media que a
/// altura sobe monotonamente entre a geração 4 e a 5 e que a ponta bate com os inteiros —
/// e **um salto satisfaz as três coisas**: com o crescimento desligado, a planta muda de
/// tamanho de uma vez em `4,1` e fica parada até `5,0`. Monótono, sim; crescimento, não.
/// *A feature é a CONTINUIDADE, então é a continuidade que se mede.*
///
/// A régua é o maior degrau de um passo contra a subida TOTAL do intervalo: com dez passos,
/// um crescimento uniforme dá `0,10` a cada um, e um salto dá `1,00` num só.
#[test]
fn a_fractional_generation_grows_continuously_instead_of_jumping() {
    const STEPS: usize = 10;
    let hs: Vec<f32> = (0..=STEPS)
        .map(|k| height(&default_tree(4.0 + k as f32 / STEPS as f32)))
        .collect();
    for w in hs.windows(2) {
        assert!(w[1] >= w[0] - 1e-5, "a altura recuou: {hs:?}");
    }
    let rise = hs[STEPS] - hs[0];
    assert!(
        rise > 1e-3,
        "a geracao inteira tem de ser mais alta: {hs:?}"
    );
    let worst = hs.windows(2).map(|w| w[1] - w[0]).fold(f32::MIN, f32::max);
    // Uniforme seria `1/STEPS`; um salto seria `1`. A barra fica a meio caminho do
    // desastre, longe do ruído de qual ramo por acaso é o mais alto.
    assert!(
        worst < rise * 0.35,
        "um passo sozinho levou {:.0}% da subida — isto e' um SALTO, nao um crescimento: {hs:?}",
        worst / rise * 100.0
    );
    // ⚠️ E o fecho nas PONTAS: `4.0` é a geração 4 inteira, e `5.0` é a 5 inteira.
    assert!((hs[0] - height(&default_tree(4.0))).abs() < 1e-6);
    assert!((hs[STEPS] - height(&default_tree(5.0))).abs() < 1e-4);
}

/// ⚠️ **Só a geração MAIS NOVA cresce.** O resto da planta fica exactamente onde estava —
/// senão o slider encolheria a árvore inteira em vez de fazer o rebento aparecer.
#[test]
fn only_the_youngest_generation_grows_and_the_rest_stands_still() {
    let a = default_tree(4.0);
    let b = default_tree(4.5);
    let (pa, pb) = (
        match a.get("P") {
            Some(Column::Vec2(v)) => v.clone(),
            _ => panic!(),
        },
        match b.get("P") {
            Some(Column::Vec2(v)) => v.clone(),
            _ => panic!(),
        },
    );
    let gb = scal(&b, "gen");
    let old = gb.iter().filter(|g| **g < 5.0).count();
    assert_eq!(
        old,
        pa.len(),
        "a parte velha tem de ser a arvore de 4 geracoes inteira"
    );
    // ⚠️ **Não se compara por ÍNDICE.** A travessia é em profundidade, então um elemento novo
    // nasce NO MEIO da lista e empurra os seguintes — a 1.ª redacção deste gate emparelhava
    // `i` com `i` e acusava a árvore inteira de se mexer. O que se preserva é a ORDEM dos
    // velhos, que é o que a filtragem por geração recupera.
    let old_now: Vec<[f32; 2]> = pb
        .iter()
        .zip(gb.iter())
        .filter(|(_, g)| **g < 5.0)
        .map(|(p, _)| *p)
        .collect();
    for (i, (p, q)) in pa.iter().zip(old_now.iter()).enumerate() {
        assert!(
            (p[0] - q[0]).abs() < 1e-5 && (p[1] - q[1]).abs() < 1e-5,
            "o elemento velho {i} mexeu-se: {p:?} -> {q:?}"
        );
    }
}

/// O plano de gerações, incluindo o lixo que um documento carregado pode trazer.
#[test]
fn the_generation_plan_is_total() {
    assert_eq!(generation_plan(0.0), (0, (0, 1.0)));
    assert_eq!(generation_plan(3.0), (3, (3, 1.0)));
    assert_eq!(generation_plan(f32::NAN), (0, (0, 1.0)));
    assert_eq!(generation_plan(-2.0), (0, (0, 1.0)));
    let (g, (y, f)) = generation_plan(4.25);
    assert_eq!((g, y), (5, 5));
    assert!((f - 0.25).abs() < 1e-6);
}

/// ⚠️ **Quando o orçamento satura, a geração que sobrou é INTEIRA** — não uma fracção de uma
/// geração que nunca chegou a existir.
#[test]
fn a_saturated_derivation_does_not_shrink_the_generation_it_did_finish() {
    // `F -> FF` duplica: 32 gerações não cabem em `MAX_MODULES` de maneira nenhuma.
    //
    // ⚠️ **A geração em que ela pára é DERIVADA do tecto, nunca escrita à mão** — a 1.ª
    // redacção cravou `17` e ficou vermelha no dia em que a medição moveu o tecto para
    // `262 144`. Um número copiado de outra constante é a mesma constante escrita duas vezes.
    let last_whole = crate::MAX_MODULES.ilog2();
    let saturated = probe_build("F", "F -> FF", 31.5, &[]);
    let whole = probe_build("F", "F -> FF", last_whole as f32, &[]);
    assert_eq!(
        saturated.count(),
        whole.count(),
        "a derivacao para em 2^{last_whole} modulos (mais a raiz que a tartaruga planta)"
    );
    assert_eq!(saturated.count(), crate::MAX_MODULES + 1);
}

/// Todo param declarado tem um hint, e todo hint nomeia um param declarado **ou** um dos
/// dois text params — o censo local que impede um knob inalcançável no painel.
#[test]
fn every_declared_param_has_a_hint_and_every_hint_has_a_home() {
    for p in MANIFEST.params {
        assert!(
            PARAM_HINTS.iter().any(|h| h.param == p.name),
            "o param {} nao tem hint — ele existe no modelo e o painel nao o mostra",
            p.name
        );
    }
    for h in PARAM_HINTS {
        let declared = MANIFEST.params.iter().any(|p| p.name == h.param);
        let text = h.param == AXIOM_PARAM || h.param == RULES_PARAM;
        assert!(declared || text, "o hint {} nao tem param nenhum", h.param);
    }
    assert!(
        MANIFEST.params.len() + 2 <= ph2d_panel_motion_params_row_cap(),
        "o no declara mais linhas do que o painel pinta"
    );
}

/// O tecto de linhas do painel de params, repetido aqui como NÚMERO e não como dependência.
///
/// ⚠️ Este crate é uma folha e não pode depender do painel; o censo de verdade
/// (`the_panel_shows_every_param_of_every_node`) corre na shell e mede TODOS os nós. Este é
/// só a rede local — se ele e o painel discordarem, é o da shell que manda.
const fn ph2d_panel_motion_params_row_cap() -> usize {
    24
}

/// Um `Generations` negativo, `NaN` ou enorme não pode custar nada nem emitir lixo.
#[test]
fn untrusted_generations_never_costs_and_never_empties() {
    for g in [-3.0, f32::NAN, f32::INFINITY, 0.0] {
        let s = default_tree(g);
        assert!(s.count() >= 1, "sempre pelo menos a raiz (g = {g})");
        assert!(
            s.count() < 4,
            "e nada de derivar com g = {g}: {}",
            s.count()
        );
    }
}

/// Um axioma ou umas regras VAZIAS caem no default — um text param apagado não pode apagar
/// a planta.
#[test]
fn an_emptied_text_param_falls_back_to_the_factory_grammar() {
    let a = probe_build("", "", 5.0, &[]);
    let b = default_tree(5.0);
    assert_eq!(a.count(), b.count());
}

/// ⭐⭐⭐ **SUBIR AS GERAÇÕES NUNCA ENCURTA A PLANTA** — a lei que o report do Enio enuncia,
/// para TODA família de gramática.
///
/// ⚠️⚠️ **Report de 2026-08-28: *"a cada ramo que vai nascer tudo se apaga e aparece de vez"***.
/// Medido: com o arbusto clássico do ABOP a altura caía a 25 % em cada cruzamento de geração e
/// voltava a subir — `13,5 → 10,1 → 40,5 → 30,4`.
///
/// O mecanismo: aquela regra **reescreve o próprio símbolo que desenha**, então ao fim de cada
/// passagem TODO módulo de desenho é da geração nova. «O rebento» era a planta inteira, e
/// escalá-lo escalava tudo. *A lei estava certa e o conjunto a que ela se aplica estava vazio
/// de contraste.*
///
/// ⚠️ **As três famílias estão aqui de propósito**, e a do meio é a que falhava: um gate só
/// sobre a gramática de fábrica (que tem `F` terminal) fica verde para sempre — foi assim que
/// isto shipou. *Uma fixtura que não contém o fenómeno aprova a cura de qualquer coisa.*
#[test]
fn raising_the_generations_never_shortens_the_plant() {
    for (name, axiom, rules) in [
        ("crescimento (F terminal)", DEFAULT_AXIOM, DEFAULT_RULES),
        ("refinamento (o F reescreve-se)", "F", "F -> F[+F]F[-F]F"),
        ("duplicacao pura", "F", "F -> FF"),
    ] {
        let mut prev = f32::MIN;
        let mut heights = Vec::new();
        for k in 0..=32 {
            let g = 2.0 + k as f32 * 0.125;
            let h = height(&probe_build(axiom, rules, g, &[]));
            heights.push((g, h));
            // ⚠️ **A tolerância é RELATIVA à planta, e a razão é a âncora `1/spread`.** Ela
            // põe a geração nova por cima da anterior a menos do erro de `f32` acumulado numa
            // cadeia de milhares de módulos — um `1e-4` absoluto media isso em vez do recuo.
            // ⛔ E ela não pode ser folgada: `2 %` da altura ainda apanha o pisca-pisca de
            // 28/08 (que era **25 %**) e a família de saltos que esta wave curou (10-31 %).
            assert!(
                h >= prev * 0.98 - 1e-4,
                "{name}: a altura RECUOU de {prev} para {h} em g = {g} — a planta apaga-se e \
                 volta. Percurso: {heights:?}"
            );
            prev = h;
        }
        // E o CONTROLE: a varredura tem de ter feito a planta crescer de facto, senão a
        // monotonia acima seria a de uma linha constante.
        assert!(
            heights.last().unwrap().1 > heights[0].1 * 1.5,
            "{name}: a fixtura nao cresceu o bastante para a monotonia querer dizer algo: \
             {heights:?}"
        );
    }
}

/// ⚠️ **E a fracção continua VIVA onde ela tem sujeito** — a metade oposta.
///
/// Sem isto, a cura acima passaria com a fracção desligada em toda a parte: a planta saltaria
/// entre inteiros sempre, e o crescimento contínuo — que é a razão de existir deste nó —
/// morreria em silêncio.
#[test]
fn the_fraction_is_still_alive_where_something_old_survives() {
    let a = height(&default_tree(4.0));
    let b = height(&default_tree(4.5));
    let c = height(&default_tree(5.0));
    assert!(
        b > a + 1e-4 && b < c - 1e-4,
        "com `F` terminal a meia geracao tem de ficar ENTRE as duas inteiras: {a} / {b} / {c}"
    );
    // ⭐⭐⭐ **E NA GRAMÁTICA DE REFINAMENTO ELA TAMBÉM ESTÁ VIVA, E É LINEAR** — 2026-08-29.
    //
    // ⚠️⚠️ **Esta metade já afirmou as DUAS coisas contrárias, e as duas eram honestas no dia
    // em que foram escritas.** Primeiro `assert_eq!` (*"a fracção não tem sujeito"*), enquanto
    // não se sabia de onde ela devia partir. Depois `assert_eq!` outra vez, quando o dono do
    // produto previu que não ficaria bom. Agora a interpolação: ele smokou, retirou a
    // previsão, e apontou o defeito que restava — *"não é linear"*.
    //
    // A régua aqui é a que a queixa dele pede: a **DERIVADA**. Não basta a meia geração ficar
    // entre as duas inteiras (isso já era verdade quando a curva subia a `3,05` e voltava a
    // `3,00`) — o passo tem de ser CONSTANTE ao longo da travessia.
    let r = |g| height(&probe_build("F", "F -> F[+F]F[-F]F", g, &[]));
    const N: usize = 16;
    let hs: Vec<f32> = (0..=N).map(|k| r(4.0 + k as f32 / N as f32)).collect();
    let d: Vec<f32> = hs.windows(2).map(|w| w[1] - w[0]).collect();
    let (lo, hi) = (
        d.iter().copied().fold(f32::MAX, f32::min),
        d.iter().copied().fold(f32::MIN, f32::max),
    );
    let mean = d.iter().sum::<f32>() / d.len() as f32;
    assert!(mean > 1e-4, "a planta tem de CRESCER na travessia: {hs:?}");
    assert!(lo > 0.0, "nenhum passo pode ANDAR PARA TRAS: {d:?}");
    // ⚠️ **A ondulação, que é a queixa dele em número.** Antes da normalização a Koch dava
    // `2,3×` e o Dragon `4,2×`; depois, `0,0×`. A barra em `0,25×` é folgada face ao que a
    // medição dá e aperta muito face ao que ela apanhava.
    let ripple = (hi - lo) / mean;
    assert!(
        ripple < 0.25,
        "a rampa ondula {ripple:.2}x — o crescimento nao e' linear, que foi o report do Enio \
         de 2026-08-29. Passos: {d:?}"
    );
}

// ─────────────────────────────────────────────────────────────────────────────────────────
// O MODO GUIADO — a metade que mede a PLANTA (a que mede o TEXTO vive no `shape_tests.rs`).
// ─────────────────────────────────────────────────────────────────────────────────────────

/// A largura do que a planta ocupa em `x` — a régua do LEQUE.
fn span_x(s: &ph2d_nodegraph::attr::Stream) -> f32 {
    match s.get("P") {
        Some(Column::Vec2(v)) => {
            let lo = v.iter().map(|p| p[0]).fold(f32::MAX, f32::min);
            let hi = v.iter().map(|p| p[0]).fold(f32::MIN, f32::max);
            hi - lo
        }
        _ => 0.0,
    }
}

/// Uma planta GUIADA — o texto passa a ser irrelevante, e é isso que o gate abaixo prova.
fn guided(overrides: &[(&str, f32)]) -> ph2d_nodegraph::attr::Stream {
    guided_at(5.0, overrides)
}

/// A mesma, com o número de gerações à escolha de quem mede.
fn guided_at(generations: f32, overrides: &[(&str, f32)]) -> ph2d_nodegraph::attr::Stream {
    let mut ov = vec![(param::MODE, MODE_GUIDED as f32)];
    ov.extend_from_slice(overrides);
    probe_build(DEFAULT_AXIOM, DEFAULT_RULES, generations, &ov)
}

/// ⭐⭐⭐ **NO GUIADO O TEXTO NÃO É LIDO** — a afirmação inteira do modo.
///
/// ⚠️ *Um controle que não faz nada não é pintado*, e o inverso é pior: um texto que o nó
/// lesse a meias faria o painel esconder a caixa **enquanto ela ainda mandava**. O gate
/// alimenta o nó com uma gramática que não se parece com nada e exige o MESMO stream.
///
/// ⚠️ **E o CONTROLE é a outra metade**: a mesma gramática de lixo em `Grammar` tem de dar
/// outra coisa. Sem ele, um `build` que ignorasse o texto nos DOIS modos passaria isto.
#[test]
fn the_guided_mode_does_not_read_the_authored_text_and_the_grammar_mode_does() {
    let a = guided(&[]);
    let b = probe_build(
        "Q",
        "Q -> QQQQ[+Q][-Q]",
        5.0,
        &[(param::MODE, MODE_GUIDED as f32)],
    );
    assert_eq!(
        a.count(),
        b.count(),
        "o guiado leu o texto: {} contra {}",
        a.count(),
        b.count()
    );
    assert_eq!(
        height(&a).to_bits(),
        height(&b).to_bits(),
        "o guiado leu o texto"
    );

    // O CONTROLE: em `Grammar` aquela gramática tem de produzir outra planta.
    let c = probe_build("Q", "Q -> QQQQ[+Q][-Q]", 5.0, &[]);
    assert_ne!(
        a.count(),
        c.count(),
        "a gramatica de lixo deu a MESMA planta que os sliders — o texto nao esta a ser lido \
         em modo nenhum, e as duas metades deste gate seriam vacuas"
    );
}

/// ⭐ **O nó recém-dropado desenha uma árvore** — no modo em que ele de facto abre.
///
/// ⚠️ É o gate irmão de `the_factory_defaults_draw_a_tree_that_branches`, e ele passou a ser
/// necessário no dia em que o default do `Mode` deixou de ser a gramática: aquele mede o
/// caminho que o artista **já não** encontra primeiro.
#[test]
fn the_node_as_dropped_from_the_palette_draws_a_branching_tree() {
    let s = guided(&[]);
    assert!(s.count() > 30, "so {} elementos", s.count());
    assert!(height(&s) > 1.0, "altura {}", height(&s));
    let parent = scal(&s, "parent");
    let mut sorted = parent.clone();
    sorted.sort_by(f32::total_cmp);
    sorted.dedup();
    assert!(
        sorted.len() < parent.len(),
        "sem pais repetidos nao ha ramo"
    );
}

/// ⭐⭐ **OS QUATRO SLIDERS DE FORMA MEXEM NA PLANTA** — um por um, cada um contra o default.
///
/// ⚠️ **É o gate que a pergunta do Enio pede**: de que serve trocar duas caixas de texto por
/// quatro sliders se algum deles for inerte? A régua de cada um é a grandeza que ele PROMETE
/// mover, e não «alguma coisa mudou» — um `assert_ne` sobre a contagem passaria com o slider
/// ligado ao knob errado.
#[test]
fn every_shape_slider_moves_the_thing_its_label_promises() {
    let base = guided(&[]);

    // `Branches`: mais rebentos, mais elementos — e o leque abre.
    let three = guided(&[(param::BRANCHES, 3.0)]);
    assert!(
        three.count() > base.count(),
        "Branches: {} contra {}",
        three.count(),
        base.count()
    );

    // `Angle`: o leque é mais LARGO.
    //
    // ⚠️⚠️ **A medição tem de ser numa planta RASA, e a 1.ª redacção deste gate reprovou
    // sobre produto correto por não o ser.** A `5` gerações o leque SATURA: a `60°` por
    // nível, ao terceiro o ramo já aponta para baixo e volta a fechar a largura — medido,
    // `2,68` contra `2,35`, uns miseráveis 14 % para um ângulo 2,4× maior. Numa planta de
    // **duas** gerações há exactamente uma bifurcação, e aí a largura é o que o slider diz:
    // `sin 60° / sin 25° = 2,05×`. *Uma fixtura em que o fenómeno satura não distingue duas
    // respostas — a régua não estava errada, a planta é que era funda demais.*
    let fan = |a: f32| span_x(&guided_at(2.0, &[(param::ANGLE, a)]));
    assert!(
        fan(60.0) > fan(25.0) * 1.8,
        "Angle: leque {} contra {}",
        fan(60.0),
        fan(25.0)
    );

    // `Trunk Segments`: o mesmo número de bifurcações, mas a planta é MAIS ALTA.
    let tall = guided(&[(param::SEGMENTS, 3.0)]);
    assert!(
        height(&tall) > height(&base) * 1.5,
        "Trunk Segments: altura {} contra {}",
        height(&tall),
        height(&base)
    );

    // `Bend`: a planta perde a simetria de espelho — o centro de massa sai do eixo.
    let cx = |s: &ph2d_nodegraph::attr::Stream| match s.get("P") {
        Some(Column::Vec2(v)) => v.iter().map(|p| p[0]).sum::<f32>() / v.len() as f32,
        _ => 0.0,
    };
    let curved = guided(&[(param::BEND, 20.0)]);
    assert!(
        cx(&curved).abs() > cx(&base).abs() + 0.1,
        "Bend: centro em {} contra {}",
        cx(&curved),
        cx(&base)
    );

    // `Variation`: a MESMA forma com sementes diferentes deixa de ser a mesma planta.
    let v1 = guided(&[(param::VARIATION, 0.6), (param::SEED, 1.0)]);
    let v9 = guided(&[(param::VARIATION, 0.6), (param::SEED, 9.0)]);
    assert_ne!(
        height(&v1).to_bits(),
        height(&v9).to_bits(),
        "Variation: as duas sementes deram plantas gemeas"
    );
    // ⚠️ E o CONTROLE: SEM variação as duas sementes têm de dar a MESMA planta, senão o
    // `assert_ne` acima estaria a medir a semente e não o slider.
    let s1 = guided(&[(param::SEED, 1.0)]);
    let s9 = guided(&[(param::SEED, 9.0)]);
    assert_eq!(
        height(&s1).to_bits(),
        height(&s9).to_bits(),
        "sem Variation a semente nao pode mudar nada — a gramatica derivada e deterministica"
    );
}

/// ⚠️ **O CRESCIMENTO FRACCIONÁRIO CONTINUA A VALER NO GUIADO.**
///
/// A razão de existir deste nó é animar o `Generations`, e a gramática derivada não pode
/// perder isso — ela reescreve `A`, que **não** é o símbolo que desenha (`F`), então a
/// fracção tem sujeito. *Uma gramática gerada que não crescesse seria a feature principal
/// perdida no default.*
#[test]
fn the_derived_grammar_still_grows_continuously_with_a_fractional_generation() {
    let hs: Vec<f32> = (0..=12)
        .map(|k| {
            let g = 3.0 + k as f32 * 0.25;
            height(&probe_build(
                DEFAULT_AXIOM,
                DEFAULT_RULES,
                g,
                &[(param::MODE, MODE_GUIDED as f32)],
            ))
        })
        .collect();
    let rise = hs[hs.len() - 1] - hs[0];
    assert!(rise > 0.05, "a planta guiada nao cresceu: {hs:?}");
    // ⚠️ **Continuidade e não monotonia** — a mesma correcção que a mutação ML7 forçou: um
    // SALTO satisfaz «sobe sempre», e é exactamente o que a fracção existe para não ter.
    let worst = hs
        .windows(2)
        .map(|w| (w[1] - w[0]).abs())
        .fold(0.0f32, f32::max);
    assert!(
        worst < 0.35 * rise,
        "um passo de {worst} contra uma subida total de {rise} — isto e um salto, nao um \
         crescimento: {hs:?}"
    );
}
