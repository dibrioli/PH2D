//! **O LÁPIS** — desenhar à mão livre (W1 do plano 25, o pedido do Enio).
//!
//! Até aqui **todo caminho deste app nascia de cliques discretos** (a caneta) ou de um arrasto
//! dimensionado (as shape tools). Este módulo é o gesto que faltava: arrastar e a curva sair.
//!
//! # O desenho, em três frases
//!
//! 1. **A captura é a `ShapeTool`, não um canal novo.** O path nasce na cena no press e é
//!    reescrito a cada move — então o preview ao vivo, o undo de UM passo, a seleção no release
//!    e o render saem **de graça** pelo caminho normal (é o padrão que o `shape.rs` estabeleceu).
//!    Um canal de preview próprio seria uma segunda resposta a *"como se vê o que se está
//!    desenhando"*.
//! 2. **O ajuste é AO VIVO.** O Illustrator e o Inkscape mostram o rabisco cru e trocam por uma
//!    curva no instante em que se solta — e é aí que o artista descobre que a curva não é a que
//!    ele desenhou. Aqui o que se vê **é** o resultado: a cada move o traço é decimado e
//!    reajustado por [`crate::hobby`]. Medido (`measure_pencil_fit`, release): o move mais caro
//!    de um traço de **2000 amostras fica ABAIXO da resolução do relógio (< 0,001 ms)** — o
//!    decimador é a única parte que vê todas as amostras, e o solver só vê os nós (8 a 10 num
//!    traço inteiro). Ajustar no release não compraria nada.
//! 3. **A fidelidade é uma DISTÂNCIA, não uma contagem.** O decimador é Ramer-Douglas-Peucker
//!    com tolerância em **pixels de tela** (convertida em mundo no press): é a mesma grandeza que
//!    o slider *Fidelity* do Illustrator expõe, e é invariante ao zoom — desenhar ampliado dá
//!    mais nós porque o artista de fato controlou mais detalhe.
//!
//! ⚠️ **RDP preserva PICOS, e é isso que o torna certo aqui.** Ele nunca corta um extremo local,
//! então uma quina desenhada de propósito sobrevive a qualquer tolerância. O preço é o simétrico:
//! o tremor da mão também é um extremo local, e nenhuma tolerância o remove — quem o remove é o
//! **estabilizador**, que filtra a amostra na ENTRADA (W1b). São duas perguntas, e ter dois
//! controles é o que impede um deles de mentir.
//!
//! ⚠️ **O `flip_draw` da shell tem um RDP também**, sobre `Vec2` f32 e interpolando pressão para
//! um traço de RASTER. Não é a mesma porta e não pode ser: aquele mora na shell (que esta crate
//! não vê) e serve um pipeline diferente. O que os dois compartilham é a *lei* — que é padrão de
//! livro e está escrita nos dois lugares.

use crate::PenStyle;
use crate::pencil_width::{PenDynamics, WidthSource, width_stops};
use ph2d_vec_scene::{VecPath, VecPathId, VecScene, VecVertex, WidthStops};

/// Tolerância do decimador, em **pixels de tela**, quando ninguém escolhe (o *Fidelity*
/// default).
///
/// MEDIDO (`measure_pencil_fidelity`, sobre um S de 120 px de largura, 480 amostras, ±0,4 px de
/// tremor sintético) — nós resultantes e desvio máximo da curva IDEAL (não da mão, que treme):
///
/// | fidelity | nós | desvio máx |
/// |---|---|---|
/// | 0,5 px | 32 | 1,04 px |
/// | 1 px | 13 | 0,77 px |
/// | **2 px** | **9** | **0,77 px** |
/// | 4 px | 8 | 1,06 px |
/// | 8 px | 4 | 3,55 px |
///
/// ⚠️ **O desvio NÃO cresce com a tolerância até 4 px, e é isso que escolhe o default:** entre
/// 0,5 e 2 px o que o decimador remove é o TREMOR (o desvio até melhora, de 1,04 para 0,77),
/// porque as amostras que ele corta são as que se afastam da curva. Só a 8 px ele começa a comer
/// a FORMA (3,55 px). O joelho é 2 px, e é onde nove nós descrevem um S inteiro — uma contagem
/// que caberia na mão de quem for editá-los.
pub const DEFAULT_FIDELITY_PX: f64 = 2.0;

