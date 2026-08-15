//! **A QUINA** (W10) — passar raspando por baixo de uma beirada em vez de bater
//! nela.
//!
//! O gesto que esta lei existe para perdoar é o mais comum de um platformer: o
//! jogador pula rente a uma plataforma, a **cabeça** dele encosta na quina por
//! dois ou três centímetros, o solver come a velocidade vertical inteira, e o
//! personagem despenca de um pulo que — para o olho — tinha passado.
//!
//! # ⚠️ PREDITIVA, nunca reativa — e a diferença não é estilo
//!
//! A versão reativa (detectar o contato e devolver a velocidade que ele comeu)
//! **inventa energia**: o solver já resolveu aquele tique, e escrever por cima do
//! resultado dele é discutir com quem acabou de decidir. Pior, ela é
//! *observável* — o personagem para por um quadro e depois salta.
//!
//! Aqui a pergunta é feita **antes**: dado que estou subindo a `rel_up`, o
//! próximo tique me leva a `rel_up · dt` mais alto; há teto ali? Se houver, e se
//! um deslocamento LATERAL pequeno me livrar dele, o personagem é **deslocado**
//! (posição, não impulso) e o contato **nunca acontece**. Nada é devolvido porque
//! nada foi tirado.
//!
//! # ⚠️ Um DESLOCAMENTO, e é por isso que ele não vira `Motor`
//!
//! [`crate::PlayerStep::nudge`] é medido em METROS e a ponte o aplica escrevendo
//! a translação do corpo — a velocidade não é tocada. A alternativa (um `boost`
//! lateral de `escape / dt`) dá o mesmo deslocamento no tique e deixa o
//! personagem **derivando de lado** a vários metros por segundo depois, porque
//! ninguém a remove: a assistência viraria um empurrão. Correção de quina é
//! assistência de POSIÇÃO em todo produto que a tem, e é por isso.
//!
//! # O sensor é um PERFIL, e a razão é que o número é uma DISTÂNCIA
//!
//! *"A cabeça bate?"* responde-se com um raio. *"Quanto preciso andar de lado
//! para não bater?"* não — ela pede saber **onde a quina está**, e a forma barata
//! e explícita de saber isso é amostrar o teto ao longo da largura do corpo mais
//! o alcance da assistência, dos dois lados ([`corner_offsets`]).
//!
//! ⚠️ **A grade é uma porta única** (`corner_offsets`), perguntada por quem
//! CASTA (a ponte) e por quem INTERPRETA (esta lei). Duas cópias da mesma
//! aritmética deslocariam o perfil de meia célula em relação ao corpo, e o
//! sintoma seria um personagem empurrado para dentro da quina de que ele estava
//! fugindo.
//!
//! ⚠️ **Uma amostra representa uma CÉLULA**, não um ponto: a amostra `i` fala por
//! `[oᵢ − passo/2, oᵢ + passo/2]`, então o corpo deslocado de `d` toca essa célula
//! quando `|oᵢ − d| ≤ meia_largura + passo/2`. É meio passo mais conservador que
//! a geometria exata — a assistência dispara um pouco antes do necessário, que é
//! o lado seguro de errar.

use crate::{JumpConfig, Vec2, perp_cw};

/// Quantos raios o perfil do teto tem.
///
/// ⚠️ **Ele decide quanto do alcance autorado se PERDE, e o número foi medido
/// antes de ser escolhido.** Uma amostra fala por uma célula de largura `passo`,
/// então a lei não pode afirmar que a beirada está a menos de meia célula de
/// onde a viu — e essa meia célula sai do alcance útil: **o encosto que a
/// assistência de fato salva é `corner_reach − passo/2`**.
///
/// O primeiro corte usava **25** e o passo saía 2,7 cm num corpo de 40 cm: um
/// encosto de 10 cm **não era salvo** com o alcance em 12 cm (medido em
/// `measure_corner`), porque a meia célula mais o arredondamento comiam os 2 cm
/// de folga. Com **65** o passo cai para 1,0 cm e a perda para 0,5 cm.
///
/// ⚠️ **E o custo NÃO foi o que decidiu, porque ele não existe:** o sensor
/// inteiro (perfil + as duas laterais) mede **+0,0002 ms por tique de subida**
/// — cerca de 8 ns por raio, e só nos tiques em que o personagem sobe. Escolher
/// 25 para "economizar" seria pagar precisão por nada.
pub const CORNER_SAMPLES: usize = 65;

