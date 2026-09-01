//! ⭐⭐⭐ **A DOBRA** — a curvatura, a parede que a satura, o alcance que a mede, e o mapa.
//!
//! # Por que ela saiu da [`crate::stack`]
//!
//! A dobra é a resposta mais longa daquele arquivo: ela tem uma **parede com dois lados medidos**
//! (o gradiente diz que não paga, a imagem diz que paga), um **piso de ponto** que existe por causa
//! do corte de ramo do `atan2`, e um mapa cujo comentário vale mais que o código. Com os três
//! escritos por extenso o `stack.rs` passava dos **700** do gate de LOC da workspace.
//! ⚠️ **A cura é partir para irmão, nunca uma entrada na allowlist.**
//!
//! ⚠️ O [`super::soft_clamp`] fica lá: ele é **partilhado** com a torção, e mudá-lo de casa faria a
//! banda ter dois donos.

use crate::stack::soft_clamp;
use fidget::context::Tree;

/// ⭐⭐⭐ **A CURVATURA que a dobra de facto aplica, já SATURADA na parede da peça.**
///
/// ⛔ **A parede é do DOCUMENTO, não uma escolha:** acima de `κ·W = 1` (com `W` a meia-largura na
/// direcção do centro do arco) a matéria dobra-se sobre si própria, o mapa deixa de ser injectivo e
/// o campo devolve lixo **nos dois sentidos** — fura e deixa fantasma. Só quem vê a peça pode
/// impô-la, e é por isso que ela mora aqui e não num `MAX_*`: *uma vara fina aguenta muito mais
/// dobra do que um bloco atarracado, e um teto escrito à mão seria o caminho lento a mandar no
/// rápido.*
///
/// ⚠️ **Satura, não recusa** — é a lei da porta deste módulo (o `set_dim` do prisma já a paga).
///
/// # ⛔⛔⛔ A PAREDE FOI DESLIGADA E MEDIDA — ela paga-se, e quem o diz é a IMAGEM (2026-08-31)
///
/// A justificação original desta parede era um `‖∇f‖ = 1,72` que **já não existe**: aquele número
/// vinha do corte de ramo do `atan2` (ver [`bend`]), e curá-lo levou `[Bend]` sozinha a `0,8130`.
/// ⚠️ *Quem cura a causa de um número tem de reconferir todo limite que aquele número justificava.*
///
/// Desligada a saturação (`k` sem `clamp`) e medido A/B:
///
/// | fixtura | `‖∇f‖` com parede | sem parede | ponta em `+X`, `0,25`→`0,50` voltas |
/// |---|---:|---:|---|
/// | barra fina `0,10×0,10×0,80` | `0,14`–`0,17` | `0,10`–`0,12` | `0,3600` parado ⇢ `0,4650` → **passa do enquadramento** |
/// | bloco `0,35³` | `0,32`–`0,42` | `0,33`–`0,47` | `0,3750` parado ⇢ **passa do enquadramento** |
///
/// ⇒ pelo gradiente a parede **não compra nada** e custa a metade de cima do slider. ⛔⛔ **E pela
/// IMAGEM ela compra tudo:** sem ela, `the_bend_draws_what_an_honest_march_draws` na barra fina a
/// `1,0` volta dá **`478` de `1 610` pixels** em desacordo com a marcha honesta, com o pior desvio
/// em **`180,0°`** — a peça desenha-se do avesso onde o mapa se dobra sobre si próprio.
///
/// ⭐⭐⭐ **É a lição de 2026-08-30 lida ao contrário, e as duas juntas fazem a lei:** um gate de
/// gradiente diz *«pode furar»* e não diz *«fura»*; e diz *«não fura»* sem dizer *«desenha certo»*.
/// **Nenhuma das duas réguas é suficiente sozinha, e nenhuma manda na outra.**
///
/// ⇒ a parede **FICA**, e o recurso dela tem nome: a **injectividade do mapa** em `κ·W = 1`, onde a
/// matéria passa pelo centro do arco. ⏳ O preço — a metade de cima do slider a não mover a peça
/// numa barra fina — é real e **não está curado**; a cura publicada é o **ombro** (a mesma que a
/// torção usa), e é wave própria.
pub(crate) fn bend_curvature(turns: f32, ball: crate::bounds::Ball) -> f64 {
    let k = f64::from(turns) * std::f64::consts::TAU;
    let w = bend_wall(ball);
    if w <= 0.0 || !w.is_finite() {
        return k;
    }
    let tecto = BEND_FOLD_MARGIN / w;
    k.clamp(-tecto, tecto)
}

