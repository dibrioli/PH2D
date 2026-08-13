//! **A INÉRCIA DO GESTO** — a velocidade que o traço tem e o arremesso que ela produz (o `Speed
//! Shapes` do Alchemy; plano 38 W2). Módulo filho de [`super`], então mantém o acesso privado aos
//! campos de [`Stroke`]; separado por assunto, e porque o pai estava a onze linhas do teto de LOC.
//!
//! ⚠️ **A MAGNITUDE é `Δarco / dt` no TIQUE, e ela foi ESCOLHIDA POR MEDIÇÃO** (plano 38 W0.1): a
//! mesma curva desenhada com 8 e com 512 eventos dá um deslocamento por evento que varia **73×** —
//! isso é o DISPOSITIVO, não o gesto —, o arco entre dabs é **constante por construção** (o
//! espaçamento o fixa ⇒ zero informação), e o arco por quadro fica **plano em 1,00–1,03**. É a lei
//! que o relevo desta casa aprendeu quatro vezes: *a grandeza é fato do CAMINHO e do RELÓGIO, nunca
//! de quão fino o motor amostrou o caminho*.
//!
//! ⚠️ **MAS a magnitude medida assim é uma ESCADA, e foi ISSO que reprovou a v1** (Enio 2026-08-13:
//! *"speed não é igual o Alchemy"*). Um valor por tique, constante dentro do quadro, saltando na
//! fronteira ⇒ a tinta sai como uma **fileira de arcos deslocados e DESCONECTADOS**, um por quadro.
//! Medido num arco rápido (quarto de círculo r=150 em 6 quadros): o maior vão entre dabs vizinhos
//! valia **25× o passo nominal em `Amount = 1` e 99× em `Amount = 4`** — a linha não era uma linha.
//! O Alchemy arremessa **por ponto gravado**, então a curva dele sai contínua.
//!
//! **A cura é a RAMPA, e ela não tem constante mágica:** entre dois tiques a velocidade usada por
//! cada dab caminha da anterior para a nova, ao longo do **arco daquele quadro** — que é justamente o
//! quanto de caminho o quadro produziu. O degrau vira uma rampa que dura exatamente um quadro de
//! percurso, em qualquer velocidade, sem ninguém escolher um número.
//!
//! ⚠️ **UM lugar computa, todos leem** ([`Stroke::speed_px_s`]): o Sketchy quer esta grandeza para o
//! *distance-opacity* e um Splatter futuro para a direção do respingo, e duas fórmulas para a mesma
//! grandeza é a falha de duas-portas que este módulo já pagou quatro vezes.

use super::*;

impl Stroke {
    /// Fecha o quadro: mede a velocidade do gesto (arco percorrido ÷ `dt`) e arma a **rampa** que os
    /// dabs do próximo quadro percorrem.
    ///
    /// ⚠️ **O [`Stroke::tick`] a chama ANTES de perguntar qual é o método de traço**, e a ordem é a
    /// feature: o `speed_px_s` é do GESTO, não de um método — medi-la depois do desvio do Airbrush a
    /// deixaria zerada em todo pincel que não fosse ele, e o arremesso nasceria morto num produto
    /// cujos gates de unidade ficariam **todos verdes**.
    pub(super) fn note_tick_speed(&mut self, dt: f32) {
        if dt <= 0.0 {
            return;
        }
        let arc = (self.arc_len - self.speed_arc_mark).max(0.0);
        // A medida do quadro, e o comprimento sobre o qual a rampa vai até ela: o arco DESTE quadro.
        // ⚠️ Um quadro sem percurso tem `ramp_len = 0`, e a rampa então salta direto — que é o certo:
        // **parar de mover É parar de arremessar**, na hora, sem uma cauda de tinta seguindo o dedo
        // parado.
        self.speed_px_s = arc / dt;
        self.speed_ramp_len = arc;
        self.speed_arc_mark = self.arc_len;
    }

