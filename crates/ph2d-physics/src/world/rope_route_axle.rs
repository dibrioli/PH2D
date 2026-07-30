//! **O EIXO COMPOSTO** — dois contatos, uma rotação, e o quociente que sai de
//! eliminá-la (ADR-0131, W-Pulley W-Weston).
//!
//! Módulo FILHO de [`super`] e não irmão: ele lê o [`RopeWheel`] e responde a UMA
//! pergunta que a rota faz — *quanto um metro de corda passa a valer depois deste
//! contato?* —, e o corte é por responsabilidade: lá mora a **geometria pura** (a
//! tangente comum, o arco, o lado do abraço), aqui a **lei da máquina** (que
//! contatos são o mesmo eixo, e o que a rotação compartilhada faz com o
//! orçamento). Nasceu do cap de 700 LOC quando a talha de Weston chegou.
//!
//! ⚠️ **É `pub use`ado pelo pai**, então `rope_route::crossing_gear` continua
//! sendo o caminho — nenhum consumidor aprendeu um módulo novo.

use super::RopeWheel;

/// **Os dois contatos de um eixo COMPOSTO**, nos índices que a rota lhes deu
/// (W-Weston) — `(primeiro, segundo)`.
///
/// `None` para toda roldana de eixo próprio (o caso de sempre), e também para um
/// par que **este modelo não sabe segurar**, por três razões que ficam nomeadas:
///
/// - **um contato só** com o número do eixo: não há rotação a eliminar, então ele
///   é uma roldana comum;
/// - **três ou mais** contatos no mesmo eixo: a eliminação deixa de dar UMA
///   restrição (cada contato acrescenta uma equação e o `θ` é um só), e um
///   orçamento pesado não a expressa — ver a nota do [`crossing_gear`];
/// - **o retorno por um diâmetro que não é MENOR** que o de ida. `r ≥ R` é uma
///   Weston montada ao contrário, e o sinal decide: com `r = R` a máquina está
///   **travada** (a carga não anda por mais que se puxe) e com `r > R` ela
///   **inverte** (puxar o esforço BAIXA a carga), que é um orçamento de
///   *diferença* e não de soma — `λ ≥ 0` deixaria de significar *a corda só
///   puxa*. Recusar o par aqui faz os dois contatos voltarem a ser roldanas
///   comuns, e é a **row** que impede o artista de chegar nisto.
///
/// ⚠️ Recusar por ARITMÉTICA e não por pânico: com `R > r > 0` o denominador de
/// [`crossing_gear`] é estritamente positivo, então não há divisão por zero a
/// guardar em lugar nenhum a jusante.
#[must_use]
pub fn axle_pair(wheels: &[RopeWheel], i: usize) -> Option<(usize, usize)> {
    let axle = wheels.get(i)?.axle;
    if axle == 0 {
        return None;
    }
    let mut on_this_axle = wheels.iter().enumerate().filter(|(_, w)| w.axle == axle);
    let (first, entry) = on_this_axle.next()?;
    let (second, ret) = on_this_axle.next()?;
    if on_this_axle.next().is_some() {
        return None;
    }
    (entry.radius > ret.radius && ret.radius > 0.0).then_some((first, second))
}

