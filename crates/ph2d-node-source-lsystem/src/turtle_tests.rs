//! Gates da **tartaruga** — o que a cadeia vira quando é desenhada, e a invariante do rig
//! que faz disto uma árvore em vez de uma nuvem.

use super::*;
use crate::derive::axiom_modules;

fn nop(_: &str) -> f32 {
    0.0
}

fn setup() -> Setup {
    Setup {
        angle: 90.0,
        step: 1.0,
        width: 1.0,
        width_scale: 0.5,
        length_scale: 0.5,
        root_angle: 90.0,
        tropism: 0.0,
        tropism_angle: -90.0,
        youngest: (0, 1.0),
        // ⚠️ **Igual ao `youngest.1` ⇒ a lei do recém-nascido fica INERTE nesta fixtura**, que é
        // o que ela quer: os gates daqui medem uma lei de cada vez sobre uma cadeia escrita à
        // mão. Quem contém o fenómeno do recém-nascido é `tests/newborn_law.rs`.
        newborn: 1.0,
        angle_frac: 1.0,
        // ⚠️ **Os defaults do PRODUTO**, e a assimetria é a decisão de 2026-08-29: o
        // comprimento cresce (é o que sempre shipou) e o ângulo **não** (ver a recusa
        // medida em `tests/growth_is_two_laws.rs`).
        // ⚠️ **LOCAL na fixtura de base**, de propósito: os gates da invariante do rig medem o
        // contrato do `rig.*`, e é ele que exige o ângulo local. O modo de MUNDO (o default do
        // produto) tem gates próprios.
        orient_world: false,
        leaf_first_level: 0.0,
        leaf_angle: 0.0,
        leaf_spread: 0.0,
        leaf_effects: true,
        seed: 1.0,
    }
}

fn draw(src: &str, set: &Setup) -> Stream {
    let p: &dyn Fn(&str) -> f32 = &nop;
    walk(&axiom_modules(src, p), set)
}

fn scal(s: &Stream, name: &str) -> Vec<f32> {
    match s.get(name) {
        Some(Column::Scalar(v)) => v.clone(),
        _ => panic!("coluna escalar {name}"),
    }
}

fn vec2(s: &Stream, name: &str) -> Vec<[f32; 2]> {
    match s.get(name) {
        Some(Column::Vec2(v)) => v.clone(),
        _ => panic!("coluna vec2 {name}"),
    }
}

/// Três passos a direito: a raiz mais três elementos, empilhados.
#[test]
fn a_straight_stem_stacks_element_on_element_out_of_the_root() {
    let s = draw("FFF", &setup());
    assert_eq!(s.count(), 4, "a raiz mais tres");
    let p = vec2(&s, "P");
    for (i, q) in p.iter().enumerate() {
        assert!(q[0].abs() < 1e-4, "elemento {i} fora do eixo: {q:?}");
        assert!((q[1] - i as f32).abs() < 1e-4, "elemento {i} em y = {i}");
    }
    assert_eq!(scal(&s, "parent"), vec![-1.0, 0.0, 1.0, 2.0]);
}

/// ⭐ **Um ramo pendura-se onde foi aberto** — e ao fechar, a haste continua do MESMO sítio.
///
/// Em `F[+F]F`, o segundo e o terceiro elementos desenhados penduram-se os dois no primeiro.
/// É a assinatura de um galho; uma cadeia única não a produz.
#[test]
fn a_branch_and_the_stem_after_it_hang_off_the_same_element() {
    let s = draw("F[+F]F", &setup());
    let parent = scal(&s, "parent");
    assert_eq!(s.count(), 4, "raiz + tres F");
    assert_eq!(parent[2], 1.0, "o F do ramo pendura no F da haste");
    assert_eq!(parent[3], 1.0, "e o F depois do `]` tambem");
}