/// Passo mínimo entre amostras aceitas, em **pixels de tela**. Abaixo disto o move é ignorado:
/// um cursor parado num pixel geraria centenas de amostras coincidentes, e nós coincidentes são
/// o caso degenerado que o solver tem de defender-se de ([`crate::hobby`]).
const MIN_SAMPLE_PX: f64 = 1.5;

/// Comprimento total mínimo do gesto, em **pixels de tela**, para o traço ser commitado. Abaixo
/// disto foi um clique perdido — o mesmo limiar de intenção que a `ShapeTool` aplica ao arrasto.
const MIN_STROKE_PX: f64 = 3.0;

/// O lápis: as amostras do gesto em curso + o path vivo que as mostra.
#[derive(Default)]
pub struct Pencil {
    /// Estilo do traço (a shell o sincroniza da tool a cada frame, como na `ShapeTool`).
    style: PenStyle,
    /// O path vivo entre press e release; `None` = ocioso.
    active: Option<VecPathId>,
    /// O último path commitado — a shell o seleciona no release.
    committed: Option<VecPathId>,
    /// As amostras CRUAS em mundo, na ordem em que a mão as deixou.
    samples: Vec<[f64; 2]>,
    /// O que cada amostra carregava além da posição — a pressão do dispositivo e o carimbo de
    /// tempo. Anda LADO A LADO com [`Self::samples`] (mesmo comprimento, sempre): é o que a
    /// [`crate::pencil_width`] lê para derivar a largura, e um descompasso ali devolve um perfil
    /// vazio em vez de adivinhar.
    dyns: Vec<PenDynamics>,
    /// Unidades de mundo por pixel de tela, capturadas no press. Congelada no gesto de
    /// propósito: dar zoom no meio de um traço não pode mudar a densidade de nós do começo dele.
    px_to_world: f64,
    /// Tolerância do decimador em pixels de tela (o *Fidelity*).
    fidelity_px: f64,
}

impl Pencil {
    /// Espelha o estilo da tool (traço/preenchimento correntes). A shell chama a cada frame.
    pub fn set_style(&mut self, style: PenStyle) {
        self.style = style;
    }

    /// Ajusta a **fidelidade** (tolerância do decimador, em px de tela). Vale do próximo move em
    /// diante — o traço em curso re-ajusta na hora, que é o que faz o slider ser legível.
    pub fn set_fidelity_px(&mut self, px: f64) {
        self.fidelity_px = px.max(0.0);
    }

    /// Há um traço em curso?
    #[must_use]
    pub fn is_active(&self) -> bool {
        self.active.is_some()
    }

    /// O path VIVO do gesto — o irmão exato do `ShapeTool::active_path`.
    ///
    /// ⚠️ Quem pergunta isto é o assentamento de origens (`settle_origins`): a geometria deste
    /// path é reescrita em **MUNDO** a cada move, então assentá-lo no meio do gesto soma
    /// geometria + `Transform` e a tinta sai deslocada do cursor **pelo ponto onde o arrasto
    /// começou** (medido: 1,5897 unidades de mundo ≈ 353 px). Um gesto que escreve mundo por
    /// frame tem de ser ANUNCIÁVEL, senão o sistema que o pula não o enxerga.
    #[must_use]
    pub fn active_path(&self) -> Option<VecPathId> {
        self.active
    }

    /// O último traço commitado (para a shell selecioná-lo).
    #[must_use]
    pub fn selected(&self) -> Option<VecPathId> {
        self.committed
    }

