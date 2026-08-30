//! ⭐⭐⭐ **O CRESCIMENTO SÃO DUAS LEIS, E CADA FAMÍLIA DE GRAMÁTICA PRECISA DE UMA.**
//!
//! # O caminho até aqui, porque ele é a lição
//!
//! 1. **Enio, 2026-08-29:** *"porque vários presets não têm crescimento suave, mas em saltos?"*
//!    Medido: quatro dos oito saltavam (`53–137 %` de pior passo contra `5–8 %`).
//! 2. **Pesquisa:** o L-System SOP do Houdini tem **dois** interruptores — *Continuous length*
//!    e *Continuous angles* — e eu tinha construído só o primeiro.
//! 3. **Enio:** *"faça os demais"* → construí, e ele previu *"os que vc tentou corrigir não
//!    ficarão bons"*. Shipei **desligado**, com a medição que concordava com ele.
//! 4. **Ele SMOKOU e retirou a previsão:** *"Melhorou muito. Mas o crescimento dos que não
//!    cresciam suavemente não é linear."*
//! 5. Medi a **DERIVADA** em vez do pior passo, e ele tinha razão pela segunda vez — e a causa
//!    não era onde eu supunha.
//!
//! # A partição, medida (`examples/preset_report.rs`, `examples/probe_curve.rs`)
//!
//! | família | razão de expansão por geração | como anima |
//! |---|---|---|
//! | Tree · Fern · Wild · Sprig | `1,63 → 1,06` (**converge para 1**) | a PONTA estica de zero |
//! | Bush · Weed | `3,00` · `2,03` (**constante**) | a figura REFINA-SE |
//! | Koch · Dragon | `3,00` · `~1,41`, e são CURVAS | idem |
//!
//! # A causa da não-linearidade, e ela não era a que eu supunha
//!
//! **Bush e Weed já eram perfeitamente lineares** (ondulação `0,0×`). Quem não era eram as
//! CURVAS: a Koch subia a `3,05` e **voltava** a `3,00` (`2,3×`), o Dragon subia a `1,62` e
//! voltava a `1,51` (`4,2×`).
//!
//! ⚠️ **As duas rampas brigavam.** O comprimento crescia linearmente enquanto as dobras, ao
//! abrir, **encurtavam** a projecção — a `90°` uma zig-zag ocupa menos do que a mesma linha
//! meio aberta. O produto tem um pico no meio, e o último quinto do slider andava **para trás**.
//! *Andar para trás é o que se vê da cadeira.*
//!
//! ⇒ A cura não é uma constante: é **normalizar pelo que se mede**. A cada instante mede-se o
//! tamanho com as dobras onde elas estão, e escolhe-se o comprimento que põe a figura na rampa
//! recta entre a geração anterior e a nova inteira. Ondulação depois: **`0,0×` nas quatro**.
//!
//! ⛔ **Duas curas ANTERIORES foram medidas e não bastam** — não as reconstrua:
//! - o **`Step Scale`** sozinho (o *Step Size Scale* do Houdini) deixa a figura do mesmo
//!   tamanho — `1/3` é exactamente a razão de Bush e Koch — e o melhor que uma varredura de
//!   oito valores alcança é `105 %` de pior passo. *Estável em tamanho ≠ contínuo na forma.*
//! - a **âncora como CONSTANTE** (`1/spread`, contada da gramática) está errada por
//!   construção: a `F -> F[+F]F[-F]F` põe **5** módulos por cada um e cresce **3,00×** (dois
//!   estão dentro de parênteses e não estendem o caminho), e a `F -> F+F-F-F+F` põe 5 sem
//!   parênteses e cresce `3,00×` na mesma, porque as viragens a dobram. *A razão é geométrica.*

use ph2d_node_source_lsystem as ls;
use ph2d_nodegraph::attr::{Column, Stream};

/// Quantas direções o OBSERVADOR amostra. ⚠️ **`64`, e o produto usa `16`** — de propósito:
/// um oráculo que partilhasse o `K` não veria uma mudança de `K`.
const OBSERVER_DIRECTIONS: usize = 64;