/// Quão longe a peça chega na direcção em que a dobra a comprime (o `X` local).
///
/// ⛔⛔ **Era `|cx| + raio`, e o raio não é a extensão de X** (2026-09-01, report do Enio
/// *«muitíssimo lento»*). Esta grandeza é a **parede** (`piso = ρ − W`) e é ela que decide se o
/// divisor `1/(1 − κW)` satura no tecto de `10`: numa pilha `[Bend, Twist, Taper]` o envelope tinha
/// raio `1,507` e extensão em X de `0,671` — **2,2×**, e bastava para pôr `κW` acima da margem e
/// cobrar `10` onde a conta honesta cobra `2,0`.
///
/// ⚠️ **A caixa é lícita aqui porque é ela que a marcha recorta** — ver [`crate::bounds::Ball::aabb`],
/// que desde a mesma data devolve as meias-extensões e não o cubo. *Ler a caixa com um recorte
/// cúbico seria uma parede dentro da região avaliada, e uma parede dentro do recorte fura.*
/// ⛔⛔⛔ **E ele mede o RECORTE, margem incluída** — não a caixa justa (2026-09-01).
///
/// Esta grandeza é a **parede**: o `piso = ρ − W` congela a secção além dela, e o divisor cobra
/// `ρ/piso`. Se `W` for menor do que a região que o avaliador percorre, a parede cai **dentro** dela
/// e o campo estica onde ninguém cobrou. Medido ao ler a caixa justa com a margem de `1 %` já
/// activa: `[Bend]` a `0,25`/`0,5`/`1,0` voltas dá `‖∇f‖ = 1,2699` e `[Array, Bend]` dá `1,5037`.
///
/// ⚠️ *Uma parede tem de estar no fim da região avaliada, e a região avaliada é a que a
/// [`crate::bounds_clip::march_clip`] devolve.* É a mesma lei que o `deformado` e o divisor já
/// tinham aprendido — **medir contra a caixa que a marcha percorre**, e não contra uma mais
/// pequena.
pub(crate) fn bend_reach(b: crate::bounds::Ball) -> f64 {
    let (lo, hi) = crate::bounds_clip::march_clip(b);
    f64::from(hi[0].abs().max(lo[0].abs()))
}

/// ⛔⛔⛔ **A PAREDE DA CURVATURA NÃO É O ALCANCE DO RECORTE** — e as duas leram o mesmo número até
/// 2026-09-01.
///
/// O [`bend_reach`] responde *«até onde o `X` vai dentro do recorte?»*, que é o que o **piso** e o
/// **divisor** precisam de saber. Esta responde a outra coisa: *«quanto pode esta peça dobrar antes
/// de deixar de ser uma peça?»* — e o `κ·W ≤ `[`BEND_FOLD_MARGIN`] que ela impõe só cobre a metade
/// **radial** do problema (o lado de dentro a colapsar no centro do arco).
///
/// ⛔ **Ele não cobre o ENROLAMENTO.** Uma barra `0,10 × 0,10 × 0,80` com a parede lida na caixa
/// tem `W = 0,10`, logo tecto `κ = 9,0`: a uma volta pedida (`κ = 6,28`) ela sai **sem saturar**,
/// com `ρ = 0,159` e um eixo de `1,6` de comprimento — `10,05 rad`, **1,6 voltas**. A peça
/// atravessa-se a si própria e o mapa deixa de ser injectivo.
///
/// ⚠️ **A esfera cobria isso por acidente**, porque numa barra ela é a ALTURA. Medido ao trocá-la
/// pela caixa: o CONTROLE de `the_bend_draws_what_an_honest_march_draws` cai — a «barra forte»
/// passa a desenhar **898** pixels de interior contra os milhares que a fixtura exige, que é a peça
/// enrolada sobre si mesma.
///
/// ⇒ fica a esfera, e **o tecto morto do slider `Turns` continua ABERTO** (`CLAUDE.md` §5): curá-lo
/// é escrever a cerca que falta — `κ · comprimento_do_eixo < 2π` —, não trocar este número.
fn bend_wall(b: crate::bounds::Ball) -> f64 {
    f64::from(b.center[0].abs() + b.radius.max(0.0))
}

