//! ⭐⭐⭐ **O BRILHO** — a luz forte que vaza para fora da própria forma.
//!
//! O ingrediente **`7`** do [`docs/Render3d/01`], a `W7`. Ele lê o quadro em **HDR**, tira o que
//! passa de um limiar, borra isso numa cadeia de resoluções e soma de volta.
//!
//! # ⛔⛔ Porque ele é uma crate PRÓPRIA e não mais um botão da [`ph2d_style`]
//!
//! Aquela é a lei **por PIXEL** que os dois motores partilham. Esta lê os **VIZINHOS** ⇒ é um
//! **passe**, e pô-la lá obrigaria a crate da lei a receber um G-buffer. A recusa está escrita, com
//! este mecanismo, no `docs/Render3d/11` §11 — *para o contorno desenhado, três dias antes de este
//! ficheiro existir*.
//!
//! # ⭐ A GEOMETRIA é portada; o CORTE é NOSSO — e as duas metades foram medidas
//!
//! O oráculo é o **Godot 4.7.2 (MIT**, lido no artefacto — porta aberta, porta-se com atribuição),
//! corrido sem interface. Medido em `15` corridas (`docs/Render3d/12` §3):
//!
//! - ⭐⭐ **A cadeia é um *mip chain*** e o raio a meia altura **DUPLICA por nível**
//!   (`3 · 9 · 20 · 38 · 55` px num quadro de `512`). Isso porta-se, e é o que este ficheiro faz.
//! - ⛔⛔ **O corte dele NÃO é uma lei**: o limiar move o halo `5 %` a `entrada 4,0` (a quantização
//!   de 8 bits) e `27 %` sobre a faixa `0..3` a `entrada 2,0`, a escala **não move nada** nas
//!   quatro células, e com limiar `3` e entrada `2` **ainda brilha**. *Portar isso seria herdar um
//!   painel em que «Threshold» não faz o que o nome diz.*
//!
//! ⇒ o corte abaixo é **declarado**, com o joelho a ser o controlo fino que o report do dono sobre
//! a camada de estilo pedia um nível acima, e com a **degenerescência gateada**: `knee = 0` é o
//! corte duro **exacto**.