/// ⭐⭐⭐ **O TAMANHO, POR UM OBSERVADOR INVARIANTE À ROTAÇÃO — e as duas réguas anteriores
/// estavam na MESMA família do defeito.**
///
/// ⚠️⚠️ **Report do Enio, 2026-08-30** (*"em dragon enquanto cresce parece piscar"*): a lei do
/// crescimento normalizava `max(w, h)`, que não é invariante à rotação, e a curva do dragão
/// **roda `45°` por geração**. Quando a caixa trocava de lado longo o tamanho verdadeiro
/// **estagnava e depois arrancava** — menor passo do arrasto a `4,5 %` do passo médio.
///
/// ⛔ **E este gate estava VERDE.** A janela é metade da razão (ele amostrava UMA travessia de
/// geração e o recuo do Dragon vivia na anterior — ver [`whole_slider`]); a outra metade é
/// esta régua, e ⚠️ **a 1.ª redacção desta nota dizia que a diagonal era CEGA, e a mutação
/// refutou-a**: ela é **falsa acusadora**.
///
/// Matriz medida em 2026-08-30 (produto × observador × janela):
///
/// | produto | observador | janela | `no_refiner_stalls` |
/// |---|---|---|---|
/// | curado | Cauchy | 3 gerações | ✅ Dragon `55,2 %` |
/// | régua `max(w,h)` | Cauchy | 3 gerações | ❌ Dragon `5,2 %` |
/// | régua `max(w,h)` | Cauchy | 1 travessia | ❌ |
/// | **curado** | **diagonal** | 3 gerações | ❌ **Dragon `25,6 %` — FALSO** |
///
/// ⇒ *uma figura que roda tem a caixa a respirar sem mudar de tamanho*, então qualquer régua
/// alinhada aos eixos lê a rotação como uma paragem — e condena o produto certo. A diagonal
/// cura o joelho de `max(w, h)` (a correcção de 2026-08-29, que continua certa) e **não** cura
/// a rotação: a AABB de um quadrado varia `41 %` ao rodá-lo `45°`.
///
/// ⇒ o observador é a **largura média de Cauchy**: `média_u(max⟨P,u⟩ − min⟨P,u⟩)`. Sem
/// centroide — as duas réguas invariantes que o têm foram medidas e rejeitadas (ver
/// `turtle::mean_width`) — e com trigonometria REAL e `K = 64` contra os `16` do produto.
///
/// ⛔⛔ **E é preciso ser honesto sobre o que isso NÃO compra**, porque a 1.ª redacção desta
/// nota exagerou-o (*«para ser um oráculo e não uma segunda cópia da mesma construção»*) e a
/// auditoria adversarial de 2026-08-30 corrigiu-a: isto é a MESMA lei com mais resolução, não
/// uma segunda opinião — a largura média converge em `O(1/K²)`, então `K = 64` é o `K = 16` com
/// o erro dividido por 16. ⚠️ **A independência DESCEU nesta mudança**: no `HEAD` o observador
/// era a diagonal da caixa, uma construção genuinamente diferente do `max(w, h)` do produto.
///
/// ⇒ o que impede este ficheiro de se auto-aprovar não é o observador: são os gates que
/// afirmam a LEI na porta onde ela vive (`turtle_tests::the_ruler_is_the_mean_width_…`) e a
/// POSE em vez do tamanho (`turtle_tests::the_newest_generation_opens_its_folds_…`) — os dois
/// nasceram daquela auditoria, e sem eles duas mutações sobreviviam a 85 testes.
fn size(s: &Stream) -> f32 {
    let Some(Column::Vec2(v)) = s.get("P") else {
        return 0.0;
    };
    if v.is_empty() {
        return 0.0;
    }
    let mut total = 0.0f64;
    for k in 0..OBSERVER_DIRECTIONS {
        let a = std::f32::consts::PI * k as f32 / OBSERVER_DIRECTIONS as f32;
        let (c, sn) = (a.cos(), a.sin());
        let mut lo = f32::MAX;
        let mut hi = f32::MIN;
        for q in v {
            let t = q[0] * c + q[1] * sn;
            lo = lo.min(t);
            hi = hi.max(t);
        }
        total += f64::from(hi - lo);
    }
    (total / OBSERVER_DIRECTIONS as f64) as f32
}

fn at(p: &ls::Preset, g: f32, over: &[(&str, f32)]) -> f32 {
    let mut o: Vec<(&str, f32)> = vec![
        (ls::param::MODE, ls::MODE_GRAMMAR as f32),
        (ls::param::ANGLE, p.angle),
        (ls::param::STEP, p.step),
        (ls::param::WIDTH, p.width),
    ];
    o.extend_from_slice(over);
    size(&ls::probe_build(p.axiom, p.rules, g, &o))
}

/// A travessia da penúltima geração para a última, amostrada fino: `(tamanhos, passos)`.
fn crossing(p: &ls::Preset, over: &[(&str, f32)]) -> (Vec<f32>, Vec<f32>) {
    const N: usize = 24;
    let g0 = (p.generations - 1.0).max(2.0).floor();
    let hs: Vec<f32> = (0..=N)
        .map(|k| at(p, g0 + k as f32 / N as f32, over))
        .collect();
    let d = hs.windows(2).map(|w| w[1] - w[0]).collect();
    (hs, d)
}

/// ⭐⭐⭐ **O SLIDER INTEIRO, e não uma travessia** — a segunda cegueira do report de
/// 2026-08-30.
///
/// [`crossing`] amostra a penúltima→última geração, e o recuo do Dragon vivia em `10,60`–`10,76`
/// com o slider a chegar a `12`: **o gate media um intervalo em que o defeito não estava**.
/// *Uma janela de amostragem é uma afirmação sobre onde o defeito pode viver.*
///
/// ⚠️ As **três** últimas gerações, ao passo do slider (`0,02`), que é o que a mão de facto
/// atravessa. As anteriores desenham figuras que o molde não enquadra.
fn whole_slider(p: &ls::Preset) -> (Vec<f32>, Vec<f32>) {
    let (g0, _) = window(p);
    let n = ((p.generations - g0) / 0.02).round() as usize;
    let hs: Vec<f32> = (0..=n).map(|k| at(p, g0 + k as f32 * 0.02, &[])).collect();
    let d = hs.windows(2).map(|w| w[1] - w[0]).collect();
    (hs, d)
}

/// `(primeira geração do arrasto, quantas gerações ele atravessa)`.
fn window(p: &ls::Preset) -> (f32, usize) {
    let g0 = (p.generations - 3.0).max(1.0);
    (g0, (p.generations - g0).round() as usize)
}