/// **Quanto um metro de corda passa a valer DEPOIS de cruzar o contato `i`** — a
/// porta única da lei de pesos da rota.
///
/// # A eliminação, que é a wave inteira
///
/// Um contato ENGATADO (a corda não escorrega nele — um dente de corrente, uma
/// canaleta que morde) transfere material de um segmento para o outro na taxa
/// `s·ρ·θ̇`, com `s` o lado do abraço. Chamando `S₀` o trecho antes do 1º contato,
/// `S₁` o que os dois contatos ABRAÇAM e `S₂` o de depois:
///
/// ```text
/// dS₀ = −s₁ρ₁ dθ
/// dS₁ = +s₁ρ₁ dθ − s₂ρ₂ dθ
/// dS₂ = +s₂ρ₂ dθ
/// ```
///
/// O `θ` é UM (os dois contatos são o mesmo eixo), então eliminá-lo entre as duas
/// primeiras dá **uma** restrição escalar — e ela é um orçamento PESADO, do mesmo
/// tipo que a rota já soma:
///
/// ```text
/// S₀ + [s₁ρ₁/(s₁ρ₁ − s₂ρ₂)] · S₁ = L₀
/// ```
///
/// Com um só sentido de abraço (`s₁ = s₂`, que é o que o [`resolve_sides`]
/// garante para um par) o sinal cai fora e o peso é **`R/(R−r)`**. Com uma
/// cadernal MÓVEL dentro de `S₁` os dois ramos dela carregam esse peso, então a
/// vantagem da máquina é **`2R/(R−r)`** — a talha de Weston, e ela não é digitada
/// em lugar nenhum: sai das duas circunferências que o artista desenhou.
///
/// ⚠️ **`S₂` recebe peso ZERO, e isso é a corda MORTA.** Três segmentos não se
/// dobram em um orçamento (a eliminação de um `θ` entre TRÊS equações deixa duas
/// restrições, e uma combinação linear delas não impõe as duas), então o modelo
/// segura a máquina que existe: numa Weston o ramo depois do retorno é o lado
/// SOLTO da alça de mão — ele pende e não carrega nada. Peso zero é exatamente
/// isso, dito por aritmética: `0 × qualquer coisa` mantém tudo a jusante solto
/// sem um `if` para o próximo nó esquecer.
///
/// ⚠️ E o peso zero dá de graça a **força de um enrolamento terminal**: o
/// Jacobiano do 2º contato ([`wheel_jacobian`]) vira `u_entra·w − u_sai·0`, ou
/// seja o eixo sente só o puxão que CHEGA — que é o que um cabo que termina
/// enrolado no tambor de fato faz.
///
/// Sem par de eixo isto é a [`RopeWheel::gear`] de sempre, e **`1.0` para toda
/// roldana comum é exato no IEEE-754** — a âncora de regressão da família.
///
/// # MEDIDO: a vantagem sai, e ela sai de circunferências GORDAS
///
/// `tests/measure_weston.rs`, contrapeso de 1 kg, `R = 0,5`, cada linha testada a
/// −20% e +20% do equilíbrio previsto (previsão com bracket, **nunca** bisseção —
/// uma busca binária sobre este sistema já mentiu nesta linha):
///
/// | peso `R/(R−r)` | `r` de retorno | previsto `2R/(R−r)` | −20% | +20% |
/// |---|---|---|---|---|
/// | 2 | 0,2500 | **4** | 3,20 kg sobe | 4,80 kg desce |
/// | 4 | 0,3750 | **8** | 6,40 kg sobe | 9,60 kg desce |
/// | 8 | 0,4375 | **16** | 12,80 kg sobe | 19,20 kg desce |
/// | 16 | 0,4688 | **32** | 25,60 kg sobe | 38,40 kg desce |
///
/// ⚠️ **A coluna do `r` é a razão de a máquina existir.** Vantagem 32 sai de duas
/// circunferências de 0,500 e 0,469 — quase do mesmo tamanho. O tambor ADJACENTE
/// do W4 chega ao mesmo 32 com `r_saída = 0,031`, um tambor de espessura de fio de
/// cabelo. É a *diferença* de dois raios gordos que é pequena, e é por isso que a
/// Weston foi inventada.
///
/// O oráculo INDEPENDENTE é o próprio tambor adjacente com `r_saída = R − r`, que
/// produz o MESMO orçamento por um caminho de código que já shipava: no equilíbrio
/// previsto os dois ficam quase parados (Weston 0,0021 m/s · adjacente −0,031 m/s
/// no peso 2; 0,00025 contra −0,022 no peso 16 — o resíduo é a geometria dos
/// enlaces, que não é exatamente 180°, e não a lei).
///
/// # ⛔ NÃO há teto, e a medição é que decidiu
///
/// A mesma sonda varreu o peso até 131 072 (`r = 0,499996`) procurando o número
/// que um teto pudesse citar, e **não existe um**:
///
/// | peso | `L₀` (m) | −20% (sobe?) | +20% (desce?) |
/// |---|---|---|---|
/// | 32 | 308,7 | +0,0149 | −0,0149 |
/// | 128 | 1 228,5 | +0,0034 | −0,0042 |
/// | 512 | 4 907,6 | +0,0004 | −0,0014 |
/// | 2 048 | 19 624,0 | −0,0003 | −0,0008 |
/// | 131 072 | 1 255 808,4 | −0,0005 | −0,0005 |
///
/// Nada explode, nada vira `NaN`, nada oscila — o [`axle_pair`] já garante
/// denominador estritamente positivo, então não há divisão por zero a guardar. O
/// que acontece é que **a carga anda `1/peso` do que o esforço anda**, e a partir
/// de ~2 048 o movimento cai abaixo da resolução de `f32` em `C = Σ w·l − L₀`
/// (a `L₀` de 19 624 m tem passo representável de ~2 mm) ⇒ as duas colunas param
/// de discordar e a máquina deixa de ser dirigível.
///
/// **Isso é o que o DESENHO diz**, não um modo de falha: dois diâmetros a 6 µm um
/// do outro são um diferencial que não se vê mover. Capar o peso seria capar o
/// desenho — o §0 na sua forma exata (*nunca deixe o fallback definir o produto*)
/// —, e o remédio certo é o readout da vantagem na row, não um número que
/// contradiz as duas circunferências.
#[must_use]
pub fn crossing_gear(wheels: &[RopeWheel], i: usize) -> f32 {
    match axle_pair(wheels, i) {
        Some((first, second)) if i == first => {
            weston_gear(wheels[first].radius, wheels[second].radius)
        }
        Some(_) if is_axle_return(wheels, i) => 0.0,
        _ => wheels[i].gear(),
    }
}