// ⭐⭐⭐ **O QUE O ARTISTA AUTORA NO HALO** — movido do [`ph2d_render::motion_fx_params`] em
// 2026-09-19, por ordem do dono: *«nosso bloom original é muito melhor. retire essa implementação
// godot»*.
//
// ⛔⛔ **A 1.ª redacção desta crate tinha um modelo PRÓPRIO** — limiar, joelho, intensidade e
// **sete pesos por nível**, portados do oráculo Godot. Ele era pior por duas razões medidas:
// os sete pesos são sete controlos onde o nosso tem **um** (o [`BloomParams::radius`]), e quatro
// deles nascem a zero, o que dá quatro fileiras que se leem como mortas. *Um modelo de botões
// copiado de outro programa não é melhor por ser de outro programa.*
//
// ⚠️ **E o tipo é UM, não dois a concordar por promessa:** o [`ph2d_render`] re-exporta este, logo
// o halo do Motion e o do campo implícito autoram-se com a MESMA estrutura. *Duas cópias com uma
// nota a dizer «mantenha-as iguais» é exactamente o que diverge no dia em que uma ganha um campo.*
/// The three glow knobs the document carries (doc 67). Plain data — the `fx.glow`
/// node authors it, the shell hands it to [`bloom_over`](MotionFx::bloom_over).
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct BloomParams {
    /// Brightness above which a pixel starts to glow (premult `max(r,g,b)`).
    /// `1.0` = only genuinely HDR (emissive) pixels bloom.
    pub threshold: f32,
    /// Soft-knee width around the threshold — the glow ramps in over
    /// `[threshold-knee, threshold]` instead of switching on hard.
    pub knee: f32,
    /// Multiplier on the accumulated glow before it is added to the scene.
    pub intensity: f32,
    /// Scales the upsample tent radius — a wider radius spreads the halo further.
    pub radius: f32,
    /// `0` pulls the glow to grey (a white bloom), `1` keeps the source colour.
    pub saturation: f32,
    /// Multiplies the (desaturated) glow — default white `[1,1,1,1]` is a no-op.
    pub tint: [f32; 4],
    /// **A ANAMORFOSE** — a razão entre o alcance do halo ao longo de
    /// [`Self::angle`] e o alcance perpendicular a ele (doc 89 folha 11). `1` é o
    /// halo redondo que sempre shipou; `>1` estica na direção do ângulo e aperta na
    /// outra, que é o *streak* anamórfico do cinema (o `Glow Dimensions H/V` do AE,
    /// o *Anamorphic Ratio* do Unity, a *Bloom Convolution* do Unreal).
    ///
    /// ⚠️ **Ela mora na TENDA do upsample, não na cadeia de mips.** Os mips são a
    /// máquina que arredonda as quinas da fonte (é literalmente o que os torna
    /// melhores que um box blur largo); torcê-los tornaria a queda direcional em
    /// TODAS as escalas e o halo perderia o miolo. A tenda de 9 taps é onde a
    /// referência põe a razão, e é o passe que corre uma vez por nível.
    pub stretch: f32,
    /// A direção do *streak*, em GRAUS — a unidade autorada única do app. Sem
    /// efeito em [`Self::stretch`] `= 1` (um círculo rodado é o mesmo círculo), e o
    /// `ParamGate` do nó esconde-a ali.
    pub angle: f32,
    /// **O TETO do bright-pass** — o antídoto dos *fireflies* (o `Clamp` do Bloom do
    /// Unity URP). `0` = **desligado**, o caminho literal que sempre shipou.
    ///
    /// ⚠️ **O recurso é a REPRESENTAÇÃO, e o número está medido.** O `tint` de uma
    /// instância é `[f32; 4]` **sem clamp** (doc 67 §4), e o bright-pass não limita
    /// nada: para `brightness → ∞` a contribuição tende a `1` e a saída tende ao
    /// próprio `c`. Então um único elemento com `tint = 5000` entra inteiro na
    /// cadeia, espalha-se por seis níveis de mip e lava a tela. O único teto que
    /// existe hoje é o do FORMATO — `Rgba16Float` guarda até **65 504** e depois é
    /// `inf`, que envenena a soma de toda a cadeia. Este param é o teto AUTORADO,
    /// que é o que a referência expõe.
    pub clamp: f32,
    /// **A OPERAÇÃO do halo** (doc 89 folha 11, o *Glow Operation* do AE): `0` = `Add`, o passe
    /// aditivo que sempre shipou; `1` = `Screen`.
    ///
    /// ⚠️ **`Multiply` não existe aqui, e é uma decisão medida pela navalha do §0**: o halo
    /// compõe-se sobre a cena já desenhada, sem profundidade, então um modo que ESCUREÇA
    /// pintaria por cima do que estivesse à frente. `Screen` (`a + b − ab`) é monótono e nunca
    /// escurece — é o único dos três do AE que sobrevive a essa navalha.
    pub operation: f32,
    /// **De que o bright-pass se alimenta** (o *Glow Based On* do AE): `0` = a luminância
    /// premultiplicada (o de sempre), `1` = o **alfa**.
    ///
    /// ⚠️ Com luma, uma silhueta **preta e opaca** não tem nada acima do limiar e nunca acende;
    /// com alfa ela acende pela COBERTURA. É a diferença entre um halo de EMISSÃO e uma AURA.
    pub source: f32,
    /// **QUANTO a máscara de sujidade acende** (doc 89 folha 11) — o `Dirt Intensity` do Unity
    /// URP / o `Bloom Dirt Mask Intensity` do Unreal. `0` = o passe de sempre.
    ///
    /// ⚠️ **Ele soma-se ao `tint`, não multiplica o halo** — `glow · (tint + dirt·isto)`, a forma
    /// da referência. A distinção importa: uma máscara que multiplicasse não poderia ACRESCENTAR
    /// cor, e um mapa de sujidade é uma fotografia colorida de pó e riscos.
    ///
    /// ⚠️ **A identidade do quadro NÃO vive neste número.** Sem imagem escolhida o binding leva
    /// uma textura preta de 1×1, então `dirt = 0` e a soma é literal — inclusive com este knob
    /// alto, que é o estado em que um artista fica ao apagar o nome da imagem.
    pub dirt_intensity: f32,
}

/// Quantas operações de composição existem — o tamanho do array de pipelines.
///
/// ⚠️ **É esta contagem que a lista de rótulos do nó tem de ter** (`OPERATION_LABELS`), e quem
/// liga as duas pontas é um gate na shell: um nó é uma folha e não alcança o `ph2d-render`,
/// então sem ele um modo a mais no dropdown seria escolhível e silenciosamente rebaixado.
pub const COMPOSITE_OPERATIONS: usize = 2;