/// A ondulação da rampa: `(maior passo − menor passo) / passo médio`. `0` = recta.
fn ripple(d: &[f32]) -> f32 {
    let mean = d.iter().sum::<f32>() / d.len() as f32;
    if mean.abs() < 1e-9 {
        return f32::MAX;
    }
    let lo = d.iter().copied().fold(f32::MAX, f32::min);
    let hi = d.iter().copied().fold(f32::MIN, f32::max);
    (hi - lo) / mean.abs()
}

/// ⭐⭐⭐ **NENHUM MOLDE ANDA PARA TRÁS** — o defeito que o Enio de facto viu.
///
/// ⚠️ **A régua é a DERIVADA, e é isso que a queixa dele nomeia.** Uma barra sobre o pior
/// passo não a apanharia: a Koch tinha `17 %` de pior passo (perfeitamente aceitável) **e**
/// encolhia no último quinto do slider. *Um salto e um recuo são defeitos diferentes, e só o
/// segundo se lê como «não é linear».*
#[test]
fn no_preset_ever_shrinks_while_the_generations_rise() {
    for p in ls::PRESETS {
        // ⚠️ O SLIDER INTEIRO — ver [`whole_slider`]: com a travessia só, este gate ficou
        // verde sobre o recuo que o Enio viu em 2026-08-30.
        let (hs, d) = whole_slider(p);
        assert!(
            hs[hs.len() - 1] > hs[0] * 1.005,
            "{}: a travessia tem de CRESCER: {hs:?}",
            p.label
        );
        let lo = d.iter().copied().fold(f32::MAX, f32::min);
        assert!(
            lo > -1e-4,
            "{}: um passo ANDOU PARA TRAS ({lo}) — a figura encolhe no fim do slider: {d:?}",
            p.label
        );
    }
}

/// ⭐⭐⭐ **E OS QUE REFINAM NÃO ONDULAM MAIS DO QUE OS QUE O DONO JÁ ACEITAVA.**
///
/// ⚠️⚠️ **A barra é COMPARATIVA de propósito, e a medição obrigou-me a isso.** Eu tinha
/// escrito `ripple < 0.25` para os oito e o gate reprovou no **Sprig — `1,5×`, o pior dos
/// oito** —, que é um dos quatro de que o Enio **nunca se queixou**. *A minha noção de «linear»
/// não é a dele; a dele é a família que ele aceitou.*
///
/// ⇒ A referência é o pior dos que crescem pela ponta. Antes da cura os que refinam davam
/// `2,3×` (Koch) e `4,2×` (Dragon) contra `1,5×`; depois dão `0,0×`–`0,7×`. A barra
/// discrimina, e não foi escolhida para passar.
///
/// ⚠️ E as duas famílias saem da razão de expansão **medida**, nunca de uma lista de nomes.
#[test]
fn the_refiners_ripple_no_worse_than_the_tip_growers_the_owner_accepted() {
    let mut tips: Vec<(&str, f32)> = Vec::new();
    let mut refiners: Vec<(&str, f32)> = Vec::new();
    for p in ls::PRESETS {
        let g0 = (p.generations - 1.0).max(2.0).floor();
        let growth = at(p, g0 + 1.0, &[]) / at(p, g0, &[]).max(1e-6);
        let r = ripple(&crossing(p, &[]).1);
        // Quem REFINA: a figura cresce por um factor constante > 1,5 a cada geração.
        if growth > 1.5 {
            &mut refiners
        } else {
            &mut tips
        }
        .push((p.label, r));
    }
    // ⚠️ **O CONTROLE**: as duas famílias têm de estar POVOADAS. Sem ele, o gate passaria num
    // mundo onde todos os moldes são da mesma espécie — e a lei da outra estaria morta.
    assert!(
        tips.len() >= 3 && refiners.len() >= 3,
        "as duas familias tem de existir no corpus: {tips:?} / {refiners:?}"
    );
    let worst_tip = tips.iter().map(|(_, r)| *r).fold(0.0f32, f32::max);
    for (label, r) in &refiners {
        assert!(
            *r <= worst_tip,
            "{label} ondula {r:.2}x, pior que o {:.2}x do pior dos que crescem pela ponta \
             ({tips:?}) — o report do Enio de 2026-08-29 voltou",
            worst_tip
        );
    }
}

