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
//! ⛔ **O que isto NÃO é:** não há velocidade ANGULAR (doc 109 §6) — o impulso é translacional, e o
//! atrito continua a ser a projecção de posição que o [`super::atrito`] descreve. Dar giro aqui
//! seria um corpo rígido a sério, que é outra obra.

use super::{GRAUS, Pecas, atrito, dot, manifesto};

/// **UMA passagem de velocidade sobre os contactos**, sobre as posições `p` em que eles de facto
/// aconteceram (as de ANTES da separação). `vel` é reescrito no sítio.
///
/// ⚠️ **Uma passagem, não `varreduras`**: a separação de posição itera porque é uma projecção que
/// converge; um impulso é uma troca de momento que acontece **uma vez** por contacto. Repeti-lo
/// multiplicaria o ressalto pelo número de varreduras.
///
/// # Panics
///
/// Se `vel` não tiver o comprimento de `p` — duas colunas da mesma corrente com comprimentos
/// diferentes não são uma pergunta com resposta.
pub fn impulsos(
    p: &[[f32; 2]],
    vel: &mut [[f32; 2]],
    giro: &mut [f32],
    pecas: &Pecas<'_>,
    dt: impl Fn(usize) -> f32,
) {
    let n = p.len();
    assert_eq!(vel.len(), n, "uma velocidade por peca");
    assert_eq!(giro.len(), n, "um giro por peca");
    assert_eq!(pecas.colisores.len(), n, "um colisor por peca");
    assert_eq!(pecas.pesos.len(), n, "um peso por peca");
    assert_eq!(pecas.inv_inercia.len(), n, "uma inercia por peca");
    let inv_inercia = pecas.inv_inercia;
    let massa = |w: f32, inv_i: f32, b: f32| w + inv_i * b * b;
    // ⚠️ A MESMA porta que o `separate` usa (`super::ativo`) — o que conta como peça é uma lei só.
    let ativo: Vec<bool> = (0..n)
        .map(|i| super::ativo(p[i], pecas.colisores[i].as_ref()))
        .collect();
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
            // ⚠️ A normal é a MESMA nos dois pontos de um trecho (é a face que a dá), então o
            // impulso do par calcula-se UMA vez — repartir por ponto não mudaria a soma e faria
            // duas trocas de momento onde a física tem uma.
            let Some(c) = m.pontos().first() else {
                continue;
            };
            let vrel = dot([vel[lo][0] - vel[hi][0], vel[lo][1] - vel[hi][1]], c.normal);
            let soma = pecas.pesos[lo] + pecas.pesos[hi];
            // Dois obstáculos não têm momento a trocar.
            if soma <= 0.0 {
                continue;
            }
            let passo = dt(lo).max(dt(hi));
            // ⭐⭐⭐ **A FORÇA NORMAL DE UM CONTACTO EM REPOUSO É A PENETRAÇÃO** — e sem esta linha o
            // atrito desaparece exactamente onde ele mais importa.
            //
            // ⛔⛔ A 1.ª redacção prendia o tecto de Coulomb à velocidade de APROXIMAÇÃO (`vrel`), e
            // num contacto assente ela é **zero**: a peça já não se aproxima de nada. *O atrito
            // ficava ligado só no instante do embate e desligado no resto do tempo* — e as fixturas
            // que o apanharam foram as dos discos, que não têm gravidade nenhuma e portanto `vrel`
            // exactamente `0`.
            //
            // ⚠️ **As duas leituras são a MESMA grandeza:** numa pilha sob gravidade a penetração
            // por sub-passo é `~g·dt²`, logo `pen/dt ≈ g·dt`, que é exactamente o `vrel` que a
            // gravidade repõe a cada sub-passo. Tomar o MAIOR das duas cobre o embate (onde manda a
            // velocidade) e o repouso (onde manda o peso), sem duas leis.
            let normal_ref = vrel.max(c.penetracao / passo.max(f32::MIN_POSITIVE));
            let (clo_c, chi_c) = (clo.centro(p[lo]), chi.centro(p[hi]));
            let (mlo, mhi) = (pecas.material(lo), pecas.material(hi));
            let e = atrito::salto(mlo.salto, mhi.salto);
            // ⚠️ **O impulso NORMAL só responde a quem se aproxima** — um contacto empurra, nunca
            // puxa. O ATRITO, esse, age em repouso também: é por isso que a guarda é aqui e não
            // à entrada do par.
            let j = if vrel > 0.0 {
                (1.0 + e) * vrel / soma
            } else {
                0.0
            };
            let (dlo, dhi) = (j * pecas.pesos[lo], j * pecas.pesos[hi]);
            let mut novo_lo = [
                vel[lo][0] - c.normal[0] * dlo,
                vel[lo][1] - c.normal[1] * dlo,
            ];
            let mut novo_hi = [
                vel[hi][0] + c.normal[0] * dhi,
                vel[hi][1] + c.normal[1] * dhi,
            ];
            // ⭐⭐⭐ **E A METADE TANGENCIAL — o ATRITO ao nível da VELOCIDADE.**
            //
            // ⚠️⚠️ **Sem ela o impulso normal deixa a pilha MAIS solta do que a lei que substituiu**,
            // e a medição di-lo: o rodopio da `=114` sobe de `3,0..3,1°` para `24,8..33,6°`. A razão
            // é que a lei velha matava a velocidade ABSOLUTA — um sorvedouro de energia que também
            // comia o deslize. Com o momento a conservar-se, o que trava uma pilha é o atrito, e
            // ⛔ o desta crate era **só posicional**: ele desfaz o deslize já acontecido e não tira
            // a velocidade que o vai repetir no tique seguinte.
            //
            // A lei é a de Coulomb sobre a tangente, com o tecto no impulso NORMAL deste mesmo
            // contacto — *é o mesmo `j`, e é isso que a torna uma lei e não um amortecedor*:
            // uma peça que mal encosta mal é travada, e uma que carrega peso é travada muito.
            let t = c.tangente();
            let (bt_lo, bt_hi) = (c.braco_tangente(clo_c), c.braco_tangente(chi_c));
            // ⭐⭐⭐ **A velocidade tangencial é a do PONTO DE CONTACTO, não a do centro** — e o
            // ponto de contacto de um corpo que RODA move-se mesmo com o centro parado.
            //
            // ⛔ Sem este termo uma bola a girar **não esfrega** contra o chão: ela gira para
            // sempre, sem atrito nenhum, porque o centro dela está quieto. Em 2D a contribuição da
            // rotação na tangente é `ω · (r · n)` — a mesma alavanca [`Contacto::braco_tangente`]
            // que reparte o impulso, pela identidade `r × perp(n) = r · n`.
            //
            // ⚠️ A velocidade angular sai do que a peça JÁ rodou neste passo (o `spin` integrado,
            // mais o que o contacto lhe acrescentou), dividido pelo passo — este modelo não tem
            // coluna de velocidade angular, e é isso que o doc 109 §6 nomeia.
            let omega = |i: usize| {
                pecas
                    .deslize
                    .map_or(0.0, |d| d.girou_antes.get(i).copied().unwrap_or(0.0))
                    .to_radians()
                    / passo.max(f32::MIN_POSITIVE)
            };
            let vt = dot([novo_lo[0] - novo_hi[0], novo_lo[1] - novo_hi[1]], t) + omega(lo) * bt_lo
                - omega(hi) * bt_hi;
            let mu = atrito::mu(mlo.atrito, mhi.atrito);
            let (mut glo, mut ghi) = (0.0_f32, 0.0_f32);
            if mu > 0.0 && vt != 0.0 {
                // ⭐⭐⭐ **E ELE REPARTE-SE ENTRE TRAVAR E RODAR**, pela massa efectiva ao longo da
                // tangente — exactamente como a lei POSICIONAL que ele substitui (doc 109 §7):
                //
                // ```text
                //   kt = w + invI · (r · n)²          jt = clamp(vt / Σkt, ±μ·jn)
                //   Δv = −t · jt · w                  Δω = −(r·n) · jt · invI
                // ```
                //
                // ⭐ **Numa bola pousada isto dá o rolamento de manual, ao bit:** `kt = w + 2w = 3w`,
                // logo a translação leva `⅓` e a rotação `⅔`, e a soma no ponto de contacto é
                // exactamente `−vt`. *A bola deixa de derrapar porque começou a rodar, não porque
                // travou* — e sem esta metade um disco **PÁRA A SECO**, que é o que a auditoria do
                // doc 111 §6 mediu (`ω·R ≈ 0` a todo `μ`).
                let kt = massa(pecas.pesos[lo], inv_inercia[lo], bt_lo)
                    + massa(pecas.pesos[hi], inv_inercia[hi], bt_hi);
                if kt > 0.0 {
                    // ⚠️ O tecto sai do impulso normal SEM o salto: o ressalto devolve energia na
                    // normal e não compra aderência nenhuma na tangente.
                    let tecto = mu * (normal_ref / soma);
                    let jt = (vt / kt).clamp(-tecto, tecto);
                    novo_lo = [
                        novo_lo[0] - t[0] * jt * pecas.pesos[lo],
                        novo_lo[1] - t[1] * jt * pecas.pesos[lo],
                    ];
                    novo_hi = [
                        novo_hi[0] + t[0] * jt * pecas.pesos[hi],
                        novo_hi[1] + t[1] * jt * pecas.pesos[hi],
                    ];
                    // ⚠️ `jt` é uma VELOCIDADE e a coluna `rot` é um ÂNGULO: a conversão é o `dt`
                    // deste par. *É o único sítio em que o passo volta a ser preciso, e por um
                    // motivo diferente do de antes* — ali ele punha tecto a uma velocidade
                    // inventada; aqui converte uma velocidade angular real num ângulo.
                    glo = -bt_lo * jt * inv_inercia[lo] * passo * GRAUS;
                    ghi = bt_hi * jt * inv_inercia[hi] * passo * GRAUS;
                }
            }
            // ⚠️ Um `NaN` que entre por uma coluna torta não contamina a cena inteira.
            if novo_lo
                .iter()
                .chain(&novo_hi)
                .chain(&[glo, ghi])
                .all(|x| x.is_finite())
            {
                vel[lo] = novo_lo;
                vel[hi] = novo_hi;
                giro[lo] += glo;
                giro[hi] += ghi;
            }
        }
    }
}
