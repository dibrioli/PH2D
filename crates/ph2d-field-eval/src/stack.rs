//! ⭐ **A PILHA DE MODIFICADORES** — o que um nó faz à forma dele depois de ela existir: casca,
//! afastamento, espelho, matriz, repetição radial e inclinação.
//!
//! # Por que ela saiu do `lib.rs`
//!
//! O `lib.rs` desta crate é a **ponte**: documento → árvore → malha. A pilha é uma resposta
//! completa e fechada dentro dela, com as três constantes medidas do [`taper`] ao lado — e o
//! arquivo passou dos **700** do gate de LOC da workspace. ⚠️ **A cura é partir para irmão, nunca
//! uma entrada na allowlist.**
//!
//! ⚠️ E partiu **duas** vezes: a família da dobra vive hoje na [`crate::stack_bend`], porque as
//! duas medições que a defendem (a parede e o piso de ponto) precisam de ser lidas por extenso.

use crate::ops;
use crate::stack_bend::{BEND_FOLD_MARGIN, bend, bend_curvature, bend_reach};
use crate::stack_mirror::dobra;
use crate::stack_taper::{taper, taper_divisor, taper_floor, taper_reach};
use fidget::context::Tree;
use ph2d_field::Unary;

/// ⭐ **A pilha de modificadores de um nó**, aplicada na ordem em que ela está.
///
/// ⚠️ **A ordem importa e é por isso que ela é uma lista**: encascar-e-afastar não é afastar-e-
/// encascar. `|f| − t` seguido de `− d` dá uma parede mais grossa; `f − d` seguido de `| | − t` dá
/// uma parede da mesma espessura noutro sítio. Um conjunto sem ordem teria de escolher uma em
/// silêncio.
pub(crate) fn stacked(inner: &Tree, mods: &[Unary], local: crate::bounds::Ball) -> Tree {
    let mut acc = inner.clone();
    // ⭐⭐ **O bordo anda AO LADO da árvore** (2026-08-30) — a torção precisa de saber quão longe do
    // eixo a peça chega **naquele ponto da pilha**, e um `Array` antes dela muda essa resposta.
    // ⚠️ A lei de cada passo é a do [`crate::bounds::step_mod`], e não uma segunda cópia dela.
    let mut ball = local;
    // ⛔⛔⛔ **HÁ UM DEFORMADOR DE ESPAÇO NESTA PILHA?** — e a pergunta é sobre a pilha INTEIRA, não
    // sobre o que já passou (2026-08-31).
    //
    // ⚠️ É a mesma lei que o `divisor` aprendeu três linhas acima, e ela mordeu de novo por
    // **falta de a aplicar aqui**: a caixa de recorte da marcha é o envelope do FIM da pilha, então
    // um deformador **posterior** alarga a região onde a repetição é avaliada — e ali a lei das duas
    // células deixa de bastar. Um `deformado` que só olha para trás vê o mundo de antes.
    //
    // Medido, os MESMOS modificadores com a ordem trocada:
    //
    // | pilha | `‖∇f‖` a `40³` |
    // |---|---:|
    // | `[Shell, Array, Twist]` (deformador **depois**) | **`2 224,31`** |
    // | `[Shell, Twist, Array]` (deformador **antes**) | `0,38` |
    // | `[Radial, Bend, Radial]` | **`507,09`** |
    // | `[Bend, Radial, Radial]` | `0,28` |
    //
    // ⇒ **`5 000×` de diferença, e a única coisa que muda é a ordem.**
    //
    // ⭐ **Uma pilha sem deformador nenhum continua byte-idêntica** — a bandeira fica `false` do
    // princípio ao fim, que é o caso de omissão. E onde a lei das duas células já estava certa, as
    // células a mais entram por `min` e **não mexem na superfície**: elas só revelam matéria que
    // estava a ser perdida.
    // ⭐⭐⭐ **E um ESPELHO fora da origem é da mesma família** — ver
    // [`crate::stack_mirror::desloca_a_seccao`], que traz a medição.
    let deformado = mods.iter().any(|m| {
        matches!(
            m,
            Unary::Twist { .. } | Unary::Bend { .. } | Unary::Taper { .. }
        ) || crate::stack_mirror::desloca_a_seccao(m)
    });
    for m in mods {
        acc = match *m {
            // ⭐ A casca inteira: o módulo de uma distância É a distância à mesma superfície vista
            // dos dois lados, e afastá-la meia espessura para cada lado dá a parede.
            Unary::Shell { thickness } => ops::offset(&acc.abs(), f64::from(thickness) * 0.5),
            Unary::Offset { distance } => ops::offset(&acc, f64::from(distance)),
            // ⭐ **Dobra do domínio**: `x → |x|`. O que existe de um lado passa a existir dos dois, e
            // o campo continua uma distância exata — não há costura a fechar, que é o mesmo motivo
            // de a booleana e a casca não poderem falhar.
            Unary::Mirror { offset } => {
                acc.remap_xyz(dobra(Tree::x(), offset), Tree::y(), Tree::z())
            }
            // ⭐ Os outros dois eixos, pela MESMA lei — ver [`ph2d_field::Unary::MirrorZ`] para a
            // cerca que caiu.
            Unary::MirrorY { offset } => {
                acc.remap_xyz(Tree::x(), dobra(Tree::y(), offset), Tree::z())
            }
            Unary::MirrorZ { offset } => {
                acc.remap_xyz(Tree::x(), Tree::y(), dobra(Tree::z(), offset))
            }
            // ⭐⭐⭐ **A LEI É A DE SEMPRE; O EIXO ENTRA POR FORA** — ver [`conjugado`], e note que
            // para o eixo de omissão ele é a **identidade ao bit**.
            Unary::Array {
                count,
                spacing,
                joint,
                axis,
            } => conjugado(&acc, axis, ph2d_field::mods::ARRAY_AXIS, |t| {
                array(t, count, f64::from(spacing), joint, deformado)
            }),
            Unary::Radial { count, joint, axis } => {
                conjugado(&acc, axis, ph2d_field::mods::RADIAL_AXIS, |t| {
                    radial(t, count, joint, deformado)
                })
            }
            // ⚠️ **O piso do `k` sai da bola LOCAL, e não da corrente** — ver [`taper_floor`].
            Unary::Taper { slope, axis } => {
                let piso = taper_floor(
                    f64::from(slope),
                    local.to_canonical(axis.shift_to(ph2d_field::mods::TAPER_AXIS)),
                );
                conjugado(&acc, axis, ph2d_field::mods::TAPER_AXIS, |t| {
                    taper(t, f64::from(slope), piso)
                })
            }
            // ⚠️ O `reach` é lido do bordo **antes** deste passo — é o pior raio-xy que o avaliador
            // toca, e é o que o lema do minorante pede (o máximo no SEGMENTO, e `r` é convexo).
            Unary::Twist {
                turns,
                lower,
                upper,
                falloff,
                axis,
            } => conjugado(&acc, axis, ph2d_field::mods::TWIST_AXIS, |t| {
                twist(
                    t,
                    f64::from(turns) * std::f64::consts::TAU,
                    f64::from(lower),
                    f64::from(upper),
                    f64::from(falloff),
                )
            }),
            Unary::Bend {
                turns,
                lower,
                upper,
                falloff,
                axis,
            } => {
                // ⛔⛔ **A BOLA TAMBÉM SE CONJUGA.** A parede e o alcance da dobra lêem
                // `center[0]`/`center[2]` — coordenadas **canónicas**. Com a bola do mundo, escolher
                // Y daria a parede medida no eixo errado, e uma parede pequena de menos **fura**.
                let s = axis.shift_to(ph2d_field::mods::BEND_AXIS);
                let canon = ball.to_canonical(s);
                let depois = crate::bounds::step_mod(ball, *m).to_canonical(s);
                conjugado(&acc, axis, ph2d_field::mods::BEND_AXIS, |t| {
                    bend(
                        t,
                        // ⛔ **A parede da dobra mede-se contra o ENVELOPE**, que é a caixa que a
                        // marcha percorre — ver [`bend_curvature`].
                        bend_curvature(turns, canon),
                        f64::from(lower),
                        f64::from(upper),
                        f64::from(falloff),
                        // A MESMA extensão que o divisor lê — ver [`step_divisor`].
                        bend_reach(depois),
                    )
                })
            }
        };
        ball = crate::bounds::step_mod(ball, *m);
    }
    // ⭐⭐⭐ **O DIVISOR SAI DA PORTA, e não de um segundo laço** — a [`stack_divisor`] é a mesma
    // função que o `field_shrink` chama, e ela faz a mesma travessia da bola que este laço faz.
    // *Enquanto eram duas contas, a curvatura da dobra podia ser uma aqui e outra lá — e era.*
    let divisor = stack_divisor(mods, local);
    if divisor == 1.0 {
        // ⭐ **IDENTIDADE AO BIT** numa pilha sem deformador — a divisão por `1,0` seria exacta em
        // `f64`, mas a árvore ganharia um nó, e o gate de forma da fita mede a árvore.
        acc
    } else {
        acc / Tree::constant(divisor)
    }
}

