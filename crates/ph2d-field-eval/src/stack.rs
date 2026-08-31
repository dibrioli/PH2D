//! ⭐ **A PILHA DE MODIFICADORES** — o que um nó faz à forma dele depois de ela existir: casca,
//! afastamento, espelho, matriz, repetição radial e inclinação.
//!
//! # Por que ela saiu do `lib.rs`
//!
//! O `lib.rs` desta crate é a **ponte**: documento → árvore → malha. A pilha é uma resposta
//! completa e fechada dentro dela, com as três constantes medidas do [`taper`] ao lado — e o
//! arquivo passou dos **700** do gate de LOC da workspace. ⚠️ **A cura é partir para irmão, nunca
//! uma entrada na allowlist.**

use crate::ops;
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
    // ⛔⛔⛔ **O DIVISOR DE UM PASSO MEDE-SE CONTRA A CAIXA QUE A MARCHA PERCORRE** — e ela é a do
    // **FIM** da pilha, não a corrente (2026-08-30, e eu aprendi isto três vezes no mesmo dia).
    //
    // O recorte da marcha é a AABB da bola final (`Scene::clip`), e um modificador **posterior** pode
    // aumentá-la: o campo de um passo anterior passa a ser avaliado mais longe do que o bordo dele
    // dizia. Medido: `[Taper, Array]` lia `‖∇f‖ = 1,0572` **dentro do recorte** — e não é artefacto
    // de grelha, porque o número **não muda** de `ε = 1e-3` para `1e-5`.
    //
    // ⛔⛔ **E não é a bola do FIM, é o ENVELOPE**: a repetição radial re-centra no eixo, logo a
    // pilha não é monótona — com a do fim, `[Taper, Radial]` foi a `730,5`.
    let final_ball = crate::bounds::envelope(local, mods);
    // ⭐⭐⭐ **O DIVISOR ACUMULA E APLICA-SE UMA VEZ, NO FIM** (2026-08-30) — e isto é uma CURA.
    //
    // ⛔ Enquanto cada deformador dividia na hora, ele mudava a **unidade** do campo, e todo número
    // GEOMÉTRICO a jusante atravessava a conversão sem saber: `|f/L| − t/2` cruza zero onde
    // `|f| = L·t/2`, ⇒ **parede `L·t`**. Medido (`measure_the_wall_after_a_warp`, parede pedida
    // `0,060`): `Taper 1,0 + Shell` entregava **`0,180`** — `1 + 2·declive` exacto —, e
    // `Twist 1,0 + Shell` entregava **`0,337`**, `5,62×`.
    //
    // ⚠️ **É defeito PRÉ-EXISTENTE**: a inclinação carrega-o desde a W18, e o filete e o afastamento
    // depois dela erram pelo mesmo factor. A torção só o tornou grande.
    //
    // ⭐ Dividir no fim é correcto **e** mais apertado: o `Shell` (`|f|−t`), o `Offset` (`f−d`) e o
    // `min`/`max` preservam Lipschitz, então o tecto da pilha inteira continua a ser o produto dos
    // `σ` — e o número que o artista escreveu volta a valer o que diz.
    let mut divisor = 1.0f64;
    // ⛔⛔ **Já passou um DEFORMADOR DE ESPAÇO por aqui?** — a pergunta que a repetição radial tem de
    // fazer, e que ela não podia fazer sozinha. Ver [`radial`].
    let mut deformado = false;
    for m in mods {
        acc = match *m {
            // ⭐ A casca inteira: o módulo de uma distância É a distância à mesma superfície vista
            // dos dois lados, e afastá-la meia espessura para cada lado dá a parede.
            Unary::Shell { thickness } => ops::offset(&acc.abs(), f64::from(thickness) * 0.5),
            Unary::Offset { distance } => ops::offset(&acc, f64::from(distance)),
            // ⭐ **Dobra do domínio**: `x → |x|`. O que existe de um lado passa a existir dos dois, e
            // o campo continua uma distância exata — não há costura a fechar, que é o mesmo motivo
            // de a booleana e a casca não poderem falhar.
            Unary::Mirror => acc.remap_xyz(Tree::x().abs(), Tree::y(), Tree::z()),
            // ⭐ Os outros dois eixos, pela MESMA lei — ver [`ph2d_field::Unary::MirrorZ`] para a
            // cerca que caiu.
            Unary::MirrorY => acc.remap_xyz(Tree::x(), Tree::y().abs(), Tree::z()),
            Unary::MirrorZ => acc.remap_xyz(Tree::x(), Tree::y(), Tree::z().abs()),
            Unary::Array {
                count,
                spacing,
                joint,
            } => array(&acc, count, f64::from(spacing), joint, deformado),
            Unary::Radial { count, joint } => radial(&acc, count, joint, deformado),
            Unary::Taper { slope } => taper(&acc, f64::from(slope)),
            // ⚠️ O `reach` é lido do bordo **antes** deste passo — é o pior raio-xy que o avaliador
            // toca, e é o que o lema do minorante pede (o máximo no SEGMENTO, e `r` é convexo).
            Unary::Twist {
                turns,
                lower,
                upper,
                falloff,
            } => twist(
                &acc,
                f64::from(turns) * std::f64::consts::TAU,
                f64::from(lower),
                f64::from(upper),
                f64::from(falloff),
            ),
            Unary::Bend {
                turns,
                lower,
                upper,
                falloff,
            } => bend(
                &acc,
                // ⛔ **A parede da dobra mede-se contra o ENVELOPE**, que é a caixa que a marcha
                // percorre — ver [`bend_curvature`]. Com a bola local, `[Bend]` sozinha rasgava.
                bend_curvature(turns, ball),
                f64::from(lower),
                f64::from(upper),
                f64::from(falloff),
                // A MESMA extensão que o divisor lê — ver [`step_divisor`].
                bend_reach(crate::bounds::step_mod(ball, *m)),
            ),
        };
        divisor *= step_divisor(*m, final_ball);
        deformado |= matches!(
            m,
            Unary::Twist { .. } | Unary::Bend { .. } | Unary::Taper { .. }
        );
        ball = crate::bounds::step_mod(ball, *m);
    }
    if divisor == 1.0 {
        // ⭐ **IDENTIDADE AO BIT** numa pilha sem deformador — a divisão por `1,0` seria exacta em
        // `f64`, mas a árvore ganharia um nó, e o gate de forma da fita mede a árvore.
        acc
    } else {
        acc / Tree::constant(divisor)
    }
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
pub(crate) fn step_divisor(m: Unary, ball: crate::bounds::Ball) -> f64 {
    match m {
        // ⚠️ **A extensão é a de DEPOIS do passo**, como na dobra: o avaliador é preso à AABB da bola
        // já inclinada, e ela é maior. Ler a de antes deixava `[Array, Taper]` em `1,1438`.
        Unary::Taper { slope } => taper_divisor(f64::from(slope), taper_reach(ball)),
        Unary::Twist { turns, .. } => {
            let k = f64::from(turns) * std::f64::consts::TAU;
            twist_sigma(k.abs() * axis_reach(ball).abs())
        }
        // ⭐ `σ = max(1, ρ/Rr) = max(1, 1/(1 − κ·W))` — o lado de DENTRO da dobra comprime-se no
        // espaço material, e é lá que o campo estica. A saturação da curvatura garante `κ·W < 1`.
        Unary::Bend { turns, .. } => {
            let k = bend_curvature(turns, ball);
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
        | Unary::Mirror
        | Unary::MirrorY
        | Unary::MirrorZ
        | Unary::Array { .. }
        | Unary::Radial { .. } => 1.0,
    }
}

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
/// # ⛔⛔ A BOLA É A DO ENVELOPE, e ler a local era um vermelho de UM CLIQUE (2026-08-30)
///
/// A parede existe para o mapa inverso não passar pelo **centro do arco** (`ρ = 1/κ`), onde ele é
/// singular. Quem decide onde o mapa é avaliado não é a peça: é a **caixa de recorte da marcha**,
/// que é a AABB do envelope da pilha. Com a bola **local** a parede garantia `κ·W_local < 0,9` e o
/// avaliador ia até `W_env`, muito maior — e ali `κ·W > 1`: **o mapa dobra-se e o campo devolve
/// lixo**.
///
/// Medido, `‖∇f‖` dentro do recorte, numa caixa `0,35³` com a dobra **sozinha**:
///
/// | voltas | com a bola local | com o envelope |
/// |---|---:|---:|
/// | `0,05` | `0,83` | `0,83` |
/// | `0,12` | **`1,72`** | `0,49` |
/// | `0,25` | `0,95` | `0,48` |
/// | `0,50` | **`1,24`** | `0,48` |
///
/// ⭐⭐ **E o slider deixa de MORRER.** Com a bola local a saturação era fixa, e numa barra fina a
/// ponta parava em `0,3817` a partir de `0,25` voltas — `0,25`, `0,50` e `1,00` davam **a mesma
/// peça**. Com o envelope a parede acompanha a dobra e a ponta continua a andar
/// (`0,2633 → 0,3083 → 0,3417`).
///
/// ⚠️ **O preço, dito com número:** no meio da faixa a dobra fica **~30 % mais fraca** (a ponta de
/// uma barra fina a `0,12` voltas vai de `0,2983` para `0,2100`). A cura que devolveria a força sem
/// devolver o lixo é o **ombro** — a mesma que a torção já usa —, e é wave própria.
pub(crate) fn bend_curvature(turns: f32, ball: crate::bounds::Ball) -> f64 {
    let k = f64::from(turns) * std::f64::consts::TAU;
    let w = bend_reach(ball);
    if w <= 0.0 || !w.is_finite() {
        return k;
    }
    let tecto = BEND_FOLD_MARGIN / w;
    k.clamp(-tecto, tecto)
}

/// Quão longe a peça chega na direcção em que a dobra a comprime (o `X` local).
pub(crate) fn bend_reach(b: crate::bounds::Ball) -> f64 {
    f64::from(b.center[0].abs() + b.radius.max(0.0))
}

/// Quanto da parede do vinco a dobra pode usar.
///
/// ⚠️ **Não é um épsilon de gosto:** em `κ·W = 1` o lado de dentro colapsa no centro do arco e o
/// divisor `1/(1−κW)` vai a infinito. Nove décimos deixa a dobra ir bem além do que um artista pede
/// (um `U` fechado) e mantém o divisor abaixo de `10`.
const BEND_FOLD_MARGIN: f64 = 0.9;

/// ⭐⭐⭐ **A DOBRA** — o eixo `Z` curva-se no plano `XZ`, com curvatura `κ`.
///
/// Mapa inverso, com `ρ = 1/κ` e o centro do arco em `(ρ, 0, 0)`:
///
/// ```text
/// a = ρ − X ;  b = banda(Z) ;  Rr = ‖(a, b)‖
/// x = ρ − Rr ;  z = atan2(b, a)·ρ ;  y = Y
/// ```
///
/// ⚠️ **A banda entra no `b`, e não no `z` de saída**: é ela que faz a dobra agir só num troço, e
/// fora dele o resto da peça segue **recto** em vez de continuar a curvar.
///
/// ⚠️ **As duas linhas do bloco 2×2 do jacobiano são ORTOGONAIS**, com normas `1` e `ρ/Rr` — logo os
/// valores singulares são exactamente `{1, ρ/Rr, 1}` e o tecto é `max(1, ρ/Rr)`. *Ao contrário da
/// torção, a esticadela é anisotrópica, e por isso nenhuma correcção escalar a torna exacta.*
fn bend(inner: &Tree, k: f64, lower: f64, upper: f64, falloff: f64, reach: f64) -> Tree {
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
    let banda = soft_clamp(
        &Tree::z(),
        lower.min(upper),
        upper.max(lower),
        falloff.max(0.0),
    );
    let a = Tree::constant(rho) - Tree::x() * Tree::constant(s);
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
    let rr = crate::ops::safe_sqrt(a.clone().square() + banda.clone().square()).max(piso);
    let x = (Tree::constant(rho) - rr) * Tree::constant(s);
    let z = banda.atan2(a) * Tree::constant(rho);
    inner.remap_xyz(x, Tree::y(), z)
}

/// Quão longe do **eixo Z local** a peça chega — o `R` de que a torção tira o divisor.
///
/// ⚠️ O centro de uma bola pode estar fora do eixo (um `Array` empurra-o), e o que conta é o ponto
/// mais distante: `‖(cx, cy)‖ + raio`.
pub(crate) fn axis_reach(b: crate::bounds::Ball) -> f64 {
    f64::from(b.center[0].hypot(b.center[1]) + b.radius.max(0.0))
}

/// ⭐ **A inclinação (draft/taper)** — e o **primeiro operador deste módulo que não é exato**.
///
/// A secção transversal escala por `k(y) = 1 + slope·y`: o ponto vai para o espaço não-inclinado
/// (`x/k`, `y`, `z/k`) e o valor volta multiplicado por `k` — a mesma receita de duas metades que a
/// [`place`] usa para a escala uniforme, e pela mesma razão (sem a segunda metade o campo deixa de
/// ser uma distância).
///
/// # ⚠️ Por que ele não pode ser exato, e o que se paga em vez disso
///
/// A escala **varia com `y`**, e é essa variação que estraga: `∇g` ganha um termo de ordem
/// `slope·f` que a multiplicação por `k` não cancela. Perto da superfície (`f ≈ 0`) o erro
/// desaparece — que é onde a marcha mais precisa dele —, mas longe ele **superestima**, e
/// superestimar é o erro que faz o raio saltar por cima da peça.
///
/// A cura é dividir por `1 + |slope|`, o que torna o campo um **bound conservador**: ele nunca
/// passa da distância verdadeira, e a marcha continua correta. O preço é o número de passos, e ele
/// está medido em `measure_taper_cost` — é dali que sai o
/// [`ph2d_field::mods::MAX_TAPER_SLOPE`].
///
/// ⚠️ **O piso em `k` impede a inversão.** Em `y = −1/slope` a secção colapsa e, passando disso,
/// ela **vira do avesso** — a peça sairia com o interior para fora. Preso a [`TAPER_FLOOR`], o que
/// acontece além do ápice é a secção ficar congelada nele, que é uma forma e não um defeito.
fn taper(inner: &Tree, slope: f64) -> Tree {
    if slope == 0.0 || !slope.is_finite() {
        return inner.clone();
    }
    let k = (Tree::constant(1.0) + Tree::y() * Tree::constant(slope)).max(TAPER_FLOOR);
    let shrunk = inner.remap_xyz(Tree::x() / k.clone(), Tree::y(), Tree::z() / k.clone());
    // ⚠️ **A divisão saiu daqui e é feita UMA vez no fim da pilha** — ver [`stacked`], e a medição
    // que a obrigou. O factor continua a ser este, e continua a ser dele.
    shrunk * k
}

/// Por quanto a inclinação divide — ver [`TAPER_SAFETY`] e o doc do [`taper`].
///
/// ⛔⛔ **O `alcance` entrou em 2026-08-30, e ele faltava desde a W18.** A tabela que escolheu o
/// `TAPER_SAFETY` foi medida numa peça **centrada e de tamanho um**; o termo que ela corrige cresce
/// com a distância ao eixo `Y` (é `x·s/k²` que reentra no gradiente), logo uma peça larga — ou uma
/// **matriz** antes da inclinação — passa por cima dele. Medido: `[Array, Taper]` dava
/// `‖∇f‖ = 1,5049` **dentro da caixa de recorte**, alcançável em dois cliques.
///
/// ⚠️ **`max(1, alcance)`**: nunca menos do que a tabela original concedeu, senão a cura tornaria
/// uma peça pequena MENOS segura do que ela é hoje.
pub(crate) fn taper_divisor(slope: f64, alcance: f64) -> f64 {
    if slope == 0.0 || !slope.is_finite() {
        1.0
    } else {
        TAPER_SAFETY.mul_add(slope.abs() * alcance.abs().max(1.0), 1.0)
    }
}

/// Quão longe do **eixo Y** a peça chega — a inclinação escala `x` e `z` em torno dele.
///
/// ⚠️ Irmão do [`axis_reach`], e num eixo diferente: cada modificador nomeia o seu, que é a lei que
/// as primitivas deste módulo já seguem.
pub(crate) fn taper_reach(b: crate::bounds::Ball) -> f64 {
    f64::from(b.center[0].hypot(b.center[2]) + b.radius.max(0.0))
}

/// O menor fator de secção que a inclinação admite — ver [`taper`].
///
/// ⚠️ Não é um épsilon de gosto: abaixo dele o `x/k` explode e o campo passa a devolver números que
/// a marcha lê como "muito longe" dentro da própria peça. Um centésimo é duas ordens de grandeza
/// abaixo da secção nominal, o que põe o ápice bem fora de qualquer peça enquadrada.
const TAPER_FLOOR: f64 = 0.01;

/// Quanto o divisor da inclinação cresce por unidade de declive — **medido, e a primeira tentativa
/// estava errada**.
///
/// ⚠️ A conta que eu escrevi primeiro dividia por `1 + |slope|`, derivada à mão. A sonda
/// `measure_taper_cost` **refutou-a**: `‖∇f‖` continuava acima de 1 em todo o alcance, ou seja o
/// campo **superestimava** — exatamente a falha que a divisão existe para evitar.
///
/// | declive | `‖∇f‖` máx com `1 + s` | com `1 + 2s` |
/// |---|---|---|
/// | 0,25 | **1,12** ⛔ | 0,93 ✅ |
/// | 0,50 | **1,20** ⛔ | 0,90 ✅ |
/// | 1,00 | **1,30** ⛔ | 0,87 ✅ |
/// | 2,00 | **1,40** ⛔ | 0,84 ✅ |
///
/// *Uma derivação à mão é uma hipótese; a tabela é o facto.* O `2` é o degrau que a medição deu —
/// com ele `‖∇f‖ ≤ 1` em todo o alcance, que é a condição de a marcha não atravessar a peça.
const TAPER_SAFETY: f64 = 2.0;

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
fn soft_clamp(z: &Tree, lo: f64, hi: f64, w: f64) -> Tree {
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
