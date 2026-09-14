//! ⭐⭐ **O CAMPO DE DESLOCAMENTO DO ESFREGÃO, e a média que o faz VIAJAR** —
//! a lei do [`Verb::SmearMultires`] (`SPEC_unblocked_brushes.md` §5.2).
//!
//! ⚠️ **Este ficheiro existe pela MESMA razão que o [`super::stroke_hc`]:** a
//! lei não cabe numa função pura por-vértice. Ela lê o campo `D` dos VIZINHOS,
//! e depois de um vértice ser movido o `D` dele já não é recuperável da
//! posição. ⇒ o campo é preenchido numa passada de preparação (irmã do
//! `fit_plane`, do `alpha_frame` e do `fill_hc_disp`, que também correm uma vez
//! por dab) e o alvo sai inteiro de um passe só.
//!
//! # A lei, em duas linhas
//!
//! ```text
//! D[u]  = p[u] − R[u]                          // nos nós TOCADOS, por dab
//! D′[v] = ( D[v] + Σ_w g(w)·D[w] ) / (1 + Σ_w g(w))
//!         com  g(w) = max(0, −(d̂ · ê_w))
//! ```
//!
//! — `ê_w` é a direcção de `R[v]` para `R[w]` **na superfície de referência**,
//! e `d̂` é a do [`crate::SmearMode`].
//!
//! ⭐ **Três coisas que a leitura rápida inverte:**
//!
//! 1. `g` é a parte **NEGATIVA** do cosseno ⇒ só os vizinhos **a montante**
//!    contribuem. É isso que transporta o relevo em vez de o borrar.
//! 2. A vizinhança é medida na **referência**, nunca nas posições deslocadas —
//!    é por isso que esfregar repetidamente não deforma a topologia.
//! 3. O peso próprio é **`1` FIXO**, fora da normalização dos outros. É o
//!    travão que impede a vizinhança de dominar a média por mais vizinhos que
//!    haja a montante.
//!
//! ⚠️ **A ORLA COME DESLOCAMENTO, e é um artefacto TOLERADO** (espec §5.4): o
//! campo só é posto em dia nos nós tocados e a média lê vizinhos que podem
//! estar fora deles — esses entram com o valor que o campo tinha, **zero** no
//! primeiro dab. ⭐ O zero é a cura **publicada** de uma regressão em que
//! vizinhos sem valor definido propagavam `NaN` pela malha; o que a cura NÃO
//! fez foi alargar a actualização à vizinhança. *Reproduzimo-la: uma fixture
//! que a contenha não é «o nosso defeito» (espec §8.3).*

use super::*;

/// **QUE FRACÇÃO DO RAIO o relevo percorre por dab** — ver
/// [`SculptStroke::passagens_do_dab`].
///
/// ⭐ **Ela é a densidade do ORÁCULO**, não um número escolhido: as fixturas da
/// espec §7 correm com `6 146` vértices e raio `0,35` ⇒ `8,3` arestas por raio,
/// e `1/8,3 = 0,12`. Ali a lei devolve **uma** passagem e a saída é a de hoje ao
/// bit.
const FRACCAO_DO_RAIO: f32 = 0.12;

/// **O TECTO DE PASSAGENS, e o recurso dele é o RELÓGIO DO DAB.**
///
/// Cada passagem é uma varredura da pegada vezes a valência. Medido na peça de
/// omissão da escultura (`98 306` vértices, raio `0,35` ⇒ `~3 400` vértices na
/// pegada), com o orçamento de `8 ms` que este módulo já usa como *kill* de dab
/// — a tabela vive no gate `mede_o_esfregao`.
const MAX_PASSAGENS: usize = 24;

/// **Quantos vértices da pegada entram na média da aresta.** ⚠️ Uma AMOSTRA e
/// não a pegada inteira: a resposta é uma escala, e uma escala não precisa de
/// todos os pontos — mas precisa de vários, porque um vértice só cairia num
/// pólo ou numa lasca.
const AMOSTRAS_DA_ARESTA: usize = 64;

