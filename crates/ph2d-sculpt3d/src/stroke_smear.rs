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
    pub(super) fn fill_smear_d(&mut self, mesh: &Mesh, needed: bool) {
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
        let own = self.smear_d.get(s).copied().unwrap_or([0.0; 3]);
        let Some(dir) = unit(brush.smear_mode.direction(dab.path, dab.center, r_v)) else {
            return live;
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
            [
                r_v[0] + num[0] * inv,
                r_v[1] + num[1] * inv,
                r_v[2] + num[2] * inv,
            ],
            w * brush.strength,
        )
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