impl Default for BloomParams {
    fn default() -> Self {
        Self {
            threshold: 1.0,
            knee: 0.6,
            intensity: 0.8,
            radius: 1.0,
            saturation: 1.0,
            tint: [1.0, 1.0, 1.0, 1.0],
            stretch: 1.0,
            angle: 0.0,
            clamp: 0.0,
            operation: 0.0,
            source: 0.0,
            dirt_intensity: 0.0,
        }
    }
}

/// ⭐⭐⭐ **O BRILHO DE UMA CENA** — o que o artista autora, mais o interruptor.
///
/// ⚠️ **O interruptor vive AQUI e não no [`BloomParams`], e a razão é o consumidor:** no Motion a
/// presença do nó `fx.glow` no grafo **é** o interruptor (o doc dele di-lo por extenso), e uma cena
/// 3D não tem grafo — ela precisa de um sim/não. *Pôr o `enabled` no tipo partilhado daria ao
/// Motion um segundo interruptor ao lado do que ele já tem, e dois interruptores podem discordar.*
#[derive(Copy, Clone, Debug, PartialEq, Default)]
pub struct Bloom {
    /// O passe corre?
    pub enabled: bool,
    /// O que o artista autora — a MESMA estrutura que o halo do Motion.
    pub params: BloomParams,
}

impl Bloom {
    /// ⚠️ **Quantos degraus a cadeia tem** — ver [`levels_that_fit`]. Ela é DERIVADA da resolução e
    /// não um número autorado: *o nosso modelo não tem pesos por nível, tem um RAIO*.
    pub const LEVELS: usize = 7;

    /// ⭐ **Este quadro paga o passe?** — sem interruptor, sem intensidade ou sem raio, não.
    #[must_use]
    pub fn contributes(&self) -> bool {
        self.enabled && self.params.intensity > 0.0 && self.params.radius > 0.0
    }

    /// ⭐⭐⭐ **QUANTOS NÚMEROS ESTE BRILHO TEM** — e o painel, a arrumação e o gémeo do dispositivo
    /// contam-nos **daqui**, nunca à mão.
    ///
    /// ⚠️ **O `tint` conta como TRÊS** (uma cor é três canais e uma amostra só), e o alfa dele fica
    /// de fora: o halo compõe-se por soma, e um alfa num somando não tem leitor.
    pub const SLOTS: usize = 12;

    /// ⭐⭐⭐ **A ARRUMAÇÃO, e há UMA** — a ordem em que o painel a lê e o dispositivo a há-de ler.
    ///
    /// ```text
    ///  0 ligado · 1 limiar · 2 joelho · 3 intensidade · 4 raio · 5 saturação
    ///  6..8 tinta (r,g,b) · 9 tecto · 10 estiramento · 11 ângulo
    /// ```
    ///
    /// ⚠️ **O interruptor viaja como número** (`0`/`1`): a arrumação de um uniforme não tem
    /// booleanos, e ter DUAS travessias seria a segunda máquina ao lado de uma que funciona.
    #[must_use]
    pub fn pack(&self) -> [f32; Self::SLOTS] {
        let p = &self.params;
        [
            f32::from(u8::from(self.enabled)),
            p.threshold,
            p.knee,
            p.intensity,
            p.radius,
            p.saturation,
            p.tint[0],
            p.tint[1],
            p.tint[2],
            p.clamp,
            p.stretch,
            p.angle,
        ]
    }

    /// A volta do [`Bloom::pack`]. ⚠️ **Ida-e-volta com gate**, porque uma arrumação com um só
    /// sentido é meia arrumação.
    #[must_use]
    pub fn unpack(v: &[f32; Self::SLOTS]) -> Self {
        Self {
            // ⚠️ `> 0,5` e não `!= 0`: um arrasto entrega o meio do curso, e um `0,49` tem de
            // decidir para um lado. *Um interruptor sem ponto de viragem declarado é um que muda de
            // ideias no último bit.*
            enabled: v[0] > 0.5,
            params: BloomParams {
                threshold: v[1],
                knee: v[2],
                intensity: v[3],
                radius: v[4],
                saturation: v[5],
                // ⚠️ O alfa da tinta fica no neutro: ele não tem leitor nesta composição.
                tint: [v[6], v[7], v[8], 1.0],
                clamp: v[9],
                stretch: v[10],
                angle: v[11],
                ..BloomParams::default()
            },
        }
    }