impl SculptStroke {
    /// **PREENCHE o `D` da pegada deste dab** — o passo 3 da espec §5.2, e um
    /// no-op para os outros trinta verbos.
    ///
    /// ⚠️ **O buffer só é ALOCADO por quem o usa**, como o `hc_b`: a `resize`
    /// com zeros cresce com o `touched` **e** dá `[0,0,0]` a quem nunca foi
    /// processado — que é exactamente o *«campo cujo valor inicial é zero em
    /// todo vértice»* da espec, e a metade que produz a orla do §5.4.
    ///
    /// ⚠️ **O `needed` é do CHAMADOR**, pela lição que o irmão já pagou: ele
    /// perguntava `brush.verb != …` e ficou errado no dia em que a mesma lei
    /// passou a ser alcançável por outra porta. Aqui a pergunta é *«o verbo em
    /// mãos lê isto?»*, e quem sabe é quem chama.
    ///
    /// ⚠️ **Sem referência fotografada ele não escreve NADA** — o `get` devolve
    /// `None` e o campo fica a zeros. É a mesma rede do alvo: *a lei não
    /// inventa uma superfície*, e um traço sem pilha fica inerte em vez de
    /// esfregar contra uma referência fabricada.
    pub(super) fn fill_smear_d(&mut self, mesh: &Mesh, brush: &Brush, dab: &Dab, needed: bool) {
        if !needed {
            return;
        }
        self.smear_d.resize(self.touched.len(), [0.0; 3]);
        // Índice e não `for &v in &self.footprint`: o laço escreve em
        // `self.smear_d`, e o empréstimo imutável da lista fecharia a porta.
        for i in 0..self.footprint.len() {
            let v = self.footprint[i] as usize;
            let Some(&r) = self.reference.get(v) else {
                continue;
            };
            let p = mesh.positions()[v];
            let s = self.slot[v] as usize;
            self.smear_d[s] = [p[0] - r[0], p[1] - r[1], p[2] - r[2]];
        }
        // ⭐⭐⭐ **AS PASSAGENS QUE TIRAM A LEI DA GRELHA** — ver
        // [`passagens_do_dab`]. A média do anel transporta UMA aresta; `n − 1`
        // relaxações aqui mais a do alvo transportam `n`, e `n` é escolhido
        // para que a distância percorrida seja uma fracção do RAIO.
        let n = self.passagens_do_dab(mesh, dab);
        for _ in 1..n {
            self.relaxa_smear_d(mesh, brush, dab);
        }
    }

    /// ⭐⭐⭐ **QUANTAS PASSAGENS ESTE DAB CORRE** — a lei que tira o pincel da
    /// dependência da GRELHA.
    ///
    /// ⛔⛔ **O report do dono (2026-09-14): *«a intensidade parece baixa mesmo
    /// no máximo»*, e ele tinha razão com número.** A média do anel transporta
    /// o relevo **UMA ARESTA** por dab — medido, `0,15 ×` a aresta média —,
    /// logo o efeito é **inversamente proporcional à densidade da malha**:
    ///
    /// | vértices | aresta | raio em arestas | 1.º dab | 12 dabs |
    /// |---|---|---|---|---|
    /// | `98 306` | `0,01154` | `30,3` | `0,00175` | `0,0297` |
    /// | `393 218` | `0,00577` | `60,6` | `0,00088` | `0,0149` |
    ///
    /// *Dobrar a densidade corta o efeito ao meio, exactamente.* Contra uma
    /// bossa de `0,200`, doze dabs movem `15 %` dela na peça de omissão e
    /// `7,5 %` na subdividida — e na peça do roteiro (`1,5 M`) `~3,7 %`, que é
    /// invisível. ⚠️ É a doença que esta casa já nomeou três vezes: *um efeito
    /// que segue a TESSELAÇÃO em vez da geometria*.
    ///
    /// ⇒ `n − 1` relaxações na preparação, mais a do alvo, transportam `n`
    /// arestas. Com `n` escolhido para que a distância seja uma **fracção do
    /// RAIO**, o pincel deixa de saber quantos vértices a peça tem.
    ///
    /// ⭐⭐ **A FRACÇÃO NÃO É ESCOLHIDA — ela é a densidade do próprio oráculo.**
    /// As fixturas da espec correm com `6 146` vértices e raio `0,35`, ou seja
    /// **`8,3` arestas por raio**; ali esta lei devolve `n = 1` e a nossa saída
    /// é a de hoje **ao bit**. ⇒ ela **não toca** no regime onde a paridade foi
    /// medida, e corrige só o que as fixturas nunca cobriram. *Uma correcção
    /// calibrada no lado aprovado não pode contradizê-lo.*
    ///
    /// ⚠️ **O TECTO nomeia o recurso, e é o RELÓGIO DO DAB:** cada passagem
    /// custa uma varredura da pegada vezes a valência, e o orçamento é os `8 ms`
    /// que este módulo já usa como *kill* de dab. A tabela está em
    /// [`MAX_PASSAGENS`].
    ///
    /// ⚠️ **A aresta é medida NA PEGADA e não na malha inteira:** o pincel está
    /// onde está, e uma peça com densidade variável daria uma média global que
    /// não descreve sítio nenhum. O custo é uma varredura de anel sobre uma
    /// AMOSTRA — ver [`AMOSTRAS_DA_ARESTA`].
    fn passagens_do_dab(&self, mesh: &Mesh, dab: &Dab) -> usize {
        let aresta = self.aresta_da_pegada(mesh);
        if aresta <= 0.0 || !aresta.is_finite() {
            return 1;
        }
        let n = (FRACCAO_DO_RAIO * dab.radius / aresta).round();
        if !n.is_finite() {
            return 1;
        }
        (n as usize).clamp(1, MAX_PASSAGENS)
    }

