//! **A ROTA DA CORDA** — a geometria pura de uma corda que passa por N roldanas
//! de raio próprio, tangenciando a superfície de cada uma (ADR-0131, W-Pulley
//! W1; plano [`docs/Physics/03_plano_polia.md`] §5).
//!
//! O que a v1 do W-Pulley chamava de "roldana" era um **ponto**: a corda ia do
//! corpo até ele e mudava de direção ali, o que é uma polia de raio zero. O
//! artista pediu diâmetro, e um diâmetro muda três coisas de uma vez — por onde a
//! corda passa (a superfície, não o centro), quanto dela existe (o arco), e o
//! quanto a roldana gira (a mesma corda por um raio maior gira menos).
//!
//! # Uma função responde ao par inteiro
//!
//! Ponto→círculo e círculo→círculo **não são dois casos**: um ponto é um círculo
//! de raio zero, e a fórmula da tangente comum já o contém. Escrever os dois
//! separados seria a segunda resposta que diverge — a representação apaga o caso
//! especial, como a bola limitada do Painter apagou quatro cercas.
//!
//! Sejam duas rodas `(C₁, r₁, s₁)` e `(C₂, r₂, s₂)`, onde `s = ±1` diz **de que
//! lado a corda passa** (`+1` = a corda vira à esquerda ali, e o centro fica à
//! esquerda dela). A tangência exige que os dois centros estejam à distância `r`
//! da reta, cada um do seu lado:
//!
//! ```text
//! D = C₂ − C₁ ,  R = s₂·r₂ − s₁·r₁
//! D = ℓ·u + R·perp(u)            (perp = giro de +90°)
//! ⇒ ℓ = √(|D|² − R²)   e   u = (ℓ·D − R·perp(D)) / |D|²
//! T₁ = C₁ − s₁·r₁·perp(u) ,  T₂ = C₂ − s₂·r₂·perp(u)
//! ```
//!
//! ⚠️ **`|D| > |R|` é a condição de existência**, e ela é o guarda de degeneração
//! honesto: com os lados IGUAIS `R` é a diferença dos raios (uma roda dentro da
//! outra não tem tangente externa); com os lados OPOSTOS `R` é a soma (rodas que
//! se tocam não têm tangente cruzada). Recusar aqui é o que impede um `NaN` de
//! chegar ao `physics_ecs_c9`.
//!
//! # O comprimento inclui o ARCO, e o Jacobiano NÃO
//!
//! `L = Σ|tangentes| + Σ rᵢ·|θᵢ|`, com `θ` o ângulo que a corda vira na roda.
//! Mas a derivada de `L` em relação à âncora de um corpo é **exatamente** o
//! versor daquele último trecho: os pontos de tangência deslizam quando a âncora
//! se move, e a variação do arco **cancela** a variação do trecho (teorema do
//! envelope — o ponto de tangência é estacionário por construção).
//!
//! É por isso que o kernel de impulso quase não muda, e é o mesmo fato que, no
//! W3, dará o Jacobiano de uma roldana MONTADA num corpo — `∂L/∂C = −(u_in +
//! u_out)`, a resultante que também é a carga de ruptura daquele centro. **Uma
//! conta, dois consumidores.**
//!
//! # Determinismo
//!
//! Só `+ − * /`, comparação e `sqrt` — todos exatos no IEEE-754 — mais **UM**
//! transcendental, o `libm::atan2f` do ângulo de arco, pinado cross-OS pela lei 6
//! (o mesmo motivo do `libm::sincosf` do W-AreaFrame). `f32::atan2` viria da libm
//! da plataforma e este número alimenta o hash.

