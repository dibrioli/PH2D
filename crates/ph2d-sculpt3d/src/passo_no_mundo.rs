//! ⭐⭐⭐ **O PASSO MEDIDO SOBRE A SUPERFÍCIE** — a cura do vinco pontilhado
//! junto à silhueta (report do dono, 2026-09-16; `SPEC_pincel_afiado.md` §16).
//!
//! # O defeito, com número
//!
//! O passo de um traço arrastado é contado em píxeis de ECRÃ. Um passo de `p`
//! píxeis sobre uma superfície cuja normal faz `θ` com a vista percorre, na
//! superfície, `(p / ppu) / cos θ` — e `1/cos θ` cresce sem tecto na silhueta.
//! Medido na nossa casa: numa esfera de raio `1` a `400` px por unidade, com o
//! pincel de raio `0,1`, o salto entre dabs vale **`0,59` do raio do pincel**
//! junto à borda contra **`0,10`** no meio. Os dabs deixam de se sobrepor, e o
//! vinco sai como uma fila de toques separados.
//!
//! ⚠️ **O alvo tem o MESMO defeito com o pincel de fábrica** (espec §16.2,
//! re-derivado por dois auditores): ali a ondulação do vinco vai de `0,002` no
//! meio a `0,977` na borda, com a profundidade a cair `5,19×`. *O nosso produto
//! é fiel; o que esta porta cura é um defeito herdado.*
//!
//! # A lei, e ela é NOSSA
//!
//! ⛔⛔ **O alvo tem um eixo que cura o pontilhado, e ele NÃO se copia** (espec
//! §16.9): lá o passo em mundo é comparado contra o acerto **VIVO**, e o acerto
//! vivo afunda com o vinco — uma realimentação que compra `10×` a profundidade
//! junto à borda, dependência da **taxa de amostragem do rato** (`−30 %` a
//! `−51 %`) e do **sentido do gesto** (`3,3×`). Isso é o traço a deixar de ser
//! facto do CAMINHO, que é a lei que esta casa pagou seis vezes no Painter.
//!
//! ⇒ aqui o passo é medido sobre a superfície **CONGELADA no pen-down**:
//!
//! ```text
//! passo_de_mundo = (espaçamento_% / 50) · raio_de_mundo
//! percorre-se o caminho do cursor em ECRÃ, fino;
//! cada candidato projecta-se na superfície do PEN-DOWN;
//! acumula-se a corda entre projecções consecutivas;
//! fechado um passo, carimba-se — e o CENTRO do dab continua a ser o acerto VIVO.
//! ```
//!
//! ⭐ **As três propriedades que isso compra, e que o eixo do alvo perde:** sem
//! realimentação (a superfície congelada não se move), sem dependência da taxa
//! de amostragem (o resíduo viaja, como no [`crate::walk`]) e sem dependência do
//! sentido.
//!
//! ⚠️ **Num plano de frente para a vista esta lei e a do ecrã COINCIDEM** por
//! construção (`cos θ = 1`), e é isso que mantém intacto todo o corpus de
//! paridade deste pincel — que corre em planos e num cilindro percorrido ao
//! longo do eixo.

use ph2d_mesh::{Hit, Mesh};

/// **Quantos dabs um único píxel do caminho pode fechar.**
///
/// ⚠️ **O recurso é o RELÓGIO DO DAB, e o tecto é dele que sai:** junto à
/// silhueta um píxel de ecrã cobre arco sem tecto (espec §16.6 mede `1,91` dabs
/// por píxel a `84°`, e a razão cresce com `1/cos θ`), e sem cerca um gesto que
/// roce a borda pediria dezenas de dabs por píxel de rato.
///
/// ⚠️ **Repetir o dab no mesmo sítio não é o mesmo que N dabs espalhados**, e é
/// a resposta honesta: abaixo de um píxel o artista não consegue endereçar a
/// superfície, e a auto-limitação do vinco (§4) faz a repetição CONVERGIR em vez
/// de explodir.
pub const MAX_DABS_POR_PASSO: u32 = 8;

/// **O passo deste verbo medido no MUNDO**, ou `None` quando ele não declara um
/// espaçamento próprio (e então vale a régua de ecrã da casa).
#[must_use]
pub fn passo_no_mundo(pincel: &crate::Brush, raio_de_mundo: f32) -> Option<f32> {
    if !pincel.verb.mede_o_passo_no_mundo() {
        return None;
    }
    crate::espacamento_do_traco(pincel)
        .map(|pct| crate::passo_de_um_espacamento(pct, raio_de_mundo))
}

