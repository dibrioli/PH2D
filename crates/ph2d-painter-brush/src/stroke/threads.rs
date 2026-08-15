//! **OS FIOS — o traço que se costura a si mesmo** (plano 38 W3 e W4). Guarda-se a memória do traço
//! e, a cada dab novo, ligam-se pontos dela com fios finos de opacidade baixa.
//!
//! **DOIS tipos, UM produtor.** O que muda entre eles é só a pergunta que escolhe os pares:
//!
//! - **[`LineKind::Sketchy`]** (o *neighbour points* do Ze Frank → Harmony → Krita *Sketch*)
//!   pergunta *quem está perto no CANVAS* — sai o hachurado de lápis que ninguém imita à mão.
//! - **[`LineKind::Wire`]** (o *Curve brush engine* do Krita) pergunta *quem está perto no
//!   PERCURSO* — sai o arame/laço, porque numa curva a corda corta a quina.
//!
//! ⚠️ **As duas perguntas coincidem num traço que não volta sobre si mesmo e divergem exatamente
//! onde ele volta** — a mesma distinção que o `Magnetify` liga e desliga dentro do Sketchy. É por
//! isso que o W4 não trouxe geometria nova: se tivesse trazido, o W3 estaria errado.
//!
//! ⚠️ **Um fio NÃO é uma corrente de dabs, e a W0.3 mediu por quê:** a densidade 1,0 põe **16 mil px
//! de fio** num traço de 312 px, e um fio fino carimbado à spacing de sub-pixel poria a conta em
//! 10⁴–10⁶ dabs. Ele é um **segmento**, publicado por um canal próprio ([`Stroke::take_threads`]) e
//! rasterizado por área exata — que é como o Harmony e o Krita o desenham.
//!
//! ⚠️ **A `Density` é o ORÇAMENTO, não um enfeite** (W0.3): a um diâmetro de alcance os fios somam
//! **~50× o arco do traço**; a quatro, **~700×**. O teto sai DERIVADO daí, não escolhido — ver
//! [`crate::line_kind::SKETCHY_DENSITY_MAX`].

use super::{Dab, Stroke};
use crate::line_kind::LineKind;

/// **A MEMÓRIA que os fios consomem** — ela mora aqui, e não no [`Stroke`], porque é o produtor que
/// sabe o que cada campo significa e o que o mantém correto.
///
/// ⚠️ **Só é populada com um tipo que costura armado:** um `Vec` que cresce em todo traço do app
/// seria memória paga por quem nunca escolheu o tipo.
#[derive(Debug, Clone, Default)]
pub struct ThreadMemory {
    /// A vizinhança: `[x, y, arco]` — os dois primeiros em px de canvas e o terceiro o
    /// [`Dab::arc_len`] daquele centro.
    ///
    /// ⚠️ **O arco não é enfeite: ele é o que separa as duas perguntas da família.** *Perto no
    /// canvas* e *perto no percurso* são coisas diferentes — o Krita chama a primeira de Magnetify
    /// ligado (dois trechos que se aproximaram) e a segunda de desligado (a porção ativa), e o
    /// [`LineKind::Wire`] é a segunda como tipo próprio. Sem o arco ao lado do ponto, nenhuma das
    /// duas é exprimível.
    pts: Vec<[f32; 3]>,
    /// Os fios costurados desde o último [`Stroke::take_threads`].
    out: Vec<Thread>,
    /// ⚠️ **Fluxo de RNG PRÓPRIO.** Sortear os fios do `rng` do traço adiantaria o mesmo fluxo que o
    /// Jitter (posição, escala, rotação, cor) consome — e então **ligar o Sketchy mudaria o jitter
    /// de um pincel**, em silêncio. Dois consumidores, dois fluxos.
    rng: u64,
}

impl ThreadMemory {
    /// Nasce vazia, com o fluxo de sorteio desviado do `seed` do traço.
    pub(super) fn new(seed: u64) -> Self {
        Self {
            pts: Vec::new(),
            out: Vec::new(),
            rng: seed ^ 0x2545_F491_4F6C_DD1D,
        }
    }

    /// Esvazia — é isto que impede um fio de ligar dois TRAÇOS.
    pub(super) fn clear(&mut self) {
        self.pts.clear();
        self.out.clear();
    }
}

