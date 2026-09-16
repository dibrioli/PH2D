//! ⭐⭐⭐ **O IMPULSO DE UM PAR** — a velocidade responde ao contacto trocando MOMENTO entre as duas
//! peças, repartido pelas massas delas.
//!
//! ## O report que o obrigou
//!
//! > *«quando aumento bounciness as colisões não são realistas. Quando uma caixa bate na outra, não
//! > parecem ter a mesma massa. É como se uma fosse muito mais pesada que a outra?»*
//! > — o dono, 2026-09-15.
//!
//! Ele leu-o exactamente. A lei que estava no `sim.step` era **por PEÇA**, sobre a velocidade
//! **ABSOLUTA** de cada uma, na direcção em que a correcção de posição a tinha empurrado: a que
//! chegava a andar via aproximação e travava; a que estava **parada** tinha velocidade zero, logo o
//! motor concluía *«esta não se aproxima de nada»* e **não lhe tocava**. ⇒ nunca havia troca de
//! momento, e a peça atingida comportava-se como uma **parede**. Medido, com duas caixas iguais e a
//! primeira a `1,0 u/s`:
//!
//! ```text
//!   bounciness |  vel da que bate |  vel da PARADA |  momento (era 1,00)
//!         0,00 |           0,0000 |         0,0000 |              0,00
//!         0,50 |          −0,5000 |         0,0000 |             −0,50
//!         1,00 |          −1,0000 |         0,0000 |             −1,00
//! ```
//!
//! ⚠️⚠️ **O momento não se perdia: INVERTIA-SE.** Com massas iguais a resposta certa é `0,50 / 0,50`
//! sem salto, e **trocarem** (`0,00 / 1,00`) com salto máximo.
//!
//! ## A lei
//!
//! O impulso clássico, sobre a velocidade **RELATIVA** na normal do par:
//!
//! ```text
//!   vrel = (v_lo − v_hi) · n          (n vai do índice MENOR para o MAIOR)
//!   se vrel <= 0: separam-se, e um contacto não puxa  ⇒  nada
//!   j = (1 + e) · vrel / (w_lo + w_hi)
//!   v_lo −= n · j · w_lo              v_hi += n · j · w_hi
//! ```
//!
//! ⭐ **Ela honra as três leis que a anterior honrava, e por construção:**
//! - *duas peças que NASCEM sobrepostas e paradas não ganham velocidade* — `vrel = 0` ⇒ `j = 0`;
//! - *só se responde a quem se move PARA DENTRO* — é a guarda `vrel > 0`;
//! - *um obstáculo (`w = 0`) devolve o salto inteiro* — com `w_hi = 0` a conta dá `v_lo = −e·v`,
//!   que é o ressalto contra uma parede, exacto.
//!
//! ⭐⭐ E acrescenta a que faltava: **o momento CONSERVA-SE** (`w_lo·Δv_lo + w_hi·Δv_hi = 0` termo a
//! termo), que é o que faz duas peças iguais partilharem o choque em vez de uma servir de parede.
//!
//! ## ⭐⭐⭐ E a partir de 2026-09-16 ele é um SOLVER, não uma passagem — as [`Leis`]
//!
//! O 8.º report do dono (*«umas caixas rodam, outras parecem não rodar»*) pediu **velocidade
//! angular**, e a 1.ª tentativa de a dar PARTIU A PILHA (doc 111 §8.3: rodopio `0,7..2,4 → 77,5..153,2`).
//! A causa não era a velocidade angular — era o que faltava à volta dela, e está medido no doc 111
//! §9. Cada peça em falta é hoje um campo desta struct, e o **controlo** ([`Leis::HOJE`]) reproduz a
//! lei anterior. ⚠️ *Um knob aqui não é configuração: é a coluna de uma tabela que já foi medida* —
//! quem lhe mexer sem re-medir a `=114` está a escolher uma célula ao acaso.
//!
//! ⚠️ **A única diferença de BITS para a lei de 2026-09-15 é a forma do ressalto** — ela escrevia
//! `(1+e)·vrel` e o solver escreve `vrel + e·vrel₀`, porque com mais de uma iteração o ressalto tem
//! de sair de um ALVO fixo (senão ele recompõe-se a cada varredura e a peça ganha energia). Com
//! `e = 0` — o default do produto — as duas contas são o mesmo bit; acima disso separam-se por 1 ULP.