/// ⭐⭐ **A INVARIANTE DO RIG: um osso nunca estica.**
///
/// `‖P[i] − P[pai]‖ == len[i]` para todo elemento, sobre uma gramática que usa ramos, saltos,
/// espessura, passo variável e marcas. É o que torna a saída deste nó legítima no contrato
/// `rig.*` — e o que se perderia em silêncio se a posição fosse calculada de outra maneira
/// que não a do `rig.fk`.
#[test]
fn every_bone_measures_exactly_its_own_length() {
    let mut set = setup();
    set.angle = 33.0;
    let s = draw("F[+F!F]-F\"F f F[-FJ]F", &set);
    let (p, parent, len) = (vec2(&s, "P"), scal(&s, "parent"), scal(&s, "len"));
    assert!(
        s.count() > 6,
        "a fixtura tem de ter elementos: {}",
        s.count()
    );
    for i in 0..s.count() {
        let par = parent[i];
        if par < 0.0 {
            continue;
        }
        let j = par as usize;
        let d = (p[i][0] - p[j][0]).hypot(p[i][1] - p[j][1]);
        assert!(
            (d - len[i]).abs() < 1e-4,
            "elemento {i}: len {} mas mede {d}",
            len[i]
        );
    }
}

/// ⚠️ **Um salto (`f`) faz nascer uma RAIZ nova, e é por isso que a invariante aguenta.**
///
/// Se o elemento depois do salto se pendurasse no anterior, `‖P − P[pai]‖` deixaria de ser
/// `len` — e o contrato do rig passaria a ser falso só nos documentos que usam `f`.
#[test]
fn a_jump_starts_a_new_root_instead_of_a_stretched_bone() {
    let s = draw("F f F", &setup());
    let parent = scal(&s, "parent");
    let roots = parent.iter().filter(|p| **p < 0.0).count();
    assert_eq!(roots, 2, "a raiz da planta e a que nasce depois do salto");
    // E o CONTROLE: sem o salto há uma raiz só.
    let one = draw("F F", &setup());
    assert_eq!(scal(&one, "parent").iter().filter(|p| **p < 0.0).count(), 1);
}

/// ⚠️ **Sem um `!` na gramática e com `Width = 1`, a coluna `size` é EXACTAMENTE a
/// identidade** — o nó não pode redimensionar a cena por existir.
#[test]
fn without_a_width_command_the_size_column_is_exactly_the_identity() {
    let s = draw("F[+F]F", &setup());
    for (i, sz) in vec2(&s, "size").iter().enumerate() {
        assert_eq!(
            sz.map(f32::to_bits),
            ph2d_nodegraph::attr::SIZE_IDENTITY.map(f32::to_bits),
            "elemento {i}: {sz:?}"
        );
    }
    // E o `!` de facto afina — senão o gate acima passaria com o comando morto.
    let thin = draw("F!F", &setup());
    let sz = vec2(&thin, "size");
    assert!(
        (sz[2][0] - 0.5).abs() < 1e-6,
        "o `!` multiplica pelo Width Scale, deu {}",
        sz[2][0]
    );
}

/// **`%` corta o resto do ramo**, e o que vem depois do `]` continua.
#[test]
fn the_cut_drops_the_rest_of_its_branch_and_only_that() {
    let full = draw("F[+FFF]F", &setup());
    let cut = draw("F[+F%FF]F", &setup());
    assert_eq!(full.count(), 6, "raiz + 5 F");
    assert_eq!(
        cut.count(),
        4,
        "raiz + o F da haste + 1 no ramo + o de depois"
    );
    // A haste depois do `]` sobreviveu: o último elemento pendura no primeiro F.
    let parent = scal(&cut, "parent");
    assert_eq!(*parent.last().unwrap(), 1.0);
}