/// Um fio: o segmento `a → b` em pixels de canvas. Sem largura nem cor — quem os carrega é o
/// [`crate::BrushSpec`], porque um fio é uma linha do MESMO traço, não um objeto próprio.
pub type Thread = [f32; 4];

impl Stroke {
    /// **A PORTA por onde um fio é publicado.** Ela existe porque a Symmetry tem de ser aplicada
    /// aqui e em nenhum outro lugar: três produtores (Sketchy, Wire e as travessas da FITA) chegam
    /// a este canal, e uma cópia do `push_symmetric_segment` por produtor é a regra que o quarto
    /// nasce sem — o feixe sairia sem espelho num tipo e com espelho nos outros, e nada no
    /// compilador ligaria as duas metades.
    ///
    /// ⚠️ É também o que mantém a `out` PRIVADA: quem produz um fio diz *este segmento é um fio*,
    /// nunca *empurre isto no vetor*.
    pub(super) fn sew(&mut self, seg: Thread) {
        crate::symmetry::push_symmetric_segment(&mut self.threads.out, seg, &self.spec.symmetry);
    }

    /// Registra o centro de um dab na memória do traço e costura os fios que ele fecha.
    ///
    /// ⚠️ **A vizinhança é do TRAÇO, nunca do canvas:** a memória nasce vazia em cada
    /// [`Stroke::begin`], então um traço novo jamais liga um fio ao anterior. Sem isso o Sketchy
    /// costuraria o desenho inteiro à medida que o artista trabalha.
    pub(super) fn note_thread_point(&mut self, dab: &Dab) {
        if !self.spec.sews_threads() {
            return;
        }
        let p = dab.center;
        match self.spec.line_kind {
            LineKind::Sketchy => self.sew_neighbours(p, dab),
            LineKind::Wire => self.sew_history(p, dab),
            // ⚠️ **A FITA costura fios e NÃO lê esta memória** — as travessas dela ligam os DOIS
            // trilhos no MESMO instante, e quem sabe onde os dois estavam é o tique
            // ([`Stroke::sew_rungs`]), não o dab. Cair no braço vazio e empurrar o ponto faria o
            // `pts` crescer o traço inteiro para ninguém o ler; sair aqui é o que mantém o custo da
            // memória com quem de facto a consome.
            _ => return,
        }
        // ⚠️ O ponto entra na memória DEPOIS de costurar — senão o dab novo seria vizinho de si
        // mesmo, e o par degenerado (`a == b`) pintaria nada consumindo um fio do orçamento.
        self.threads.pts.push([p[0], p[1], dab.arc_len]);
    }

    /// **SKETCHY** — quem está perto no CANVAS.
    fn sew_neighbours(&mut self, p: [f32; 2], dab: &Dab) {
        // ⚠️ Pela porta única — o alcance da vizinhança sai de UM lugar (ver o doc dela).
        let reach = self.spec.sketchy_reach_px();
        let density = self.spec.sketchy_density.clamp(0.0, 1.0);
        // **MAGNETIFY — ele escolhe QUE PARES viram fio, nunca com que força um fio desenha.**
        //
        // ⚠️ A lei é a do manual do Krita, verbatim: *"It's what causes curve lines to form between
        // two close line sections … With Magnetify off, the curve line just forms on either side of
        // the current active portion of your connection line."* Ligado, todo par dentro do alcance
        // costura — inclusive dois trechos que o traço aproximou depois de dar uma volta inteira.
        // Desligado, só a **porção ativa** do traço: os pontos que o dedo acabou de deixar.
        //
        // ⚠️ **E a régua de "porção ativa" é o ARCO, não a contagem** — a distinção já estava escrita
        // logo abaixo, como o motivo de a varredura não poder cortar por arco: *o arco entre dois
        // pontos limita por baixo a distância entre eles, então passado `reach` de arco nenhum ponto
        // mais antigo está dentro do raio EXCETO quando o traço volta sobre si mesmo*. Voltar sobre
        // si mesmo é exatamente o que o Magnetify liga e desliga.
        let magnetify = self.spec.sketchy_magnetify;
        if reach > 0.0 && density > 0.0 {
            let r2 = reach * reach;
            // ⚠️ **A varredura tem TETO de CONSULTAS, não de arco** — é ele que mantém o custo linear
            // no traço em vez de quadrático, e o número é MEDIDO ([`SKETCHY_SCAN_CAP`]). Cortar por
            // arco aqui seria escolher o Magnetify OFF para todo mundo.
            let n = self.threads.pts.len();
            let from = n.saturating_sub(crate::line_kind::SKETCHY_SCAN_CAP);
            for i in from..n {
                let q = self.threads.pts[i];
                // Sem Magnetify o candidato tem de estar na porção ATIVA do percurso. O corte é
                // estritamente mais apertado que o do raio (arco ≥ corda), então o teste de distância
                // abaixo segue governando o alcance nos dois modos.
                if !magnetify && (dab.arc_len - q[2]) > reach {
                    continue;
                }
                let d2 = (q[0] - p[0]).powi(2) + (q[1] - p[1]).powi(2);
                if d2 > r2 {
                    continue;
                }
                // Sorteio determinístico (splitmix64 do próprio traço — HR-5, replays reproduzíveis).
                if crate::jitter::next_f32(&mut self.threads.rng) > density {
                    continue;
                }
                self.sew([p[0], p[1], q[0], q[1]]);
            }
        }
    }