/// Quanto da parede do vinco a dobra pode usar.
///
/// ⚠️ **Não é um épsilon de gosto:** em `κ·W = 1` o lado de dentro colapsa no centro do arco e o
/// divisor `1/(1−κW)` vai a infinito. Nove décimos deixa a dobra ir bem além do que um artista pede
/// (um `U` fechado) e mantém o divisor abaixo de `10`.
/// # ⛔⛔⛔ E ELE TEM PREÇO, medido em 2026-08-31 (report do Enio: *«algumas combinações muito lentas»*)
///
/// Ele fixa **dois** números de uma vez: quanta dobra a peça aceita **e** quanto a marcha paga. O
/// divisor da dobra é `1/(1 − margem)` no instante em que a parede morde — logo `0,9` cobra **`10×`
/// em toda dobra, sempre**. Varrido (barra `0,10 × 0,10 × 0,80`, `160²`):
///
/// | margem | ponta a `0,25`/`0,50` voltas | divisor | passos/raio (`[Bend]`) |
/// |---:|---|---:|---:|
/// | **`0,90`** | `0,3800` | `10,00` | `72,2` |
/// | `0,88` | `0,3800` | `8,33` | `60,9` |
/// | `0,86` | `0,3600` | `7,14` | `52,7` |
/// | `0,80` | `0,3400` | `5,00` | `37,7` |
/// | `0,75` | `0,3400` | `4,00` | `30,6` |
/// | `0,60` | `0,2800` | `2,50` | `19,8` |
///
/// ⇒ de `0,90` para `0,75`: **`17 %` menos dobra por `2,4×` menos custo**. ⚠️ `0,80` é
/// **estritamente dominado** por `0,75` (mesma ponta, `19 %` mais caro).
///
/// ⛔ **Fica em `0,9` até o dono decidir** — é uma troca de PRODUTO (quanto a peça dobra contra
/// quanto ela custa a desenhar), e não um defeito. *Um número que só o dono pode escolher não se
/// escolhe sozinho.*
///
/// ⭐ E ele **não é a cura da lentidão**: o desperdício grande é o **produto** dos divisores
/// (`24,7×` de folga provada em `[Bend, Twist, Taper]`), e a cura dele é o divisor por região —
/// ver `ph2d_field_render::what_a_stack_of_deformers_costs_the_march`.
pub(crate) const BEND_FOLD_MARGIN: f64 = 0.9;