    /// ⭐⭐ **O SANEAMENTO é na PORTA**, como o do estilo — a partir daqui o número viaja para a
    /// thread que desenha, e ela não tem cerca.
    ///
    /// ⚠️ **O que não é número vira o valor de fábrica e não zero:** um `NaN` no limiar com `0`
    /// faria a cena inteira brilhar, e *uma recusa não pode ser mais destrutiva do que o pedido*.
    #[must_use]
    pub fn sanitized(mut self) -> Self {
        let f = BloomParams::default();
        let so_finito = |v: f32, fabrica: f32| if v.is_finite() { v.max(0.0) } else { fabrica };
        let p = &mut self.params;
        p.threshold = so_finito(p.threshold, f.threshold);
        p.knee = so_finito(p.knee, f.knee);
        p.intensity = so_finito(p.intensity, f.intensity);
        p.radius = so_finito(p.radius, f.radius);
        p.saturation = so_finito(p.saturation, f.saturation);
        p.clamp = so_finito(p.clamp, f.clamp);
        p.stretch = so_finito(p.stretch, f.stretch);
        // ⚠️ O ÂNGULO pode ser negativo — ele é uma direcção, não uma grandeza.
        p.angle = if p.angle.is_finite() {
            p.angle
        } else {
            f.angle
        };
        for c in &mut p.tint {
            *c = so_finito(*c, 1.0);
        }
        self
    }
}

/// ⭐⭐ **QUANTOS NÍVEIS CABEM NUM QUADRO DESTE TAMANHO** — derivado, nunca fixo.
///
/// # ⛔ Porque não são sempre os sete
///
/// Medido no oráculo (`docs/Render3d/12` §3.4): num quadro de `512` px os níveis `5`, `6` e `7`
/// têm o borrão **maior que a moldura**, e a medição do raio deles lê a moldura em vez do nível.
/// *Um nível cujo borrão não cabe no quadro não acrescenta forma: acrescenta uma constante.*
///
/// A cadeia pára quando o lado menor deixa de ter pelo menos [`LADO_MINIMO`] píxeis.
#[must_use]
pub fn levels_that_fit(w: usize, h: usize) -> usize {
    let mut lado = w.min(h);
    let mut n = 0;
    while n < Bloom::LEVELS && lado >= LADO_MINIMO * 2 {
        lado /= 2;
        n += 1;
    }
    n
}

/// O lado mais pequeno que um nível da cadeia pode ter.
///
/// ⚠️ **`4` e não `1`:** abaixo disto o borrão de um nível é a **média do quadro inteiro**, que é
/// uma constante aditiva e não uma forma — e ela apareceria como um véu cinzento que sobe com o
/// brilho da cena. É o mesmo mecanismo que faz a fábrica do alvo acender só três dos sete níveis.
pub const LADO_MINIMO: usize = 4;

/// ⭐⭐⭐ **O CORTE — quanto de um pixel entra na cadeia.**
///
/// A luminância de pico decide, e o resultado é uma **fracção** que multiplica a cor: assim a
/// MATIZ do que brilha é a do pixel, e não a do canal que passou.
///
/// ```text
/// l  = max(r, g, b)
/// s  = clamp(l − t + k, 0, 2k)         a rampa suave, centrada no limiar
/// b  = max(s² / (4k), l − t)           o joelho por baixo, o corte duro por cima
/// w  = max(b, 0) / max(l, ε)           a fracção
/// ```
///
/// ⭐ **Com `k = 0` o primeiro ramo desaparece** (`s ≡ 0`) e sobra `max(l − t, 0)`, que é o corte
/// duro **exacto** — há gate sobre isso, e é ele que torna o joelho um *controlo* em vez de uma
/// aproximação permanente.
///
/// ⭐⭐⭐ **E o TECTO (`clamp`) entra ANTES de tudo, por CANAL** — o antídoto do *firefly*: um pixel
/// que estourou não pode arrastar a cena inteira para o halo.
///
/// ⛔⛔ **Ele FALTAVA aqui até 2026-09-19, e a fileira `Clamp` do painel era um BOTÃO MORTO** — a
/// lei estava escrita no shader que já shipava (`bloom.wgsl`, `fs_prefilter`) e o
/// [`BloomParams::clamp_limit`] tinha **um** leitor no repositório inteiro, do outro lado. *Um fio
/// completo, uma fileira pintada, um valor que chega ao tipo — e a LEI a não o ler.*
///
/// ⚠️ **Por CANAL e não na luminância**, e a razão vem do shader: limitar a luminância e reescalar
/// mudaria o **matiz** do pixel que estourou. ⚠️ Com o knob desligado o tecto é o maior finito do
/// `Rgba16Float`, logo o `min` não morde nada que um quadro consiga guardar.
#[must_use]
pub fn bright(rgb: [f32; 3], b: &BloomParams) -> [f32; 3] {
    // ⭐⭐⭐ **O TECTO, ANTES da luminância e POR CANAL** — ver a nota acima e o gate
    // [`crate::lei_tests::o_tecto_do_corte_morde_e_a_omissao_fica_ao_bit`].
    let lim = b.clamp_limit();
    let rgb = [rgb[0].min(lim), rgb[1].min(lim), rgb[2].min(lim)];
    let l = rgb[0].max(rgb[1]).max(rgb[2]);
    if l.is_nan() || l <= 0.0 {
        return [0.0; 3];
    }
    let k = b.knee.max(0.0);
    let duro = l - b.threshold;
    let acima = if k > 0.0 {
        let s = (duro + k).clamp(0.0, 2.0 * k);
        (s * s / (4.0 * k)).max(duro)
    } else {
        duro
    };
    if acima.is_nan() || acima <= 0.0 {
        return [0.0; 3];
    }
    let w = acima / l;
    [rgb[0] * w, rgb[1] * w, rgb[2] * w]
}