/// **O acumulador do caminho sobre a superfície congelada.**
///
/// ⚠️ **O resíduo VIAJA, como no [`crate::walk`]** — e é ele que faz um traço
/// lento depositar a mesma densidade que um rápido. Zerá-lo a cada candidato
/// devolveria a dependência da taxa de amostragem pela porta dos fundos.
#[derive(Clone, Debug, Default)]
pub struct CaminhoNoMundo {
    anterior: Option<[f32; 3]>,
    acumulado: f32,
    /// **O passo de ecrã PREVISTO para o candidato seguinte** — ver
    /// [`Self::percorre`].
    ///
    /// ⚠️⚠️ **Ele vive no CAMINHO e não no segmento, e isso é uma lei e não
    /// arrumação:** um evento de rato pode ser de UM píxel, e uma previsão que
    /// recomeçasse a cada evento nunca chegaria a afinar — o caso mais comum do
    /// artista (arrastar devagar) seria exactamente o que ficaria sem cura.
    /// ⛔ Medido: com a previsão local, o mesmo caminho a `1` e a `4` píxeis por
    /// evento divergia **`20,8 %`** numa banda, que é a dependência da taxa de
    /// amostragem a entrar pela porta dos fundos.
    previsto: Option<f32>,
}

impl CaminhoNoMundo {
    /// Um caminho que ainda não andou.
    #[must_use]
    pub fn novo() -> Self {
        Self::default()
    }

    /// **ANDA até o próximo ponto congelado do caminho** e devolve quantos
    /// passos ele FECHA (`0` quando ainda não vale um).
    ///
    /// ⚠️ **O primeiro ponto nunca fecha passo nenhum:** ele é a âncora, e quem
    /// carimba no pen-down é o chamador — a mesma repartição do `walk`, cujo
    /// primeiro ponto devolvido está já a um passo da âncora.
    ///
    /// ⚠️ **Um passo não-finito ou não-positivo devolve `0` e não anda**: a lei
    /// não existe aí, e deixar passar entregaria uma divisão que carimba a peça
    /// inteira.
    pub fn avanca(&mut self, congelado: [f32; 3], passo: f32) -> u32 {
        if !passo.is_finite() || passo <= 0.0 || !congelado.iter().all(|c| c.is_finite()) {
            return 0;
        }
        let Some(anterior) = self.anterior.replace(congelado) else {
            return 0;
        };
        let d = {
            let (dx, dy, dz) = (
                congelado[0] - anterior[0],
                congelado[1] - anterior[1],
                congelado[2] - anterior[2],
            );
            (dx * dx + dy * dy + dz * dz).sqrt()
        };
        self.acumulado += d;
        if self.acumulado < passo {
            return 0;
        }
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let fecha = ((self.acumulado / passo).floor() as u32).min(MAX_DABS_POR_PASSO);
        // ⚠️ **O resíduo é o que SOBRA depois dos passos fechados**, e não zero:
        // é a mesma lei do carry do [`crate::walk`], e é ela que torna o traço
        // um facto do caminho e não da taxa de eventos.
        self.acumulado -= passo * f32::from(u16::try_from(fecha).unwrap_or(u16::MAX));
        fecha
    }

    /// Esquece o caminho — o pen-up.
    pub fn esquece(&mut self) {
        *self = Self::novo();
    }