/// Uma roldana na rota: onde ela está, que tamanho tem, e **de que lado a corda
/// passa**.
///
/// `side` é `+1` (a corda vira à esquerda ao passar) ou `−1` (à direita). Ele é
/// **resolvido em autoria** e congelado no play — uma corda real não troca de
/// lado da polia no meio da corrida sem sair da canaleta, e um lado recomputado
/// por frame pisca perto da configuração degenerada, o que muda o comprimento e
/// dá um puxão.
///
/// ⚠️ **Nem todo campo daqui é geometria**, e isso é deliberado: `id`,
/// `break_force` e o par `body`/`local` do W3 são *o que a rota consome sobre uma
/// roldana*, não o que ela desenha. O MÓDULO é a geometria pura; a STRUCT é a
/// roldana. As funções deste arquivo leem `centre`/`radius`/`side` e mais nada.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct RopeWheel {
    /// O centro, em **mundo** — e para uma roldana MONTADA (W3) ele é
    /// **derivado**, não autorado: [`super::pulley::refresh_mounts`] o reescreve
    /// da pose viva do corpo uma vez por sub-passo, na ARENA, que é a mesma lista
    /// que o desenho lê. Uma segunda cópia refrescada à parte faria o solver e o
    /// overlay discordarem sobre onde a roldana está.
    pub centre: [f32; 2],
    /// **Em que corpo esta roldana está MONTADA** (W3) — `None` é uma roldana
    /// pregada no cenário, e é isso que toda roldana era até aqui.
    ///
    /// É este campo que traz a vantagem mecânica de volta: uma roldana montada
    /// num corpo que se move é a **cadernal móvel** de uma talha, e o corpo passa
    /// a ser sustentado por DOIS ramos da mesma corda. Ver o Jacobiano em
    /// [`wheel_jacobian`].
    pub body: Option<rapier2d::dynamics::RigidBodyHandle>,
    /// Onde no corpo o EIXO está, no frame local dele.
    ///
    /// Local e nunca mundo, pela mesma razão que as âncoras de joint são locais
    /// (W-AnchorFollow): guardar mundo faria o eixo **caminhar pelo corpo**
    /// conforme ele gira. Ignorado quando [`Self::body`] é `None`.
    pub local: [f32; 2],
    /// O raio, em metros. `0` reduz ao modelo de ponto do W-Pulley v1 — e essa
    /// redução é **exata**, o que a torna a âncora de regressão da wave.
    ///
    /// Numa roldana DIFERENCIAL (W4) este é o raio por onde a corda **ENTRA**;
    /// ver [`Self::radius_out`].
    pub radius: f32,
    /// **O raio por onde a corda SAI** — `None` numa roldana comum, que é o que
    /// toda roldana era até o W4.
    ///
    /// Uma roldana com dois raios é um **tambor diferencial**: a corda chega
    /// enrolando num diâmetro e sai enrolando noutro, presos ao mesmo eixo. É
    /// daqui que a vantagem mecânica contínua nasce — e ela é o **quociente de
    /// duas circunferências que o artista desenha**, nunca um número digitado.
    ///
    /// ⚠️ **Isto é o que substitui o `ratio` que o W1 aposentou.** Aquele campo
    /// descrevia *"uma talha diferencial com o eixo invisível"* (§3 do plano); aqui
    /// o eixo é visível, tem duas circunferências, e o número **cai delas**.
    ///
    /// ⚠️ **Duas roldanas CONCÊNTRICAS seriam a leitura ingênua e são impossíveis:**
    /// a tangente comum exige `|C₂−C₁| > |s₂r₂ − s₁r₁}|`, que dois círculos de
    /// mesmo centro nunca satisfazem ⇒ a rota inteira seria recusada. Um eixo é UM
    /// nó, e é por isso que os dois raios moram na mesma roldana.
    ///
    /// Não-positivo é tratado como *não é um diferencial* — uma regra, dois
    /// consumidores ([`Self::radius_out`] e [`Self::gear`]), em vez de uma
    /// geometria que diz uma coisa e uma engrenagem que diz outra.
    pub radius_out: Option<f32>,
    /// `+1` = a corda vira à ESQUERDA aqui; `−1` = à direita.
    pub side: i8,
    /// **Quem esta roldana É, através das trocas de arena** — o `stable_name_id`
    /// do nome dela (W2).
    ///
    /// A arena é reconstruída por dispatch, então o que uma roldana ACUMULOU (a
    /// carga de pico do eixo) e o fato de ela ter ROMPIDO não podem ser guardados
    /// por posição: acrescentar uma roldana deslocaria os índices e o eixo partido
    /// migraria para a vizinha. Mesma chave, mesmo motivo, que o `id` da corda.
    pub id: u64,
    /// **O que este EIXO aguenta**, newtons — `∞` é uma roldana que não parte, e
    /// é também o que uma roldana que ninguém dimensionou carrega.
    ///
    /// A carga aqui **não é a tensão da corda**: é a RESULTANTE que o desvio
    /// produz (`T·|u_saída − u_entrada|`), então um enlace de 180° carrega `2T` e
    /// um que quase não desvia a corda carrega quase nada. É a mesma conta do
    /// Jacobiano — uma conta, dois consumidores.
    pub break_force: f32,
}