/// ⭐⭐⭐ **O PASSE: soma o halo ao quadro, no sítio.**
///
/// `hdr` é **linear de CENA** — antes da exposição e do tonemapper. ⚠️ *Ler depois deles é o que o
/// `docs/Render3d/01` §2 chama de «sem o `1`, o bloom mente»*: o tonemapper comprime justamente o
/// que havia para colher.
///
/// ⚠️ **Devolve cedo quando não há o que somar** — ver [`Bloom::contributes`] —, e é isso que faz o
/// caminho de omissão custar **zero**.
pub fn apply(hdr: &mut [[f32; 3]], w: usize, h: usize, b: &Bloom) {
    let add = halo(hdr, w, h, b);
    if add.is_empty() {
        return;
    }
    for (px, a) in hdr[..w * h].iter_mut().zip(add) {
        for c in 0..3 {
            px[c] += a[c];
        }
    }
}

/// ⭐⭐⭐ **O HALO SOZINHO** — o que o brilho ACRESCENTA, sem o somar.
///
/// # ⚠️ Porque a porta devolve o halo em vez de só o somar
///
/// Quem compõe pode não ter o quadro em cena-linear para somar: no modelador o **FUNDO** é uma cor
/// que o artista dá em BYTES e que nunca passou pelo olhar, logo não há como o converter de volta.
/// ⛔ E tratar peça e fundo por regras diferentes está **refutado por medição** neste módulo
/// (`docs/Render3d/10` §11.3): *um `if` por pixel desenha a fronteira entre os dois ramos*, e a
/// cura que o fez pintou um fio serrilhado na silhueta.
///
/// ⇒ o halo sai daqui em **cena-linear** — lido do HDR, que é o que o torna honesto — e quem compõe
/// aplica-lhe o olhar e soma-o **com a mesma regra em todo pixel**.
///
/// Devolve **vazio** quando não há o que somar, e é isso que faz a omissão custar zero.
#[must_use]
pub fn halo(hdr: &[[f32; 3]], w: usize, h: usize, b: &Bloom) -> Vec<[f32; 3]> {
    if !b.contributes() || w == 0 || h == 0 || hdr.len() < w * h {
        return Vec::new();
    }
    let n = levels_that_fit(w, h);
    if n == 0 {
        return Vec::new();
    }

    // (1) o corte, na resolução cheia.
    let corte: Vec<[f32; 3]> = hdr[..w * h].iter().map(|c| bright(*c, &b.params)).collect();

    // (2) DESCE a cadeia com o filtro de 13 taps.
    let mut mips: Vec<(Vec<[f32; 3]>, usize, usize)> = Vec::with_capacity(n);
    let (mut lw, mut lh) = (w, h);
    let mut actual = corte;
    for _ in 0..n {
        let (dw, dh) = (lw / 2, lh / 2);
        actual = desce13(&actual, lw, lh);
        lw = dw;
        lh = dh;
        mips.push((actual.clone(), lw, lh));
    }

    // (3) SOBE um degrau de cada vez, somando. ⭐ É esta a metade que dá a qualidade: cada nível é
    // re-borrado por TODAS as tendas mais finas por onde passa, e é isso que dissolve os cantos.
    //
    // ⭐⭐⭐ **A soma é ADITIVA e sem pesos — é o NOSSO modelo.** A 1.ª redacção desta crate tinha
    // **sete pesos por nível** (o `glow_levels/1..7` do Godot) e isso era pior por medição: sete
    // controlos onde este tem **um**, e quatro deles a nascer em zero — quatro fileiras que se leem
    // como mortas. O tamanho do halo é o [`BloomParams::radius`], que estica a TENDA.
    #[allow(clippy::cast_precision_loss)]
    let aspecto = w as f32 / h.max(1) as f32;
    // ⭐⭐⭐ **A base da tenda sai da PORTA DO TIPO, e a minha cópia dela foi APAGADA.** Eu tinha-a
    // reescrito nesta crate; ao mover o `BloomParams` para cá, o método dele veio junto — e o
    // compilador obrigou-o a vir (*um `impl` inerente para um tipo de outra crate é proibido*).
    // *A linguagem disse a coisa certa: a lei de um dado mora onde o dado mora.*
    let raios = b.params.upsample_basis(aspecto);
    let (topo, tw, th) = mips[n - 1].clone();
    let mut acc = topo;
    let (mut aw, mut ah) = (tw, th);
    for k in (0..n - 1).rev() {
        let (mip, mw, mh) = &mips[k];
        let mut subido = sobe_tenda(&acc, aw, ah, *mw, *mh, raios);
        for (d, s) in subido.iter_mut().zip(mip) {
            for c in 0..3 {
                d[c] += s[c];
            }
        }
        acc = subido;
        aw = *mw;
        ah = *mh;
    }

    // (4) o último degrau, de volta ao quadro.
    let mut halo = sobe_tenda(&acc, aw, ah, w, h, raios);

    // (5) e a COR do halo — dessatura, tinge e escala, na ordem do gémeo que já shipa.
    //
    // ⚠️ **A dessaturação é para a LUMINÂNCIA**, com os pesos de sempre: com `saturation = 0` o halo
    // sai cinzento e com `1` guarda a cor da fonte. *É um dos knobs que o nosso modelo tem e o do
    // Godot não.*
    let p = &b.params;
    for px in &mut halo {
        let luz = 0.2126f32.mul_add(px[0], 0.7152f32.mul_add(px[1], 0.0722 * px[2]));
        for (canal, tinta) in px.iter_mut().zip(&p.tint) {
            *canal = luz.mul_add(1.0 - p.saturation, *canal * p.saturation) * tinta * p.intensity;
        }
    }
    halo
}