    /// **A VELOCIDADE do gesto no último quadro, em px/s** — a grandeza medida, não a rampeada. Zero num traço que ainda não teve um tique e em
    /// toda [`Stroke`] recém-construída — o que é o que torna o arremesso **estruturalmente inerte**
    /// na família dos shape editors: cada preenchimento de forma constrói uma `Stroke` FRESCA, e a
    /// mão nunca correu por ela.
    #[must_use]
    pub fn speed_px_s(&self) -> f32 {
        self.speed_px_s
    }

    /// Avança a velocidade deste dab pela **rampa**: ela caminha do valor do tique anterior para o do
    /// tique atual ao longo do arco que aquele quadro percorreu.
    ///
    /// ⚠️ **O comprimento da rampa é MEDIDO, não escolhido** — é o arco do próprio quadro, então ela
    /// dura um quadro de percurso a 300 px/s e a 3 000 px/s igualmente. Uma constante em pixels
    /// suavizaria demais o gesto lento e de menos o rápido, que é exatamente o defeito que ela existe
    /// para curar.
    fn advance_speed(&mut self, arc: f32) {
        let ds = (arc - self.speed_dab_arc).max(0.0);
        self.speed_dab_arc = arc;
        if self.speed_ramp_len <= f32::EPSILON {
            self.throw_speed_px_s = self.speed_px_s;
            return;
        }
        let t = (ds / self.speed_ramp_len).clamp(0.0, 1.0);
        self.throw_speed_px_s += (self.speed_px_s - self.throw_speed_px_s) * t;
    }

    /// **O ARREMESSO** — onde a tinta cai quando o gesto tem inércia (manual do Alchemy, verbatim:
    /// *"throw the line beyond the actual pen position"*).
    ///
    /// A tinta é lançada ao longo do **heading** (a EMA da tangente do caminho — o mesmo vetor que o
    /// Rake cavalga) por `velocidade × antecipação`, e a antecipação é `Amount` **quadros** de
    /// [`crate::line_kind::SPEED_LOOKAHEAD_S`]. Sem heading (o primeiro dab de um traço) não há
    /// direção a arremessar e a tinta fica onde o dedo a pôs.
    ///
    /// ⚠️ **A velocidade é avançada AQUI, por dab** (a rampa de [`Self::advance_speed`]) — é o que
    /// mantém a linha CONTÍNUA em vez de uma fileira de arcos deslocados, um por quadro.
    ///
    /// ⚠️ **O `dab_at` o compõe ANTES do jitter, e a ordem NÃO é load-bearing** — a mutação que a
    /// inverte sobreviveu à suíte inteira, e verificar (em vez de escrever um gate) diz por quê: o
    /// `apply_jitter` soma um deslocamento que **não depende da posição** (só do raio e do RNG), e o
    /// arremesso é outra translação — **duas translações COMUTAM**, então as duas ordens dão o mesmo
    /// ponto, ao bit. Ler nessa ordem é ler a frase (*o gesto move onde a tinta cai, o pincel espalha
    /// em torno de onde ela caiu*), e afirmar que ela é CORREÇÃO seria a mesma forma de over-claim que
    /// a comutatividade do IEEE-754 já custou a esta linha no warp da aquarela.
    ///
    /// ⚠️ **Isto move a TINTA, nunca o CAMINHO:** o `last_pos`, o `accum` do espaçamento e o
    /// `arc_len` continuam sendo o que a mão fez. Se o arremesso realimentasse o caminho, a
    /// velocidade se somaria a si mesma e o traço fugiria da tela por composição — e, pior, o
    /// espaçamento passaria a depender do arremesso, que é a dependência de amostragem que este
    /// módulo já curou quatro vezes.
    pub(super) fn throw(&mut self, pos: [f32; 2], arc: f32) -> [f32; 2] {
        if self.spec.line_kind == LineKind::None {
            return pos;
        }
        self.advance_speed(arc);
        let a = self.spec.line_speed_amount;
        let d = self.heading;
        if a > 0.0 && (d[0] != 0.0 || d[1] != 0.0) {
            let k = self.throw_speed_px_s * a * crate::line_kind::SPEED_LOOKAHEAD_S;
            return [pos[0] + d[0] * k, pos[1] + d[1] * k];
        }
        pos
    }
}