    /// **A ARESTA MÉDIA SOB O PINCEL** — a unidade em que a lei do anel
    /// trabalha.
    fn aresta_da_pegada(&self, mesh: &Mesh) -> f32 {
        let n = self.footprint.len();
        if n == 0 {
            return 0.0;
        }
        let passo = (n / AMOSTRAS_DA_ARESTA).max(1);
        let mut soma = 0.0f64;
        let mut contadas = 0u32;
        for i in (0..n).step_by(passo) {
            let v = self.footprint[i] as usize;
            let p = mesh.positions()[v];
            for &nb in mesh.adjacency().vert_verts.neighbours(v) {
                let q = mesh.positions()[nb as usize];
                let d = [q[0] - p[0], q[1] - p[1], q[2] - p[2]];
                soma += f64::from((d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt());
                contadas += 1;
            }
        }
        if contadas == 0 {
            return 0.0;
        }
        (soma / f64::from(contadas)) as f32
    }

    /// **UMA passagem de relaxação do campo `D`**, sobre a pegada.
    ///
    /// ⚠️ **O buffer é DUPLO**, e tem de ser: a média lê o `D` dos vizinhos, e
    /// escrever no sítio a meio do laço faria metade dos vértices ler o valor
    /// novo e metade o velho — um Gauss-Seidel dependente da ORDEM da pegada,
    /// que é a mesma doença que o `stroke_hc` documenta ter evitado.
    fn relaxa_smear_d(&mut self, mesh: &Mesh, brush: &Brush, dab: &Dab) {
        self.smear_scratch.clear();
        self.smear_scratch.reserve(self.footprint.len());
        for i in 0..self.footprint.len() {
            let v = self.footprint[i];
            let s = self.slot[v as usize] as usize;
            self.smear_scratch
                .push(self.media_do_anel(mesh, brush, dab, v, s));
        }
        for i in 0..self.footprint.len() {
            let s = self.slot[self.footprint[i] as usize] as usize;
            self.smear_d[s] = self.smear_scratch[i];
        }
    }

    /// **O `D` deste vizinho** — o dele se o traço já o tocou, **zero** se não.
    ///
    /// ⚠️ **Zero e não «pular»**, e a diferença é o DIVISOR: pular mudaria a
    /// média para `Σg·D / Σg` sobre a metade de dentro da pegada, e o
    /// deslocamento saltaria na fronteira do pincel. A espec escreve o campo
    /// como zerado no início do traço, e um vizinho nunca tocado tem zero
    /// **como valor**, não como ausência.
    fn smear_neighbour(&self, nb: u32) -> [f32; 3] {
        let ni = nb as usize;
        if self.stamp[ni] == self.epoch {
            self.smear_d[self.slot[ni] as usize]
        } else {
            [0.0; 3]
        }
    }

    /// **O ALVO DO ESFREGÃO** — `R[v] + D′[v]`, o passo 4 da espec §5.2.
    ///
    /// ⚠️ **Ele devolve a posição VIVA quando não há referência** — a mesma
    /// resposta do apagador, e pela mesma razão: ali o dado de entrada não
    /// existe. A recusa em voz alta é da shell; esta linha é a rede que
    /// garante que contorná-la não inventa geometria.
    ///
    /// ⚠️ **A DEGENERESCÊNCIA é a lei, não um caso de borda:** com `d̂` nulo
    /// (o [`crate::SmearMode::Drag`] com o cursor parado, ou um vértice
    /// exactamente no centro do dab nos outros dois) **nenhum** vizinho passa o
    /// teste do cosseno, a média colapsa em `D[v]` e o alvo é a posição viva.
    /// A espec mede-o: `1,3e-03` contra `2,4e-02` no traço que anda.
    ///
    /// ⚠️ **O anel entra INTEIRO, sem as duas regras de borda do
    /// [`ph2d_mesh::ring_average`]** — e a divergência é DELIBERADA. Aquelas
    /// regras existem para que um vértice de beira não seja sugado para dentro
    /// ao mediar POSIÇÕES; aqui o que se medeia é um **deslocamento**, e o alvo
    /// está ancorado em `R[v]`. Não há para onde sugar, e restringir o anel
    /// faria a borda deixar de receber o relevo que vem de dentro.
    pub(super) fn smear_target(
        &self,
        mesh: &Mesh,
        brush: &Brush,
        dab: &Dab,
        v: u32,
        s: usize,
        w: f32,
    ) -> [f32; 3] {
        let vi = v as usize;
        // ⚠️ **A posição viva é LIDA aqui e não recebida**, ao contrário do
        // irmão [`super::target::compute_target`] que a passa: ela é
        // literalmente `mesh.positions()[v]`, e o argumento a mais punha esta
        // assinatura em oito — o tecto do `clippy::too_many_arguments`. *Um
        // parâmetro que o corpo pode ler da fonte que já recebeu não é
        // informação, é ruído.*
        let live = mesh.positions()[vi];
        let Some(&r_v) = self.reference.get(vi) else {
            return live;
        };
        if unit(brush.smear_mode.direction(dab.path, dab.center, r_v)).is_none() {
            return live;
        }
        let d = self.media_do_anel(mesh, brush, dab, v, s);
        // ⭐⭐⭐ **A FORÇA ENTRA AO QUADRADO, e o quadrado é ESCRITO AQUI** —
        // espec §1.1, a mesma medição do apagador. `w` já é
        // `peso × força × pressão`, logo `w × strength` dá a **força** ao
        // quadrado e a **pressão** linear. ⛔ `w × w` elevaria a pressão e a
        // curva de queda junto.
        //
        // ⛔ **E o `clamp(f, 0, 1)` da espec §1.3 sai de graça:** `w` e
        // `strength` são ambos `[0, 1]`, logo o produto também.
        super::target::toward3(
            live,
            [r_v[0] + d[0], r_v[1] + d[1], r_v[2] + d[2]],
            w * brush.strength,
        )
    }

    /// **A MÉDIA PONDERADA DO ANEL** — o `D′[v]` da espec §5.2, a porta ÚNICA.
    ///
    /// ⚠️ **Ela tem DOIS chamadores** — a relaxação da preparação e o alvo — e é
    /// por isso que é uma porta: escrita duas vezes, a `n`-ésima passagem
    /// deixaria de ser a mesma lei da primeira, e o pincel teria uma lei para o
    /// meio do dab e outra para o fim.
    ///
    /// ⚠️ **Com `d̂` degenerado ela devolve o próprio `D[v]`** — nenhum vizinho
    /// passa o teste do cosseno, e a média colapsa. É a §5.3 escrita na
    /// aritmética em vez de num `if` no chamador.
    fn media_do_anel(&self, mesh: &Mesh, brush: &Brush, dab: &Dab, v: u32, s: usize) -> [f32; 3] {
        let vi = v as usize;
        let own = self.smear_d.get(s).copied().unwrap_or([0.0; 3]);
        let Some(&r_v) = self.reference.get(vi) else {
            return own;
        };
        let Some(dir) = unit(brush.smear_mode.direction(dab.path, dab.center, r_v)) else {
            return own;
        };
        let mut num = own;
        let mut den = 1.0f32;
        for &nb in mesh.adjacency().vert_verts.neighbours(vi) {
            let Some(&r_w) = self.reference.get(nb as usize) else {
                continue;
            };
            let Some(e) = unit([r_w[0] - r_v[0], r_w[1] - r_v[1], r_w[2] - r_v[2]]) else {
                continue;
            };
            // ⚠️ **A parte NEGATIVA do cosseno** — quem está do lado para onde a
            // mão vai tem peso zero. ⛔ O `if` é OPTIMIZAÇÃO e não lei (espec
            // §5.2): com `g == 0` as duas somas ficam iguais.
            let g = -(dir[0] * e[0] + dir[1] * e[1] + dir[2] * e[2]);
            if g <= 0.0 {
                continue;
            }
            let d_w = self.smear_neighbour(nb);
            for k in 0..3 {
                num[k] += g * d_w[k];
            }
            den += g;
        }
        let inv = 1.0 / den;
        [num[0] * inv, num[1] * inv, num[2] * inv]
    }
}

/// O vector unitário, ou `None` quando ele **não tem direcção**.
///
/// ⚠️ **O limiar é o zero exacto do quadrado do comprimento** e não um epsilon
/// escolhido: o que esta porta separa é *há direcção* de *não há*, e um epsilon
/// aqui inventaria uma terceira resposta — *«a direcção é pequena demais para
/// eu decidir»* — que a lei não tem onde pôr.
fn unit(v: [f32; 3]) -> Option<[f32; 3]> {
    let n2 = v[0] * v[0] + v[1] * v[1] + v[2] * v[2];
    if n2 <= 0.0 || !n2.is_finite() {
        return None;
    }
    let inv = 1.0 / n2.sqrt();
    Some([v[0] * inv, v[1] * inv, v[2] * inv])
}