/// **O TETO da contagem de amostras** — o MESMO número e o MESMO argumento do
/// irmão [`crate::MAX_WALL_SAMPLES`], e um só porque só há uma medição.
///
/// ⚠️ **Não é um teto de TEMPO** (`measure_player_probes::measure_what_a_sample_costs`):
/// **18 ns por raio, PLANO em N** ⇒ 257 raios custam **4,55 µs, 0,027% de um
/// quadro**, e só nos tiques de subida. O que se esgota é a **precisão de
/// representação** — o passo do perfil cai a **2,5 mm em 257** contra o
/// `normalized_allowed_linear_error` de ~**1,3 mm** com que o solver assenta.
pub const MAX_CORNER_SAMPLES: usize = 257;

/// Em quantos degraus o escape é procurado dentro do alcance.
///
/// ⚠️ **É uma resolução DIFERENTE da do perfil, e separá-las é o ponto:** o
/// perfil diz *onde está a beirada* (e erra por meia célula), a busca diz *que
/// deslocamentos experimentar*. Amarrar a busca à grade do perfil somaria os dois
/// erros; assim só o primeiro existe, e `clear(d)` é computável em qualquer `d`.
pub const CORNER_SEARCH_STEPS: usize = 16;

/// Quantos tiques de antecedência o sensor olha.
///
/// ⚠️ **`2` é literalmente a frase do plano** (*"o boost lateral no tick anterior
/// ao contato"*): o raio mede `rel_up · dt · 2`, então a quina é vista no tique
/// ANTES daquele em que a cabeça a alcançaria, e o deslocamento acontece com um
/// tique de folga. Um `1.0` agiria no mesmo tique do contato — correto na
/// aritmética e sem margem nenhuma para o arredondamento do solver.
///
/// E ele **se escala sozinho com a velocidade**: um comprimento fixo em metros
/// seria curto num pulo rápido e longo demais perto do ápice, onde o personagem
/// nem alcançaria o teto.
///
/// ⚠️ **MEDIDO: o segundo tique é MARGEM, não correção — a mutação `2.0 → 1.0`
/// sobrevive aos cinco gates de comportamento.** E o porquê é aritmética: o
/// deslocamento é de POSIÇÃO e acontece *antes* do `step` do mesmo tique, então
/// ver a quina no tique em que a cabeça a alcançaria já bastaria; a subida real
/// é ainda `½·g·dt²` menor que `rel_up · dt`, o que torna um `1.0` conservador
/// por conta própria.
///
/// Ele fica em `2.0` por dois motivos que não são "senão quebra", e ficam
/// escritos aqui em vez de descobertos por quem tentar "limpar" a constante
/// depois ([[feedback_layered_defenses_need_per_layer_gates]]): o desvio começa
/// um tique antes (o olho lê intenção, não um salto no último quadro), e a
/// margem não depende de o solver nunca fechar mais do que a velocidade do
/// início do tique previa.
pub const CORNER_LOOKAHEAD: f32 = 2.0;