/// ⭐⭐⭐ **O FILTRO DE 13 TAPS QUE DESCE** — o da referência (Call of Duty / Jimenez, SIGGRAPH 2014),
/// que é o que o [`ph2d_render`] já corre no dispositivo para o brilho do Motion.
///
/// ⛔⛔ **A 1.ª redacção desta crate usava uma média de `2×2` e subia CADA nível directamente ao
/// quadro cheio**, e o dono viu o resultado numa foto: *«bloom bizarro de baixa qualidade»*, com
/// blocos à volta das peças. A causa é aritmética — um bilinear a partir de um nível de `15` px de
/// largura para `1900` desenha a GRELHA desse nível —, e a cura é a da referência: descer com um
/// filtro largo e subir **um degrau de cada vez**.
///
/// ⚠️⚠️ **E o achado maior é que esta lei já existia neste repositório** (`ph2d-render/src/shaders/
/// bloom.wgsl`, doc 67 do Motion). *Antes de construir, MEÇA se a composição já o exprime*
/// (`CLAUDE.md` §5.0) — e eu construí um motor pior ao lado de um melhor.
fn desce13(src: &[[f32; 3]], sw: usize, sh: usize) -> Vec<[f32; 3]> {
    let (dw, dh) = (sw / 2, sh / 2);
    if dw == 0 || dh == 0 {
        return Vec::new();
    }
    #[allow(clippy::cast_precision_loss)]
    let (tx, ty) = (1.0 / sw as f32, 1.0 / sh as f32);
    let mut out = vec![[0.0f32; 3]; dw * dh];
    for j in 0..dh {
        for i in 0..dw {
            #[allow(clippy::cast_precision_loss)]
            let (u, v) = ((i as f32 + 0.5) / dw as f32, (j as f32 + 0.5) / dh as f32);
            let s = |du: f32, dv: f32| amostra_uv(src, sw, sh, u + du, v + dv);
            let (a, c, g, i2) = (
                s(-2.0 * tx, 2.0 * ty),
                s(2.0 * tx, 2.0 * ty),
                s(-2.0 * tx, -2.0 * ty),
                s(2.0 * tx, -2.0 * ty),
            );
            let (b2, d, f, h2) = (
                s(0.0, 2.0 * ty),
                s(-2.0 * tx, 0.0),
                s(2.0 * tx, 0.0),
                s(0.0, -2.0 * ty),
            );
            let e = s(0.0, 0.0);
            let (j2, k2, l2, m2) = (s(-tx, ty), s(tx, ty), s(-tx, -ty), s(tx, -ty));
            let px = &mut out[j * dw + i];
            for ch in 0..3 {
                px[ch] = e[ch].mul_add(
                    0.125,
                    (a[ch] + c[ch] + g[ch] + i2[ch]).mul_add(
                        0.031_25,
                        (b2[ch] + d[ch] + f[ch] + h2[ch])
                            .mul_add(0.0625, (j2[ch] + k2[ch] + l2[ch] + m2[ch]) * 0.125),
                    ),
                );
            }
        }
    }
    out
}