/// **O tropismo curva o passo para a direcção declarada** — e a `0` nada se move.
///
/// A régua é o ângulo de mundo do último elemento: com a tartaruga a subir (90°) e o
/// tropismo a puxar para baixo (−90°), o produto vectorial é máximo e cada passo desvia.
#[test]
fn tropism_bends_the_walk_and_zero_leaves_it_straight() {
    let mut set = setup();
    set.root_angle = 0.0; // a andar para +x
    let straight = draw("FFFF", &set);
    let w0 = *scal(&straight, "wrot").last().unwrap();
    assert!(w0.abs() < 1e-4, "sem tropismo o rumo nao muda: {w0}");

    set.tropism = 10.0;
    set.tropism_angle = -90.0; // para baixo
    let bent = draw("FFFF", &set);
    let w1 = *scal(&bent, "wrot").last().unwrap();
    assert!(
        w1 < -10.0,
        "o rumo tem de cair na direccao do tropismo, deu {w1}"
    );
    let py = vec2(&bent, "P").last().unwrap()[1];
    assert!(py < 0.0, "e a ponta tem de estar abaixo do eixo, deu {py}");
}

/// **`J` pousa um elemento SEM segmento** — a marca de folha/flor, na posição do pai e com
/// um `sym` próprio, que é o que a torna seleccionável a jusante.
#[test]
fn a_leaf_mark_lands_on_its_parent_with_no_bone_and_its_own_symbol() {
    let s = draw("FJ", &setup());
    assert_eq!(s.count(), 3);
    let (p, len, sym) = (vec2(&s, "P"), scal(&s, "len"), scal(&s, "sym"));
    assert_eq!(len[2], 0.0, "uma marca nao tem osso");
    assert_eq!(p[2], p[1], "e fica onde o pai esta");
    assert_eq!(sym[2], f32::from(b'J'));
    assert_eq!(sym[1], f32::from(b'F'), "e o tronco diz que e' um F");
}

/// A profundidade de ramo é uma coluna, e conta os colchetes ABERTOS.
#[test]
fn the_depth_column_counts_open_brackets() {
    let s = draw("F[+F[+F]]F", &setup());
    let d = scal(&s, "depth");
    assert_eq!(d[1], 0.0, "a haste esta ao nivel do chao");
    assert_eq!(d[2], 1.0, "o primeiro ramo");
    assert_eq!(d[3], 2.0, "o ramo dentro do ramo");
    assert_eq!(d[4], 0.0, "e depois dos dois `]` voltamos ao chao");
}

/// Toda letra que não é comando é **muda**: estrutura a reescrita e não desenha nada.
#[test]
fn an_unknown_letter_draws_nothing() {
    let with = draw("FXYZF", &setup());
    let without = draw("FF", &setup());
    assert_eq!(with.count(), without.count());
    assert_eq!(vec2(&with, "P"), vec2(&without, "P"));
}

/// ⭐⭐ **A FORMA APONTA PARA ONDE O RAMO CRESCE** — o report do Enio de 2026-08-28.
///
/// O lowering desenha cada instância com o ângulo da coluna **`rot`**, e o contrato do `rig.*`
/// diz que `rot` é o ângulo LOCAL. Num galho a direito o local é ≈ `0` ⇒ a forma carimbada saía
/// sempre em pé, qualquer que fosse a direcção do ramo.
///
/// ⚠️ **O CONTROLE é o modo LOCAL**: ali o `rot` de uma haste a direito TEM de ser ≈ `0`, e é
/// isso que prova que os dois modos são de facto dois — e que o local continua a servir o rig.
#[test]
fn in_growth_mode_the_shape_faces_along_its_branch() {
    let mut world = setup();
    world.orient_world = true;
    world.angle = 40.0;
    // Uma haste a subir, depois um ramo a 40° para cada lado.
    let w = draw("FF[+FF][-FF]", &world);
    let (rot, wrot) = (scal(&w, "rot"), scal(&w, "wrot"));
    for (i, (r, wr)) in rot.iter().zip(&wrot).enumerate() {
        assert!(
            (r - wr).abs() < 1e-4,
            "no modo de crescimento o elemento {i} tem de apontar para o MUNDO: {r} vs {wr}"
        );
    }
    // E os ramos apontam de facto para lados diferentes — senão o gate acima seria vacuo.
    let mut sorted = rot.clone();
    sorted.sort_by(f32::total_cmp);
    assert!(
        sorted.last().unwrap() - sorted.first().unwrap() > 70.0,
        "os dois ramos abrem 80 graus entre si: {rot:?}"
    );

    // ⚠️ O CONTROLE: em LOCAL a mesma haste sai toda a zero (nada virou em relação ao pai).
    let mut local = world;
    local.orient_world = false;
    let l = scal(&draw("FF[+FF][-FF]", &local), "rot");
    let straight = l.iter().filter(|r| r.abs() < 1e-4).count();
    assert!(
        straight >= 4,
        "em local uma haste a direito tem de ter `rot` zero: {l:?}"
    );
}