/// **Onde os raios do perfil nascem**, como deslocamentos laterais a partir do
/// centro do corpo.
///
/// O vão cobre `meia_largura + alcance` para cada lado — que é exatamente a união
/// de todas as posições que o corpo pode ocupar depois de um escape válido. Sem
/// essa margem a lei não teria como saber que o DESTINO do deslocamento está
/// livre, e empurraria o personagem de uma quina para dentro da seguinte.
#[must_use]
pub fn corner_offsets(half_width: f32, reach: f32, samples: usize) -> [f32; MAX_CORNER_SAMPLES] {
    let n = crate::wall::odd_samples(samples, MAX_CORNER_SAMPLES);
    let span = half_width.max(0.0) + reach.max(0.0);
    let last = (n - 1) as f32;
    let mut out = [0.0; MAX_CORNER_SAMPLES];
    for (i, o) in out.iter_mut().take(n).enumerate() {
        *o = -span + 2.0 * span * (i as f32) / last;
    }
    out
}

/// **O que o sensor viu acima (e ao lado) da cabeça neste tique.**
///
/// Preenchido pela ponte, consumido por [`corner_escape`]. Os três campos existem
/// porque a decisão precisa dos três: a largura diz que células o corpo ocupa, o
/// perfil diz quais estão tapadas, e a folga lateral impede que a cura seja pior
/// que a doença.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct CeilingProbe {
    /// Meia-largura do corpo — a da caixa que envolve **todas** as formas dele.
    ///
    /// ⚠️ A caixa, e não a forma: um corpo composto (W-Compound) tem várias, e
    /// perguntar à primeira é a premissa que envelheceu e custou quatro defeitos
    /// à jornada passada. A caixa é conservadora e é sempre uma.
    pub half_width: f32,
    /// `blocked[i]` = há teto ao alcance no deslocamento `corner_offsets()[i]`.
    ///
    /// ⚠️ **Dimensionado pelo TETO e não pela contagem autorada** — a cauda não
    /// usada fica `false`, e é isso que mantém este tipo sem alocação no caminho
    /// quente com a contagem a variar por personagem. Quem lê corta em
    /// [`Self::samples`].
    pub blocked: [bool; MAX_CORNER_SAMPLES],
    /// Metros LIVRES ao lado do corpo, na altura dele — `[esquerda, direita]`.
    ///
    /// ⚠️ Saturado no próprio alcance (nunca infinito): `reach` significa *"livre
    /// até onde esta assistência poderia querer ir"*, e é o que mantém a struct
    /// comparável e o gate legível.
    pub side_clear: [f32; 2],
    /// Quantas das `blocked` este perfil de facto varreu.
    ///
    /// ⚠️ **`0` é legítimo e significa *"o leque não foi perguntado"***: com o
    /// assist desarmado (`corner_reach == 0`) a ponte ainda constrói este sensor
    /// para responder o [`Self::head_blocked`], que é um FATO e não uma
    /// assistência. Quem lê o leque é o [`corner_escape`], e ele só é alcançado
    /// sob o [`corner_probe_wanted`], que exige o alcance autorado — a razão de
    /// um perfil meio-preenchido nunca chegar lá.
    pub samples: usize,
    /// **A cabeça bate neste tique?** — o `is_on_ceiling` do Godot.
    ///
    /// ⚠️ **É um FATO, e é por isso que não tem knob** ([`ceiling_fact_wanted`]
    /// não olha para config nenhuma). Os outros dois campos desta struct
    /// descrevem uma ASSISTÊNCIA opcional; este descreve o mundo, e um readout
    /// que existisse só quando uma ajuda está ligada seria um teto que aparece
    /// *às vezes*, em silêncio.
    ///
    /// ⚠️ **A régua é o que a cabeça percorre NESTE tique** (`rel_up · dt`), sem
    /// o [`JumpConfig::corner_lookahead`] que o leque usa: a antecipação é do
    /// assist, e herdá-la faria o fato dizer *"bateu"* um tique antes de bater.
    ///
    /// ⚠️ **Varredura da FORMA, não um raio** — o corpo pode ser composto
    /// (W-Compound), e um raio pelo centro atravessaria uma marquise que o ombro
    /// encosta. É o mesmo primitivo que o [`crate::Headroom`] já usa.
    pub head_blocked: bool,
}