/// ⭐⭐⭐ **A TENDA DE 9 TAPS QUE SOBE UM DEGRAU** — `[1 2 1; 2 4 2; 1 2 1] / 16`, a da referência.
///
/// ⚠️ **Ela sobe UM degrau**, e é isso que a separa da 1.ª redacção desta crate: subir do nível `k`
/// direito ao quadro é um bilinear de `2^k` para `1`, e o que ele desenha é a grelha do nível `k`.
fn sobe_tenda(
    src: &[[f32; 3]],
    sw: usize,
    sh: usize,
    dw: usize,
    dh: usize,
    base: [f32; 4],
) -> Vec<[f32; 3]> {
    let (du, dv) = ([base[0], base[1]], [base[2], base[3]]);
    let mut out = vec![[0.0f32; 3]; dw * dh];
    if sw == 0 || sh == 0 || dw == 0 || dh == 0 {
        return out;
    }
    for j in 0..dh {
        for i in 0..dw {
            #[allow(clippy::cast_precision_loss)]
            let (u, v) = ((i as f32 + 0.5) / dw as f32, (j as f32 + 0.5) / dh as f32);
            let s = |du: f32, dv: f32| amostra_uv(src, sw, sh, u + du, v + dv);
            let e = s(0.0, 0.0);
            let (b2, d, f, h2) = (
                s(dv[0], dv[1]),
                s(-du[0], -du[1]),
                s(du[0], du[1]),
                s(-dv[0], -dv[1]),
            );
            let (a, c, g, i2) = (
                s(-du[0] + dv[0], -du[1] + dv[1]),
                s(du[0] + dv[0], du[1] + dv[1]),
                s(-du[0] - dv[0], -du[1] - dv[1]),
                s(du[0] - dv[0], du[1] - dv[1]),
            );
            let px = &mut out[j * dw + i];
            for ch in 0..3 {
                px[ch] = e[ch].mul_add(
                    4.0,
                    (b2[ch] + d[ch] + f[ch] + h2[ch]).mul_add(2.0, a[ch] + c[ch] + g[ch] + i2[ch]),
                ) / 16.0;
            }
        }
    }
    out
}

/// Amostra bilinear em coordenadas de **UV**, com a borda presa — é o `textureSample` do gémeo.
fn amostra_uv(src: &[[f32; 3]], sw: usize, sh: usize, u: f32, v: f32) -> [f32; 3] {
    #[allow(clippy::cast_precision_loss)]
    let (x, y) = (u.mul_add(sw as f32, -0.5), v.mul_add(sh as f32, -0.5));
    amostra_bilinear(src, sw, sh, x, y)
}

fn amostra_bilinear(src: &[[f32; 3]], sw: usize, sh: usize, u: f32, v: f32) -> [f32; 3] {
    #[allow(clippy::cast_possible_truncation, clippy::cast_precision_loss)]
    let (x0, y0) = (u.floor(), v.floor());
    let (tx, ty) = (u - x0, v - y0);
    #[allow(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        clippy::cast_precision_loss
    )]
    let ix = |f: f32, lim: usize| -> usize { (f.max(0.0) as usize).min(lim - 1) };
    let (xa, xb) = (ix(x0, sw), ix(x0 + 1.0, sw));
    let (ya, yb) = (ix(y0, sh), ix(y0 + 1.0, sh));
    let mut out = [0.0f32; 3];
    for c in 0..3 {
        let a = src[ya * sw + xa][c] * (1.0 - tx) + src[ya * sw + xb][c] * tx;
        let b = src[yb * sw + xa][c] * (1.0 - tx) + src[yb * sw + xb][c] * tx;
        out[c] = a * (1.0 - ty) + b * ty;
    }
    out
}

/// ⭐⭐⭐ **A MESMA LEI, em WGSL** — para quem pinta no dispositivo. Ver o módulo.
#[path = "wgsl.rs"]
pub mod wgsl;

#[cfg(test)]
#[path = "lei_tests.rs"]
mod lei_tests;