/// ⭐⭐ **A ÂNCORA É O NÚMERO CERTO, E NÃO SÓ «UM NÚMERO QUE MELHORA».**
///
/// ⚠️⚠️ **Este gate nasceu de uma mutação que SOBREVIVEU.** Trocar a pose de partida de
/// `frac = 0` (viragens fechadas) por `frac = 1` (abertas) muda a âncora de `1/5` para `1/3`,
/// e um gate que só perguntava *«melhorou 2×?»* ficou verde com as duas. *Uma barra de
/// «melhorou» não distingue duas âncoras que ambas melhoram.*
///
/// A régua é o significado da âncora: com ela aplicada, a geração nova em `frac → 0` tem de ter
/// **o tamanho da anterior**. É uma identidade, não uma desigualdade.
#[test]
fn the_anchor_puts_the_new_generation_exactly_on_top_of_the_previous_one() {
    for (name, axiom, rules, angle, exact) in [
        (
            "arbusto (com ramos)",
            "F",
            "F -> F[+F]F[-F]F",
            25.7,
            Some(1.0 / 3.0),
        ),
        ("koch (curva pura)", "F", "F -> F+F-F-F+F", 90.0, Some(0.2)),
        ("duplicacao", "F", "F -> FF", 25.0, Some(0.5)),
        (
            "dragao (roda 45 graus)",
            "F",
            "F -> F+G ; G -> F-G",
            90.0,
            Some(0.5),
        ),
        // ⭐⭐ **A fixtura que NÃO é auto-semelhante** — a auditoria de 2026-08-30 mostrou que
        // as três de cima são exactamente aquelas em que a régua não pode importar: numa
        // gramática auto-semelhante achatar a geração nova devolve a anterior **a escala**, e
        // duas réguas homogéneas de grau 1 dão o MESMO número. O Weed não é cópia a escala e a
        // âncora dele **varia com a geração** (`0,401 → 0,451 → 0,476 → 0,488`), o que a torna
        // a única do corpus que uma troca de régua move.
        (
            "weed (nao auto-semelhante)",
            "X",
            "X -> F[+X]F[-X]+X ; F -> FF",
            20.0,
            None,
        ),
    ] {
        let anchor = ls::probe_anchor(axiom, rules, 4.0, &[(ls::param::ANGLE, angle)]);
        assert!(
            (0.02..1.0).contains(&anchor),
            "{name}: a ancora saiu {anchor}"
        );
        // ⭐ **O ORÁCULO ANALÍTICO**, onde ele existe: achatada, a regra põe `n` unidades em
        // linha, logo a âncora é `1/n` — `F[+F]F[-F]F` põe **3** (as duas dos ramos não
        // esticam a linha), `F+F-F-F+F` põe 5, `FF` e `F+G` põem 2. ⛔ Uma faixa larga aceitava
        // qualquer lei; isto aceita UMA resposta.
        if let Some(want) = exact {
            assert!(
                (anchor - want).abs() < 1e-4,
                "{name}: a ancora tem de ser {want}, deu {anchor}"
            );
        }
        let s = |g: f32| {
            size(&ls::probe_build(
                axiom,
                rules,
                g,
                &[
                    (ls::param::MODE, ls::MODE_GRAMMAR as f32),
                    (ls::param::ANGLE, angle),
                ],
            ))
        };
        let ratio = s(4.0 + 1e-3) / s(4.0).max(1e-6);
        assert!(
            (0.97..1.03).contains(&ratio),
            "{name}: logo depois da geracao 4 a figura mede {ratio:.3}x a de 4 — a ancora nao \
             a poe por cima da anterior (ancora = {anchor:.4})"
        );
    }
}

/// ⛔ **O ESCAPE: desligar o `Grow Angle` devolve o degrau inteiro de sempre.**
///
/// ⚠️ ⭐ As duas metades. Sem a primeira o interruptor é um knob morto; sem a segunda ele não
/// é um escape — e um artista que prefira o degrau (ou que precise de bissecar) fica sem saída.
#[test]
fn switching_the_angle_growth_off_gives_back_the_whole_step() {
    let bush = ls::PRESETS
        .iter()
        .find(|p| p.label == "Bush")
        .expect("o molde existe");
    // 1. Desligado, a fracção é INERTE: o passo é inteiro, byte a byte.
    let off = |g: f32| at(bush, g, &[(ls::param::CONTINUOUS_ANGLE, 0.0)]);
    assert_eq!(
        off(3.25).to_bits(),
        off(3.75).to_bits(),
        "desligado, o passo tem de ser INTEIRO"
    );
    // 2. E ligado (o default) ela interpola.
    assert_ne!(
        at(bush, 3.25, &[]).to_bits(),
        at(bush, 3.75, &[]).to_bits(),
        "ligado, a fraccao tem de interpolar — senao o interruptor e' um knob morto"
    );
    // 3. E o default É ligado.
    let default = ls::MANIFEST
        .params
        .iter()
        .find(|p| p.name == ls::param::CONTINUOUS_ANGLE)
        .expect("o param existe")
        .default;
    assert_eq!(default, 1.0, "o `Grow Angle` shipa LIGADO desde 2026-08-29");
}

/// ⭐ **O `Grow Length` é BYTE-INERTE numa gramática de refinamento** — e a inércia é
/// estrutural, não um defeito.
///
/// Ali quem manda no comprimento é a normalização medida; o interruptor de esticar-de-zero não
/// tem sujeito, porque não há rebento novo a sair de nada. *Onde uma lei é inerte é um facto
/// sobre a lei.*
#[test]
fn the_length_growth_switch_has_no_subject_in_a_refinement_grammar() {
    let bush = ls::PRESETS
        .iter()
        .find(|p| p.label == "Bush")
        .expect("o molde existe");
    assert_eq!(
        at(bush, 3.5, &[(ls::param::CONTINUOUS_LENGTH, 0.0)]).to_bits(),
        at(bush, 3.5, &[(ls::param::CONTINUOUS_LENGTH, 1.0)]).to_bits(),
        "numa gramatica de refinamento o Grow Length nao tem sujeito"
    );
    // ⚠️ **O CONTROLE**: numa que cresce pela ponta ele MANDA. Sem esta metade, arrancar o
    // interruptor inteiro deixaria o gate verde.
    let tree = ls::PRESETS
        .iter()
        .find(|p| p.label == "Tree")
        .expect("o molde existe");
    assert_ne!(
        at(tree, 4.5, &[(ls::param::CONTINUOUS_LENGTH, 0.0)]).to_bits(),
        at(tree, 4.5, &[(ls::param::CONTINUOUS_LENGTH, 1.0)]).to_bits(),
        "numa que cresce pela ponta o Grow Length TEM de mandar"
    );
}