impl Default for RopeWheel {
    /// **A roldana neutra: um PONTO pregado no cenário, que não parte.**
    ///
    /// Não é conveniência — é a redução exata ao modelo do W-Pulley v1 (raio zero,
    /// sem eixo montado, sem limiar), que é a âncora de regressão desta família
    /// inteira. `side: 1` é só o valor de partida; quem responde de que lado a
    /// corda passa é o [`resolve_sides`].
    ///
    /// ⚠️ **Para FIXTURES.** Um sítio de PRODUTO nomeia todo campo: com `..default()`
    /// o campo seguinte nasce neutro **em silêncio**, e o compilador deixa de ser a
    /// lista de quem precisa aprender sobre ele. As duas rotas de produto (a
    /// colheita e o `pulley_rig` da ponte) escrevem os seis.
    fn default() -> Self {
        Self {
            centre: [0.0, 0.0],
            body: None,
            local: [0.0, 0.0],
            radius: 0.0,
            radius_out: None,
            side: 1,
            id: 0,
            break_force: f32::INFINITY,
        }
    }
}

impl RopeWheel {
    /// O raio por onde a corda **SAI** — o de entrada para toda roldana comum.
    ///
    /// Porta única com a [`Self::gear`] sobre *"esta roldana é um diferencial?"*:
    /// um `radius_out` não-positivo cai aqui para o raio de entrada, e lá para
    /// engrenagem 1, de modo que as duas respostas não podem discordar.
    #[must_use]
    pub fn radius_out(&self) -> f32 {
        match self.radius_out {
            Some(r) if r > 0.0 => r,
            _ => self.radius,
        }
    }

    /// **A ENGRENAGEM desta roldana** — quanto o orçamento de corda do lado de
    /// SAÍDA vale, em unidades do lado de ENTRADA (W4).
    ///
    /// `r_entra / r_sai`, e **exatamente `1.0`** para toda roldana comum: o eixo
    /// gira uma vez, recolhe `r_entra` de um lado e paga `r_sai` do outro, então
    /// `r_sai·Δl_entra + r_entra·Δl_sai = 0`. Normalizado pelo lado de entrada, o
    /// trecho que sai conta `r_entra/r_sai` vezes.
    ///
    /// ⚠️ **`1.0` não é aproximadamente um: `x * 1.0 == x` é exato no IEEE-754**, e
    /// é isso que torna toda cena anterior byte-idêntica — a âncora de regressão
    /// desta wave é a mesma do W1, uma multiplicação que não move um bit.
    ///
    /// A vantagem mecânica que ela produz é o próprio número: puxar do lado de
    /// entrada com força `T` segura `T·gear` do lado de saída.
    #[must_use]
    pub fn gear(&self) -> f32 {
        match self.radius_out {
            Some(out) if out > 0.0 && self.radius > 0.0 => self.radius / out,
            _ => 1.0,
        }
    }
}