    /// ⭐⭐⭐ **PERCORRE um segmento do caminho de ecrã e carimba onde o ARCO
    /// manda** — a lei inteira, num sítio só, com dois chamadores (o app e a
    /// bancada).
    ///
    /// `congelado(t)` devolve onde o píxel `t` do segmento cai na superfície do
    /// **pen-down** (`None` fora da peça); `carimba(t)` põe o dab ali e devolve
    /// `false` para parar o gesto.
    ///
    /// ⭐⭐ **A granularidade dos candidatos é ADAPTATIVA, e é isso que faz a
    /// cura chegar à silhueta.** Um píxel de ecrã cobre `1/cos θ` de arco, logo
    /// junto à borda ele vale vários passos — e uma amostragem fixa de um píxel
    /// deixaria o vinco a saltar de píxel em píxel (medido: a ondulação da banda
    /// `83–88°` fica em `0,943` contra `0,974` sem cura nenhuma). ⇒ depois de
    /// cada candidato o passo de ecrã seguinte é **previsto** do arco que o
    /// último mediu.
    ///
    /// ⚠️ **A previsão lê a superfície CONGELADA e nada mais** — não é
    /// realimentação: a superfície que ela mede não se move enquanto o traço
    /// corre, que é exactamente o que separa esta lei da do alvo (§16.9).
    ///
    /// ⚠️ **O PISO do passo de ecrã é o recurso NOMEADO:** cada candidato custa
    /// um raio, e sem piso a silhueta exacta (onde `cos θ → 0`) pediria uma
    /// infinidade deles. A `1/32` de píxel um candidato por píxel vira `32`, e o
    /// tecto de dabs por passo ([`MAX_DABS_POR_PASSO`]) fecha a outra ponta.
    pub fn percorre(&mut self, de: f32, ate: f32, passo: f32, alvo: &mut dyn CarimboDoCaminho) {
        if !(de.is_finite() && ate.is_finite() && passo.is_finite() && passo > 0.0) {
            return;
        }
        let sentido = if ate >= de { 1.0 } else { -1.0 };
        let comprimento = (ate - de).abs();
        let mut andado = 0.0f32;
        let mut px = self.previsto.unwrap_or(PASSO_INICIAL_DO_CANDIDATO_PX);
        while andado < comprimento {
            andado = (andado + px).min(comprimento);
            let t = de + sentido * andado;
            let Some(p) = alvo.congelado(t) else {
                // ⛔ **Fora da peça o caminho NÃO anda**: o arco entre dois
                // pontos que não estão nela não é arco nenhum. ⚠️ E o candidato
                // seguinte volta ao passo inicial — a previsão só vale enquanto
                // houver superfície a prever.
                px = PASSO_INICIAL_DO_CANDIDATO_PX;
                self.previsto = Some(px);
                continue;
            };
            let antes = self.acumulado;
            let fecha = self.avanca(p, passo);
            for _ in 0..fecha {
                if !alvo.carimba(t) {
                    return;
                }
            }
            // O arco que ESTE candidato mediu, usado para prever o próximo.
            let arco = (self.acumulado + passo * f32::from(u16::try_from(fecha).unwrap_or(0))
                - antes)
                .max(0.0);
            px = if arco > 0.0 {
                (px * passo / arco).clamp(PISO_DO_CANDIDATO_PX, PASSO_INICIAL_DO_CANDIDATO_PX)
            } else {
                PASSO_INICIAL_DO_CANDIDATO_PX
            };
            self.previsto = Some(px);
        }
    }
}

/// ⭐⭐ **AS DUAS PERGUNTAS QUE A LEI FAZ AO MUNDO** — e é por elas viverem no
/// mesmo trait que a lei pode correr sobre a cena do app **e** sobre uma malha de
/// bancada sem uma segunda cópia dela.
///
/// ⚠️ **Um trait e não dois fechos**, e a razão é de empréstimo: quem responde às
/// duas é a mesma cena, e dois fechos sobre ela não coexistem — um lê, o outro
/// escreve. *O trait é o que torna a alternância explícita: o motor chama uma de
/// cada vez.*
pub trait CarimboDoCaminho {
    /// Onde o ponto `t` do caminho cai na superfície **CONGELADA** do pen-down —
    /// `None` fora da peça.
    fn congelado(&mut self, t: f32) -> Option<[f32; 3]>;
    /// Põe um dab no ponto `t`; `false` **pára** o gesto (o caminho saiu da peça).
    fn carimba(&mut self, t: f32) -> bool;
}

/// O passo de ecrã com que um caminho começa a ser amostrado — **um píxel**, que
/// é a resolução com que o rato entrega o gesto.
pub const PASSO_INICIAL_DO_CANDIDATO_PX: f32 = 1.0;

/// **O PISO do passo de ecrã de um candidato** — o recurso é o RAIO por
/// candidato.
///
/// ⚠️ **`1/32` de píxel, e o número diz de que recurso é:** junto à silhueta um
/// píxel cobre `1/cos θ` de arco e a lei pede um candidato por passo de mundo;
/// sem piso, `cos θ → 0` pede uma infinidade deles. A `1/32` um píxel custa no
/// máximo `32` raios, e a `89,5°` — onde um píxel já vale `23` passos — ainda
/// sobra resolução.
pub const PISO_DO_CANDIDATO_PX: f32 = 1.0 / 32.0;