/// ⭐⭐⭐ **A RÉGUA NÃO MUDA QUANDO A FIGURA RODA** — a lei que o report do Enio de 2026-08-30
/// (*"em dragon enquanto cresce parece piscar"*) comprou.
///
/// A lei do crescimento põe o que a [`mean_width`] devolve numa rampa recta. Se a régua
/// depender da ORIENTAÇÃO, uma figura que rode — e a curva do dragão roda `45°` por geração
/// por construção — estagna e depois arranca, sem que a lei saiba.
///
/// ⚠️ **A régua de até 2026-08-30 (`max(w, h)`) reprova aqui com `10,9 %`** nesta fixtura, e é
/// isso que faz este gate ser o que impede a volta. ⛔ A 1.ª redacção escrevia `32,5 %` aqui —
/// esse número é de OUTRA bancada (o pior dos oito moldes em `examples/probe_ruler.rs`), e a
/// auditoria de 2026-08-30 apanhou-o: *um número medido noutra figura não descreve esta.*
///
/// ⚠️⚠️ **E a ondulação da média é uma constante de `K`, não uma propriedade da figura** —
/// medido, um rectângulo `1×3`, uma agulha e uma de aspecto `100` dão os MESMOS `0,48 %` a
/// `K = 16`. `K = 2` dá `32,6 %` · `K = 4` dá `7,8 %` · `K = 8` dá `1,9 %` · **`K = 16` dá
/// `0,48 %`** · `K = 32` dá `0,12 %`. ⇒ a barra de `1 %` deixa passar o `16` que shipa e
/// **reprova o `8`**. *Este gate é, honestamente, um gate do `K` — o que prova qual é a LEI é o
/// [`the_ruler_is_the_mean_width_and_a_square_proves_it`], porque o máximo direccional também é
/// invariante à rotação e passaria aqui.*
#[test]
fn the_ruler_does_not_change_when_the_figure_turns() {
    // Uma figura deliberadamente ALONGADA: é onde uma caixa de eixo mais mente.
    let chain = axiom_modules("F(3)+F(1)+F(3)", &nop as &dyn Fn(&str) -> f32);
    let mut widths = vec![];
    for deg in 0..=90 {
        let mut set = setup();
        set.root_angle = deg as f32;
        widths.push(mean_width(&chain, &set));
    }
    let hi = widths.iter().copied().fold(f32::MIN, f32::max);
    let lo = widths.iter().copied().fold(f32::MAX, f32::min);
    let mean = widths.iter().sum::<f32>() / widths.len() as f32;
    let ripple = (hi - lo) / mean * 100.0;
    assert!(
        ripple < 1.0,
        "a regua muda {ripple:.2} % so' por a figura rodar — ela tem de ser invariante, senao \
         a lei do crescimento normaliza a orientacao em vez do tamanho"
    );
    // ⚠️ **O CONTROLE**: a figura tem de ser mesmo alongada, senao o gate mede um disco e
    // qualquer regua passa. `max(w,h)` sobre ELA varia muito — e e' o que se esta a recusar.
    let mut axis = vec![];
    for deg in 0..=90 {
        let mut set = setup();
        set.root_angle = deg as f32;
        let s = walk(&chain, &set);
        let Some(Column::Vec2(v)) = s.get("P") else {
            panic!("a coluna P existe")
        };
        let w = v.iter().map(|q| q[0]).fold(f32::MIN, f32::max)
            - v.iter().map(|q| q[0]).fold(f32::MAX, f32::min);
        let h = v.iter().map(|q| q[1]).fold(f32::MIN, f32::max)
            - v.iter().map(|q| q[1]).fold(f32::MAX, f32::min);
        axis.push(w.max(h));
    }
    let am = axis.iter().sum::<f32>() / axis.len() as f32;
    let ar = (axis.iter().copied().fold(f32::MIN, f32::max)
        - axis.iter().copied().fold(f32::MAX, f32::min))
        / am
        * 100.0;
    assert!(
        ar > 10.0,
        "a fixtura tem de ser alongada: a caixa de eixo so' varia {ar:.1} % ao roda-la, entao \
         este gate passaria com uma regua qualquer"
    );
}