/// O que a rota entrega ao kernel de impulso.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct RopeRoute {
    /// Do primeiro ponto de tangência PARA a âncora em A, unitário — a direção
    /// em que o impulso do ramo A age.
    pub dir_a: [f32; 2],
    /// O mesmo na ponta B.
    pub dir_b: [f32; 2],
    /// `Σ|tangentes| + Σ arcos`, em metros — **pesado pela engrenagem** de cada
    /// trecho (W4). Sem tambor diferencial todo peso é `1.0` e isto é a soma
    /// simples que sempre foi.
    pub length: f32,
    /// **A engrenagem acumulada na ponta B** — `1.0` sem tambor diferencial.
    ///
    /// A ponta A é a REFERÊNCIA e vale sempre `1.0` por construção (a engrenagem
    /// só começa a contar depois da primeira roldana), então publicar um
    /// `weight_a` seria um campo que só sabe dizer um número. A vantagem mecânica
    /// da corda **é este valor**: a tensão em B é `weight_b` vezes a de A.
    pub weight_b: f32,
    /// **O maior peso da rota** — `1.0` sem tambor diferencial.
    ///
    /// ⚠️ **Ele existe porque o W4 FALSIFICA uma premissa escrita no
    /// [`super::pulley::PulleyDesc::break_force`]:** *"a corda é inextensível,
    /// logo a tensão é uniforme"*. Isso vale enquanto a corda DESLIZA sobre as
    /// roldanas — e um tambor diferencial é exatamente o lugar onde ela não
    /// desliza: os dois lados carregam tensões diferentes, e é dessa diferença
    /// que a vantagem mecânica nasce.
    ///
    /// Então o limiar de ruptura tem de ser comparado contra o lado MAIS
    /// carregado, senão uma corda com engrenagem 5 arrebentaria com cinco vezes a
    /// carga que o artista dimensionou, em silêncio. O máximo, e não o de B: com
    /// dois tambores em série o pico pode estar num trecho do meio.
    pub weight_max: f32,
}

/// Abaixo disto um trecho não tem direção definida e normalizar produziria `NaN`.
/// Mesma constante e mesmo motivo do `MIN_BRANCH` do modelo de ponto.
const MIN_SEG: f32 = 1.0e-4;

/// Os pontos de tangência de um passo da rota, mais a direção do trecho.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Tangent {
    /// Onde a corda LARGA a roda anterior (ou a âncora, se for a primeira).
    pub from: [f32; 2],
    /// Onde ela ENCOSTA na roda seguinte (ou a âncora, se for a última).
    pub to: [f32; 2],
    /// Unitário, de `from` para `to`.
    pub dir: [f32; 2],
    /// `|to − from|`.
    pub len: f32,
    /// **Quanto um metro DESTE trecho vale no orçamento da corda** (W4) —
    /// `1.0` em toda corda sem tambor diferencial, e é isso que mantém tudo o
    /// que já existia byte-idêntico.
    ///
    /// É o produto das [`RopeWheel::gear`] de todas as roldanas que a corda já
    /// atravessou até aqui, andando de A para B. Um tambor de dois raios
    /// **engrena** o resto da rota, e dois deles compõem — o que é o que dois
    /// eixos em série de fato fazem.
    pub weight: f32,
}

#[inline]
fn perp(v: [f32; 2]) -> [f32; 2] {
    [-v[1], v[0]]
}