use super::{Contacto, GRAUS, Pecas, atrito, dot, manifesto};

/// **As leis que o solver de velocidade corre.** O default é [`Leis::HOJE`], que é a lei medida e
/// aprovada pelo dono em 2026-09-15 — *toda medição desta família começa por ela como CONTROLO.*
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Leis {
    /// O impulso escreve **velocidade angular** (que persiste) em vez de um ângulo do tique só.
    ///
    /// Report do dono: *«umas caixas rodam, outras parecem não rodar»* — sem isto uma peça atingida
    /// fora do centro roda `0,5341°` **uma vez** e congela (doc 111 §8.2).
    pub angular: bool,
    /// Cada ponto do [`super::Manifesto`] é uma restrição com o **seu** `λ`, como já acontece na
    /// posição ([`super::varredura`]).
    ///
    /// ⛔ Com um ponto só, uma caixa apoiada por uma FACE é sustentada por um SÍTIO: o impulso
    /// aplica-lhe binário, ela roda, o apoio desloca-se, e nasce um balanço que nada amortece.
    pub por_ponto: bool,
    /// Varreduras de velocidade, com o `λ` **acumulado** e preso em `≥ 0` entre elas.
    ///
    /// ⚠️ O que o acumulador compra não é precisão: é o **sinal**. Sem ele cada varredura só sabe
    /// empurrar (`λ ≥ 0` por varredura), então uma varredura que sobrepassa não pode ser desfeita
    /// pela seguinte, e a pilha ganha energia a cada uma.
    pub iteracoes: usize,
    /// Abaixo desta velocidade de aproximação o **ressalto não responde** (`e` efectivo `= 0`).
    ///
    /// Numa pilha assente a gravidade repõe `g·dt` de aproximação a cada sub-passo, e com `e > 0`
    /// isso é energia devolvida para sempre.
    pub limiar_salto: f32,
    /// O tecto de Coulomb sai do `λ` **deste contacto** em vez da estimativa `max(vrel, pen/dt)`.
    pub tecto_por_lambda: bool,
    /// A separação de POSIÇÃO continua a rodar as peças (doc 109 §6), a par da velocidade angular.
    ///
    /// ⚠️ **Este campo não é lido aqui: quem o honra é o CHAMADOR**, porque o dono dele é o
    /// [`super::separate`], que ainda não recebe as leis. Ele vive nesta struct para as leis do
    /// contacto se declararem todas num sítio — *e é dívida nomeada: se ele decidir alguma coisa,
    /// passa a ser um argumento do `separate`.*
    pub giro_posicional: bool,
}