impl CeilingProbe {
    /// Um perfil sem nada acima — o neutro, e o que uma cena sem teto produz.
    #[must_use]
    pub fn clear(half_width: f32, reach: f32) -> Self {
        Self {
            half_width,
            blocked: [false; MAX_CORNER_SAMPLES],
            side_clear: [reach, reach],
            samples: CORNER_SAMPLES,
            head_blocked: false,
        }
    }
}

/// **Vale a pena perguntar se a cabeça bate?** — a porta do FATO.
///
/// ⚠️ **Ela NÃO é o [`corner_probe_wanted`], e a diferença é o knob.** Aquela é a
/// porta de uma ASSISTÊNCIA e exige `corner_reach > 0`, porque uma ajuda
/// desligada não deve custar um raio. Esta descreve o MUNDO, e um fato não é
/// opt-in: um `is_on_ceiling` que só existisse com a assistência de quina armada
/// seria um readout falso na maioria dos tiques, em silêncio.
///
/// ⚠️ **Ela é o SUPERCONJUNTO da outra** (`corner_probe_wanted` = isto **e**
/// `corner_reach > 0`), e é isso que mantém o assist byte-idêntico: com o
/// alcance no ponto de partida (`0,12`) as duas coincidem, então na configuração
/// que shipa o sensor **já rodava** e o fato custa uma varredura a mais, nunca
/// uma invocação nova.
///
/// - **Subindo**, porque uma cabeça que desce não bate em teto nenhum.
/// - **No ar**, porque de pé sob uma marquise o personagem não está a subir
///   contra ela — o que ele tem ali é falta de espaço, que é a pergunta do
///   [`crate::Headroom`], não esta.
#[must_use]
pub fn ceiling_fact_wanted(grounded: bool, rel_up: f32) -> bool {
    !grounded && rel_up > 0.0
}

/// **Vale a pena perguntar ao mundo?**
///
/// ⚠️ Porta única, e o motivo é o custo: a ponte a pergunta para decidir se casta
/// os [`CORNER_SAMPLES`] raios, e a lei a pergunta para decidir se age. Duas
/// cópias da condição divergiriam no dia em que uma delas ganhasse um caso
/// especial, e o sintoma seria a assistência ora existir ora não, sem nada na
/// tela a explicar.
///
/// - **Subindo**, porque uma cabeça que desce não bate em beirada nenhuma.
/// - **No ar**, porque de pé sob uma marquise o personagem não está indo a lugar
///   nenhum — deslocá-lo ali seria a ferramenta andando sozinha.
/// - Com **alcance autorado**, porque `0` desliga a assistência inteira.
#[must_use]
pub fn corner_probe_wanted(cfg: &JumpConfig, grounded: bool, rel_up: f32) -> bool {
    cfg.corner_reach > 0.0 && cfg.corner_reach.is_finite() && !grounded && rel_up > 0.0
}