/// **A tangente comum a duas rodas, cada uma pelo lado pedido.**
///
/// Um ponto é uma roda de raio zero (o `side` dele é ignorado pela aritmética,
/// porque ele entra multiplicado pelo raio) — é isso que faz esta ser a única
/// função de geometria da rota.
///
/// `None` quando os círculos estão próximos demais para a tangente existir; ver o
/// cabeçalho do módulo.
#[must_use]
pub fn tangent(c1: [f32; 2], r1: f32, s1: i8, c2: [f32; 2], r2: f32, s2: i8) -> Option<Tangent> {
    let d = [c2[0] - c1[0], c2[1] - c1[1]];
    let dd = d[0] * d[0] + d[1] * d[1];
    let rr = f32::from(s2) * r2 - f32::from(s1) * r1;
    let inner = dd - rr * rr;
    if inner <= 0.0 || dd < MIN_SEG * MIN_SEG {
        return None;
    }
    let len = inner.sqrt();
    if len < MIN_SEG {
        return None;
    }
    let pd = perp(d);
    let dir = [
        (len * d[0] - rr * pd[0]) / dd,
        (len * d[1] - rr * pd[1]) / dd,
    ];
    let pu = perp(dir);
    let from = [
        c1[0] - f32::from(s1) * r1 * pu[0],
        c1[1] - f32::from(s1) * r1 * pu[1],
    ];
    let to = [
        c2[0] - f32::from(s2) * r2 * pu[0],
        c2[1] - f32::from(s2) * r2 * pu[1],
    ];
    Some(Tangent {
        from,
        to,
        dir,
        len,
        // O peso é da ROTA, não do par de círculos: quem o conhece é o
        // [`route`], que anda a corda de A para B acumulando as engrenagens.
        weight: 1.0,
    })
}

/// O ângulo com que a corda vira ao passar por uma roda de lado `side`, em
/// radianos e **com sinal**.
///
/// ⚠️ O sinal do `atan2` sozinho não basta: ele devolve o ângulo no intervalo
/// `(−π, π]`, e uma roda com enlace maior que meia volta viraria pelo lado
/// errado. O `side` diz qual dos dois sentidos é o real, então o ângulo é
/// **desdobrado** para aquele sentido — é assim que um enlace de 270° mede 270°
/// e não −90°.
#[must_use]
pub fn turn_angle(u_in: [f32; 2], u_out: [f32; 2], side: i8) -> f32 {
    let cross = u_in[0] * u_out[1] - u_in[1] * u_out[0];
    let dot = u_in[0] * u_out[0] + u_in[1] * u_out[1];
    let mut t = libm::atan2f(cross, dot);
    if side > 0 && t < 0.0 {
        t += std::f32::consts::TAU;
    } else if side < 0 && t > 0.0 {
        t -= std::f32::consts::TAU;
    }
    t
}

/// **O Jacobiano de uma roldana montada** — `∂L/∂C`, o quanto a rota se alonga
/// quando o EIXO dela se move (W3).
///
/// ⚠️ **Duas leituras da mesma expressão, e é preciso saber que são a mesma**,
/// senão alguém "corrige" uma na outra:
///
/// - no vocabulário deste módulo, onde [`Tangent::dir`] aponta **para a frente ao
///   longo da corda**, é `u_entra − u_sai` — mover a roda ao longo do trecho que
///   chega o alonga, e ao longo do que sai o encurta;
/// - no vocabulário da FÍSICA, onde os dois versores apontam **para fora do
///   eixo** (as duas direções em que a corda puxa), é `−(u_in + u_out)`, que é
///   como o cabeçalho deste módulo e o do [`super::rope_load`] a escrevem.
///
/// São a mesma conta porque o versor de fora do trecho que chega é `−u_entra`.
///
/// ⚠️ **O ARCO não entra** — o ponto de tangência desliza e a variação do arco
/// cancela a do trecho (teorema do envelope, o cabeçalho deste módulo). É por isso
/// que uma roldana montada custa ao kernel exatamente uma linha.
///
/// **Uma conta, TRÊS consumidores:** o impulso no eixo (`pulley::apply`), a massa
/// efetiva dele, e a carga de ruptura daquele centro (`rope_load::ledger_axles`),
/// que é `T·|∂L/∂C|`. `None` quando a lista de trechos não descreve a roda `i`.
#[must_use]
pub fn wheel_jacobian(legs: &[Tangent], i: usize) -> Option<[f32; 2]> {
    // O trecho que CHEGA nesta roda é o `i`, o que SAI é o `i+1`.
    let inbound = legs.get(i)?;
    let outbound = legs.get(i + 1)?;
    // ⚠️ **Cada lado entra com o PESO dele** (W4): num tambor diferencial os dois
    // trechos valem diferente no orçamento, então a resultante no eixo é a
    // diferença dos versores JÁ engrenados — e é isso que faz a carga do eixo de
    // um diferencial ser a soma de duas tensões distintas, que é o que ele de
    // fato segura. Sem tambor os dois pesos são `1.0` e isto é a subtração de
    // sempre, ao bit.
    Some([
        inbound.dir[0] * inbound.weight - outbound.dir[0] * outbound.weight,
        inbound.dir[1] * inbound.weight - outbound.dir[1] * outbound.weight,
    ])
}