impl Leis {
    /// **A lei que shipou em 2026-09-15** (doc 111 §5.12 + §7) — o CONTROLO de toda medição desta
    /// família, e o que o produto corria antes da wave da velocidade angular.
    pub const HOJE: Self = Self {
        angular: false,
        por_ponto: false,
        iteracoes: 1,
        limiar_salto: 0.0,
        tecto_por_lambda: false,
        giro_posicional: true,
    };
    /// ⭐⭐⭐ **A LEI QUE O PRODUTO CORRE desde 2026-09-16** — a resposta ao 8.º report do dono
    /// (*«umas caixas rodam, outras parecem não rotacionar»*), medida célula a célula na `=114`
    /// (doc 111 §9). As quatro peças são **indivisíveis**, e cada uma sozinha lê-se como fracasso:
    ///
    /// ```text
    ///   lei                                     | tremor        | rodopio    | salto | y
    ///   ----------------------------------------|---------------|------------|-------|------
    ///   HOJE (o controlo)                       | 0,036..0,049  |  2,0.. 2,8 |  1,58 | −2,42
    ///   + angular                               | 0,454..1,992  | 38,7..123,3|  1,69 | −2,55
    ///   + angular + por_ponto                   | 0,365..0,855  | 32,5.. 66,4|  1,65 | −2,49
    ///   + angular + por_ponto + 8 iter          | 0,840..0,905  | 42,1.. 43,6|  0,87 | −2,47
    ///   ESTA (as três + sem giro posicional)    | 0,016..0,028  |  2,6.. 3,1 |  2,42 | −2,43
    /// ```
    ///
    /// ⚠️ **A coluna do SALTO está no tecto de «assentada» de `0,2°/tique`, que é o que a cena
    /// usava quando esta varredura correu** — e ele apertou para `0,1` no mesmo dia, porque a
    /// `0,2` uma peça que anda `2,4°` na janela conta como parada. No tecto de hoje a coluna lê
    /// **`1,57` (HOJE)** contra **`0,74` (ESTA)**; a tabela e o porquê vivem ao lado da constante,
    /// em `motion_state_pilha_demo_salto::QUIETO`.
    ///
    /// ⚠️ **Porque `8` iterações:** `1` lê `72..77` de rodopio e `2` lê `3,6..38,2` (não convergiu);
    /// `4` já lê `3,0..4,1`; `16`, `32` e `64` leem o mesmo que `8` e custam até `1,73 ms` contra
    /// `0,66`. *O `8` é onde a curva assenta, não uma preferência.*
    ///
    /// ⛔ **E `tecto_por_lambda` foi medido e REFUTADO nesta combinação** (`rodopio 3,0..24,7`, com
    /// a dispersão de volta): o tecto de Coulomb pela estimativa `max(vrel, pen/dt)` é mais estável
    /// que o `λ` acumulado, e a razão é que o `λ` da 1.ª varredura ainda não conhece a carga.
    pub const EM_VIGOR: Self = Self {
        angular: true,
        por_ponto: true,
        iteracoes: 8,
        limiar_salto: 0.0,
        tecto_por_lambda: false,
        giro_posicional: false,
    };
}

impl Default for Leis {
    fn default() -> Self {
        Self::HOJE
    }
}

/// **O que o impulso escreve.** Três colunas porque são três grandezas diferentes, e qual delas
/// recebe a rotação é [`Leis::angular`] que decide — nunca as duas, que seria contá-la a dobrar.
pub struct Movimento<'a> {
    /// A velocidade linear, reescrita no sítio.
    pub vel: &'a mut [[f32; 2]],
    /// O ÂNGULO (graus) que o contacto rodou neste tique — a lei **sem** velocidade angular.
    pub giro: &'a mut [f32],
    /// A VELOCIDADE ANGULAR (graus/s) que o contacto acrescentou — a lei **com**.
    pub spin: &'a mut [f32],
}

/// Uma restrição de velocidade: um PONTO de contacto de um par, com o `λ` dele.
struct Restricao {
    lo: usize,
    hi: usize,
    n: [f32; 2],
    t: [f32; 2],
    /// A alavanca da NORMAL (`r × n`) em `[lo, hi]` — quanto o impulso normal roda cada lado.
    bn: [f32; 2],
    /// A alavanca da TANGENTE (`r · n`) em `[lo, hi]`.
    bt: [f32; 2],
    /// Massa efectiva na normal e na tangente.
    kn: f32,
    kt: f32,
    /// O alvo do ressalto: `e · vrel₀`, já filtrado pelo [`Leis::limiar_salto`].
    restituicao: f32,
    mu: f32,
    /// O impulso NORMAL de referência da lei de 2026-09-15 — `max(vrel, pen/dt) / Σw`. É dele que
    /// sai o tecto de Coulomb e o do rolamento, quando o [`Leis::tecto_por_lambda`] está desligado.
    normal_fixo: f32,
    /// ⭐ **O ROLAMENTO de cada lado** `[lo, hi]` — é da PEÇA, nunca do par (ver [`rolamento_um`]).
    rolar: [f32; 2],
    /// O passo deste par, para converter uma velocidade angular num ângulo.
    passo: f32,
    /// O `λ` acumulado ao longo das varreduras.
    lambda: f32,
    lambda_t: f32,
    lambda_r: [f32; 2],
}