/// **O menor deslocamento lateral que livra a cabeça** — `None` quando não há
/// nada a livrar, ou quando nenhum deslocamento dentro do alcance resolve.
///
/// ⚠️ **Os dois `None` são o mesmo para o chamador, e isso é desenho:** nada
/// bloqueando ⇒ não há o que corrigir; um teto de verdade ⇒ não se deve corrigir
/// (o personagem tem de bater, senão a assistência vira teletransporte). Separá-
/// los daria ao chamador uma distinção que ele não usa.
///
/// A busca é por magnitude crescente, e o empate (uma obstrução simétrica sobre
/// a cabeça — uma estalactite no eixo) resolve-se para a DIREITA. É arbitrário e
/// é determinístico, que é o que o `physics_ecs_c9` exige.
#[must_use]
pub fn corner_escape(probe: &CeilingProbe, reach: f32) -> Option<f32> {
    let w = probe.half_width;
    if !w.is_finite() || w <= 0.0 || !reach.is_finite() || reach <= 0.0 {
        return None;
    }
    let n = crate::wall::odd_samples(probe.samples, MAX_CORNER_SAMPLES);
    let offs = corner_offsets(w, reach, n);
    let step = offs[1] - offs[0];
    if !step.is_finite() || step <= 0.0 {
        return None;
    }
    // Uma amostra fala por uma CÉLULA de largura `step` (ver o topo do módulo).
    let half_cell = step * 0.5;
    let clear = |d: f32| {
        !offs[..n]
            .iter()
            .zip(probe.blocked[..n].iter())
            .any(|(&o, &b)| b && (o - d).abs() <= w + half_cell)
    };

    if clear(0.0) {
        return None;
    }
    for k in 1..=CORNER_SEARCH_STEPS {
        let d = reach * k as f32 / CORNER_SEARCH_STEPS as f32;
        if d <= probe.side_clear[1] && clear(d) {
            return Some(d);
        }
        if d <= probe.side_clear[0] && clear(-d) {
            return Some(-d);
        }
    }
    None
}

/// O escape como DESLOCAMENTO em mundo, ao longo do eixo horizontal do `up`.
///
/// ⚠️ O eixo sai do `up` pela mesma `perp_cw` que a caminhada usa — um literal
/// `[1, 0]` aqui seria a segunda resposta que diverge no dia em que a gravidade
/// lateral alcançar o player.
#[must_use]
pub fn corner_nudge(probe: Option<&CeilingProbe>, cfg: &JumpConfig, up: Vec2) -> Vec2 {
    let Some(p) = probe else {
        return [0.0, 0.0];
    };
    let Some(d) = corner_escape(p, cfg.corner_reach) else {
        return [0.0, 0.0];
    };
    let axis = perp_cw(up);
    [axis[0] * d, axis[1] * d]
}

#[cfg(test)]
mod tests {
    use super::*;

    const UP: Vec2 = [0.0, 1.0];
    const W: f32 = 0.3;
    const REACH: f32 = 0.15;

    fn cfg() -> JumpConfig {
        JumpConfig::STARTING_POINT
    }

    /// Um perfil com tudo à ESQUERDA de `edge` tapado — a beirada canônica.
    fn ledge_on_the_left(edge: f32) -> CeilingProbe {
        let offs = corner_offsets(W, REACH, CORNER_SAMPLES);
        let mut blocked = [false; MAX_CORNER_SAMPLES];
        for (i, &o) in offs.iter().enumerate().take(CORNER_SAMPLES) {
            blocked[i] = o <= edge;
        }
        CeilingProbe {
            half_width: W,
            blocked,
            side_clear: [REACH, REACH],
            samples: CORNER_SAMPLES,
            // Parte da cabeça está tapada, então ela ESTÁ a bater — o valor que
            // mantém a fixture coerente consigo mesma. O `corner_escape` não lê
            // este campo; ele é do readout.
            head_blocked: true,
        }
    }

    /// A grade é simétrica, cobre o corpo MAIS o alcance, e é monotônica.
    #[test]
    fn the_grid_spans_the_body_plus_the_reach_on_both_sides() {
        let offs = corner_offsets(W, REACH, CORNER_SAMPLES);
        assert!((offs[0] + (W + REACH)).abs() < 1.0e-6, "{:?}", offs[0]);
        assert!(
            (offs[CORNER_SAMPLES - 1] - (W + REACH)).abs() < 1.0e-6,
            "{:?}",
            offs[CORNER_SAMPLES - 1]
        );
        assert!(
            (offs[CORNER_SAMPLES / 2]).abs() < 1.0e-6,
            "o centro e' zero"
        );
        // ⚠️ A CONTAGEM, nao o array: ele e' dimensionado pelo TETO desde a
        // W-Probes2, e a cauda nao usada fica em zero.
        for pair in offs[..CORNER_SAMPLES].windows(2) {
            assert!(pair[1] > pair[0], "a grade e' crescente");
        }
    }