/// **Resolver a rota inteira**, escrevendo os `wheels.len() + 1` trechos em
/// `out`.
///
/// `out` é do CHAMADOR e é limpo aqui: o passe de polias roda uma vez por
/// sub-passo, então uma alocação por rota apareceria no gate de zero-alloc do
/// caminho quente.
///
/// `None` quando qualquer trecho é degenerado — a corda inteira é pulada, que é a
/// mesma recusa que o modelo de ponto faz e pela mesma razão: uma rota com um
/// trecho sem direção não tem impulso definido, e meia rota seria pior que
/// nenhuma.
pub fn route(
    anchor_a: [f32; 2],
    anchor_b: [f32; 2],
    wheels: &[RopeWheel],
    out: &mut Vec<Tangent>,
) -> Option<RopeRoute> {
    out.clear();
    let mut prev_c = anchor_a;
    let mut prev_r = 0.0;
    let mut prev_s = 1_i8;
    // **A engrenagem acumulada** (W4). A ponta A é a referência, então ela parte
    // de `1.0` e só se move ao ATRAVESSAR um tambor de dois raios.
    let mut weight = 1.0_f32;
    for w in wheels {
        let mut leg = tangent(prev_c, prev_r, prev_s, w.centre, w.radius, w.side)?;
        leg.weight = weight;
        out.push(leg);
        // Daqui para a frente a corda está do outro lado do eixo: ela sai
        // enrolando no raio de SAÍDA, e um metro dela passa a valer `gear` vezes.
        weight *= w.gear();
        prev_c = w.centre;
        prev_r = w.radius_out();
        prev_s = w.side;
    }
    let mut tail = tangent(prev_c, prev_r, prev_s, anchor_b, 0.0, 1)?;
    tail.weight = weight;
    out.push(tail);

    let mut length = 0.0;
    let mut weight_max = 1.0_f32;
    for t in out.iter() {
        length += t.len * t.weight;
        weight_max = weight_max.max(t.weight);
    }
    // Os arcos: cada roda vive ENTRE dois trechos, e o que ela acrescenta é o
    // pedaço de circunferência que a corda abraça.
    //
    // ⚠️ **Pelo raio e pelo peso de ENTRADA**, e a escolha é declarada: a corda
    // abraça o tambor em que ela CHEGOU. Num diferencial o enlace se reparte
    // entre os dois diâmetros, mas o que ele acrescenta é quase constante e o
    // `L0` o absorve — o que move a carga são os trechos LIVRES, e esses estão
    // pesados exatamente. Numa roldana comum os dois raios e os dois pesos são o
    // mesmo, então isto é o arco de sempre, ao bit.
    for (i, w) in wheels.iter().enumerate() {
        if w.radius <= 0.0 {
            continue;
        }
        length += out[i].weight * w.radius * turn_angle(out[i].dir, out[i + 1].dir, w.side).abs();
    }

    // As duas pontas: o versor aponta do ponto de tangência PARA a âncora, que é
    // a direção em que afastar o corpo estica a corda.
    let first = out[0];
    let last = out[out.len() - 1];
    Some(RopeRoute {
        dir_a: [-first.dir[0], -first.dir[1]],
        dir_b: [last.dir[0], last.dir[1]],
        length,
        weight_b: last.weight,
        weight_max,
    })
}