/// **O SOLVER DE VELOCIDADE** sobre as posições `p` em que os contactos de facto aconteceram (as de
/// ANTES da separação). `mov` é reescrito no sítio.
///
/// ⚠️ **Os manifestos constroem-se UMA vez** e as varreduras correm sobre eles: a geometria é a do
/// início do tique, e recalculá-la entre varreduras poria o solver a perseguir um alvo que ele
/// próprio move. *É também o que torna `iteracoes = 8` mais barato que oito chamadas desta função.*
///
/// # Panics
///
/// Se as colunas de `mov` não tiverem o comprimento de `p` — duas colunas da mesma corrente com
/// comprimentos diferentes não são uma pergunta com resposta.
pub fn impulsos(
    p: &[[f32; 2]],
    mov: &mut Movimento<'_>,
    pecas: &Pecas<'_>,
    dt: impl Fn(usize) -> f32,
    leis: Leis,
) {
    let n = p.len();
    assert_eq!(mov.vel.len(), n, "uma velocidade por peca");
    assert_eq!(mov.giro.len(), n, "um giro por peca");
    assert_eq!(mov.spin.len(), n, "um spin por peca");
    assert_eq!(pecas.colisores.len(), n, "um colisor por peca");
    assert_eq!(pecas.pesos.len(), n, "um peso por peca");
    assert_eq!(pecas.inv_inercia.len(), n, "uma inercia por peca");
    // ⚠️ A velocidade angular de ENTRADA, em radianos/s. Ela sai do que o `spin` já rodou neste
    // passo (doc 109 §6: este modelo não tem coluna de velocidade angular própria do contacto), e
    // é a mesma leitura que a lei de 2026-09-15 fazia para o atrito ver uma bola a girar.
    let mut w: Vec<f32> = (0..n)
        .map(|i| {
            let passo = dt(i);
            if passo <= 0.0 {
                return 0.0;
            }
            pecas
                .deslize
                .map_or(0.0, |d| d.girou_antes.get(i).copied().unwrap_or(0.0))
                .to_radians()
                / passo
        })
        .collect();
    let w0 = w.clone();
    let mut restricoes = monta(p, pecas, &dt, leis, mov.vel, &w);
    for _ in 0..leis.iteracoes.max(1) {
        for r in &mut restricoes {
            resolve_um(r, mov, &mut w, pecas, leis);
        }
    }
    if leis.angular {
        for i in 0..n {
            let d = (w[i] - w0[i]) * GRAUS;
            if d.is_finite() {
                mov.spin[i] += d;
            }
        }
    }
}