    /// **A contagem é AUTORADA, e o default é o mundo de sempre.**
    ///
    /// ⚠️ Duas metades: a grade honra o número pedido (o passo encolhe), e o
    /// default reproduz `CORNER_SAMPLES` — é isso que mantém byte-idêntico todo
    /// player já autorado.
    #[test]
    fn the_profile_honours_the_authored_count() {
        let d = corner_offsets(W, REACH, CORNER_SAMPLES);
        let fine = corner_offsets(W, REACH, 129);
        assert!(
            (fine[128] - (W + REACH)).abs() < 1.0e-6,
            "129 amostras cobrem o mesmo vao: {:?}",
            fine[128]
        );
        let step_d = d[1] - d[0];
        let step_f = fine[1] - fine[0];
        assert!(
            step_f < step_d * 0.55,
            "o dobro das amostras corta o passo pela metade: {step_d} -> {step_f}"
        );
        // E o clamp para IMPAR: 128 pedido tem de dar a MESMA grade de 129.
        let even = corner_offsets(W, REACH, 128);
        assert_eq!(
            even[..129],
            fine[..129],
            "uma contagem PAR sobe para a impar seguinte (o meio e' a ancora)"
        );
    }

    /// **Céu limpo não move ninguém.** O neutro, e é o que toda cena sem teto é.
    #[test]
    fn nothing_above_means_no_correction() {
        let p = CeilingProbe::clear(W, REACH);
        assert_eq!(corner_escape(&p, REACH), None);
        assert_eq!(corner_nudge(Some(&p), &cfg(), UP), [0.0, 0.0]);
    }

    /// **A quina rasa é livrada, e para o lado LIVRE.**
    ///
    /// A beirada cobre 3 cm do lado esquerdo da cabeça (borda em `−0,27`), então
    /// o escape é para a direita e tem a ordem daquela sobreposição.
    ///
    /// **Mutação que deve sangrar:** devolver `Some(-d)` antes de `Some(d)` — o
    /// personagem seria empurrado para DENTRO da beirada.
    #[test]
    fn a_shallow_clip_is_escaped_towards_the_free_side() {
        let p = ledge_on_the_left(-0.27);
        let d = corner_escape(&p, REACH).expect("uma quina rasa TEM de ser livrada");
        assert!(
            d > 0.0,
            "o escape foge da beirada, que esta' a esquerda: {d}"
        );
        assert!(d <= REACH, "e cabe no alcance autorado: {d}");
        // O deslocamento é da ordem da sobreposição (3 cm), com a quantização da
        // grade e a meia-célula de conservadorismo por cima.
        assert!(d < 0.12, "e nao e' o alcance inteiro: {d}");
    }

    /// **Um TETO de verdade não é uma quina, e a lei não o perdoa.**
    ///
    /// É a metade que impede a assistência de virar teletransporte: com a cabeça
    /// inteira tapada, nenhum deslocamento livra, e o personagem bate — que é o
    /// que ele tem de fazer.
    ///
    /// **Mutação que deve sangrar:** devolver `Some(reach)` quando a busca falha.
    #[test]
    fn a_real_ceiling_is_not_forgiven() {
        let p = CeilingProbe {
            half_width: W,
            blocked: [true; MAX_CORNER_SAMPLES],
            side_clear: [REACH, REACH],
            samples: CORNER_SAMPLES,
            head_blocked: true,
        };
        assert_eq!(corner_escape(&p, REACH), None);
    }

    /// **Uma quina FUNDA demais também não** — o alcance é o que o artista
    /// autorou, e passar dele seria a ferramenta decidindo por ele.
    #[test]
    fn a_clip_deeper_than_the_reach_is_refused() {
        // Beirada cobrindo até `+0.1`: sobreposição de 0,4 m sobre um corpo de
        // 0,6 m de largura, muito além dos 0,15 de alcance.
        let p = ledge_on_the_left(0.1);
        assert_eq!(corner_escape(&p, REACH), None);
    }