// ===== AUDIT LENS TEMP PROBE — REMOVER =====
fn mw_k(chain: &[Module], set: &Setup, k: usize) -> f32 {
    let s = walk(chain, set);
    let Some(Column::Vec2(v)) = s.get("P") else {
        return 0.0;
    };
    let mut acc = 0.0f64;
    for i in 0..k {
        let a = std::f32::consts::PI * i as f32 / k as f32;
        let (c, sn) = (a.cos(), a.sin());
        let mut lo = f32::MAX;
        let mut hi = f32::MIN;
        for q in v {
            let t = q[0] * c + q[1] * sn;
            lo = lo.min(t);
            hi = hi.max(t);
        }
        acc += f64::from(hi - lo);
    }
    (acc / k as f64) as f32
}

#[test]
fn audit_probe_fixture() {
    let chain = axiom_modules("F(3)+F(1)+F(3)", &nop as &dyn Fn(&str) -> f32);
    let s = walk(&chain, &setup());
    let Some(Column::Vec2(v)) = s.get("P") else {
        panic!()
    };
    println!("PONTOS = {v:?}");
    let w = v.iter().map(|q| q[0]).fold(f32::MIN, f32::max)
        - v.iter().map(|q| q[0]).fold(f32::MAX, f32::min);
    let h = v.iter().map(|q| q[1]).fold(f32::MIN, f32::max)
        - v.iter().map(|q| q[1]).fold(f32::MAX, f32::min);
    println!(
        "CAIXA a 0 graus: w = {w}  h = {h}  aspecto = {}",
        (w / h).max(h / w)
    );
    // ondulacoes ao rodar
    let mut axis = vec![];
    let mut prod = vec![];
    let mut k4 = vec![];
    let mut k8 = vec![];
    let mut k64 = vec![];
    for deg in 0..=90 {
        let mut set = setup();
        set.root_angle = deg as f32;
        let s = walk(&chain, &set);
        let Some(Column::Vec2(v)) = s.get("P") else {
            panic!()
        };
        let w = v.iter().map(|q| q[0]).fold(f32::MIN, f32::max)
            - v.iter().map(|q| q[0]).fold(f32::MAX, f32::min);
        let h = v.iter().map(|q| q[1]).fold(f32::MIN, f32::max)
            - v.iter().map(|q| q[1]).fold(f32::MAX, f32::min);
        axis.push(w.max(h));
        prod.push(mean_width(&chain, &set));
        k4.push(mw_k(&chain, &set, 4));
        k8.push(mw_k(&chain, &set, 8));
        k64.push(mw_k(&chain, &set, 64));
    }
    let rip = |v: &Vec<f32>| {
        let m = v.iter().sum::<f32>() / v.len() as f32;
        (v.iter().copied().fold(f32::MIN, f32::max) - v.iter().copied().fold(f32::MAX, f32::min))
            / m
            * 100.0
    };
    println!("ondulacao AXIS max(w,h) = {:.3} %", rip(&axis));
    println!("ondulacao PRODUTO (K=16, dir())  = {:.3} %", rip(&prod));
    println!("ondulacao K=4  (f32 trig) = {:.3} %", rip(&k4));
    println!("ondulacao K=8  (f32 trig) = {:.3} %", rip(&k8));
    println!("ondulacao K=64 (f32 trig) = {:.3} %", rip(&k64));
}