/// ⭐⭐⭐ **A LEI DE UM MODIFICADOR NOUTRO EIXO** — conjugação, e não um operador por eixo.
///
/// Cada modificador tem **uma** lei, escrita no eixo canónico dele ([`ph2d_field::mods::ARRAY_AXIS`]
/// e irmãos). O eixo que o artista escolhe entra por fora:
///
/// ```text
/// f_A = P⁻¹ ∘ f_canónico ∘ P
/// ```
///
/// com `P` a levar o eixo escolhido ao canónico. Como todos estes operadores são **remapeamentos do
/// domínio** (`remap_xyz`), isto é literalmente dois remapeamentos a mais — e para `A` igual ao
/// canónico é a **identidade ao bit** (nem um nó a mais na árvore), que é o que faz toda peça já
/// autorada continuar a mesma.
///
/// ⛔⛔ **`P` é CÍCLICA, nunca uma troca de dois eixos** — ver [`ph2d_field::Axis`]. Uma troca tem
/// determinante `−1`: ela espelha a peça, e uma torção espelhada **gira ao contrário**.
///
/// ⚠️ **O valor do campo não é tocado**, e é isso que torna isto barato: uma permutação de
/// coordenadas é uma isometria, logo não há divisor a pagar — ao contrário da inclinação, que
/// deforma.
fn conjugado(
    inner: &Tree,
    de: ph2d_field::Axis,
    para: ph2d_field::Axis,
    f: impl FnOnce(&Tree) -> Tree,
) -> Tree {
    let s = de.shift_to(para);
    if s == 0 {
        // ⭐ **IDENTIDADE AO BIT** no eixo de omissão — a mesma cerca do `divisor == 1.0` na
        // [`stacked`], e pela mesma razão: a árvore ganharia nós e todo golden mudaria de valor.
        return f(inner);
    }
    let c = [Tree::x(), Tree::y(), Tree::z()];
    let leva = |i: usize| c[i % 3].clone();
    // `P⁻¹` na entrada, `P` na saída — ver [`ph2d_field::Axis::to_canonical`], que é a MESMA
    // permutação aplicada ao centro da bola de bordo.
    let dentro = inner.remap_xyz(leva(s), leva(1 + s), leva(2 + s));
    f(&dentro).remap_xyz(leva(3 - s), leva(4 - s), leva(5 - s))
}