#[cfg(test)]
#[path = "passo_no_mundo_tests.rs"]
mod tests;

/// ⭐⭐⭐ **O MESMO PONTO DE BARRO, depois de o traço o ter movido** — a segunda
/// metade da cura da silhueta.
///
/// O raio pica a superfície **congelada**; a face e as coordenadas baricêntricas
/// do acerto são lidas ali e aplicadas às posições **vivas**. ⇒ o centro do dab
/// **afunda com o vinco** (a auto-limitação do §4 fica intacta, medido) e
/// **não escorrega ao longo da superfície**.
///
/// ⚠️⚠️ **O escorregamento é o que sobra do defeito depois do passo no mundo, e
/// ele está medido:** um raio rasante que pica a superfície VIVA move o acerto
/// `δ/cos θ` ao longo do arco quando o barro afunda `δ` — junto à silhueta isso
/// vale vários passos, e os dabs amontoam-se e rareiam. Medido nas bandas em que
/// o traço corre em regime, a ondulação cai de `0,070 · 0,118 · 0,251` para
/// **`0,018 · 0,024 · 0,064`**.
///
/// ⛔ **E não é o cursor CONGELADO**, que seria a cura preguiçosa: com ele o
/// vinco deixa de se auto-limitar (medido `0,24` de profundidade contra `0,206`,
/// e a crescer), porque a auto-limitação *é* o cursor a afundar.
///
/// ⚠️ **O triângulo é o de MELHOR ajuste, nunca «o primeiro que aceita»:** num
/// quadrilátero o acerto pode cair a um epsilon da diagonal e uma cascata com
/// tolerância rejeita os dois — apagando o dab exactamente onde o raio é mais
/// rasante.
#[must_use]
pub fn levado_pela_deformacao(congelada: &Mesh, viva: &Mesh, hit: &Hit) -> Option<[f32; 3]> {
    let face = *congelada.faces().get(hit.face as usize)?;
    let v = face.verts();
    let mut melhor: Option<(f32, [f32; 3])> = None;
    for k in 1..v.len().saturating_sub(1) {
        let (a, b, c) = (v[0] as usize, v[k] as usize, v[k + 1] as usize);
        let (pa, pb, pc) = (
            *congelada.positions().get(a)?,
            *congelada.positions().get(b)?,
            *congelada.positions().get(c)?,
        );
        let Some((wa, wb, wc)) = baricentricas(pa, pb, pc, hit.point) else {
            continue;
        };
        let (qa, qb, qc) = (
            *viva.positions().get(a)?,
            *viva.positions().get(b)?,
            *viva.positions().get(c)?,
        );
        let dentro = wa.min(wb).min(wc);
        let p = [
            qa[0] * wa + qb[0] * wb + qc[0] * wc,
            qa[1] * wa + qb[1] * wb + qc[1] * wc,
            qa[2] * wa + qb[2] * wb + qc[2] * wc,
        ];
        if p.iter().all(|c| c.is_finite()) && melhor.as_ref().is_none_or(|(d, _)| dentro > *d) {
            melhor = Some((dentro, p));
        }
    }
    melhor.map(|(_, p)| p)
}

/// As coordenadas baricêntricas de `p` no triângulo — `None` se ele degenera.
fn baricentricas(a: [f32; 3], b: [f32; 3], c: [f32; 3], p: [f32; 3]) -> Option<(f32, f32, f32)> {
    let sub = |u: [f32; 3], v: [f32; 3]| [u[0] - v[0], u[1] - v[1], u[2] - v[2]];
    let (v0, v1, v2) = (sub(b, a), sub(c, a), sub(p, a));
    let dot = |u: [f32; 3], v: [f32; 3]| u[0] * v[0] + u[1] * v[1] + u[2] * v[2];
    let (d00, d01, d11, d20, d21) = (
        dot(v0, v0),
        dot(v0, v1),
        dot(v1, v1),
        dot(v2, v0),
        dot(v2, v1),
    );
    let den = d00 * d11 - d01 * d01;
    if !den.is_finite() || den.abs() <= f32::MIN_POSITIVE {
        return None;
    }
    let vb = (d11 * d20 - d01 * d21) / den;
    let vc = (d00 * d21 - d01 * d20) / den;
    Some((1.0 - vb - vc, vb, vc))
}