/// **De que lado a corda passa em cada roda** — o (7) do pedido do artista.
///
/// Ponto fixo: chuta pela poligonal dos CENTROS, resolve a rota com esse chute,
/// re-lê o sentido de giro que os trechos de fato fazem, e repete. Converge em
/// uma ou duas rodadas para toda montagem sã (medido em
/// `tests/measure_rope_route.rs`); o cap existe para o caso patológico, onde a
/// resposta é *fique com o último*, nunca *itere para sempre*.
///
/// ⚠️ Roda por AUTORIA, não por frame — ver o cabeçalho do `RopeWheel`.
pub fn resolve_sides(
    anchor_a: [f32; 2],
    anchor_b: [f32; 2],
    wheels: &mut [RopeWheel],
    scratch: &mut Vec<Tangent>,
) {
    if wheels.is_empty() {
        return;
    }
    // O chute: o sentido de giro da poligonal que liga âncora → centros → âncora.
    for i in 0..wheels.len() {
        let prev = if i == 0 {
            anchor_a
        } else {
            wheels[i - 1].centre
        };
        let next = if i + 1 == wheels.len() {
            anchor_b
        } else {
            wheels[i + 1].centre
        };
        let c = wheels[i].centre;
        let a = [c[0] - prev[0], c[1] - prev[1]];
        let b = [next[0] - c[0], next[1] - c[1]];
        wheels[i].side = sign_or(a[0] * b[1] - a[1] * b[0], wheels[i].side);
    }
    for _ in 0..MAX_SIDE_PASSES {
        if route(anchor_a, anchor_b, wheels, scratch).is_none() {
            return;
        }
        let mut changed = false;
        for i in 0..wheels.len() {
            let (u_in, u_out) = (scratch[i].dir, scratch[i + 1].dir);
            let s = sign_or(u_in[0] * u_out[1] - u_in[1] * u_out[0], wheels[i].side);
            if s != wheels[i].side {
                wheels[i].side = s;
                changed = true;
            }
        }
        if !changed {
            return;
        }
    }
}

/// Quantas vezes o ponto fixo do lado pode reavaliar antes de aceitar o que tem.
///
/// **MEDIDO** em `tests/measure_rope_route.rs` — 18 montagens, de 1 a 6 roldanas
/// em zigue-zague, com três espalhamentos:
///
/// | roldanas | 1 | 2 | 3 | 4 | 5 | 6 |
/// |---|---|---|---|---|---|---|
/// | passadas até assentar | 1 | 1 | 1 | 1 | 1 | 1 |
///
/// ⚠️ **UMA passada em todo caso são** — o chute pela poligonal dos centros já
/// É o ponto fixo, e a re-avaliação existe para confirmá-lo. (Eu havia escrito
/// *"1 ou 2"* antes de medir; a medição é mais forte que a estimativa, e a
/// diferença é o tipo de número que ninguém re-mede depois.)
///
/// O cap fica em 4 para o caso patológico — uma montagem que oscile —, e ali
/// *ficar com o último* é a resposta certa: o artista tem o override por roda.
const MAX_SIDE_PASSES: usize = 4;

/// O sinal de `x`, ou `fallback` quando ele é exatamente zero.
///
/// Zero é a corda **colinear** — nem esquerda nem direita —, e ali o lado
/// anterior é a única resposta que não faz a roda pular de um lado ao outro por
/// ruído de `f32`.
#[inline]
fn sign_or(x: f32, fallback: i8) -> i8 {
    if x > 0.0 {
        1
    } else if x < 0.0 {
        -1
    } else {
        fallback
    }
}

#[cfg(test)]
#[path = "rope_route_tests.rs"]
mod tests;