impl BloomParams {
    /// O índice do pipeline de composição — grampeado à lista que de facto existe.
    ///
    /// ⚠️ **Grampeado, e não `unwrap`**: o número vem de um param que um documento carregado ou
    /// uma edição por MCP pode ter posto fora da faixa, e escolher um pipeline que não existe
    /// seria um `panic` no meio do quadro. Fora da faixa cai no `Add`, que é o neutro.
    #[must_use]
    pub fn operation_tag(&self) -> usize {
        if self.operation.is_finite() && self.operation >= 0.5 {
            1
        } else {
            0
        }
    }

    /// `1` quando o bright-pass lê o ALFA, `0` quando lê a luminância — o número que o shader
    /// compara com `0,5`. Um valor lixo conta como a luminância, o caminho de sempre.
    #[must_use]
    pub fn source_flag(&self) -> f32 {
        if self.source.is_finite() && self.source >= 0.5 {
            1.0
        } else {
            0.0
        }
    }

    /// Pack the soft-knee curve the prefilter shader expects:
    /// `(threshold, threshold-knee, 2·knee, 0.25/knee)` (COD/Karis).
    pub fn prefilter_curve(&self) -> [f32; 4] {
        let knee = self.knee.max(1e-4);
        [
            self.threshold,
            self.threshold - knee,
            2.0 * knee,
            0.25 / knee,
        ]
    }
}

impl BloomParams {
    /// **A BASE 2×2 da tenda do upsample**, em UV: `[du.x, du.y, dv.x, dv.y]`.
    ///
    /// Os 9 taps deixam de ser `(±x, ±y)` e passam a ser `(±du ±dv)`. No neutro
    /// (`stretch = 1`) o caminho é **LITERAL** e devolve `[fr, 0, 0, fr·aspect]`,
    /// que reconstrói tap a tap os offsets de sempre — `uv + (−du + dv)` é
    /// `uv + (−fr, fr·aspect)`, a mesma soma, ao bit.
    ///
    /// ⚠️ **A anisotropia é calculada em PIXELS e convertida no fim.** O `aspect`
    /// existe para o halo sair redondo na tela; aplicá-lo antes da rotação faria o
    /// ângulo significar coisas diferentes em janelas diferentes — o mesmo `45°`
    /// apontaria para outro sítio ao redimensionar.
    pub fn upsample_basis(&self, aspect: f32) -> [f32; 4] {
        let fr = BASE_FILTER_RADIUS * self.radius.max(0.0);
        let s = self.stretch.max(MIN_STRETCH);
        if s == 1.0 {
            return [fr, 0.0, 0.0, fr * aspect];
        }
        let (c, sn) = cos_sin_cycles(self.angle / 360.0);
        // Ao longo do ângulo alarga por `s`; perpendicular aperta por `1/s`, para o
        // «raio» continuar a ser a média geométrica dos dois e o knob não mudar a
        // ENERGIA do halo, só a forma dele.
        let (ax, ay) = (c * fr * s, sn * fr * s);
        let (bx, by) = (-sn * fr / s, c * fr / s);
        [ax, ay * aspect, bx, by * aspect]
    }

    /// O teto do bright-pass, como o shader o quer: `0` (desligado) vira o maior
    /// finito do `Rgba16Float`, que é o teto que a REPRESENTAÇÃO já impunha — então
    /// o `min` do shader é um no-op sobre qualquer valor que o RT consiga guardar.
    pub fn clamp_limit(&self) -> f32 {
        if self.clamp > 0.0 {
            self.clamp
        } else {
            F16_MAX
        }
    }
}

/// Upsample tent radius in UV at `radius = 1` (the mip chain does the heavy
/// spreading; this is the per-level tent overlap). Scaled by `BloomParams::radius`.
pub const BASE_FILTER_RADIUS: f32 = 0.006;
/// Piso da anamorfose: abaixo disto o eixo estreito colapsa e a tenda deixa de
/// cobrir o próprio texel (o `1/s` explodiria o outro eixo).
pub const MIN_STRETCH: f32 = 0.05;
/// O maior finito representável em `Rgba16Float` — o teto que o formato do RT já
/// impõe, e o valor com que o clamp desligado passa pelo `min` sem morder.
pub const F16_MAX: f32 = 65_504.0;

/// ⭐ **A aritmética da FASE** (o seno polinomial que a anamorfose lê) vive no irmão — ver
/// [`trig`]. ⛔ Corte por responsabilidade e por tecto de LOC, nunca por isenção.
#[path = "trig.rs"]
mod trig;
pub(crate) use trig::cos_sin_cycles;