/// Os manifestos do tique, um ponto de cada vez, com as massas efectivas e o alvo do ressalto já
/// calculados — tudo o que não muda entre varreduras.
fn monta(
    p: &[[f32; 2]],
    pecas: &Pecas<'_>,
    dt: &impl Fn(usize) -> f32,
    leis: Leis,
    vel: &[[f32; 2]],
    w: &[f32],
) -> Vec<Restricao> {
    let n = p.len();
    let inv_inercia = pecas.inv_inercia;
    let massa = |peso: f32, inv_i: f32, b: f32| peso + inv_i * b * b;
    // ⚠️ A MESMA porta que o `separate` usa (`super::ativo`) — o que conta como peça é uma lei só.
    let ativo: Vec<bool> = (0..n)
        .map(|i| super::ativo(p[i], pecas.colisores[i].as_ref()))
        .collect();
    let mut out = Vec::new();
    // ⚠️ O laço é `lo < hi` e a normal vai do MENOR para o MAIOR — a mesma ordem do par que o
    // `separate` usa, para os dois lados de um contacto serem exactamente opostos.
    for lo in 0..n {
        if !ativo[lo] {
            continue;
        }
        for hi in (lo + 1)..n {
            if !ativo[hi] {
                continue;
            }
            let (Some(clo), Some(chi)) = (pecas.colisores[lo], pecas.colisores[hi]) else {
                continue;
            };
            let Some(m) = manifesto(&clo, p[lo], &chi, p[hi], (lo + hi) % 2 == 0) else {
                continue;
            };
            let soma = pecas.pesos[lo] + pecas.pesos[hi];
            // Dois obstáculos não têm momento a trocar.
            if soma <= 0.0 {
                continue;
            }
            let passo = dt(lo).max(dt(hi));
            let (clo_c, chi_c) = (clo.centro(p[lo]), chi.centro(p[hi]));
            let (mlo, mhi) = (pecas.material(lo), pecas.material(hi));
            let e = atrito::salto(mlo.salto, mhi.salto);
            let mu = atrito::mu(mlo.atrito, mhi.atrito);
            // ⭐⭐⭐ **UM ou DOIS pontos** — ver [`Leis::por_ponto`]. A lei de 2026-09-15 lia o
            // primeiro e o seu comentário dizia *«a normal é a mesma nos dois, logo calcula-se uma
            // vez»*: verdade para a TRANSLAÇÃO e falso para o binário, que é o que a wave curou.
            let pontos: &[Contacto] = if leis.por_ponto {
                m.pontos()
            } else {
                &m.pontos()[..1]
            };
            for c in pontos {
                let bn = [c.braco(clo_c), c.braco(chi_c)];
                let bt = [c.braco_tangente(clo_c), c.braco_tangente(chi_c)];
                // ⭐ Na normal a alavanca só entra quando a rotação PERSISTE: sem velocidade
                // angular o impulso normal não roda ninguém, e pôr a alavanca na massa efectiva
                // deixaria a peça a receber menos impulso por uma rotação que não vai acontecer.
                let kn = if leis.angular {
                    massa(pecas.pesos[lo], inv_inercia[lo], bn[0])
                        + massa(pecas.pesos[hi], inv_inercia[hi], bn[1])
                } else {
                    soma
                };
                let kt = massa(pecas.pesos[lo], inv_inercia[lo], bt[0])
                    + massa(pecas.pesos[hi], inv_inercia[hi], bt[1]);
                let vn = normal_relativa(c, lo, hi, bn, vel, w, leis);
                // ⭐⭐⭐ **A FORÇA NORMAL DE UM CONTACTO EM REPOUSO É A PENETRAÇÃO** — e sem esta
                // linha o atrito desaparece exactamente onde ele mais importa.
                //
                // ⛔⛔ A 1.ª redacção prendia o tecto de Coulomb à velocidade de APROXIMAÇÃO, e num
                // contacto assente ela é **zero**: a peça já não se aproxima de nada. *O atrito
                // ficava ligado só no instante do embate e desligado no resto do tempo* — e as
                // fixturas que o apanharam foram as dos discos, que não têm gravidade nenhuma.
                //
                // ⚠️ **As duas leituras são a MESMA grandeza:** numa pilha sob gravidade a
                // penetração por sub-passo é `~g·dt²`, logo `pen/dt ≈ g·dt`, que é exactamente o
                // `vrel` que a gravidade repõe. Tomar o MAIOR cobre o embate e o repouso.
                let normal_ref = vn.max(c.penetracao / passo.max(f32::MIN_POSITIVE));
                out.push(Restricao {
                    lo,
                    hi,
                    n: c.normal,
                    t: c.tangente(),
                    bn,
                    bt,
                    kn,
                    kt,
                    // ⚠️ O ressalto sai de um ALVO FIXO, medido na velocidade de ENTRADA. Recalculá-lo
                    // a cada varredura fá-lo-ia compor-se `iteracoes` vezes.
                    restituicao: if vn > leis.limiar_salto { e * vn } else { 0.0 },
                    mu,
                    normal_fixo: normal_ref / soma,
                    rolar: [mlo.rolar, mhi.rolar],
                    passo,
                    lambda: 0.0,
                    lambda_t: 0.0,
                    lambda_r: [0.0, 0.0],
                });
            }
        }
    }
    out
}