/// ⭐⭐⭐ **POR QUANTO UM MODIFICADOR ENCOLHE O CAMPO** — a lei, num sítio só.
///
/// Um deformador de espaço devolve um **minorante** da distância, e o preço é este número: o campo
/// vale `1/divisor` do que valeria. ⚠️ Ele é lido pela [`stacked`] (que o aplica) **e** pela
/// [`crate::field_shrink`] (que diz à marcha quantos passos a mais isso custa) — *uma lei com dois
/// leitores é uma porta; escrita duas vezes, são duas respostas que divergem.*
///
/// ⚠️ **A bola é a de ANTES deste passo**, como no [`crate::bounds::step_mod`]: é dela que a torção
/// tira o alcance do eixo.
/// ⚠️ **A `ball` é a do FIM da pilha**, e não a de antes deste passo — ver a nota na [`stacked`].
/// ⭐⭐⭐ **O DIVISOR DA PILHA INTEIRA — a porta, e a mesma travessia que a [`stacked`] faz.**
///
/// ⛔⛔⛔ **Ela existe porque havia DUAS contas** (2026-09-01): a [`stacked`] acumulava a dela num
/// laço e o `field_shrink` acumulava a dele noutro, e nada as obrigava a concordar. A dobra tira a
/// **curvatura** da bola de ANTES de cada passo, e o segundo laço só tinha o envelope: enquanto o
/// bordo era folgado as duas saturavam no tecto de `10` e concordavam **por acidente**; com o bordo
/// apertado nada satura e o campo passou a furar (`‖∇f‖ = 1,29` numa dobra de um clique).
///
/// # ⛔⛔ O DIVISOR DE UM PASSO MEDE-SE CONTRA A CAIXA QUE A MARCHA PERCORRE
///
/// E ela é o **ENVELOPE** da pilha, não a bola corrente nem a do fim (2026-08-30). O recorte da
/// marcha é a AABB da bola final (`Scene::clip`), e um modificador **posterior** pode aumentá-la: o
/// campo de um passo anterior passa a ser avaliado mais longe do que o bordo dele dizia. Medido:
/// `[Taper, Array]` lia `‖∇f‖ = 1,0572` **dentro do recorte** — e não é artefacto de grelha, porque
/// o número não muda de `ε = 1e-3` para `1e-5`. ⛔ E não é a bola do FIM: a repetição radial
/// **re-centra** no eixo, logo a pilha não é monótona — com a do fim, `[Taper, Radial]` foi a `730,5`.
///
/// ⇒ **duas bolas por passo**: o `fim` (o envelope, onde o passo é avaliado) e a `corrente` (onde a
/// árvore está quando aquele modificador é aplicado). *Escrever só uma é o que deixava as duas
/// contas divergirem.*
/// ⭐ **Os factores do divisor, um por passo** — a porta da sonda que mede a ESTRUTURA da folga.
///
/// ⚠️ Ela existe porque a pergunta *«o bound composto é o produto ou o máximo?»* não se responde com
/// o produto já colapsado num número. Ver `what_a_stack_of_deformers_costs_the_march`.
#[doc(hidden)]
#[must_use]
pub fn stack_divisor_factors(mods: &[Unary], local: crate::bounds::Ball) -> Vec<f64> {
    let fim = crate::bounds::envelope(local, mods);
    let mut corrente = local;
    let mut out = Vec::with_capacity(mods.len());
    for m in mods {
        out.push(step_divisor(*m, fim, corrente));
        corrente = crate::bounds::step_mod(corrente, *m);
    }
    out
}

pub(crate) fn stack_divisor(mods: &[Unary], local: crate::bounds::Ball) -> f64 {
    let produto: f64 = stack_divisor_factors(mods, local).iter().product();
    // ⭐⭐⭐ **E o bound da COMPOSIÇÃO entra por `min`** (2026-09-02) — ver [`crate::bounds_lip`].
    //
    // O produto acima cobra `σ(J₀)·…·σ(Jₙ)`, cada um no pior ponto da caixa **independentemente**
    // dos outros; a verdade é o `σ_max` do PRODUTO das matrizes. ⛔ O `min` é a cerca: a lei nova
    // nunca pode ser pior do que a que já defende o sítio, e ela **abstém-se** (`None`) em toda
    // pilha que não saiba modelar.
    match crate::bounds_lip::stack_lipschitz(mods, local) {
        Some(novo) if novo < produto => novo,
        _ => produto,
    }
}

