//! **O ACCUMULATE** — de onde a distância é medida, e mais nada.
//!
//! ⚠️ **Este arquivo descrevia a coisa errada até 2026-08-11.** Ele chamava o
//! interruptor de *"a lei que troca o envelope por uma integral de linha"* e
//! media uma pilha normalizada pelo passo (`ACCUM_PER_DAB`). O rótulo do
//! original é *"Accumulate (no limit per stroke)"*, e o mecanismo dele é
//! **UM**: a distância sai da posição VIVA em vez do proxy congelado no
//! primeiro toque — ver [`crate::ref_kernels::Origin`], que o guarda num tipo
//! justamente para ninguém achar que ele multiplica alguma coisa.
//!
//! ⚠️ **E a FIXTURE media o oposto do produto.** Ela fixava o centro do dab na
//! esfera ORIGINAL, e o mecanismo inteiro depende de o centro subir com a tinta
//! (no produto ele é `hit.point`, o acerto do raycast contra a malha **viva** —
//! `sculpt3d_input.rs`). Com o centro parado, o vértice sobe e se AFASTA, a
//! distância viva CRESCE e o armado sai mais fraco que o desarmado; medido:
//!
//! | passadas | centro fixo (armado ÷ desarmado) | centro na superfície viva |
//! |---|---|---|
//! | 1 | 0,900× | 0,995× |
//! | 2 | 0,759× | **1,225×** |
//! | 4 | 0,691× | **1,495×** |
//!
//! A coluna da direita é o produto, e é a da referência: **desarmado o pincel
//! se esgota** (0,116 → 0,192 → 0,264, sublinear) e **armado ele não tem
//! limite** (0,116 → 0,236 → 0,395, linear nas passadas).

use super::*;
use ph2d_mesh::{Mesh, shapes};

fn sphere() -> Mesh {
    shapes::uv_sphere(32, 48, 1.0)
}

/// O raio de mundo de um pincel que cobre uma calota confortável da esfera.
const R: f32 = 0.35;

/// O ponto da superfície **VIVA** sob `(x, 0)`.
///
/// ⚠️ **É a premissa inteira do mecanismo, e a fixture antiga não a tinha.** No
/// produto o centro do dab é `hit.point`, o acerto do raycast contra a malha
/// viva; ele SOBE com a tinta. Um centro parado na esfera original faz a
/// distância viva crescer em vez de acompanhar, e o gate mediria o inverso do
/// que o interruptor faz — ver a tabela no cabeçalho.
fn live_centre(mesh: &Mesh, x: f32) -> [f32; 3] {
    let mut best = [x, 0.0, 1.0];
    let mut bd = f32::MAX;
    for p in mesh.positions() {
        if p[2] < 0.0 {
            continue;
        }
        let d = (p[0] - x).abs() + p[1].abs();
        if d < bd {
            bd = d;
            best = *p;
        }
    }
    best
}

/// **Uma passada reta pelo topo da esfera**, carimbada no passo geométrico que o
/// produto usa, com o centro **na superfície viva**.
///
/// ⚠️ Ela anda em `x` sobre o polo `+Z`: é a fatia da esfera onde o olho (`-Z`)
/// vê tudo de frente, então o culling não entra na conta e o gate mede a LEI,
/// não a visibilidade.
fn sweep(mesh: &mut Mesh, stroke: &mut SculptStroke, brush: &Brush, passes: usize) {
    let step = crate::min_spacing(R);
    let (from, to) = (-0.5_f32, 0.5_f32);
    let n = ((to - from) / step).floor() as usize;
    for _ in 0..passes {
        for k in 0..=n {
            let x = step.mul_add(k as f32, from);
            let c = live_centre(mesh, x);
            stroke.dab(
                mesh,
                brush,
                &Dab::at(c, R, [0.0, 0.0, -1.0]),
                Symmetry::default(),
            );
        }
    }
}