/// A velocidade relativa na NORMAL, no ponto do contacto.
///
/// ⚠️ **O termo `ω·(r × n)` só entra com [`Leis::angular`]** — e é ele que faz uma peça a girar ser
/// vista a aproximar-se pela ponta que desce. Sem velocidade angular não há `ω` que persista, e
/// lê-lo ali punha o impulso normal a responder a uma rotação que já foi deitada fora.
fn normal_relativa(
    c: &Contacto,
    lo: usize,
    hi: usize,
    bn: [f32; 2],
    vel: &[[f32; 2]],
    w: &[f32],
    leis: Leis,
) -> f32 {
    let linear = dot([vel[lo][0] - vel[hi][0], vel[lo][1] - vel[hi][1]], c.normal);
    if leis.angular {
        linear + w[lo] * bn[0] - w[hi] * bn[1]
    } else {
        linear
    }
}

/// Uma restrição, uma varredura: o impulso normal e depois o de Coulomb, os dois com o `λ` preso.
fn resolve_um(
    r: &mut Restricao,
    mov: &mut Movimento<'_>,
    w: &mut [f32],
    pecas: &Pecas<'_>,
    leis: Leis,
) {
    let (lo, hi) = (r.lo, r.hi);
    let (peso_lo, peso_hi) = (pecas.pesos[lo], pecas.pesos[hi]);
    let (inv_lo, inv_hi) = (pecas.inv_inercia[lo], pecas.inv_inercia[hi]);
    // ⭐⭐ **O IMPULSO NORMAL, com o `λ` ACUMULADO preso em `≥ 0`.**
    //
    // ⚠️ **A guarda não é sobre o impulso desta varredura, é sobre o TOTAL** — é isso que deixa uma
    // varredura desfazer o que a anterior sobrepassou sem nunca o contacto chegar a PUXAR.
    if r.kn > 0.0 {
        let vn = {
            let linear = dot(
                [
                    mov.vel[lo][0] - mov.vel[hi][0],
                    mov.vel[lo][1] - mov.vel[hi][1],
                ],
                r.n,
            );
            if leis.angular {
                linear + w[lo] * r.bn[0] - w[hi] * r.bn[1]
            } else {
                linear
            }
        };
        let alvo = (r.lambda + (vn + r.restituicao) / r.kn).max(0.0);
        let dj = alvo - r.lambda;
        if dj.is_finite() && dj != 0.0 {
            r.lambda = alvo;
            aplica(mov, lo, hi, r.n, dj, peso_lo, peso_hi);
            if leis.angular {
                gira(w, lo, hi, r.bn, dj, inv_lo, inv_hi);
            }
        }
    }
    atrito_um(r, mov, w, pecas, leis);
    // ⚠️ **O rolamento corre DEPOIS do atrito e lê o `ω` já corrigido por ele** — a mesma ordem da
    // taça (`sim.collide`): os dois escrevem a mesma grandeza, e lidos do mesmo `ω` este desfaria
    // parte do giro que aquele acabou de dar. E corre com `μ = 0` também: achatar-se não depende
    // de esfregar.
    rolamento_um(r, w, pecas, leis);
}

/// O impulso NORMAL de referência deste contacto — a porta única dos dois tectos (Coulomb e
/// rolamento), para as duas leis nunca lerem normais diferentes.
fn normal(r: &Restricao, leis: Leis) -> f32 {
    if leis.tecto_por_lambda {
        r.lambda
    } else {
        r.normal_fixo
    }
}