fn step_divisor(m: Unary, ball: crate::bounds::Ball, corrente: crate::bounds::Ball) -> f64 {
    // ⛔⛔ **A BOLA LÊ-SE NO REFERENCIAL CANÓNICO DO MODIFICADOR** (Enio, 2026-08-31) — os três
    // `*_reach` abaixo lêem `center[0]`/`center[1]`/`center[2]` **por índice**, e um índice lido no
    // eixo errado dá um `R` pequeno de menos ⇒ um divisor pequeno de menos ⇒ o campo **fura**.
    // A permutação é a MESMA da [`conjugado`] e da [`crate::bounds::step_mod`].
    let s = crate::bounds::axis_shift_of(m);
    let ball = ball.to_canonical(s);
    let corrente = corrente.to_canonical(s);
    match m {
        // ⚠️ **A extensão é a de DEPOIS do passo**, como na dobra: o avaliador é preso à AABB da bola
        // já inclinada, e ela é maior. Ler a de antes deixava `[Array, Taper]` em `1,1438`.
        Unary::Taper { slope, .. } => taper_divisor(f64::from(slope), taper_reach(ball)),
        Unary::Twist { turns, .. } => {
            let k = f64::from(turns) * std::f64::consts::TAU;
            twist_sigma(k.abs() * axis_reach(ball).abs())
        }
        // ⭐ `σ = max(1, ρ/Rr) = max(1, 1/(1 − κ·W))` — o lado de DENTRO da dobra comprime-se no
        // espaço material, e é lá que o campo estica. A saturação da curvatura garante `κ·W < 1`.
        Unary::Bend { turns, .. } => {
            // ⛔⛔⛔ **A CURVATURA É A QUE A ÁRVORE USA, e não a que este envelope daria**
            // (2026-09-01). A [`stacked`] dobra por `bend_curvature(turns, bola_ANTES_desta_dobra)`;
            // este divisor lia `bend_curvature(turns, ENVELOPE)`, e as duas saturam contra paredes
            // diferentes. Enquanto o bordo era folgado **as duas batiam no tecto de `10`** e
            // concordavam por acidente; com o bordo apertado nada satura e a diferença aparece:
            // `κ_árvore = 1,554` contra `κ_divisor = 1,083`, `σ` verdadeiro `5,69` contra um
            // divisor de `2,35` ⇒ `‖∇f‖ = 1,29` numa dobra de um clique.
            //
            // ⭐ Com a MESMA `κ` a cobertura é demonstrável nos dois ramos do piso:
            // `σ = ρ/max(ρ − alcance_depois, ρ/10)`, e `alcance_depois ≤ alcance_envelope` ⇒
            // `σ ≤ 1/(1 − min(κ·W_env, margem))`, que é exactamente o que se cobra aqui.
            //
            // ⚠️ **Defeito PRÉ-EXISTENTE**, e a saturação dupla era o que o escondia.
            let k = bend_curvature(turns, corrente);
            // ⛔⛔ **A extensão é a de DEPOIS da dobra, e ler a de antes foi um vermelho medido.**
            // O avaliador é preso à AABB da bola **já dobrada** (`Scene::clip`), e ela é maior: com
            // `0,05` voltas o alcance vai de `0,736` para `1,64`, e o tecto verdadeiro de `ρ/Rr`
            // sobe de `1,30` para `2,06`. *Um divisor calculado numa caixa mais pequena do que a que
            // o raio percorre é um divisor pequeno de mais — e pequeno de mais fura.*
            let w = bend_reach(ball);
            (1.0 / (1.0 - (k.abs() * w).min(BEND_FOLD_MARGIN))).max(1.0)
        }
        // Os outros são exactos: eles lêem o campo, não o remodelam.
        Unary::Shell { .. }
        | Unary::Offset { .. }
        | Unary::Mirror { .. }
        | Unary::MirrorY { .. }
        | Unary::MirrorZ { .. }
        | Unary::Array { .. }
        | Unary::Radial { .. } => 1.0,
    }
}

