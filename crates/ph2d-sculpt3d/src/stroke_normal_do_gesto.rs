//! **A NORMAL QUE OS GESTOS TANGENCIAIS LEEM** — o plano de que o
//! [`Verb::Thumb`] e o [`Verb::Nudge`] subtraem o puxão.
//!
//! ⚠️⚠️ **Ela NÃO é a normal do [`super::plane`], e as três diferenças são
//! load-bearing** (`docs/3D/cleanroom/SPEC_pull_brushes.md` §5.2):
//!
//! 1. o **raio é uma fracção** do raio do pincel ([`Brush::normal_radius_frac`],
//!    que nasce em `0,5`) — a normal descreve a superfície **sob o miolo**, não
//!    sob a pegada inteira;
//! 2. o peso é a curva **suave fixa** (`3f² − 2f³`), nunca a curva que o artista
//!    escolheu para o pincel — *a forma do dab é do artista; a leitura da
//!    superfície não é*;
//! 3. os vértices caem em **DOIS baldes** — os que olham para a câmara e os que
//!    lhe dão as costas —, e o balde da frente **ganha sempre**.
//!
//! ⚠️ **O terceiro item é o que salva uma peça fina.** Sem os baldes, os dois
//! lados de uma parede fina entram na mesma soma, cancelam-se, e a normal
//! colapsa — o gesto ficaria sem plano de onde subtrair, justamente onde o
//! artista mais usa um polegar.
//!
//! ⚠️ **A pose de leitura é do VERBO**, pela mesma lei do irmão: o gesto
//! ancorado lê a pose do pen-down (ele nunca volta a perguntar à superfície) e o
//! gesto que viaja lê a pose VIVA, reamostrada a cada evento.
//!
//! # ⛔ O que NÃO é observável daqui, e a razão é GEOMETRIA
//!
//! ⚠️⚠️ **Trocar os dois baldes um pelo outro não muda saída nenhuma nestes dois
//! verbos, e isso está MEDIDO** (mutação de 2026-09-13, que sobreviveu à bancada
//! inteira). O motivo não é o corpus: é que o consumidor desta normal é a
//! **componente tangencial**, e ela é **quadrática em `n`** (`Δ − n·(n·Δ)`) —
//! logo é cega ao SINAL. Numa peça fina os dois baldes carregam normais
//! **antiparalelas**, e antiparalelo é exactamente a diferença que um sinal
//! apaga.
//!
//! ⇒ **os baldes ficam porque a lei é essa** (e porque a alternativa — somar
//! tudo num balde só — faz a soma CANCELAR e a normal degenerar numa peça
//! fina), mas quem os quiser gatear precisa de um consumidor que leia a
//! DIRECÇÃO, não o plano. *Uma propriedade que o consumidor não consegue
//! distinguir não se prova com o consumidor.*

use super::*;

/// O peso da amostragem: a curva suave, **fixa**, sobre a fracção do raio.
///
/// ⚠️ **Ela é escrita aqui e não lida do [`Falloff`] do pincel de propósito** —
/// o `Falloff` é a forma que o artista escolheu para o CARIMBO, e trocá-lo faria
/// a leitura da superfície mudar com um knob que não fala sobre ela. Um gate
/// nomeia esta independência.
pub(super) fn peso_da_amostra(d: f32, r: f32) -> f32 {
    let f = 1.0 - d / r;
    (f * f * (3.0 - 2.0 * f)).clamp(0.0, 1.0)
}

impl SculptStroke {
    /// **A normal do gesto**, ou `None` quando não há amostra nenhuma dentro da
    /// fracção do raio (pegada vazia, ou raio fraccionado a zero).
    ///
    /// ⚠️ **O `None` não é um erro a esconder:** quem chama cai na normal do
    /// plano do carimbo, que existe sempre — e é a única resposta finita quando
    /// a superfície não respondeu.
    pub(super) fn normal_do_gesto(
        &self,
        mesh: &Mesh,
        brush: &Brush,
        dab: &Dab,
    ) -> Option<[f32; 3]> {
        let r = dab.radius * brush.normal_radius_frac;
        // ⚠️ **`is_finite` ANTES do sinal, e não um `r <= 0.0` sozinho:** um
        // `NaN` compara falso com tudo, logo `r <= 0.0` deixá-lo-ia passar e a
        // soma sairia envenenada — que é o defeito que o `normalizar` existe
        // para não ter de curar a jusante.
        if !r.is_finite() || r <= 0.0 {
            return None;
        }
        // A pose de leitura: congelada no gesto ancorado, viva no que viaja.
        let congelada = matches!(brush.verb, Verb::Thumb);
        let (mut frente, mut verso) = ([0.0f64; 3], [0.0f64; 3]);
        for &v in &self.footprint {
            let s = self.slot[v as usize] as usize;
            let p = if congelada {
                self.base_pos[s]
            } else {
                mesh.positions()[v as usize]
            };
            let d = {
                let (dx, dy, dz) = (
                    p[0] - dab.center[0],
                    p[1] - dab.center[1],
                    p[2] - dab.center[2],
                );
                (dx * dx + dy * dy + dz * dz).sqrt()
            };
            if d > r {
                continue;
            }
            let n = if congelada {
                self.base_nrm[s]
            } else {
                mesh.normals()[v as usize]
            };
            let peso = f64::from(peso_da_amostra(d, r));
            // ⚠️ **O `<= 0` é o mesmo teste que o [`super::plane`] usa para o
            // conjunto frontal** — o `eye` aponta da câmara para a cena, então
            // um vértice virado para quem olha tem produto negativo. Escrever o
            // teste ao contrário aqui poria o balde do VERSO a ganhar, e a
            // normal sairia invertida numa peça fina: o gesto espalmaria para o
            // lado errado do plano.
            let de_frente = n[0] * dab.eye[0] + n[1] * dab.eye[1] + n[2] * dab.eye[2] <= 0.0;
            let balde = if de_frente { &mut frente } else { &mut verso };
            for k in 0..3 {
                balde[k] += f64::from(n[k]) * peso;
            }
        }
        normalizar(frente).or_else(|| normalizar(verso))
    }
}