    /// **Press** — abre o traço com a 1ª amostra e põe o path vivo na cena. Devolve o id dele.
    pub fn on_press(
        &mut self,
        scene: &mut VecScene,
        p: [f64; 2],
        px_to_world: f64,
        dyn_in: PenDynamics,
    ) -> VecPathId {
        // Um gesto que não foi fechado (não deveria acontecer) não deixa lixo na cena.
        if let Some(id) = self.active.take() {
            scene.remove_path(id);
        }
        self.samples.clear();
        self.dyns.clear();
        self.samples.push(p);
        self.dyns.push(dyn_in);
        self.px_to_world = px_to_world.max(f64::MIN_POSITIVE);
        if self.fidelity_px <= 0.0 {
            self.fidelity_px = DEFAULT_FIDELITY_PX;
        }
        let id = scene.push_path(self.build());
        self.active = Some(id);
        id
    }

    /// **Move** — aceita a amostra se ela andou o passo mínimo e reescreve o traço vivo.
    /// Devolve `true` se a cena mudou (a shell não precisa repintar de graça).
    pub fn on_drag(&mut self, scene: &mut VecScene, p: [f64; 2], dyn_in: PenDynamics) -> bool {
        if self.active.is_none() {
            return false;
        }
        let Some(&last) = self.samples.last() else {
            return false;
        };
        if dist(last, p) < MIN_SAMPLE_PX * self.px_to_world {
            return false;
        }
        self.samples.push(p);
        self.dyns.push(dyn_in);
        self.rebuild(scene)
    }

    /// **Release** — fecha o gesto. `true` se o traço foi commitado; `false` se o gesto foi curto
    /// demais para ser um traço (e aí o path vivo sai da cena e a shell cancela o undo pendente).
    pub fn on_release(&mut self, scene: &mut VecScene) -> bool {
        let Some(id) = self.active.take() else {
            return false;
        };
        if self.travel() < MIN_STROKE_PX * self.px_to_world || self.samples.len() < 2 {
            scene.remove_path(id);
            self.samples.clear();
            self.dyns.clear();
            return false;
        }
        self.committed = Some(id);
        self.samples.clear();
        self.dyns.clear();
        true
    }

    /// **Aborta** o traço em curso (o botão direito, o Esc) — o path vivo some sem deixar rastro.
    pub fn cancel(&mut self, scene: &mut VecScene) {
        if let Some(id) = self.active.take() {
            scene.remove_path(id);
        }
        self.samples.clear();
        self.dyns.clear();
    }

    /// **O perfil de largura que este gesto pede**, na fonte dada (W1d).
    ///
    /// Lê as amostras VIVAS, então serve o preview ao vivo — a shell o pergunta a cada move e
    /// pendura o resultado na forma. Depois do release as amostras já saíram, e a resposta é
    /// vazia: quem guarda o perfil daí em diante é o componente que a shell armou no último move.
    ///
    /// ⚠️ **Nada aqui toca a cena.** O lápis produz a CURVA; a espessura dela é uma relação que
    /// vive no componente (ADR-0148), e misturar as duas aqui poria geometria de fita dentro do
    /// caminho autorado — exatamente o que o preview vivo existe para não fazer.
    #[must_use]
    pub fn width_stops(&self, source: WidthSource) -> WidthStops {
        width_stops(source, &self.samples, &self.dyns)
    }

    /// As amostras cruas do gesto — para sondas e gates (o preview ao vivo é o path na cena).
    #[must_use]
    pub fn samples(&self) -> &[[f64; 2]] {
        &self.samples
    }

    /// O comprimento percorrido pelo gesto, em mundo.
    fn travel(&self) -> f64 {
        self.samples
            .windows(2)
            .map(|w| dist(w[0], w[1]))
            .sum::<f64>()
    }

    /// Reescreve a geometria do path vivo. `true` se ele existe.
    fn rebuild(&self, scene: &mut VecScene) -> bool {
        let Some(id) = self.active else {
            return false;
        };
        let path = self.build();
        let Some(live) = scene.path_mut(id) else {
            return false;
        };
        live.verts = path.verts;
        true
    }