/// Quão longe do **eixo Z local** a peça chega — o `R` de que a torção tira o divisor.
///
/// ⚠️ O centro de uma bola pode estar fora do eixo (um `Array` empurra-o), e o que conta é o ponto
/// mais distante: `‖(cx, cy)‖ + raio`.
///
/// # ⛔⛔⛔ RECUSA MEDIDA (2026-09-01): aqui a CAIXA não serve, e a esfera não é folga por acaso
///
/// A tentação é óbvia — a [`crate::bounds_mods::axis_distance`] sabe ler a caixa, e numa barra
/// `0,34 × 0,11 × 0,62` ela dá `0,505` contra os `0,717` da esfera, o que corta o divisor da torção
/// de `9,12` para `6,50` (**`1,4×` mais barato**). ⛔ E a peça **FURA**: a duas voltas,
/// `a_shorter_step_finds_exactly_the_same_piece` acusa **1 pixel** a mudar quando o passo é dividido
/// por quatro. *Um pixel é toda a prova de que precisa: um passo seguro nunca acha mais peça ao ser
/// encurtado.*
///
/// ⭐ **Bisectado, e a caixa de recorte está ILIBADA**: com o `aabb` a devolver as meias-extensões e
/// este alcance de volta à esfera, o gate é verde; com este alcance na caixa e o `aabb` no cubo, é
/// vermelho. ⇒ o defeito é deste número, e de mais nenhum.
///
/// ⚠️ **O mecanismo é o da nota do [`twist`]:** aquela tabela (`σ = t/2 + √(1 + t²/4)`, sem constante
/// ajustada) foi medida com `R` **a ser esta esfera**. A álgebra do valor singular é exacta *no
/// ponto*; o divisor é **uma constante** tirada em `R`, e a margem que fazia a constante bastar
/// vinha de `R` ser folgado. *Uma recusa medida responde à pergunta que lhe foi feita — e a que foi
/// feita ali tinha esta esfera dentro.*
///
/// ⇒ apertar isto exige **re-medir a tabela do [`twist`]**, não trocar uma linha.
pub(crate) fn axis_reach(b: crate::bounds::Ball) -> f64 {
    f64::from(b.center[0].hypot(b.center[1]) + b.radius.max(0.0))
}

/// ⭐⭐⭐ **A TORÇÃO (twist)** — o segundo operador de espaço deste módulo, e o irmão do [`taper`].
///
/// O ponto vai para o espaço **não torcido** rodando `(x, y)` por `−k·z`, e o valor volta como está:
/// ao contrário da inclinação, cada fatia de `z` sofre uma **rotação**, que é uma isometria — não há
/// escala para desfazer.
///
/// # ⚠️ Onde ela deixa de ser uma distância, e o tecto EXACTO disso
///
/// O jacobiano do mapa inverso tem as duas primeiras colunas ortonormais e a terceira igual a
/// `(k·q_y, −k·q_x, 1)` — o termo que a rotação ganha por variar com `z`. Com `t = k·r`
/// (`r = √(x²+y²)`, que a rotação preserva), a matriz `JᵀJ` restringida ao plano que importa é
/// `[[1, t], [t, 1 + t²]]`, e o maior valor singular sai em forma fechada:
///
/// ```text
/// σ_max(J) = t/2 + √(1 + t²/4)
/// ```
///
/// ⚠️ **E ele é MAIOR do que o `√(1 + t²)` que a intuição sugere** — `1,618` contra `1,414` em
/// `t = 1`. *A derivação à mão do irmão já tinha sido refutada uma vez por medir a coisa errada; aqui
/// a álgebra fecha, e a tabela confirma.*
///
/// # ⛔⛔ E a MEDIÇÃO refutou a FORMA do divisor, não apenas a constante
///
/// Dividir por `σ_max(k·r)` **no ponto** parece mais apertado e é **pior**: o divisor varia com o
/// ponto e a derivada dele reentra em `∇(f/d) = ∇f/d − f·∇d/d²`, e o segundo termo cresce **com o
/// próprio divisor**. Medido a uma volta por unidade, com a margem a subir:
/// `1,78 · 2,11 · 2,32 · 2,51 · 2,55` — *subir a margem PIORA*.
///
/// ⭐ O divisor **constante** `σ_max(k·R)` — com `R` o alcance do eixo, lido do bordo — não tem
/// gradiente próprio, e a tabela fecha **sem constante ajustada**:
///
/// | voltas/un | `σ(k·R)` | `‖∇f‖` |
/// |---:|---:|---:|
/// | 0,05 | `1,1421` | `0,9617` |
/// | 0,30 | `2,0802` | `0,8167` |
/// | 1,00 | `5,5129` | `0,7068` |
/// | 2,00 | `10,7559` | `0,7039` |
///
/// ⚠️ **É a diferença com o [`taper`], e ela é do OPERADOR e não do cuidado:** ali o divisor tem de
/// ser medido porque a escala varia com `y` **dentro** da conta; aqui a álgebra fecha e a medição só
/// confirma. *Uma constante ajustada é o que se escreve quando a demonstração não fecha — e quando
/// ela fecha, escrevê-la à mesma seria esconder que fechou.*
/// ⭐⭐⭐ **O OMBRO da banda** — um `clamp` cujas quinas são arredondadas, com meia-largura `w`.
///
/// # ⛔ O report que o obrigou
///
/// Enio, 2026-08-30, com a seta na dobra: *«smoke ok mas muito dura a transição»*.
///
/// ⚠️ **E a régua não era a normal.** Medida atravessando o fim da banda, ela é **contínua**
/// (`0,787°` a um passo de `0,005`, exactamente proporcional ao passo ⇒ sem salto). O que salta é a
/// **CURVATURA**: o giro da normal por unidade de altura passa de `0,0` para `157,3 °/un` de um
/// lado ao outro. *É a mesma lei que a junção tangente deste repo já pagou — G1 sem ser G2 —, e o
/// que o olho lê como quina é a taxa, não o ângulo.*
///
/// # A conta, e por que ela é de graça
///
/// `soft_clamp = smin(smax(z, lo, w), hi, w)` com o `smin`/`smax` polinomial. A derivada de cada um
/// vive em `[0, 1]` (`1 − h/2` de um lado, `h/2` do outro), logo o **declive nunca passa de 1** e o
/// tecto `σ` da torção **não se mexe** — o ombro não custa um passo de marcha.
///
/// ⚠️ **A meia-largura é limitada a metade da banda**: acima disso os dois ombros misturam-se e o
/// `smin`/`smax` come o meio da rampa, que é o operador a mentir sobre o ângulo total.
pub(crate) fn soft_clamp(z: &Tree, lo: f64, hi: f64, w: f64) -> Tree {
    let meia = (hi - lo).abs() * 0.5;
    let w = w.min(meia);
    if w <= 0.0 || !w.is_finite() {
        return z.clone().max(lo).min(hi);
    }
    let suave = |a: Tree, b: f64, cima: bool| {
        let d = (a.clone() - Tree::constant(b)).abs();
        let h = (Tree::constant(w) - d).max(0.0) * Tree::constant(1.0 / w);
        let corda = h.square() * Tree::constant(w * 0.25);
        if cima {
            a.max(b) + corda
        } else {
            a.min(b) - corda
        }
    };
    suave(suave(z.clone(), lo, true), hi, false)
}