/// A travessia pelo **`Growth`** (`0 → 1`), que é o arrasto que o artista de facto faz.
fn full_drag(p: &ls::Preset) -> Vec<f32> {
    const N: usize = 24;
    (0..=N)
        .map(|k| {
            at(
                p,
                p.generations,
                &[(ls::param::GROWTH, k as f32 / N as f32)],
            )
        })
        .collect()
}

/// **Quem REFINA e quem cresce pela PONTA — pela decisão DO PRODUTO.**
///
/// ⚠️⚠️ Isto tinha um critério PRÓPRIO (a razão entre duas gerações consecutivas) e
/// **discordava do nó no Dragon**, cuja razão oscila (`2,00 · 1,50 · 1,67 · 1,40 · 1,57`): ele
/// caía na família errada e o gate media a lei que o produto não lhe aplica. *Um gate que
/// classifica por conta própria testa a sua própria classificação.*
fn is_refiner(p: &ls::Preset) -> bool {
    ls::probe_growth_ratio(p.axiom, p.rules, &[(ls::param::ANGLE, p.angle)]) > 1.0
}

/// ⭐⭐⭐ **O ARRASTO INTEIRO CRESCE POR IGUAL** — o report do Enio de 2026-08-29
/// (*"ainda não linear"*), depois de eu ter medido a coisa errada duas vezes.
///
/// ⚠️⚠️ **Eu media UMA travessia de geração; ele arrasta o slider TODO.** Dentro de uma
/// travessia a rampa já era recta; ao longo do arrasto ela é **exponencial**, porque cada
/// geração multiplica a figura por uma razão constante (`3,00` no Bush e na Koch, medido). O
/// Bush andava `+0,017` no início e `+0,157` no fim (`9×`); o Dragon `+0,007` contra `+0,335`
/// (`48×`). *Uma régua sobre um trecho não vê a curvatura do percurso.*
///
/// ⚠️⚠️ **E A BARRA É ABSOLUTA, não comparativa — quatro mutações obrigaram.** A 1.ª redacção
/// comparava cada molde com *o pior dos que o dono aceitava*, e essa referência sai do MESMO
/// arrasto: uma sabotagem que piora toda a gente **sobe a barra junto** e passa. É a *razão
/// entre dois doentes* que esta casa já pagou. Os números abaixo são medidos e escritos aqui:
/// Bush `1,0` · Weed `0,7` · Koch `1,0` · Dragon `1,9`, contra `1,8`–`2,1`–`1,9`–`3,7` sem a
/// remapagem.
#[test]
fn dragging_the_growth_slider_is_even_for_the_grammars_that_multiply() {
    // A barra: o pior medido é `1,9` (Dragon). `2,2` dá folga para a máquina e reprova todos
    // os valores sem remapagem (o Dragon dava `3,7`).
    const BAR: f32 = 2.2;
    let mut seen = 0usize;
    for p in ls::PRESETS.iter().filter(|p| is_refiner(p)) {
        seen += 1;
        let hs = full_drag(p);
        let r = ripple(&hs.windows(2).map(|w| w[1] - w[0]).collect::<Vec<_>>());
        assert!(
            hs[hs.len() - 1] > hs[0] * 1.5,
            "{}: o arrasto do Growth tem de crescer: {hs:?}",
            p.label
        );
        assert!(
            r <= BAR,
            "{}: o arrasto ondula {r:.2}x (barra {BAR}) — o report de 2026-08-29 voltou",
            p.label
        );
    }
    // ⚠️ **O CONTROLE**: a família tem de existir. Um censo que varre zero responde «está tudo
    // bem» para sempre.
    assert!(seen >= 4, "so' {seen} gramaticas de refinamento no corpus");
}

/// ⭐⭐⭐ **E A REMAPAGEM NÃO TOCA EM QUEM CRESCE PELA PONTA** — a metade que uma barra sobre a
/// ondulação nunca poderia afirmar.
///
/// ⚠️ Uma gramática cuja razão CONVERGE (`1,63 → 1,06`) não é exponencial, e remapeá-la
/// **piora-a**: medido, o Tree foi de `0,5×` para `0,8×` e o Wild de `1,8×` para `2,2×` quando
/// o discriminador as apanhava. Aqui a inércia é afirmada como uma **identidade**: para elas,
/// `Growth = t` tem de dar exactamente `Generations = 1 + (G−1)·t`, que é a rampa linear.
#[test]
fn the_remap_leaves_the_tip_growers_exactly_where_they_were() {
    let mut seen = 0usize;
    for p in ls::PRESETS.iter().filter(|p| !is_refiner(p)) {
        seen += 1;
        for k in 1..8 {
            let t = k as f32 / 8.0;
            let via_growth = at(p, p.generations, &[(ls::param::GROWTH, t)]);
            let linear = at(p, 1.0 + (p.generations - 1.0) * t, &[]);
            assert_eq!(
                via_growth.to_bits(),
                linear.to_bits(),
                "{} em t = {t}: o Growth remapeou uma gramatica que CONVERGE — ela nao e' \
                 exponencial, e o logaritmo piora-a",
                p.label
            );
        }
    }
    assert!(seen >= 4, "so' {seen} gramaticas de ponta no corpus");
}