    /// ⚠️ **A PAREDE ao lado veta o escape** — sem esta metade a assistência
    /// empurra o personagem para dentro dela, e o solver o devolve com um pop.
    ///
    /// **Mutação que deve sangrar:** ignorar `side_clear`.
    #[test]
    fn a_wall_on_the_escape_side_vetoes_it() {
        let free = ledge_on_the_left(-0.27);
        let d = corner_escape(&free, REACH).expect("controle: sem parede ele livra");

        let walled = CeilingProbe {
            side_clear: [0.0, 0.0],
            ..free
        };
        assert_eq!(
            corner_escape(&walled, REACH),
            None,
            "com parede dos dois lados o escape de {d} nao pode acontecer"
        );

        // E com espaço só de UM lado, ele usa aquele lado.
        let one_side = CeilingProbe {
            side_clear: [REACH, 0.0],
            ..free
        };
        assert_eq!(
            corner_escape(&one_side, REACH),
            None,
            "a beirada esta' a esquerda e o unico espaco livre tambem: nao ha' saida"
        );
    }

    /// **`corner_reach = 0` desliga a assistência inteira** — e o desligar tem de
    /// alcançar as DUAS portas, senão a ponte castaria 25 raios por tique para
    /// alimentar uma lei que nunca age.
    #[test]
    fn zero_reach_switches_the_whole_assist_off() {
        let mut c = cfg();
        c.corner_reach = 0.0;
        assert!(!corner_probe_wanted(&c, false, 5.0), "a ponte nao pergunta");
        let p = ledge_on_the_left(-0.27);
        assert_eq!(corner_escape(&p, 0.0), None, "e a lei nao age");
        assert_eq!(corner_nudge(Some(&p), &c, UP), [0.0, 0.0]);
    }

    /// A porta do sensor: subindo E no ar, e nada mais.
    #[test]
    fn the_probe_is_wanted_only_while_rising_in_the_air() {
        let c = cfg();
        assert!(corner_probe_wanted(&c, false, 3.0));
        assert!(!corner_probe_wanted(&c, false, -3.0), "descendo nao");
        assert!(!corner_probe_wanted(&c, false, 0.0), "parado nao");
        assert!(!corner_probe_wanted(&c, true, 3.0), "no chao nao");
    }

    /// O deslocamento sai no eixo do `up`, não no Y literal.
    ///
    /// **Mutação que deve sangrar:** trocar `perp_cw(up)` por `[1, 0]`.
    #[test]
    fn the_nudge_axis_comes_from_up() {
        let p = ledge_on_the_left(-0.27);
        let flat = corner_nudge(Some(&p), &cfg(), UP);
        assert!(flat[0] > 0.0 && flat[1] == 0.0, "{flat:?}");

        // Mundo girado 90°: o "alto" é +X, e o eixo lateral vira o Y.
        let turned = corner_nudge(Some(&p), &cfg(), [1.0, 0.0]);
        assert!(
            turned[0] == 0.0 && turned[1] != 0.0,
            "o eixo tem de girar com o up: {turned:?}"
        );
    }

    /// Entrada degenerada não move ninguém.
    #[test]
    fn degenerate_input_yields_no_escape() {
        let p = ledge_on_the_left(-0.27);
        assert_eq!(corner_escape(&p, f32::NAN), None);
        assert_eq!(corner_escape(&p, -1.0), None);
        let zero_w = CeilingProbe {
            half_width: 0.0,
            ..p
        };
        assert_eq!(corner_escape(&zero_w, REACH), None);
        assert_eq!(corner_nudge(None, &cfg(), UP), [0.0, 0.0]);
    }
}