/// ⭐⭐⭐ **A DOBRA** — o eixo `Z` curva-se no plano `XZ`, com curvatura `κ`.
///
/// Mapa inverso, com `ρ = 1/κ` e o centro do arco em `(ρ, 0, 0)`:
///
/// ```text
/// a = ρ − X ;  b = Z ;  Rr = ‖(a, b)‖ ;  θ = atan2(b, a)
/// θc = banda(θ) ;  d = θ − θc
/// x = ρ − Rr·cos d ;  z = θc·ρ + Rr·sin d ;  y = Y
/// ```
///
/// # ⛔⛔⛔ A BANDA VIVE NO ÂNGULO — e escrevê-la no `z` extrudia a peça até ao INFINITO
///
/// A banda diz *«dobra só este troço; o resto segue recto»*. Recto quer dizer **rígido**: fora do
/// troço a peça é a mesma peça, transportada pela rotação que o arco tem na borda — e o eixo
/// **continua a andar**. Escrever a banda no `z` de entrada (`b = banda(Z)`) mata a segunda metade:
/// com `b` saturado, `x` e `z` deixam de depender de `Z` **de todo**, e o campo passa a ser
/// **constante ao longo do eixo**.
///
/// ⇒ a peça ganha uma **cauda semi-infinita**, e o campo dentro dela é uma planície logo à
/// superfície. Medido na peça exacta da foto do Enio (2026-08-31 — caixa `0,414 × 1,005 × 0,072`,
/// `turns 0,485`, banda `[−0,187, 0,048]`, `falloff 0,072`):
///
/// | `z` | campo em `x = −0,2` |
/// |---:|---:|
/// | `0,05` | `−0,00046` (dentro) |
/// | `0,50` | `+0,00047` |
/// | `2,00` | `+0,00047` |
/// | `20,00` | `+0,00047` |
///
/// ⛔ Uma marcha não atravessa isso: o passo é o valor do campo, e o valor não cresce. O traçado
/// queimava o orçamento inteiro a atravessar piche, **604 pixels** que a marcha honesta acerta
/// saíam vazios, e a peça aparecia **cortada por um plano vertical** — que foi o que ele
/// fotografou. *A cauda não se vê; o corte que a caixa de recorte lhe faz, sim.*
///
/// ⚠️ **E a borda da banda estava na grandeza errada:** `atan2(hi, a)` depende de `a`, logo do `x`.
/// A banda é declarada sobre o eixo **material**, então a borda dela é o ângulo `hi/ρ`, o mesmo
/// para toda a secção. A forma nova impõe isso por construção.
///
/// ⛔⛔ **Porque TODOS os gates ficaram verdes:** cada um deles — e o próprio nascimento do
/// modificador — dá à banda uma faixa que **cobre a peça inteira** (`[−2, 2]`). Aí `θc = θ`, `d = 0`
/// e as duas escritas são a mesma conta, bit a bit. *A banda é um parâmetro cuja faixa nenhuma
/// medição percorria: só o artista, arrastando o `From` e o `To`, saía do único ponto testado.*
///
/// ⚠️ **As duas linhas do bloco 2×2 do jacobiano são ORTOGONAIS**, com normas `1` e `ρ/Rr` — logo os
/// valores singulares são exactamente `{1, ρ/Rr, 1}` e o tecto é `max(1, ρ/Rr)`. *Ao contrário da
/// torção, a esticadela é anisotrópica, e por isso nenhuma correcção escalar a torna exacta.*
///
/// ⭐ **Fora da banda o mapa é uma ISOMETRIA** — na base ortonormal `(−cos d, sin d)`, `(sin d, cos d)`
/// o jacobiano fica `[[1, (ρC′/Rr)·sin d], [0, (1−C′) + (ρC′/Rr)·cos d]]`, e com `C′ = 0` isso é
/// ortogonal. ⇒ o divisor `max(1, ρ/Rr)` continua a ser um majorante em toda a parte, e **aperta**:
/// onde a dobra não age, o campo volta a ser uma distância exacta.
pub(crate) fn bend(inner: &Tree, k: f64, lower: f64, upper: f64, falloff: f64, reach: f64) -> Tree {
    if k == 0.0 || !k.is_finite() || !(lower.is_finite() && upper.is_finite()) {
        // ⭐ **IDENTIDADE AO BIT** — e aqui ela é obrigatória por mais uma razão: `κ = 0` dá
        // `ρ = ∞`, e a conta abaixo seria `0/0`.
        return inner.clone();
    }
    // ⛔⛔ **O SINAL, e ele custou um vermelho:** com `κ < 0` o centro do arco fica em `ρ < 0`, e as
    // DUAS contas trocam de sentido — `ρ − Rr` manda o ponto para `−6,4` em vez de `0`, e o
    // `atan2(0, negativo)` devolve `π` em vez de `0`. A peça **desaparecia** ao dobrar para um dos
    // lados, e só para um. *Quem escreve «e sabe para que lado» num gate é quem o apanha.*
    //
    // ⇒ dobra-se sempre para `+X` sobre o eixo **espelhado**, e espelha-se de volta: para `κ > 0` é
    // a identidade, e para `κ < 0` é a mesma conta na peça reflectida.
    let s = if k < 0.0 { -1.0 } else { 1.0 };
    let rho = (1.0 / k).abs();
    let a = Tree::constant(rho) - Tree::x() * Tree::constant(s);
    let b = Tree::z();
    // ⛔⛔ **O PISO DO RAIO, e ele é obrigatório** — a lei do [`TAPER_FLOOR`], pela mesma razão.
    //
    // A bola de bordo **cresce** com a dobra, e a marcha é presa à AABB dela: o recorte passa a
    // conter o **centro do arco**, onde `Rr → 0` e `σ = ρ/Rr → ∞`. Sem o piso, o campo devolve lixo
    // ali e o gradiente estoura — medido `‖∇f‖ = 1,0983` já **dentro** da caixa de recorte.
    //
    // ⭐ O piso é a parede da própria peça (`ρ − W`): dentro dela a conta é exacta, e além dela a
    // secção fica **congelada**, que é uma forma e não um defeito. Com ele o tecto passa a valer em
    // TODO o recorte, por maior que a caixa fique.
    let piso = (rho - reach.abs()).max(rho * (1.0 - BEND_FOLD_MARGIN));
    // ⛔⛔⛔ **O PISO É DO PONTO, e não só do raio** (2026-08-31) — e esta metade faltava.
    //
    // A caixa de recorte pode conter o **centro do arco e o que está para lá dele**: medido em
    // `[Twist, Bend]`, ela vai a `x = 1,4036` com `ρ = 1,3263`, logo `a = ρ − x` fica **negativo**.
    // Ali o `atan2(b, a)` atravessa o **corte de ramo**: `θ` salta de `+π` para `−π` ao cruzar
    // `b = 0`, e a banda clampa os dois lados em bordas opostas. O campo rasga por aritmética, não
    // por geometria.
    //
    // ⭐ Empurrar `a` para a parede é exactamente a lei que o piso já declarava — *«além dela a
    // secção fica congelada»* —, aplicada também ao ÂNGULO. Com `a ≥ piso > 0` o `atan2` vive em
    // `(−π/2, π/2)` e é contínuo, e `∂θ/∂b ≤ 1/piso` ⇒ o tecto de `∂z/∂b` é `ρ/piso`, que é
    // **exactamente** o divisor que a [`step_divisor`] já cobra.
    let a = a.max(piso);
    let rr = crate::ops::safe_sqrt(a.clone().square() + b.clone().square());
    // ⭐⭐⭐ **A BANDA VIVE NO ÂNGULO, e não no `z` do mundo** — ver o doc da função.
    //
    // ⚠️ As três constantes convertem-se dividindo por `ρ`, que é a mesma lei do `w ↦ w/ρ` da ida:
    // um comprimento ao longo do eixo é um arco. Para `κ → 0` as duas metades encolhem juntas
    // (`θ ≈ z/ρ`), e a banda continua a cobrir exactamente o mesmo troço da peça.
    let theta = b.atan2(a);
    let theta_c = soft_clamp(
        &theta,
        lower.min(upper) / rho,
        upper.max(lower) / rho,
        falloff.max(0.0) / rho,
    );
    // ⭐ **O QUE SOBRA DO ÂNGULO** — zero dentro da banda, e é isso que torna esta forma uma
    // reescrita **exacta** da anterior no caso que os gates medem: `cos 0 = 1` e `sin 0 = 0` em
    // IEEE, logo `x` e `z` saem bit a bit iguais aos de antes enquanto a banda cobrir a peça.
    let d = theta - theta_c.clone();
    let x = (Tree::constant(rho) - rr.clone() * d.clone().cos()) * Tree::constant(s);
    let z = theta_c * Tree::constant(rho) + rr * d.sin();
    inner.remap_xyz(x, Tree::y(), z)
}