/// A metade TANGENCIAL de uma restrição, numa varredura.
fn atrito_um(
    r: &mut Restricao,
    mov: &mut Movimento<'_>,
    w: &mut [f32],
    pecas: &Pecas<'_>,
    leis: Leis,
) {
    let (lo, hi) = (r.lo, r.hi);
    let (peso_lo, peso_hi) = (pecas.pesos[lo], pecas.pesos[hi]);
    let (inv_lo, inv_hi) = (pecas.inv_inercia[lo], pecas.inv_inercia[hi]);
    // ⭐⭐⭐ **E A METADE TANGENCIAL — o ATRITO ao nível da VELOCIDADE.**
    //
    // ⚠️⚠️ **Sem ela o impulso normal deixa a pilha MAIS solta do que a lei que substituiu**, e a
    // medição di-lo: o rodopio da `=114` sobe de `3,0..3,1°` para `24,8..33,6°`. A razão é que a lei
    // velha matava a velocidade ABSOLUTA — um sorvedouro de energia que também comia o deslize. Com
    // o momento a conservar-se, o que trava uma pilha é o atrito, e ⛔ o desta crate era **só
    // posicional**: ele desfaz o deslize já acontecido e não tira a velocidade que o vai repetir.
    //
    // A lei é a de Coulomb sobre a tangente, com o tecto no impulso NORMAL deste mesmo contacto —
    // *é isso que a torna uma lei e não um amortecedor*: uma peça que mal encosta mal é travada, e
    // uma que carrega peso é travada muito.
    if r.mu <= 0.0 || r.kt <= 0.0 {
        return;
    }
    // ⭐⭐⭐ **A velocidade tangencial é a do PONTO DE CONTACTO, não a do centro** — e o ponto de
    // contacto de um corpo que RODA move-se mesmo com o centro parado. Sem este termo uma bola a
    // girar não esfrega contra o chão: ela gira para sempre. Em 2D a contribuição da rotação na
    // tangente é `ω · (r · n)`, pela identidade `r × perp(n) = r · n`.
    let vt = dot(
        [
            mov.vel[lo][0] - mov.vel[hi][0],
            mov.vel[lo][1] - mov.vel[hi][1],
        ],
        r.t,
    ) + w[lo] * r.bt[0]
        - w[hi] * r.bt[1];
    // ⚠️ O tecto sai do impulso normal SEM o salto: o ressalto devolve energia na normal e não
    // compra aderência nenhuma na tangente.
    let tecto = r.mu * normal(r, leis);
    let alvo = (r.lambda_t + vt / r.kt).clamp(-tecto, tecto); // CLAMP-OK: tecto >= 0
    let djt = alvo - r.lambda_t;
    if !djt.is_finite() || djt == 0.0 {
        return;
    }
    r.lambda_t = alvo;
    aplica(mov, lo, hi, r.t, djt, peso_lo, peso_hi);
    // ⭐ **E ELE REPARTE-SE ENTRE TRAVAR E RODAR**, pela massa efectiva ao longo da tangente:
    //
    // ```text
    //   kt = w + invI · (r · n)²          jt = clamp(vt / Σkt, ±μ·jn)
    //   Δv = −t · jt · w                  Δω = −(r·n) · jt · invI
    // ```
    //
    // ⭐ **Numa bola pousada isto dá o rolamento de manual, ao bit:** `kt = w + 2w = 3w`, logo a
    // translação leva `⅓` e a rotação `⅔`, e a soma no ponto de contacto é exactamente `−vt`. *A
    // bola deixa de derrapar porque começou a rodar, não porque travou.*
    if leis.angular {
        gira(w, lo, hi, r.bt, djt, inv_lo, inv_hi);
    } else {
        // ⚠️ `jt` é uma VELOCIDADE e a coluna `rot` é um ÂNGULO: a conversão é o `dt` deste par.
        let (glo, ghi) = (
            -r.bt[0] * djt * inv_lo * r.passo * GRAUS,
            r.bt[1] * djt * inv_hi * r.passo * GRAUS,
        );
        if glo.is_finite() && ghi.is_finite() {
            mov.giro[lo] += glo;
            mov.giro[hi] += ghi;
        }
    }
}

