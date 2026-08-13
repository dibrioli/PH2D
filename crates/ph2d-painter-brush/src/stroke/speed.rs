//! **A INÉRCIA DO GESTO** — a velocidade que o traço tem e o arremesso que ela produz (o `Speed
//! Shapes` do Alchemy; plano 38 W2). Módulo filho de [`super`], então mantém o acesso privado aos
//! campos de [`Stroke`]; separado por assunto, e porque o pai estava a onze linhas do teto de LOC.
//!
//! ⚠️ **A fórmula é `Δarco / dt` no TIQUE, e ela foi ESCOLHIDA POR MEDIÇÃO** (plano 38 W0.1): a mesma
//! curva desenhada com 8 e com 512 eventos dá um deslocamento por evento que varia **73×** — isso é o
//! DISPOSITIVO, não o gesto —, o arco entre dabs é **constante por construção** (o espaçamento o fixa
//! ⇒ zero informação), e o arco por quadro fica **plano em 1,00–1,03**. É a mesma lei que o relevo
//! desta casa aprendeu quatro vezes: *a grandeza é fato do CAMINHO e do RELÓGIO, nunca de quão fino o
//! motor amostrou o caminho*.
//!
//! ⚠️ **UM lugar computa, todos leem** ([`Stroke::speed_px_s`]): o Sketchy quer esta grandeza para o
//! *distance-opacity* e um Splatter futuro para a direção do respingo, e duas fórmulas para a mesma
//! grandeza é a falha de duas-portas que este módulo já pagou quatro vezes.

use super::*;

impl Stroke {
    /// Mede a velocidade do gesto neste quadro: o arco percorrido desde o tique anterior, dividido
    /// pelo `dt` que o chamador entregou.
    ///
    /// ⚠️ **O [`Stroke::tick`] a chama ANTES de perguntar qual é o método de traço**, e a ordem é a
    /// feature: o `speed_px_s` é do GESTO, não de um método — medi-la depois do desvio do Airbrush a
    /// deixaria zerada em todo pincel que não fosse ele, e o arremesso nasceria morto num produto
    /// cujos gates de unidade ficariam **todos verdes**.
    pub(super) fn note_tick_speed(&mut self, dt: f32) {
        if dt > 0.0 {
            self.speed_px_s = (self.arc_len - self.speed_arc_mark).max(0.0) / dt;
            self.speed_arc_mark = self.arc_len;
        }
    }

    /// **A VELOCIDADE do gesto neste quadro, em px/s.** Zero num traço que ainda não teve um tique e
    /// em toda [`Stroke`] recém-construída — o que é o que torna o arremesso **estruturalmente
    /// inerte** na família dos shape editors: cada preenchimento de forma constrói uma `Stroke`
    /// FRESCA, e a mão nunca correu por ela.
    #[must_use]
    pub fn speed_px_s(&self) -> f32 {
        self.speed_px_s
    }

    /// **O ARREMESSO** — onde a tinta cai quando o gesto tem inércia (manual do Alchemy, verbatim:
    /// *"throw the line beyond the actual pen position"*).
    ///
    /// A tinta é lançada ao longo do **heading** (a EMA da tangente do caminho — o mesmo vetor que o
    /// Rake cavalga) por `velocidade × antecipação`, e a antecipação é `Amount` **quadros** de
    /// [`crate::line_kind::SPEED_LOOKAHEAD_S`]. Sem heading (o primeiro dab de um traço) não há
    /// direção a arremessar e a tinta fica onde o dedo a pôs.
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
    pub(super) fn throw(&self, pos: [f32; 2]) -> [f32; 2] {
        if self.spec.line_kind != LineKind::None {
            let a = self.spec.line_speed_amount;
            let d = self.heading;
            if a > 0.0 && (d[0] != 0.0 || d[1] != 0.0) {
                let k = self.speed_px_s * a * crate::line_kind::SPEED_LOOKAHEAD_S;
                return [pos[0] + d[0] * k, pos[1] + d[1] * k];
            }
        }
        pos
    }
}