    /// **WIRE** — quem está perto no PERCURSO.
    ///
    /// ⚠️ **A janela é de ARCO e a contagem é FIXA, e as duas metades são a mesma decisão:** *"ligue
    /// aos últimos N"* lido ao pé da letra são `N` fios por dab, com `N` = quantos dabs couberam na
    /// janela — ou seja, o desenho E o custo passariam a depender do **Spacing**. Amostrando
    /// [`WIRE_CURVES_PER_DAB`] posições igualmente espaçadas EM ARCO, as duas coisas ficam livres do
    /// espaçamento: o mesmo gesto desenha o mesmo arame com o dobro dos dabs.
    ///
    /// ⚠️ **A busca é binária porque a memória JÁ está ordenada por arco** — o arco de um traço é
    /// monotônico por construção —, então o Wire custa `O(log n)` por corda em vez de percorrer a
    /// janela. Ele não tem, e não precisa de, o teto de consultas do irmão.
    ///
    /// ⚠️ **No começo do traço a janela não está cheia, e as cordas que caem antes do 1º ponto são
    /// PULADAS** — não presas ao ponto inicial. É o comportamento que o manual do Krita descreve
    /// como consequência (*"if you set this value too high, the curve lines will only start forming
    /// relatively late"*), e prendê-las no começo desenharia um leque saindo da origem do traço.
    fn sew_history(&mut self, p: [f32; 2], dab: &Dab) {
        let window = self.spec.wire_history_px();
        if window <= 0.0 || self.threads.pts.is_empty() {
            return;
        }
        let oldest = self.threads.pts[0][2];
        #[allow(clippy::cast_precision_loss)]
        let n = crate::line_kind::WIRE_CURVES_PER_DAB as f32;
        for k in 1..=crate::line_kind::WIRE_CURVES_PER_DAB {
            #[allow(clippy::cast_precision_loss)]
            let back = window * (k as f32) / n;
            let want = dab.arc_len - back;
            if want < oldest {
                // A janela ainda não encheu até aqui — e as seguintes estão mais longe.
                break;
            }
            // O primeiro ponto cujo arco alcança o alvo (a memória é crescente em arco).
            let i = self.threads.pts.partition_point(|q| q[2] < want);
            let Some(q) = self.threads.pts.get(i) else {
                continue;
            };
            self.sew([p[0], p[1], q[0], q[1]]);
        }
    }

    /// **Drena os fios costurados desde a última chamada.** Canal PRÓPRIO, e não um `Dab` a mais:
    /// um fio tem largura e opacidade suas e é rasterizado como segmento, não carimbado.
    ///
    /// ⚠️ Quem chama é o dono do canvas, no MESMO ponto em que consome os dabs — os dois saem do
    /// mesmo gesto e têm de pousar na mesma região suja.
    pub fn take_threads(&mut self, out: &mut Vec<Thread>) {
        out.clear();
        out.append(&mut self.threads.out);
    }
}