/// ⭐⭐⭐ **O ROLAMENTO de uma restrição** (doc 111 §10) — o que faz uma peça a rolar **parar
/// sozinha**, pela MESMA porta e na MESMA forma que a taça ([`atrito::rolamento`]).
///
/// ⛔⛔ **Até 2026-09-16 o botão `Rolling` do cartão era MORTO no contacto peça×peça**, e o contrato
/// da coluna dizia-o por escrito (*«ali não há nada que este número possa travar»*): a rotação era
/// posicional e não havia velocidade angular a resistir. O doc 111 §9 deu-lha, e a frase passou a
/// ser falsa no mesmo commit — *quem move o número que tornava algo inalcançável tem de reconferir a
/// nota* (§0.0).
///
/// ⭐⭐ **Cada peça é travada contra o PRÓPRIO giro, com o PRÓPRIO rolamento** — a forma da taça,
/// termo a termo. ⛔⛔ **A forma do PAR (o `ω` relativo, com a massa angular dos dois) foi
/// construída, medida e REFUTADA:** ela é a física certa para uma bola a rolar sobre outra, e numa
/// pilha de CAIXAS é um acoplamento espúrio — uma caixa a tombar ARRASTA a vizinha parada e
/// desaloja-a. Medido na `=114`, rodopio janela a janela:
///
/// ```text
///   Rolling |  forma do par (120..180 · 480..540)  |  esta (120..180 · 480..540)
///   --------|--------------------------------------|---------------------------
///      0    |          2,66  ·  0,51               |      2,66  ·  0,51
///      0,25 |         40,98  ·  0,49               |      1,19  ·  0,03
///      0,75 |         28,90  ·  5,31               |      1,22  ·  0,02
///      1,5  |         32,42  ·  6,34               |      1,15  ·  0,02
/// ```
///
/// ⇒ o botão fazia o **contrário** do nome. Esta forma é sempre dissipativa: ela só tira giro a
/// quem o tem, e nunca o põe numa peça parada. ⚠️ O preço declarado: uma bola a rolar sobre uma
/// plataforma que GIRA é travada contra o mundo, não contra a plataforma — um caso que nenhuma cena
/// do produto tem.
///
/// ⚠️ **Sem velocidade angular não corre** — não há giro que persista para travar, e a lei de
/// 2026-09-15 fica ao bit. Com `Rolling = 0` (o default do cartão) o tecto é zero e ela também não
/// mexe em nada: a `=114` aprovada não se move.
fn rolamento_um(r: &mut Restricao, w: &mut [f32], pecas: &Pecas<'_>, leis: Leis) {
    if !leis.angular {
        return;
    }
    let n = normal(r, leis);
    for (lado, peca) in [r.lo, r.hi].into_iter().enumerate() {
        let inv = pecas.inv_inercia[peca];
        if r.rolar[lado] <= 0.0 || inv <= 0.0 {
            continue;
        }
        // ⚠️ O `clamp` da porta é sobre o momento que a peça TEM: o pior caso é parar o giro
        // nesta varredura — ⛔ nunca invertê-lo, e é por isso que o `ROLLING_MAX` não tem
        // divergência a temer.
        let momento = r.lambda_r[lado] + w[peca] / inv;
        let alvo = atrito::rolamento(momento, r.rolar[lado], n, r.bt[lado]);
        let dr = alvo - r.lambda_r[lado];
        if dr.is_finite() && dr != 0.0 {
            r.lambda_r[lado] = alvo;
            w[peca] -= inv * dr;
        }
    }
}

/// O impulso `dj` na direcção `eixo`, repartido pelas massas — `lo` recua, `hi` avança.
///
/// ⚠️ Um `NaN` que entre por uma coluna torta não contamina a cena inteira: o par é saltado.
fn aplica(
    mov: &mut Movimento<'_>,
    lo: usize,
    hi: usize,
    eixo: [f32; 2],
    dj: f32,
    peso_lo: f32,
    peso_hi: f32,
) {
    let (dlo, dhi) = (dj * peso_lo, dj * peso_hi);
    let novo_lo = [
        mov.vel[lo][0] - eixo[0] * dlo,
        mov.vel[lo][1] - eixo[1] * dlo,
    ];
    let novo_hi = [
        mov.vel[hi][0] + eixo[0] * dhi,
        mov.vel[hi][1] + eixo[1] * dhi,
    ];
    if novo_lo.iter().chain(&novo_hi).all(|x| x.is_finite()) {
        mov.vel[lo] = novo_lo;
        mov.vel[hi] = novo_hi;
    }
}

/// A metade angular do mesmo impulso: `Δω = ∓ alavanca · dj · invI`.
fn gira(
    w: &mut [f32],
    lo: usize,
    hi: usize,
    alavanca: [f32; 2],
    dj: f32,
    inv_lo: f32,
    inv_hi: f32,
) {
    let (dlo, dhi) = (-alavanca[0] * dj * inv_lo, alavanca[1] * dj * inv_hi);
    if dlo.is_finite() && dhi.is_finite() {
        w[lo] += dlo;
        w[hi] += dhi;
    }
}
