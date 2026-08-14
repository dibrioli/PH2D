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
        // ⚠️ **EMA ponderada por COMPRIMENTO, `α = Δs/(Δs + L)`** — a MESMA lei que o
        // [`crate::heading`] usa para suavizar a tangente, e não uma rampa linear até o alvo. Uma
        // rampa linear é contínua no valor e **descontínua na derivada**: no arco em que ela
        // COMPLETA, o `d(arremesso)/d(arco)` cai de golpe e o caminho da tinta ganha uma QUINA
        // (medido: 25× a virada da própria mão num arco de 3 quadros). A EMA nunca completa, então
        // não há onde pôr o canto.
        let alpha = ds / (ds + self.speed_ramp_len);
        self.throw_speed_px_s += (self.speed_px_s - self.throw_speed_px_s) * alpha;
    }

    /// **O ARREMESSO** — onde a tinta cai quando o gesto tem inércia (manual do Alchemy, verbatim:
    /// *"throw the line beyond the actual pen position"*).
    ///
    /// A tinta é lançada ao longo do **heading** (a EMA da tangente do caminho — o mesmo vetor que o
    /// Rake cavalga) por `velocidade × antecipação`, e a antecipação é [`crate::line_kind::SPEED_LOOKAHEAD_S`]:
    /// **onde a mão ESTARÁ**. Sem heading (o primeiro dab de um traço) não há direção a arremessar e a
    /// tinta fica onde o dedo a pôs.
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
        let ds = (arc - self.speed_dab_arc).max(0.0);
        self.advance_speed(arc);
        let k_now = self.throw_speed_px_s * crate::line_kind::SPEED_LOOKAHEAD_S;
        // ⚠️ **A MIRA tem janela PRÓPRIA, e o comprimento dela é o próprio ARREMESSO.**
        //
        // O heading do motor é suavizado sobre `0,1 × diâmetro` — **1,6 px** num pincel de 16 —, que é
        // o certo para o Rake: ele gira o carimbo, e um carimbo tem de ACOMPANHAR o traço. Mas o
        // arremesso não gira nada: ele **desloca** a tinta por `k` px ao longo dessa direção, e isso é
        // uma ALAVANCA. Medido num arco de 3 quadros com `k = 470 px`: um tremor residual de **0,6°**
        // no heading vira **±5 px** de ondulação lateral na tinta, e a linha — contínua e no lugar —
        // sai TORTA (24× a virada da própria mão, na escala de um diâmetro).
        //
        // A lei cai da geometria: o erro lateral é `k · Δθ`, então para ele ficar abaixo de uma fração
        // fixa do diâmetro o `Δθ` tem de encolher como `1/k` ⇒ **a janela cresce com o arremesso**.
        // Quanto mais longe você joga, mais firme a mira precisa ser — e o número não é escolhido: é
        // a distância que a própria tinta vai percorrer.
        self.throw_dir = crate::heading::advance(self.throw_dir, self.heading, ds, k_now);
        let d = self.throw_dir;
        let thrown = if d[0] == 0.0 && d[1] == 0.0 {
            pos
        } else {
            [pos[0] + d[0] * k_now, pos[1] + d[1] * k_now]
        };
        // O segmento que a TINTA saltou desde o dab anterior — o caminho que o `fill_thrown_gap`
        // vai percorrer. Guardado aqui porque este é o único lugar que conhece as duas pontas.
        //
        // ⚠️ **Só com arremesso VIVO**, e o motivo é exatidão, não economia: com `k = 0` a tinta cai
        // exatamente onde a mão passou, e o caminhador já a espaçou — o vão entre dois dabs vale o
        // passo *ao bit*. Comparar `gap > step` sobre essa igualdade decide por **ruído de vírgula
        // flutuante**, e um dab extra nasce ou não conforme a aritmética cair de um lado ou do outro.
        // Foi assim que um tipo de linha SEM arremesso (o Sketchy, que não mede velocidade) deixou de
        // ser byte-idêntico ao neutro — pego pelo gate que exige que ligar o Sketchy não mova a tinta.
        if k_now > 0.0 {
            self.fill_from = self.last_thrown.replace((thrown, arc));
        } else {
            self.fill_from = None;
            self.last_thrown = None;
        }
        thrown
    }

    /// **A PORTA UNICA por onde um dab chega ao mundo** — ela percorre o caminho que a tinta de
    /// fato descreve e so entao espelha.
    ///
    /// ⚠️ **O arremesso ESTICA o caminho da tinta, e ate esta wave ninguem re-espacava.** O motor
    /// emite um dab a cada `spacing x diametro` do caminho da MAO; o arremesso vale `v x T`, entao
    /// entre dois dabs a tinta anda `passo x (1 + d(arremesso)/d(arco))` — e num gesto de artista,
    /// que acelera na reta e freia na curva, esse fator e `1 + Δv/v` e chega facil a cinco. Medido
    /// (sonda `measure_the_gap_on_a_gesture_that_changes_speed`, pincel de raio 4): o maior vao
    /// entre dabs vizinhos valia **1,61 diametro** e a contagem de dabs era **identica** a do traco
    /// sem arremesso — a assinatura exata do defeito, *os mesmos dabs espalhados por um caminho mais
    /// longo*, que o Enio viu como *"ainda fica pontilhado"* (2026-08-13, com a foto).
    ///
    /// O Alchemy nao tem este problema porque o traco dele e um `GeneralPath` **percorrido**: uma
    /// curva tracada e continua por construcao, seja qual for a distancia entre os pontos gravados.
    /// Num motor de dabs o equivalente e este: **os pontos arremessados formam um caminho, e o
    /// espacamento e honrado ONDE A TINTA CAI, nunca onde a mao passou**.
    pub(super) fn emit(&mut self, dab: Dab, out: &mut Vec<Dab>) {
        self.note_sketchy_point(&dab);
        self.fill_thrown_gap(&dab, out);
        crate::symmetry::push_symmetric(out, dab, &self.spec.symmetry);
    }

    /// Preenche o vao que o arremesso abriu entre o dab anterior e este, no dominio da TINTA.
    ///
    /// ⚠️ **O passo e `spacing x o diametro que de fato sera carimbado aqui`**, nao o nominal — a
    /// mesma lei que o [`Stroke::walk_space`] ja aplica ao taper: espacamento e uma RAZAO, e segurar
    /// o vao fixo enquanto o dab encolhe faz a ponta sair em pontos separados.
    ///
    /// ⚠️ **O jitter e sorteado por dab preenchido**, nao copiado: a interpolacao corre nas posicoes
    /// ARREMESSADAS (pre-jitter), entao um pincel com scatter continua a espalhar em vez de desenhar
    /// uma fileira alinhada dentro do proprio vao.
    fn fill_thrown_gap(&mut self, dab: &Dab, out: &mut Vec<Dab>) {
        let (Some((from, from_arc)), Some((to, to_arc))) = (self.fill_from, self.last_thrown)
        else {
            return;
        };
        let d = [to[0] - from[0], to[1] - from[1]];
        let gap = (d[0] * d[0] + d[1] * d[1]).sqrt();
        let step = (2.0 * dab.radius_px * self.spec.spacing).max(1.0);
        if gap <= step {
            return;
        }
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let n = ((gap / step).ceil() as usize).saturating_sub(1);
        for k in 1..=n {
            #[allow(clippy::cast_precision_loss)]
            let t = k as f32 / (n + 1) as f32;
            let p = [from[0] + d[0] * t, from[1] + d[1] * t];
            let mut fill = *dab;
            fill.center = self.apply_jitter(p, dab.radius_px);
            fill.arc_len = from_arc + (to_arc - from_arc) * t;
            crate::symmetry::push_symmetric(out, fill, &self.spec.symmetry);
        }
    }
}