/// ⭐⭐ **`Growth = 1` É O NO-OP EXACTO** — e é isso que torna o param aditivo.
///
/// ⚠️ Sem esta metade, acrescentar o controlo teria mexido em toda cena e todo gate desta
/// casa. Com ela, o default não move um bit: o `Generations` continua a querer dizer gerações.
#[test]
fn growth_at_one_is_bit_identical_to_not_having_the_control() {
    for p in ls::PRESETS {
        let with = at(p, p.generations, &[(ls::param::GROWTH, 1.0)]);
        let without = at(p, p.generations, &[]);
        assert_eq!(
            with.to_bits(),
            without.to_bits(),
            "{}: o Growth em 1 tem de ser o no-op EXACTO",
            p.label
        );
    }
    // ⚠️ E o CONTROLE: abaixo de `1` ele TEM de mexer, senão o param é um knob morto.
    let bush = ls::PRESETS
        .iter()
        .find(|p| p.label == "Bush")
        .expect("existe");
    assert_ne!(
        at(bush, bush.generations, &[(ls::param::GROWTH, 0.5)]).to_bits(),
        at(bush, bush.generations, &[]).to_bits(),
        "o Growth em 0,5 nao mexeu — knob morto"
    );
}

/// ⭐⭐⭐ **A RAZÃO MEDIDA BATE COM O QUE A MATEMÁTICA DIZ** — e o oráculo é externo, não o
/// próprio medidor.
///
/// ⚠️⚠️ **Este gate nasceu de uma mutação que SOBREVIVEU**: medir a razão numa amostra só
/// (`span6/span5`) em vez da média geométrica sobre duas gerações não mudava a CLASSIFICAÇÃO
/// de nenhum molde deste corpus, então todos os gates de comportamento ficavam verdes. *Um
/// corpus que não contém o caso não pode reprovar a régua que ele quebra.*
///
/// ⇒ A régua passou a ser a EXACTIDÃO, contra factos conhecidos:
/// - o arbusto `F -> F[+F]F[-F]F` e a ilha de Koch `F -> F+F-F-F+F` são auto-semelhantes de
///   factor **3** (três `F` colineares num, e o gerador da ilha a `90°` a abranger três
///   unidades no outro) — medido: `3,0000` nos dois, ao dígito;
/// - a curva do dragão escala por **`√2`** — medido `1,4832`, `4,9 %` acima, porque a razão
///   dela **oscila** e a janela é finita.
///
/// Uma amostra só põe o dragão fora da barra; a média geométrica põe-no dentro.
#[test]
fn the_measured_ratio_agrees_with_what_the_mathematics_says() {
    let r =
        |p: &ls::Preset| ls::probe_growth_ratio(p.axiom, p.rules, &[(ls::param::ANGLE, p.angle)]);
    let of = |name: &str| {
        ls::PRESETS
            .iter()
            .find(|p| p.label == name)
            .expect("existe")
    };

    for name in ["Bush", "Koch"] {
        let got = r(of(name));
        assert!(
            (got - 3.0).abs() < 0.03,
            "{name} e' auto-semelhante de factor 3 e a medicao deu {got:.4}"
        );
    }
    let dragon = r(of("Dragon"));
    let sqrt2 = 2.0f32.sqrt();
    assert!(
        (dragon / sqrt2 - 1.0).abs() < 0.10,
        "a curva do dragao escala por raiz de 2 ({sqrt2:.4}) e a medicao deu {dragon:.4} \
         ({:+.1}%) — uma amostra so' nao aguenta a oscilacao dela",
        (dragon / sqrt2 - 1.0) * 100.0
    );
    // ⚠️ E o CONTROLE: as que CONVERGEM devolvem o neutro EXACTO, senão elas seriam remapeadas.
    for name in ["Tree", "Fern", "Wild", "Sprig"] {
        assert_eq!(
            r(of(name)),
            1.0,
            "{name} converge e tem de devolver o neutro exacto"
        );
    }
}