/// `None` quando a soma degenera — o mesmo contrato do kernel de área do
/// [`super::plane`], e pela mesma razão: um `NaN` aqui envenena a malha inteira.
fn normalizar(v: [f64; 3]) -> Option<[f32; 3]> {
    let len = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    if len == 0.0 {
        return None;
    }
    let inv = 1.0 / len;
    Some([
        (v[0] * inv) as f32,
        (v[1] * inv) as f32,
        (v[2] * inv) as f32,
    ])
}

/// **A COMPONENTE DO GESTO NO PLANO TANGENTE** — `Δ − n·(n·Δ)`, com `|n| = 1`.
///
/// ⭐ É a única linha que separa o [`Verb::Thumb`] do [`Verb::Move`], e a única
/// que separa o [`Verb::Nudge`] do [`Verb::SnakeHook`]. *Um verbo pode ser uma
/// subtracção.*
#[must_use]
pub fn tangencial(delta: [f32; 3], n: [f32; 3]) -> [f32; 3] {
    let dot = delta[0] * n[0] + delta[1] * n[1] + delta[2] * n[2];
    [
        delta[0] - n[0] * dot,
        delta[1] - n[1] * dot,
        delta[2] - n[2] * dot,
    ]
}

impl SculptStroke {
    /// ⭐⭐⭐⭐ **A DIRECÇÃO DO PUXÃO QUANDO O ARTISTA LIGA O
    /// [`Brush::puxa_pela_normal`]** — a normal do gesto, **congelada no
    /// pen-down** e guardada uma por passe de simetria.
    ///
    /// ⚠️ **Ela é o primeiro consumidor desta normal que lê a DIRECÇÃO**, e
    /// isso fecha uma dívida que o cabeçalho deste ficheiro escreve desde
    /// 2026-09-13: os dois baldes (frente/verso) eram inobserváveis porque o
    /// único consumidor era a componente **tangencial**, que é quadrática em
    /// `n` e cega ao sinal. *Aqui o sinal é a ferramenta inteira: trocá-los põe
    /// o espigão a crescer para DENTRO da peça.*
    ///
    /// # ⛔ Ela é NOSSA, e isso é uma divergência DECLARADA
    ///
    /// Nenhuma das duas referências desta casa oferece isto nestes verbos (o
    /// corpus de paridade deles não tem uma única fixtura com a coluna), logo
    /// **não há lado aprovado a copiar** — o que existe é a medição das três
    /// escolhas abaixo, e elas ficam escritas porque a próxima pessoa vai
    /// perguntar porquê.
    ///
    /// | escolha | porquê, medido |
    /// |---|---|
    /// | a normal é a do **gesto** ([`crate::stroke_normal_do_gesto`]) e não a do estimador de plano | ela lê a superfície **sob o miolo** e tem os dois baldes, logo não colapsa numa parede fina — que é onde um espigão é mais usado |
    /// | ela **CONGELA** no pen-down, uma por passe de simetria | lida viva, cada dab puxaria ao longo da normal que o dab anterior acabou de virar, e o espigão **enrola**; congelada ele sai a direito, e o traço volta a ser facto do gesto e não da taxa de eventos |
    /// | o comprimento é `‖puxão‖`, sempre para FORA | a componente do arrasto ao longo da normal é **zero** exactamente no caso que o dono descreve (a normal a apontar ao artista), logo ela entregaria um controlo inerte onde ele é mais pedido |
    ///
    /// ⚠️ **O sentido para DENTRO fica em aberto e é decisão do dono** — os dois
    /// verbos não honram o `Ctrl` hoje ([`crate::Verb::honours_invert`]), e ele
    /// já ordenou que um gesto novo se arme por **botão no painel** e não por
    /// modificador.
    ///
    ///
    /// ⚠️ **O `None` do [`Self::normal_do_gesto`] cai na normal do plano do
    /// carimbo**, que existe sempre — a mesma queda que os dois gestos
    /// tangenciais já fazem.
    pub(super) fn direccao_do_puxao(
        &mut self,
        mesh: &Mesh,
        brush: &Brush,
        dab: &Dab,
        do_plano: [f32; 3],
    ) -> [f32; 3] {
        let passe = self.passe_simetria;
        if self.normal_do_puxao.len() <= passe {
            self.normal_do_puxao.resize(passe + 1, None);
        }
        if let Some(n) = self.normal_do_puxao[passe] {
            return n;
        }
        let n = self.normal_do_gesto(mesh, brush, dab).unwrap_or(do_plano);
        self.normal_do_puxao[passe] = Some(n);
        n
    }
}