pub(crate) fn twist(inner: &Tree, k: f64, lower: f64, upper: f64, falloff: f64) -> Tree {
    if k == 0.0 || !k.is_finite() || !(lower.is_finite() && upper.is_finite()) {
        // ⭐ **IDENTIDADE AO BIT** — sem o curto-circuito a árvore ganharia `cos(0)`/`sin(0)` e o
        // valor mudaria por arredondamento em toda peça já gravada.
        return inner.clone();
    }
    // ⚠️ **A BANDA é um `clamp` do `z` que entra no ÂNGULO**, e não um corte no campo: fora dela a
    // peça roda como corpo rígido (o ângulo congela), que é o que as quatro referências fazem. Um
    // corte no campo partiria a peça em três sólidos.
    let banda = soft_clamp(
        &Tree::z(),
        lower.min(upper),
        upper.max(lower),
        falloff.max(0.0),
    );
    let angle = banda * Tree::constant(-k);
    let (c, s) = (angle.clone().cos(), angle.sin());
    let (x, y) = (Tree::x(), Tree::y());
    let untwisted = inner.remap_xyz(
        x.clone() * c.clone() - y.clone() * s.clone(),
        x * s + y * c,
        Tree::z(),
    );
    // ⚠️ **Sem dividir**: o divisor é acumulado e aplicado uma vez no fim da pilha — ver [`stacked`].
    untwisted
}

/// O tecto espectral do jacobiano do mapa inverso da torção, em `t = k·r`. Ver [`twist`].
#[must_use]
pub(crate) fn twist_sigma(t: f64) -> f64 {
    t * 0.5 + (1.0 + t * t * 0.25).sqrt()
}

/// A mesma lei com o divisor PONTUAL — a porta que a varredura refutou. Ver [`TWIST_SAFETY`].
pub(crate) fn twist_with(inner: &Tree, k: f64, safety: f64) -> Tree {
    if k == 0.0 || !k.is_finite() {
        return inner.clone();
    }
    let angle = Tree::z() * Tree::constant(-k);
    let (c, s) = (angle.clone().cos(), angle.sin());
    let (x, y) = (Tree::x(), Tree::y());
    let untwisted = inner.remap_xyz(
        x.clone() * c.clone() - y.clone() * s.clone(),
        x.clone() * s + y.clone() * c,
        Tree::z(),
    );
    // `t = k·r`, com o `r` do ponto — a rotação preserva-o, então tanto faz ler antes ou depois.
    let t = crate::ops::safe_sqrt(x.square() + y.square()) * Tree::constant(k.abs());
    let sigma = t.clone() * Tree::constant(0.5)
        + crate::ops::safe_sqrt(Tree::constant(1.0) + t.square() * Tree::constant(0.25));
    untwisted / (Tree::constant(1.0) + (sigma - Tree::constant(1.0)) * Tree::constant(safety))
}