/// ⭐⭐⭐ **NENHUMA GRAMÁTICA DE REFINAMENTO PÁRA A MEIO DO SLIDER** — o report do Enio de
/// 2026-08-30 (*"em dragon enquanto cresce (aumentando Generations) parece piscar"*).
///
/// ⚠️ **Encolher e PARAR são defeitos diferentes**, e é por isso que este gate não é o
/// [`no_preset_ever_shrinks_while_the_generations_rise`]: o Dragon nunca encolhia sob o
/// observador invariante — ele **estagnava** (menor passo a `4,5 %` do médio) e depois
/// arrancava. Uma barra sobre o sinal do passo é cega a isso.
///
/// ⛔ **A barra COMPARATIVA foi medida e rejeitada.** «Nenhum que refina pára pior que o pior
/// dos que crescem pela ponta» aceitaria tudo: o **Sprig pára por completo** (`0,0 %`) e é
/// geometria verdadeira — o tronco dele converge por `0,8^n`, então as gerações finais
/// engrossam a planta sem alargar o envelope. *Uma família cujo pior caso é `0` não é
/// referência para nada.*
///
/// ⇒ barra absoluta, com **os dois lados medidos ao lado dela**.
#[test]
fn no_refiner_stalls_while_the_generations_rise() {
    // ⚠️ **A FOLGA, e a medição que a escolheu.** O menor passo MEDIDO divide-se pelo que a
    // razão da própria gramática permite; medido, essa razão é `1,00` (Bush), `0,99` (Koch),
    // `1,03` (Weed) e **`0,86`** (Dragon, o mais apertado — a curvatura de dentro de uma
    // geração). Com `0,5` sobra `1,7×` de margem no pior, e o defeito de 2026-08-30 ficava a
    // **`0,08`** do inerente. *A barra é derivada; só esta folga é escolhida, e os dois lados
    // dela estão medidos.*
    const SLACK: f32 = 0.5;
    let mut seen: Vec<(&str, f32, f32)> = Vec::new();
    for p in ls::PRESETS {
        let r = ls::probe_growth_ratio(p.axiom, p.rules, &[(ls::param::ANGLE, p.angle)]);
        if r <= 1.0 {
            continue;
        }
        let (_, gens) = window(p);
        let (_, d) = whole_slider(p);
        let mean = d.iter().sum::<f32>() / d.len() as f32;
        let lo = d.iter().copied().fold(f32::MAX, f32::min);
        seen.push((p.label, lo / mean * 100.0, inherent_floor(r, gens)));
    }
    assert!(seen.len() >= 3, "o corpus tem de ter refinadores: {seen:?}");
    for (label, pct, floor) in &seen {
        assert!(
            *pct >= floor * SLACK,
            "{label}: o menor passo do arrasto e' {pct:.1} % do passo medio, e a razao desta \
             gramatica permitia {floor:.1} % — a figura PARA e depois arranca, que e' o \
             «piscar» de 2026-08-30. Todos (molde, medido, inerente): {seen:?}"
        );
    }
}

/// ⭐⭐ **O MENOR PASSO QUE A PRÓPRIA GRAMÁTICA PERMITE**, em % do passo médio — a barra
/// DERIVADA, e não um número escolhido (§0.0).
///
/// A lei põe cada geração numa recta do tamanho anterior ao novo, logo dentro da geração `n`
/// o declive é `r^n·(r − 1)`. Sobre `g` gerações de arrasto o menor declive é o de `n = 0` e a
/// média é `(Σ_{n<g} r^n)/g`, então o `(r − 1)` cancela e sobra `g / Σ r^n`.
///
/// ⚠️ **Isto é o preço da rampa por geração, não um defeito** — uma sequência exponencial
/// amostrada por troços tem os troços com declives diferentes por construção, e é exactamente
/// isso que o param `Growth` remapeia. Aqui ele só serve de PISO à barra: um molde que cresce
/// `3×` por geração nunca terá o passo tão uniforme como um que cresce `√2×`.
///
/// ⚠️ **A folga do gate absorve erro de MODELO, não só de máquina** (auditoria de 2026-08-30):
/// o `r` vem de [`ls::probe_growth_ratio`], que o mede nas gerações `4→6`, enquanto a janela do
/// arrasto é `generations−3 .. generations` — para o Dragon (`12`) os dois intervalos são
/// **disjuntos**, e a razão dele oscila. Medido, o quociente `medido/inerente` é `1,00` no
/// Bush, `0,99` na Koch, `1,03` no Weed e **`0,86`** no Dragon: dos `2×` de folga do `SLACK`,
/// `14 %` já estão gastos antes de o produto entrar.
fn inherent_floor(r: f32, g: usize) -> f32 {
    let sum: f32 = (0..g).map(|n| r.powi(n as i32)).sum();
    g as f32 / sum * 100.0
}