/// ⭐⭐⭐ **A RÉGUA É A MÉDIA, e não o máximo nem a caixa** — a costura que uma auditoria
/// adversarial provou faltar em 2026-08-30.
///
/// ⚠️⚠️ **MUTAÇÃO QUE SOBREVIVEU:** trocar a média das larguras pelo **MÁXIMO** delas passava
/// nos 85 testes e no clippy, e piorava o produto de forma medível (o Dragon caía de `55,2 %`
/// para `52,1 %` de pior passo). O gate de invariância não a apanha — *o máximo direccional
/// também é invariante à rotação* —, e nenhum outro afirmava qual das duas é.
///
/// A régua é a **largura média de Cauchy**, e para um convexo ela é o **`perímetro/π`**. Isso
/// é uma identidade, logo um oráculo: um quadrado de lado `1` tem de dar `4/π = 1,27324`.
///
/// ⚠️ **O piso é a discretização de `K`** (`0,48 %` a `K = 16`, medido) — este gate mata as
/// construções ERRADAS (o máximo dá `√2`, `+11 %`; a caixa de eixo dá `1`, `−21 %`), e **não**
/// consegue ver uma contaminação de poucos por cento. Quem a vê é o
/// [`the_ruler_does_not_change_when_the_figure_turns`], e é por isso que os dois existem.
#[test]
fn the_ruler_is_the_mean_width_and_a_square_proves_it() {
    let mut set = setup();
    set.angle = 90.0;
    set.root_angle = 90.0;
    let square = axiom_modules("F(1)+F(1)+F(1)+F(1)", &nop as &dyn Fn(&str) -> f32);
    let w = mean_width(&square, &set);
    let exact = 4.0 / std::f32::consts::PI;
    let err = (w - exact).abs() / exact * 100.0;
    assert!(
        err < 1.0,
        "um quadrado de lado 1 tem perimetro 4, logo largura media 4/pi = {exact:.5}; a regua \
         deu {w:.5} ({err:.2} % de erro)"
    );
    // ⚠️ **O CONTROLE, e ele NOMEIA as duas construções recusadas** — sem isto o gate passaria
    // com qualquer coisa perto de 1,27, e o que ele existe para dizer é *qual* das leis é.
    let diameter = std::f32::consts::SQRT_2;
    assert!(
        (w - diameter).abs() / diameter > 0.05,
        "a regua deu {w:.5}, que e' o DIAMETRO ({diameter:.5}) — isso e' o maximo das larguras, \
         nao a media"
    );
    assert!(
        (w - 1.0f32).abs() > 0.05,
        "a regua deu {w:.5}, que e' o lado da caixa de eixo — a regua de ate' 2026-08-30"
    );

    // ⭐⭐ **E A AGULHA, alinhada com um eixo** — é ela que fecha a janela que o quadrado
    // deixa. Uma auditoria mediu que **`4 %` da régua de eixo podiam voltar em silêncio**; num
    // quadrado essa mistura desloca o valor `0,8 %` (debaixo da barra), mas numa agulha
    // alinhada a caixa de eixo lê `L` contra os `2L/π = 0,6366 L` da média — **`+57 %`** —, e
    // `4 %` dela já são `+2,3 %`. *A fixtura que mais separa duas leis é aquela em que elas
    // mais discordam, e não a mais bonita.*
    let mut needle = setup();
    needle.root_angle = 0.0;
    let seg = axiom_modules("F(1)", &nop as &dyn Fn(&str) -> f32);
    let nw = mean_width(&seg, &needle);
    let nexact = 2.0 / std::f32::consts::PI;
    let nerr = (nw - nexact).abs() / nexact * 100.0;
    assert!(
        nerr < 1.0,
        "um segmento de comprimento 1 tem largura media 2/pi = {nexact:.5}; a regua deu \
         {nw:.5} ({nerr:.2} % de erro) — uma caixa de eixo leria 1,0"
    );
}