/// **A engrenagem de uma talha de WESTON** — `R/(R−r)`, o que um metro do trecho
/// ABRAÇADO vale em metros do trecho de esforço.
///
/// Porta única com DOIS consumidores, e o segundo é a razão de ela existir: o
/// [`crossing_gear`] a usa para pesar a rota, e o **readout da §13** a usa para
/// dizer ao artista o que as duas circunferências que ele desenhou compraram. Uma
/// segunda cópia no painel mostraria um número e o solver usaria outro — que é
/// exatamente o que uma vantagem *digitada* fazia antes do W4.
///
/// ⚠️ **Só é chamada com `R > r > 0`** (o [`axle_pair`] recusa o resto), então o
/// denominador é estritamente positivo e não há guarda de divisão a escrever. Sem
/// teto, e a medição está no [`crossing_gear`].
#[must_use]
pub fn weston_gear(entry: f32, ret: f32) -> f32 {
    entry / (entry - ret)
}

/// **Este contato é o RETORNO de um eixo composto?** (W-Weston.)
///
/// Porta única para os consumidores que precisam distinguir os dois contatos de um
/// par sem re-derivar o [`axle_pair`] — e o `spin` é um deles por uma razão de
/// física, não de gosto: **um eixo tem UMA rotação**. O giro desenhado sai do
/// contato de ENTRADA (`ω = v/R`); derivá-lo outra vez no retorno o computaria com
/// o raio errado (`v/r`) e as duas circunferências concêntricas girariam em
/// velocidades diferentes na tela, o que é o desenho de um eixo que não existe.
#[must_use]
pub fn is_axle_return(wheels: &[RopeWheel], i: usize) -> bool {
    axle_pair(wheels, i).is_some_and(|(_, second)| i == second)
}

/// **Um eixo, UM sentido de abraço** (W-Weston).
///
/// O lado do 2º contato de um par passa a ser o do 1º. Não é higiene: é o que
/// separa um diferencial que **subtrai** (`R/(R−r)`, a Weston) de um que **soma**
/// (`R/(R+r)`, uma máquina real e inútil — vantagem menor que a da cadernal
/// sozinha). Deixá-lo para a geometria adivinhar seria fazer a máquina depender de
/// onde a ponta MORTA foi largada, e a ponta morta é justamente a que não carrega
/// nada.
///
/// ⚠️ Roda DEPOIS do ponto fixo, nunca dentro dele: o chute e a re-avaliação
/// perguntam à geometria, e esta é uma regra da MÁQUINA — misturá-las faria o
/// laço perseguir um lado que ele mesmo reescreve.
///
/// No-op para toda corda sem eixo composto, que é o que toda corda era até aqui.
///
/// ⚠️ **`pub` porque há um SEGUNDO chamador, e ele é o que fecha a regra:** a
/// ponte aplica o override de abraço do artista DEPOIS do [`resolve_sides`], então
/// sem re-amarrar ali um chip clicado num dos dois contatos faria a máquina SOMAR
/// os diâmetros em vez de subtrair.
pub fn tie_axle_pairs(wheels: &mut [RopeWheel]) {
    for i in 0..wheels.len() {
        if let Some((first, second)) = axle_pair(wheels, i)
            && i == second
        {
            wheels[second].side = wheels[first].side;
        }
    }
}