    /// O path do traço: decima as amostras e ajusta a spline que passa por elas.
    fn build(&self) -> VecPath {
        VecPath {
            verts: self.fit(),
            closed: false,
            // O lápis desenha uma LINHA: preenchê-la fecharia visualmente um traço aberto.
            // Quem quiser região fecha o caminho depois (o Pen já tem esse gesto).
            fill: None,
            // A largura é autorada em px de tela e a geometria só fala mundo — a conversão usa o
            // `px_to_world` congelado no press, como a `ShapeTool`.
            stroke: Some(
                self.style
                    .stroke_spec(self.style.stroke_w_px * self.px_to_world),
            ),
            ..VecPath::default()
        }
    }

    /// As amostras → nós → vértices da spline. Uma amostra só devolve um vértice de quina (o
    /// path existe desde o press, senão o traço só apareceria no 2º move).
    fn fit(&self) -> Vec<VecVertex> {
        match self.samples.len() {
            0 => Vec::new(),
            1 => vec![VecVertex::corner(self.samples[0])],
            _ => {
                let knots = decimate(&self.samples, self.fidelity_px * self.px_to_world);
                crate::hobby::fit_hobby_open(&knots)
            }
        }
    }
}

/// **Ramer-Douglas-Peucker**: mantém as amostras cujo desvio perpendicular à corda passa de
/// `tol` (em MUNDO), descarta o resto. As duas pontas sobrevivem sempre.
///
/// Iterativo com pilha explícita, e não recursivo: um traço longo e ruidoso divide fundo, e o
/// caminho de escrita do editor não é lugar de descobrir a profundidade de pilha do usuário.
#[must_use]
pub fn decimate(samples: &[[f64; 2]], tol: f64) -> Vec<[f64; 2]> {
    if samples.len() < 3 {
        return samples.to_vec();
    }
    let mut keep = vec![false; samples.len()];
    keep[0] = true;
    keep[samples.len() - 1] = true;
    let tol2 = (tol.max(0.0)) * (tol.max(0.0));
    let mut stack = vec![(0usize, samples.len() - 1)];
    while let Some((a, b)) = stack.pop() {
        if b <= a + 1 {
            continue;
        }
        let mut worst = 0.0_f64;
        let mut at = a;
        for i in (a + 1)..b {
            let d = dist2_to_segment(samples[i], samples[a], samples[b]);
            if d > worst {
                worst = d;
                at = i;
            }
        }
        // `>` e não `>=`: com tolerância 0 um traço perfeitamente reto não ganha nós no meio.
        if worst > tol2 && at > a {
            keep[at] = true;
            stack.push((a, at));
            stack.push((at, b));
        }
    }
    samples
        .iter()
        .zip(keep)
        .filter_map(|(p, k)| k.then_some(*p))
        .collect()
}

/// Distância AO QUADRADO de `p` ao segmento `a..b` (ao quadrado para não tirar raiz por
/// amostra; a comparação com a tolerância é feita ao quadrado também).
fn dist2_to_segment(p: [f64; 2], a: [f64; 2], b: [f64; 2]) -> f64 {
    let (vx, vy) = (b[0] - a[0], b[1] - a[1]);
    let (wx, wy) = (p[0] - a[0], p[1] - a[1]);
    let vv = vx * vx + vy * vy;
    if vv <= f64::MIN_POSITIVE {
        return wx * wx + wy * wy;
    }
    // Projeta e SEGURA no segmento: fora dele a distância é à ponta, não à reta infinita — sem
    // o clamp, uma amostra que volta sobre o traço parece perto de uma corda que ela não toca.
    let t = ((wx * vx + wy * vy) / vv).clamp(0.0, 1.0);
    let (dx, dy) = (wx - t * vx, wy - t * vy);
    dx * dx + dy * dy
}

fn dist(a: [f64; 2], b: [f64; 2]) -> f64 {
    let (dx, dy) = (a[0] - b[0], a[1] - b[1]);
    (dx * dx + dy * dy).sqrt()
}

#[cfg(test)]
#[path = "pencil_tests.rs"]
mod tests;