/// ⭐⭐⭐ **A FAMÍLIA VEM DA ESTRUTURA, E AS DUAS LEIS RESPONDEM PELA MESMA PORTA.**
///
/// ⛔⛔ Isto substitui um gate que media a folga de um LIMIAR — e o limiar morreu em
/// 2026-08-30 porque se esgotou: quando a régua passou a ser invariante à rotação, o modo
/// **GUIADO (o default do nó)** caiu a `0,017 %` dele, e mexer o `Length Scale` de `0,89` para
/// `0,90` (o default do painel) saltava o tamanho **`+15,4 %`**. Varridas 8 100 combinações
/// dos knobs, o guiado chega a `1,4294` e o refinador mais fraco do corpus a `1,4791` —
/// **`3,5 %`** de separação. *Duas classes que se tocam não se separam por um número.*
///
/// As três metades, e nenhuma é opcional:
#[test]
fn the_family_comes_from_the_structure_and_the_two_laws_agree() {
    // 1. ⭐ O ORÁCULO: quem reescreve TODO o módulo que desenha refina; quem tem um `F`
    //    terminal cresce pela ponta. Escrito à mão de propósito — é a resposta, não a medição.
    let expected: &[(&str, bool)] = &[
        ("Tree", false),
        ("Fern", false),
        ("Bush", true),
        ("Weed", true),
        ("Wild", false),
        ("Koch", true),
        ("Dragon", true),
        ("Sprig", false),
    ];
    assert_eq!(
        expected.len(),
        ls::PRESETS.len(),
        "um molde novo tem de declarar a familia aqui"
    );
    for (p, (label, want)) in ls::PRESETS.iter().zip(expected) {
        assert_eq!(&p.label, label, "a ordem do oraculo segue a dos moldes");
        let got = ls::probe_grows_by_refining(p.axiom, p.rules, &[(ls::param::ANGLE, p.angle)]);
        assert_eq!(got, *want, "{label}: familia errada");
    }
    // ⚠️ **O CONTROLE**: as duas famílias têm de existir no corpus.
    let refiners = expected.iter().filter(|(_, r)| *r).count();
    assert!(
        refiners >= 3 && expected.len() - refiners >= 3,
        "o corpus tem de povoar as duas familias"
    );

    // 2. ⭐⭐ **O MODO GUIADO É DA PONTA** — o caso que o limiar classificava mal, e o `F` da
    //    gramática que ele deriva é terminal por construção.
    let guided = ls::shape::rules(&ls::shape::Shape {
        branches: 2.0,
        segments: 1.0,
        variation: 0.0,
        bend: 0.0,
    });
    for ls_scale in [0.5f32, 0.7, 0.89, 0.90, 0.95, 1.0] {
        assert!(
            !ls::probe_grows_by_refining(
                ls::shape::AXIOM,
                &guided,
                &[(ls::param::LENGTH_SCALE, ls_scale)]
            ),
            "o guiado com Length Scale {ls_scale} tem de ser da PONTA — foi ali que o limiar \
             partiu o produto ao meio"
        );
    }

    // 3. ⭐⭐⭐ **A MESMA PORTA**: a decisão que a remapagem do `Growth` toma tem de ser a
    //    que o desenho toma. A consequência OBSERVÁVEL da decisão do desenho é que numa
    //    gramática de refinamento o `Grow Length` é byte-inerte (quem manda no comprimento
    //    é a normalização medida); numa da ponta ele MANDA.
    for (p, (label, want)) in ls::PRESETS.iter().zip(expected) {
        let with = |v: f32| at(p, p.generations - 0.5, &[(ls::param::CONTINUOUS_LENGTH, v)]);
        let length_switch_is_inert = with(0.0).to_bits() == with(1.0).to_bits();
        assert_eq!(
            length_switch_is_inert, *want,
            "{label}: a remapagem do Growth diz «refina = {want}» e o DESENHO diz o contrario \
             — as duas leis voltaram a ter respostas proprias"
        );
    }
}

/// ⛔⛔ **PENDURAR UMA FOLHA NÃO MUDA A FAMÍLIA DE CRESCIMENTO DA PLANTA.**
///
/// A pergunta da família é *«sobrou algum módulo VELHO que DESENHA?»*, e uma marca de instância
/// (`J`/`K`/`M`) **não desenha** — ela diz onde pousar um objecto. Enquanto a pergunta era a
/// larga (`F | G | f | g | J | K | M`), acrescentar uma folha a um molde de REFINAMENTO
/// reclassificava-o como da ponta: um `J` nunca é reescrito, logo **acumula** e sobrevive às
/// gerações. *A planta era nova e as folhas eram velhas, e a velhice das folhas decidia por ela.*
///
/// ⚠️ **Medido em 2026-08-30**, quando os moldes ganharam âncoras de folha: com a pergunta larga
/// o `Bush` e o `Weed` trocavam de lei de comprimento e de remapagem do `Growth` só por levarem
/// uma folha. ⇒ a pergunta estreitou-se ([`turtle::draws`]), e este gate é a cerca.
///
/// ⚠️ E ele mede **as duas famílias**: uma cura que respondesse sempre «da ponta» passaria com
/// metade do oráculo.
#[test]
fn hanging_a_leaf_on_a_grammar_does_not_change_its_growth_family() {
    // Um par de cada família, com a regra decorada de uma marca de instância.
    let cases: [(&str, &str, &str, &str, bool); 4] = [
        (
            "Bush",
            "X",
            "X -> F[+X]F[-X]+X ; F -> FF",
            "X -> F[+XJ]F[-X]+X ; F -> FF",
            true,
        ),
        (
            "Weed",
            "X",
            "X -> F[+X]F[-X]+X ; F -> FF",
            "X -> F[+X]F[-XK]+X ; F -> FF",
            true,
        ),
        (
            "ponta",
            "A(step)",
            "A(s) -> F(s)![+A(s*0.7)][-A(s*0.7)]",
            "A(s) -> F(s)![+A(s*0.7)J][-A(s*0.7)J]",
            false,
        ),
        (
            "ponta-M",
            "A(step)",
            "A(s) -> F(s)![+A(s*0.7)][-A(s*0.7)]",
            "A(s) -> F(s)!M[+A(s*0.7)][-A(s*0.7)]",
            false,
        ),
    ];
    let mut refiners = 0;
    for (label, axiom, plain, decorated, refines) in cases {
        assert_eq!(
            ls::probe_grows_by_refining(axiom, plain, &[]),
            refines,
            "{label}: o oraculo descreve mal a gramatica NUA"
        );
        assert_eq!(
            ls::probe_grows_by_refining(axiom, decorated, &[]),
            refines,
            "{label}: pendurar uma folha mudou a familia — a pergunta voltou a contar marcas"
        );
        refiners += usize::from(refines);
    }
    assert_eq!(refiners, 2, "o oraculo tem de povoar as DUAS familias");
}