/// ⭐⭐⭐ **A GERAÇÃO NOVA ABRE AS DOBRAS EM VEZ DE SALTAR** — e este é o ÚNICO gate desta
/// crate que olha para um ÂNGULO.
///
/// ⚠️⚠️⚠️ **MUTAÇÃO QUE SOBREVIVEU (auditoria adversarial, 2026-08-30):** apagar o
/// `set.angle_frac` — isto é, desligar o *Continuous angles*, a metade da feature de que a lei
/// do crescimento inteira trata — deixava **85 testes e o clippy `-D warnings` verdes**.
///
/// **O mecanismo:** todos os outros gates lêem a figura por um TAMANHO, e a largura média é
/// quase cega ao dobrar — medido, a âncora do Bush com as dobras fechadas dá `0,333333` e com
/// elas abertas `0,333289`, **`0,013 %`**. E o gate que existia para o interruptor
/// (`switching_the_angle_growth_off_gives_back_the_whole_step`) é satisfeito pelo efeito
/// colateral da normalização do COMPRIMENTO, não pelo ângulo. É a espécie do `CLAUDE.md §5.0`:
/// *o consumidor que projecta o valor fora.*
///
/// ⇒ a régua deste gate é a POSE: a maior viragem que a figura contém.
#[test]
fn the_newest_generation_opens_its_folds_instead_of_snapping() {
    let chain = |g: u16| {
        let p: &dyn Fn(&str) -> f32 = &nop;
        crate::derive::derive(
            &axiom_modules("F", p),
            &crate::grammar::parse_rules("F -> F[+F]F[-F]F"),
            g,
            1,
            crate::MAX_MODULES,
            p,
        )
    };
    // A maior viragem que a figura contém, com a geração mais nova a `frac` aberta.
    let pose = |frac: f32, on: bool| {
        let d = chain(4);
        let mut set = setup();
        set.angle = 25.7;
        set.orient_world = true;
        set.youngest = (d.generations, 1.0);
        set.angle_frac = if on { frac } else { 1.0 };
        let s = walk(&d.chain, &set);
        scal(&s, "rot").iter().fold(0.0f32, |m, r| m.max(r.abs()))
    };
    // 1. ⭐ LIGADO, a pose ABRE: estritamente crescente ao longo da travessia.
    let opened: Vec<f32> = [0.0f32, 0.25, 0.5, 0.75, 1.0]
        .iter()
        .map(|f| pose(*f, true))
        .collect();
    for w in opened.windows(2) {
        assert!(
            w[1] > w[0] + 1e-3,
            "a viragem da geracao nova tem de ABRIR com a fraccao: {opened:?}"
        );
    }
    // 2. ⭐ E a excursão é GRANDE — sem isto o gate passaria com uma abertura de um milésimo.
    let span = opened[4] - opened[0];
    assert!(
        span > 10.0,
        "a pose so' abre {span:.3} graus ao longo da travessia inteira: {opened:?}"
    );
    // 3. ⛔ DESLIGADO, ela SALTA: a viragem e' a cheia em toda a travessia, byte a byte.
    let snapped: Vec<f32> = [0.0f32, 0.25, 0.5, 0.75, 1.0]
        .iter()
        .map(|f| pose(*f, false))
        .collect();
    for v in &snapped {
        assert_eq!(
            v.to_bits(),
            opened[4].to_bits(),
            "desligado, a pose tem de ser a CHEIA em toda a travessia: {snapped:?}"
        );
    }
}