/// ⭐ **A matriz radial**: `count` cópias em coroa, em torno do **Z**.
///
/// A conta é a mesma ideia da linear numa coordenada diferente: em vez de dobrar o `x`, dobra-se o
/// **ângulo**. Leva-se o ponto para a fatia dele (`θ − Δ·k`, com `Δ = 2π/count`) e avalia-se **uma**
/// forma — uma coroa de 32 custa o mesmo que uma de 2.
///
/// ⚠️ **Duas fatias**, pelo mesmíssimo motivo da linear: com uma só, uma forma que transborde a
/// fatia faz o campo **superestimar**, e superestimar é o que faz a marcha de raios saltar por cima
/// da superfície. Ver [`array`], onde o mecanismo está escrito por extenso.
///
/// ⚠️ **No eixo (`x = y = 0`) não há ângulo**, e é por isso que a conta não divide por `r`: ela
/// reconstrói o ponto por `r·cos θ'` / `r·sin θ'`, e em `r = 0` isso é a origem — a resposta certa,
/// sem caso especial e sem `NaN`.
/// ⭐⭐⭐ **QUANTAS FATIAS A JANELA TEM DE OLHAR de cada lado** — e o número é MEDIDO, sobre a faixa
/// inteira de `count`.
///
/// # ⛔⛔ O defeito que ela cura
///
/// A janela `[raw−n, raw+n]` **desliza com o ponto**. Se uma cópia de fora dela ainda puder ser a
/// mais próxima, o `min` troca de membros quando `raw` salta e o campo **descontinua** — e o que o
/// artista vê é a peça **estilhaçada**, com lascas soltas e buracos, a dois cliques do nascimento.
/// `[Taper, Radial]` media `‖∇f‖ = 730,5`, dívida desde a W18.
///
/// # ⚠️ A derivação geométrica está ERRADA, e a medição é que o disse
///
/// A conta óbvia — meia-largura angular `asin(R/d)`, e `π` quando a pegada contém o eixo — dá
/// `count/2` para toda forma nascida na origem, que é **toda** forma (a pilha corre em coordenadas
/// locais, antes da pose). Isso custa `count` avaliações da subárvore: medido a `640×360`,
/// **`79,4 ms`** num `taper + radial 64` contra `2 ms` sem deformador.
///
/// ⭐ E é conservador de mais. A matriz medida (`‖∇f‖` dentro do recorte, caixa `0,35³` com
/// `Taper 0,6`, grelha `40³`):
///
/// | janela | `c=5` | `c=6` | `c=7` | `c=10` | `c=12` | `c≥16` |
/// |---|---:|---:|---:|---:|---:|---:|
/// | `n = 1` | `561,6` | `730,5` | `1 327,5` | `1 198,7` | `3 684,7` | `0,47` |
/// | `n = 2` | `0,68` | `0,69` | `736,3` | `1 562,0` | `10 698,9` | `0,64` |
/// | **`n = 3`** | **`0,68`** | **`0,69`** | **`0,60`** | **`0,68`** | **`0,67`** | **`0,64`** |
///
/// ⚠️ **A exigência NÃO é monótona em `n`** (a `c=12` o `n=2` é pior que o `n=1`) nem em `count` (a
/// `c≥16` as cópias ficam tão densas que a união é quase um sólido de revolução e qualquer fatia
/// responde o mesmo). *Uma lei derivada de geometria não descreve isto; a varredura descreve.*
///
/// ⇒ `3` é o menor que limpa **toda** a faixa `3..=64` (o `MAX_ARRAY_COUNT` inteiro) em três
/// deformadores — `Taper 0,6`, `Taper` no máximo e `Twist`. É uma barra de corpus sobre um domínio
/// **FECHADO**, e o gate varre-o.
///
/// ⚠️ **Tecto em `count/2`**: `wedge(k)` e `wedge(k + count)` rodam o mesmo ângulo, então além de
/// meia volta as fatias repetem-se.
const RADIAL_WINDOW: u32 = 3;

fn radial(inner: &Tree, count: u32, joint: ph2d_field::Joint, deformado: bool) -> Tree {
    if count <= 1 {
        return inner.clone();
    }
    let step = std::f64::consts::TAU / f64::from(count);
    let d = Tree::constant(step);
    let r = crate::ops::safe_sqrt(Tree::x().square() + Tree::y().square());
    let theta = Tree::y().atan2(Tree::x());
    let raw = (theta.clone() / d.clone()).round();
    // A fatia vizinha é a do lado para onde o ponto pende — mesma lei da linear.
    let toward = theta.clone() / d.clone() - raw.clone();
    let other = raw.clone() + toward.compare(Tree::constant(0.0));
    let raw_mais = raw.clone() + Tree::constant(1.0);
    let wedge = |k: Tree| {
        let t = theta.clone() - d.clone() * k;
        inner.remap_xyz(r.clone() * t.clone().cos(), r.clone() * t.sin(), Tree::z())
    };
    // ⛔⛔ **No CENTRO EXACTO de uma fatia o `compare` devolve `0`**, e ali a "vizinha" é a própria
    // cópia. Com `min` isso é inofensivo; com a junta ligada a superfície move-se, e o centro de uma
    // fatia é precisamente onde a superfície daquela cópia passa. O portão é o mesmo da matriz.
    let distinta = crate::ops_joint::distinct_copies(&raw, &other);
    let duas = crate::ops_joint::union_between_copies(
        &wedge(raw.clone()),
        &wedge(other),
        joint,
        &distinta,
    );
    if !deformado {
        // ⭐ **Byte-idêntico para toda peça sem deformador** — que é o caso de omissão.
        return duas;
    }
    // ⛔⛔ **A TERCEIRA fatia, e ela existe por um defeito MEDIDO** (auditoria de 2026-08-30).
    //
    // As duas fatias bastam enquanto a forma é a mesma vista de qualquer lado. Um deformador de
    // espaço **antes** da repetição roda a secção para fora da própria fatia e torna-a **quiral**: a
    // vizinha do lado `y < 0` é a `−60°` e a do lado `y > 0` é a `+60°`, e numa forma quiral essas
    // duas não são a mesma coisa. O `min` de duas salta.
    //
    // Medido em dois cliques do artista (defaults de nascimento, `Twist` e depois `Radial`):
    // `‖∇f‖ = 40,0064` **dentro da caixa de recorte**, com o campo a saltar de `0,0035` para
    // `0,0207` entre dois pontos a `0,0005` um do outro — e `21` pixels a mudar quando o passo é
    // dividido por oito. ⚠️ **E é família:** `[Taper, Radial]` dá `37,3158`, e isso é dívida desde a
    // W18.
    //
    // ⚠️ **Só quando um deformador passou**: a terceira fatia custa mais uma cópia da forma na
    // árvore, e cobrá-la a quem não a precisa seria o caminho lento a mandar no rápido.
    let _ = raw_mais;
    // ⭐ **Quantas** — ver [`RADIAL_WINDOW`]. Com `n = 1` isto é exactamente as duas fatias
    // vizinhas que a wave anterior já olhava.
    let n = RADIAL_WINDOW.clamp(1, (count / 2).max(1));
    let mut acc = duas;
    for k in 1..=i64::from(n) {
        #[allow(clippy::cast_precision_loss)]
        let f = k as f64;
        acc = acc
            .min(wedge(raw.clone() - Tree::constant(f)))
            .min(wedge(raw.clone() + Tree::constant(f)));
    }
    acc
}