/// Quanto o vértice mais alto da calota subiu.
fn lift(before: &Mesh, after: &Mesh) -> f32 {
    let mut best = 0.0_f32;
    for (i, b) in before.positions().iter().enumerate() {
        // Só o miolo do caminho: as pontas veem meia passada por construção.
        if b[0].abs() > 0.2 || b[1].abs() > 0.1 || b[2] < 0.5 {
            continue;
        }
        let a = after.positions()[i];
        let d = [a[0] - b[0], a[1] - b[1], a[2] - b[2]];
        best = best.max((d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt());
    }
    best
}

fn drawing(accumulate: bool) -> Brush {
    Brush {
        verb: Verb::Draw,
        strength: 0.5,
        radius: R,
        accumulate,
        ..Brush::default()
    }
}

/// Uma varredura de `passes` passadas, e o quanto ela levantou o miolo.
fn measure(accumulate: bool, passes: usize) -> f32 {
    let base = sphere();
    let mut mesh = sphere();
    let mut stroke = SculptStroke::default();
    stroke.begin(&mesh);
    sweep(&mut mesh, &mut stroke, &drawing(accumulate), passes);
    lift(&base, &mesh)
}

/// **DESARMADO o pincel SE ESGOTA; ARMADO ele não tem limite** — o gate da
/// feature, e a frase é a do rótulo do original.
///
/// ⚠️ **As duas metades são um gate só de propósito:** a de cima sozinha ficaria
/// verde num motor que simplesmente enfraquecesse o pincel, e a de baixo
/// sozinha ficaria verde com o interruptor ligado por engano no default.
///
/// ⚠️ **E ele afirma a FORMA, não um número:** o desarmado é SUBLINEAR nas
/// passadas (cada uma rende menos que a anterior, porque o vértice se afasta do
/// proxy congelado e sai da pegada) e o armado é ~LINEAR (o proxy sobe junto).
#[test]
fn the_disarmed_brush_exhausts_itself_and_the_accumulate_has_no_limit() {
    let (off1, off2, off4) = (measure(false, 1), measure(false, 2), measure(false, 4));
    assert!(off1 > 1e-4, "a fixture não levantou nada");
    assert!(
        off4 < off1 * 3.0,
        "o DESARMADO não se esgotou: 1 passada {off1:.5}, 4 passadas {off4:.5} \
         (uma lei sem limite daria ~4x)"
    );
    assert!(
        off2 > off1,
        "o DESARMADO parou de progredir de todo: {off1:.5} -> {off2:.5}"
    );

    let (on1, on4) = (measure(true, 1), measure(true, 4));
    assert!(
        on4 > on1 * 3.0,
        "o ARMADO se esgotou: 1 passada {on1:.5}, 4 passadas {on4:.5} \
         (esperado ~4x)"
    );
    assert!(
        on4 > off4 * 1.3,
        "armado e desarmado terminam no mesmo lugar ({on4:.5} contra \
         {off4:.5}): o interruptor não está trocando a origem da distância"
    );
}

/// **A pilha é proporcional ao CAMINHO percorrido** — o gate que a lei do traço
/// sustenta.
///
/// ⚠️ **Ele não mede mais uma normalização, e é essa a mudança.** A versão
/// anterior dividia cada dab por `ACCUM_PER_DAB` para que a soma não dependesse
/// de quantos dabs o motor emitiu; hoje quem responde por isso é o **passo
/// exato do `walk`** (a metade 1: a lista de dabs é função do caminho,
/// `6,485 % → 0,000 %`), e a proporcionalidade é consequência dela.
#[test]
fn the_pile_is_proportional_to_the_path_walked() {
    let one = measure(true, 1);
    assert!(one > 1e-4, "a fixture não levantou nada");
    for passes in [2usize, 4] {
        let got = measure(true, passes);
        let want = one * passes as f32;
        assert!(
            (got / want - 1.0).abs() < 0.20,
            "{passes} passadas deram {got}, e uma passada vale {one} (esperado ~{want})"
        );
    }
}

/// **A PRIMEIRA passada é a MESMA nos dois modos** — o preço, medido.
///
/// ⚠️ **Este gate afirmava o contrário e o número o derrubou.** Ele dizia que a
/// primeira passada acumulada era *mais fraca* (o preço de trocar o pico do
/// envelope pela média da integral), e sob a lei nova ela mede **0,995×** — as
/// duas leis só divergem depois de a superfície ter se movido o bastante para o
/// proxy congelado e a posição viva discordarem. Um artista que arma o
/// interruptor **não perde nada na primeira pincelada**, e é isso que ele tem o
/// direito de esperar de um botão chamado *"sem limite por traço"*.
#[test]
fn the_first_pass_costs_the_artist_nothing() {
    let (off, on) = (measure(false, 1), measure(true, 1));
    assert!(off > 1e-4, "a fixture não levantou nada");
    assert!(
        (on / off - 1.0).abs() < 0.05,
        "a primeira passada difere entre os modos: desarmado {off:.5}, armado \
         {on:.5} — as duas leis têm de partir do mesmo lugar"
    );
}

/// **O interruptor é INERTE nos verbos que não carimbam** — byte a byte.
///
/// ⚠️ Quem tem âncora carrega o gesto TOTAL desde o pen-down; somar totais
/// multiplicaria o gesto pelo número de eventos de ponteiro. A porta que decide
/// é [`Verb::accumulates`], e este gate é o que prova que ela é honrada em vez
/// de meramente oferecida.
#[test]
fn the_switch_is_inert_for_every_verb_that_does_not_stamp() {
    for verb in Verb::ALL {
        // ⚠️ **A seleção é pelo GRIP, e não por `accumulates()`.** A primeira
        // versão filtrava pela própria função sob teste, e a mutação que a faz
        // devolver `true` para todo verbo esvaziava o laço — o gate passava
        // sobre NADA, verde por vácuo. O grip é o fato independente: se alguém
        // fizer o Grab acumular, este laço ainda o escolhe e a asserção sangra.
        if matches!(verb.grip(), Grip::Stamp) {
            continue;
        }
        let mut snap = [const { Vec::new() }; 2];
        for (k, accumulate) in [false, true].into_iter().enumerate() {
            let mut mesh = sphere();
            let mut stroke = SculptStroke::default();
            stroke.begin(&mesh);
            let brush = Brush {
                verb,
                accumulate,
                ..drawing(false)
            };
            // Um gesto com âncora: pega e puxa, duas vezes pelo mesmo caminho.
            for pull in [0.05_f32, 0.10] {
                stroke.dab(
                    &mut mesh,
                    &brush,
                    &Dab::pulling([0.0, 0.0, 1.0], R, [0.0, 0.0, -1.0], [pull, 0.0, 0.0]),
                    Symmetry::default(),
                );
            }
            snap[k] = mesh.positions().to_vec();
        }
        assert_eq!(
            snap[0], snap[1],
            "o accumulate mexeu no verbo {verb:?}, que não carimba"
        );
    }
}

/// **O limite por traço é UM RAIO DE PINCEL** — a forma fechada, medida.
///
/// ⚠️ **A versão anterior media IDEMPOTÊNCIA, e a família do carimbo abriu mão
/// dela** (ver `re_stamping_the_stamp_family_compounds…`). O que sobrevive da
/// intenção é a assinatura do *limite por traço*, e ela tem um número exato: a
/// distância do vértice ao proxy congelado É o quanto ele subiu, então o peso
/// zera quando essa subida alcança o raio. Furando o mesmo ponto por 400 dabs:
///
/// | dabs | desarmado | armado |
/// |---|---|---|
/// | 80 | 0,93 raios | 1,54 |
/// | 160 | 0,97 | 1,99 |
/// | 240 | 0,98 | 2,10 |
/// | 399 | **0,99** | **2,55** (e subindo) |
///
/// ⚠️ **Ele NUNCA esvazia o `last_moved`, e por isso o oráculo é a ALTURA.** O
/// peso tende a zero sem chegar lá (o falloff é contínuo), então um gate que
/// esperasse `last_moved().is_empty()` rodaria para sempre — foi a primeira
/// forma escrita, e o produto a recusou em 400 dabs com um vértice ainda vivo.
#[test]
fn the_disarmed_brush_saturates_at_one_radius_and_the_armed_one_passes_it() {
    // ⚠️ **A premissa aqui era *"o default tem de ser DESARMADO"*, e ela nunca
    // descreveu esta fixture** — o `drawing()` arma o interruptor
    // EXPLICITAMENTE nas duas pernas, então o default não entra no que este
    // gate mede. Ela ficou FALSA quando o default passou a sair da referência
    // (`Brush.js:16`), e derrubou um gate que estava certo. A premissa
    // verdadeira é a que está escrita agora: as duas pernas têm de diferir no
    // interruptor, senão o gate compara uma perna consigo mesma.
    assert_ne!(
        drawing(false).accumulate,
        drawing(true).accumulate,
        "as duas pernas têm de diferir no interruptor"
    );
    const DABS: usize = 400;
    let mut lift_radii = [0.0f32; 2];
    for (k, accumulate) in [false, true].into_iter().enumerate() {
        let base = sphere();
        let mut mesh = sphere();
        let mut stroke = SculptStroke::default();
        stroke.begin(&mesh);
        let brush = drawing(accumulate);
        // O centro fica onde o dedo está: no produto ele acompanha a superfície
        // viva, e é a distância ao PROXY que decide quem ainda está na pegada.
        for _ in 0..DABS {
            let c = live_centre(&mesh, 0.0);
            stroke.dab(
                &mut mesh,
                &brush,
                &Dab::at(c, R, [0.0, 0.0, -1.0]),
                Symmetry::default(),
            );
        }
        lift_radii[k] = lift(&base, &mesh) / R;
    }
    let (off, on) = (lift_radii[0], lift_radii[1]);
    assert!(
        off < 1.05,
        "o DESARMADO passou de um raio ({off:.2}): o limite por traço sumiu"
    );
    assert!(
        off > 0.8,
        "o DESARMADO parou cedo demais ({off:.2}): a fixture não chegou ao teto"
    );
    assert!(
        on > 1.5,
        "o ARMADO parou em {on:.2} raios: a origem viva não está sendo lida"
    );
}

/// **O CANAL DE MÁSCARA REPRODUZ O PORTE, PELA PORTA DO PRODUTO.**
///
/// ⚠️ **Este gate não existia, e é essa ausência que deixou a máscara derivar.**
/// Os treze kernels do [`crate::ref_kernels`] são gateados bit a bit contra o
/// JS EXECUTANDO (`tests/sculptgl_parity.rs`), mas *nada* afirmava que o
/// PRODUTO os reproduz — a paridade era MEDIDA por uma sonda `#[ignore]`, e uma
/// sonda que ninguém corre não impede regressão nenhuma. Foi exatamente a
/// pergunta do Enio sobre o carimbo (*"nada está idêntico! as mudanças já foram
/// linkadas?"*), e no canal a resposta era **não**: sob o envelope, dezesseis
/// esfregadas deixavam o canal onde uma o deixava.
///
/// ⚠️ **A POLARIDADE é convertida numa ponta só.** O nosso `1` é *protegido*, o
/// `free` da referência é *livre*, e `free = 1 − mask` é EXATO em `f32` (o
/// expoente não muda entre `m` e `1 − m` para `m ∈ [0,1]` a menos de um ULP no
/// próprio subtraendo). Comparar os dois crus mediria uma máscara contra o
/// complemento da outra.
///
/// ⚠️ **A BARRA É DERIVADA e cresce com as esfregadas**, porque a divergência
/// que resta é estrutural e conhecida: nós somamos o traço inteiro em `f32`
/// (`accum`) e escrevemos UMA vez; a referência escreve o canal a cada dab,
/// arredondando `N` vezes. São duas ordens de arredondada sobre a mesma soma —
/// medido, `5,96e-8` num dab e `8,3e-7` em dezesseis, ou seja **uma arredondada
/// por esfregada** e nunca mais que isso. Um nível de máscara visível é
/// `1/255 = 3,9e-3`; o pior caso aqui é **4700× menor**.
#[test]
fn the_mask_channel_reproduces_the_reference_kernel() {
    // Os defaults DE FÁBRICA da tool `Masking` do original (`Masking.js:13-16`)
    // — nenhum deles é escolhido aqui.
    const INTENSITY: f32 = 1.0;
    let radius = 0.45f32;
    let centre = [0.7f32.cos(), 0.0, 0.7f32.sin()];
    let len = (centre[0] * centre[0] + centre[2] * centre[2]).sqrt();
    let eye = [-centre[0] / len, 0.0, -centre[2] / len];
    let dab = Dab::at(centre, radius, eye);

    for n in [1usize, 2, 4, 16] {
        // --- O NOSSO, pela porta do produto.
        let mut mesh = sphere();
        let brush = Brush {
            verb: Verb::Mask,
            radius,
            strength: INTENSITY,
            ..Brush::default()
        };
        let mut st = SculptStroke::default();
        st.begin(&mesh);
        for _ in 0..n {
            st.dab(&mut mesh, &brush, &dab, Symmetry::default());
        }

        // --- O DELES, sobre a MESMA malha e a MESMA pegada.
        let clean = sphere();
        let pos: Vec<f32> = clean.positions().iter().flat_map(|p| *p).collect();
        let fp: Vec<u32> = (0..clean.vert_count() as u32)
            .filter(|&v| {
                let p = clean.positions()[v as usize];
                let d = [
                    p[0] - centre[0],
                    p[1] - centre[1],
                    p[2] - centre[2],
                ];
                d[0] * d[0] + d[1] * d[1] + d[2] * d[2] < radius * radius
            })
            .collect();
        assert!(!fp.is_empty(), "a fixture não tem pegada");
        let mut free = vec![1.0f32; clean.vert_count()];
        let c = [
            f64::from(centre[0]),
            f64::from(centre[1]),
            f64::from(centre[2]),
        ];
        for _ in 0..n {
            crate::ref_kernels::mask(
                &mut free,
                &pos,
                &fp,
                c,
                f64::from(radius) * f64::from(radius),
                f64::from(INTENSITY),
                f64::from(Brush::default().mask_hardness),
                true,
            );
        }

        // Uma arredondada de `f32` por esfregada, e nada além disso.
        let bar = n as f32 * f32::EPSILON;
        let ours = mesh.masks().expect("o canal foi pintado");
        let gap = fp
            .iter()
            .map(|&v| {
                let mine = crate::mask_ops::free_weight(ours[v as usize]);
                (mine - free[v as usize]).abs()
            })
            .fold(0.0f32, f32::max);
        assert!(
            gap <= bar,
            "{n} esfregada(s): o canal divergiu {gap:.3e} do porte (barra {bar:.3e})"
        );
    }
}

/// **E o CONTROLE: a curva do canal NÃO é a da geometria.**
///
/// ⚠️ Sem ele o gate acima é satisfeito por um produto que peça a curva errada
/// e por acaso concorde — e não concordaria, mas *"não concordaria"* é uma
/// afirmação minha, não um teste. A quártica da geometria vale `0,6875` a meio
/// raio e a do canal vale `0,3536`: quase o DOBRO, e é o que separa uma borda
/// de máscara apertada de uma borrada.
#[test]
fn the_channel_curve_is_not_the_geometry_curve() {
    let b = Brush::default();
    let mid = b.mask_weight(0.5);
    let geo = crate::Falloff::Plateau.weight(0.5);
    assert!(
        (mid - 0.353_553).abs() < 1e-5,
        "a curva do canal a meio raio é (1−0,5)^1,5 = 0,353553, e deu {mid}"
    );
    assert!(
        geo / mid > 1.8,
        "a curva da geometria tem de ser quase o dobro da do canal a meio raio, \
         e a razão deu {:.3}",
        geo / mid
    );
    // ⚠️ **E o topo da faixa é um DISCO DURO, não *"quase duro"*:** em
    // `hardness = 1` o expoente é ZERO e `x^0 == 1` em toda a pegada. É o que
    // torna o knob uma família contínua em vez de uma escolha entre curvas.
    let hard = Brush {
        mask_hardness: crate::MAX_MASK_HARDNESS,
        ..Brush::default()
    };
    for t in [0.0f32, 0.25, 0.5, 0.99] {
        assert_eq!(hard.mask_weight(t), 1.0, "o disco duro cedeu em t={t}");
    }
}

/// **O DEFAULT DO INTERRUPTOR SAI DA REFERÊNCIA, tool a tool.**
///
/// ⚠️ **Ele era `false` para os dezesseis, e isso era o pedido original do Enio
/// por responder:** *"Brush:Checkbox:Clay com accumulate **checado por
/// padrão**"*. O `Brush.js:16` do original ship `this._accumulate = true`, e a
/// tool `Brush` dele é a nossa **Draw E Clay** — o artista pegava o Clay e
/// tinha na mão uma ferramenta que a referência não tem.
///
/// ⚠️ **E a família do PLANO é mais forte que um default:** o `Flatten.js` não
/// declara `_accumulate`, e o kernel pergunta `this._accumulate === false` —
/// que em `undefined` é FALSO. `Flatten`/`Fill`/`Scrape` leem o vivo SEMPRE, e
/// nascer armado é o mais perto disso que um interruptor chega.
///
/// ⚠️ **O `Brush::default()` DERIVA do verbo dele em vez de repetir um
/// literal** — é isso que o segundo `assert` afirma, e sem ele os dois fatos
/// viveriam em dois lugares e o dia da divergência não teria testemunha.
#[test]
fn the_accumulate_default_is_the_references_tool_by_tool() {
    for verb in Verb::ALL {
        let armed = verb.default_accumulate();
        let expected = matches!(
            verb,
            Verb::Draw | Verb::Clay | Verb::Flatten | Verb::Fill | Verb::Scrape
        );
        assert_eq!(armed, expected, "{} nasceu errado", verb.label());
    }
    // Os que a referência NÃO arma: os dois que não têm o campo (Smooth/Mask),
    // os dois ganhos próprios (Pinch/Crease) e os quatro grips de gesto.
    for verb in [
        Verb::Smooth,
        Verb::Sharpen,
        Verb::Pinch,
        Verb::Magnify,
        Verb::Crease,
        Verb::Mask,
        Verb::Move,
        Verb::SnakeHook,
        Verb::Twist,
        Verb::LocalScale,
    ] {
        assert!(!verb.default_accumulate(), "{} nasceu armado", verb.label());
    }
    assert_eq!(
        Brush::default().accumulate,
        Verb::default().default_accumulate(),
        "o Brush default e o verbo default discordam sobre o interruptor"
    );
}