/// ⭐ **A matriz linear**: `count` cópias espaçadas de `spacing` no X, **sem N cópias da árvore**.
///
/// A conta é a dobra do domínio: leva-se o ponto para a célula dele (`x − s·k`, com `k` o índice da
/// célula preso a `[0, count−1]`) e avalia-se **uma** forma. É a razão de uma matriz de 64 custar o
/// mesmo que uma de 2 — numa malha ela custaria 64 vezes a geometria.
///
/// # ⚠️ Por que DUAS células, e não uma
///
/// A receita clássica (`opRepLim`) olha só a célula do ponto, e ela **superestima** a distância
/// quando a forma transborda a célula: existe uma cópia vizinha mais perto do que a da célula, e o
/// campo não a vê. Superestimar é o erro **caro** numa marcha de raios — o passo salta por cima da
/// superfície, e o sintoma é a peça com buracos, não um erro.
///
/// Olhar a célula do ponto **e a vizinha do lado para onde ele pende** custa duas avaliações da
/// subárvore e devolve a distância exata enquanto a forma couber em **1,5 células**. ⛔ Acima disso
/// o bound volta, e a cura é olhar três — que é o dobro do custo por um caso que o nascimento da
/// matriz (espaçamento = 2× a peça) já põe fora de alcance.
fn array(
    inner: &Tree,
    count: u32,
    spacing: f64,
    joint: ph2d_field::Joint,
    deformado: bool,
) -> Tree {
    if count <= 1 || spacing <= 0.0 || !spacing.is_finite() {
        return inner.clone();
    }
    let s = Tree::constant(spacing);
    let last = f64::from(count - 1);
    // O índice da célula, preso à matriz: `clamp(round(x/s), 0, count−1)`.
    let raw = (Tree::x() / s.clone()).round();
    let k = raw.max(Tree::constant(0.0)).min(Tree::constant(last));
    // ⚠️ **A vizinha é a do lado para onde o ponto PENDE**, e não uma fixa: com o sinal errado a
    // segunda avaliação cai na mesma célula metade das vezes e o gate passaria sem nada a defender.
    let toward = Tree::x() / s.clone() - k.clone();
    let neighbour = (k.clone() + toward.compare(Tree::constant(0.0)))
        .max(Tree::constant(0.0))
        .min(Tree::constant(last));
    let neighbour_mais = k.clone() + Tree::constant(1.0);
    let cell = |idx: Tree| inner.remap_xyz(Tree::x() - s.clone() * idx, Tree::y(), Tree::z());
    // ⛔⛔ **NAS PONTAS DA MATRIZ o `clamp` devolve a PRÓPRIA célula** (e no centro de uma célula o
    // `compare` faz o mesmo), e uma mistura de uma cópia consigo mesma move a superfície
    // (`blend(a, a) ≠ a`). O portão é `|vizinha − própria|` — ver [`crate::ops_joint`].
    let distinta = crate::ops_joint::distinct_copies(&k, &neighbour);
    let duas = crate::ops_joint::union_between_copies(
        &cell(k.clone()),
        &cell(neighbour),
        joint,
        &distinta,
    );
    if !deformado {
        // ⭐ **Byte-idêntico para toda peça sem deformador** — que é o caso de omissão.
        return duas;
    }
    // ⛔ **A TERCEIRA célula, pela MESMA razão da terceira fatia do [`radial`]** (2026-08-30): a lei
    // das duas células é exacta enquanto a forma cabe em ~1,5 delas, e um deformador antes da matriz
    // alarga a pegada. Medido: `[Taper, Array]` dava `‖∇f‖ = 1,0572` dentro da caixa de recorte.
    //
    // ⚠️ *Achar uma metade de uma família é motivo para procurar as outras* — e a outra era esta.
    duas.min(cell(
        (k - Tree::constant(1.0))
            .max(Tree::constant(0.0))
            .min(Tree::constant(last)),
    ))
    .min(cell(
        (neighbour_mais)
            .max(Tree::constant(0.0))
            .min(Tree::constant(last)),
    ))
}
