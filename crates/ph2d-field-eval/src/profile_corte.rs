//! ⭐⭐⭐ **O CORTE** — que arestas do contorno uma REGIÃO precisa, e as três respostas que ela tem.
//!
//! Irmão do [`super::profile_index`] por responsabilidade (teto de LOC, 2026-09-23: `728` contra
//! `700`): ali mora *o índice* — a construção, a consulta por ponto e a grelha do enrolamento —, e
//! aqui *a pergunta que uma região faz*. ⚠️ A fronteira não é de tamanho: o corte tem **três**
//! conjuntos distintos (a distância · o sinal · as meias-luas), cada um com a própria regra, e foi
//! esta wave que lhe acrescentou o quarto parâmetro (a costura do eixo).
//!
//! ⚠️ **Ele é um módulo FILHO de propósito**: os campos de [`ProfileIndex`] são privados, e um
//! filho vê os privados do pai. *Abri-los a um irmão seria pagar API pública por um corte de
//! tecto.*

use super::dist::{aresta_no_eixo, longe2, perto2, seg_box_dist2, seg_hull_dist2};
use super::{Edge, ProfileIndex};

impl ProfileIndex {
    /// As arestas que podem ser a mais próxima de **algum** ponto da caixa — ver
    /// [`Self::sd_batch_culled`].
    pub(super) fn cull(&self, lo: [f32; 2], hi: [f32; 2], out: &mut Vec<u32>) {
        self.cull_com(lo, hi, None, out);
    }

    /// ⭐⭐⭐ **O MESMO corte, sobre a POPULAÇÃO que a costura do eixo deixa** (2026-09-23).
    ///
    /// # ⛔⛔ O defeito MEDIDO que obrigou este parâmetro
    ///
    /// O [`crate::profile::sd_profile_in_region`] tirava as arestas do eixo com um `continue`
    /// **depois** de chamar o corte, e tinha escrita ao lado a nota de que um corte vazio é
    /// *«impossível — a regra do corte guarda sempre pelo menos a aresta que realiza o `dmax`»*.
    /// ⚠️ **A nota fala do CORTE e havia DOIS filtros:** o corte podia devolver exactamente a
    /// aresta da costura, o `continue` tirava-a, e a região caía no degenerado — que reconstrói a
    /// árvore **INTEIRA**.
    ///
    /// Medido no vaso da cena `5` (`1` das `24` arestas assenta no eixo): numa grelha de `32×32`
    /// em `(u, v)`, **`5` de `1 024`** células pagavam `931` linhas de WGSL em vez de `~50` —
    /// **`18×`** —, e as cinco eram as células **sobre o eixo**, à altura da costura, que é dentro
    /// do sólido e onde a marcha passa.
    ///
    /// ⇒ *a pergunta «esta aresta assenta no eixo?» tem uma PORTA* ([`Self::no_eixo`]) e o corte
    /// corre sobre a população que sobra, o que devolve à nota do degenerado a verdade que ela
    /// afirmava.
    fn cull_com(&self, lo: [f32; 2], hi: [f32; 2], sem_eixo: Option<f32>, out: &mut Vec<u32>) {
        out.clear();
        let corners = [
            [lo[0], lo[1]],
            [hi[0], lo[1]],
            [lo[0], hi[1]],
            [hi[0], hi[1]],
        ];
        let conta = |e: &Edge| sem_eixo.is_none_or(|tol| !aresta_no_eixo(e, tol));
        let mut dmax = f32::INFINITY;
        for e in self.edges.iter().filter(|e| conta(e)) {
            dmax = dmax.min(longe2(e, &corners));
        }
        for (i, e) in self.edges.iter().enumerate() {
            if conta(e) && perto2(e, seg_box_dist2(e, lo, hi)) <= dmax {
                out.push(i as u32);
            }
        }
    }

    /// ⚠️ Só para o gate: a distância² da região ao contorno da aresta `i` — ver
    /// [`seg_hull_dist2`]. É ela que tem de ser um **minorante verdadeiro**.
    #[doc(hidden)]
    #[must_use]
    pub fn probe_hull_dist2(&self, i: u32, hull: &[[f32; 2]]) -> f32 {
        seg_hull_dist2(&self.edges[i as usize], hull)
    }

    /// ⚠️ Só para o gate: o `dmax` que o corte por casco usa — ver [`Self::distance_edges_hull`].
    #[doc(hidden)]
    #[must_use]
    pub fn probe_hull_dmax(&self, hull: &[[f32; 2]]) -> f32 {
        let mut dmax = f32::INFINITY;
        for e in &self.edges {
            dmax = dmax.min(longe2(e, hull));
        }
        dmax
    }

    /// ⚠️ Só para a sonda: quantas arestas sobrevivem ao corte deste **casco convexo** (W59).
    ///
    /// ⭐ **A mesma regra, com a região a ser um polígono em vez de uma caixa.** A caixa de um tubo
    /// de viés é muito maior do que o tubo, e o `dmax` do corte cresce com o **diâmetro** da região
    /// — logo a caixa deita fora menos arestas do que a forma real deitaria.
    ///
    /// ⚠️ Ela é **sonda antes de ser produto**: a nota que a pediu diz que este é *"o único eixo que
    /// não multiplica a montagem de JIT"*, e isso é uma afirmação sobre o **preço**. Se o casco não
    /// cortar mais do que a caixa, não há obra a fazer — e esta linha já pagou quatro vezes por
    /// construir antes de medir.
    #[doc(hidden)]
    #[must_use]
    pub fn probe_cull_hull(&self, hull: &[[f32; 2]]) -> usize {
        self.distance_edges_hull(hull).len()
    }

    /// ⭐⭐⭐ **AS ARESTAS QUE A DISTÂNCIA PRECISA NESTE POLÍGONO** (W59) — a irmã do
    /// [`Self::distance_edges`], com a região a ser a forma real em vez da caixa dela.
    ///
    /// ⚠️ **A regra é a MESMA** (ver [`Self::sd_batch_culled`]): guarda-se toda aresta a menos de
    /// `dmax = min_e (máx distância de um VÉRTICE da região a e)`. O que muda é a região — e é o
    /// `dmax` que colhe: ele cresce com o **diâmetro**, e a diagonal de uma caixa é maior que a do
    /// polígono que ela envolve.
    ///
    /// ⭐ **Medido** (`the_table_of_whether_a_hull_culls_better_than_its_box`, 640×480, contra a
    /// caixa que shipava): `1,21×`–`1,28×` menos arestas na câmera de viés, `1,06×`–`1,08×` de
    /// frente. ⚠️ E a **área** cai `1,97×` para render só `1,21×` — *o corte segue o diâmetro, não a
    /// área*, e é por isso que o ganho é bem menor do que a figura sugere.
    ///
    /// ⚠️ Um polígono com menos de 3 vértices é degenerado (a região colapsou) ⇒ devolve **tudo**,
    /// que é a resposta segura.
    #[must_use]
    pub fn distance_edges_hull(&self, hull: &[[f32; 2]]) -> Vec<u32> {
        if hull.len() < 3 {
            return (0..self.edges.len() as u32).collect();
        }
        let mut dmax = f32::INFINITY;
        for e in &self.edges {
            // ⚠️ O máximo de uma função **convexa** sobre um polígono convexo está num VÉRTICE — é
            // a mesma lei que deixa a versão de caixa olhar só os quatro cantos.
            dmax = dmax.min(longe2(e, hull));
        }
        self.edges
            .iter()
            .enumerate()
            .filter(|(_, e)| perto2(e, seg_hull_dist2(e, hull)) <= dmax)
            .map(|(i, _)| i as u32)
            .collect()
    }

    /// ⚠️ Só para a sonda: quantas arestas sobrevivem ao corte desta caixa.
    #[doc(hidden)]
    #[must_use]
    pub fn probe_cull(&self, lo: [f32; 2], hi: [f32; 2]) -> usize {
        let mut v = Vec::new();
        self.cull(lo, hi, &mut v);
        v.len()
    }

    /// ⭐⭐ **As arestas que a DISTÂNCIA precisa nesta região** — a porta pública do corte.
    ///
    /// Ver [`Self::sd_batch_culled`] para a regra e para por que ela tem de ser conservadora.
    #[must_use]
    pub fn distance_edges(&self, lo: [f32; 2], hi: [f32; 2]) -> Vec<u32> {
        let mut v = Vec::new();
        self.cull(lo, hi, &mut v);
        v
    }

    /// ⭐⭐⭐ **ESTA ARESTA ASSENTA NO EIXO?** — a porta única de uma pergunta que vivia em dois
    /// sítios (ver [`Self::cull_com`]).
    ///
    /// ⚠️ **A régua são os extremos da CORDA**, e num arco isso não é o mesmo que «o arco assenta no
    /// eixo»: um arco cuja corda está no eixo sai dele até à flecha. A lei desta casa
    /// ([`crate::profile::sd_profile_in_region`]) sempre a leu assim, e reproduzi-la aqui é o que
    /// mantém a imagem **bit a bit** — *mudá-la é outra wave, e teria de ser medida na tela*.
    #[must_use]
    pub fn no_eixo(&self, i: u32, on_axis: f32) -> bool {
        aresta_no_eixo(&self.edges[i as usize], on_axis)
    }

    /// ⭐⭐⭐ **As arestas que a DISTÂNCIA precisa nesta região, com a COSTURA DO EIXO já fora.**
    ///
    /// Ver [`Self::cull_com`] para o defeito medido que a separou da [`Self::distance_edges`].
    /// ⚠️ Devolve **vazio** só quando o perfil inteiro assenta no eixo — que é o degenerado a
    /// sério, e é aí que o chamador recai na conta completa.
    #[must_use]
    pub fn distance_edges_fora_do_eixo(
        &self,
        lo: [f32; 2],
        hi: [f32; 2],
        on_axis: f32,
    ) -> Vec<u32> {
        let mut v = Vec::new();
        self.cull_com(lo, hi, Some(on_axis), &mut v);
        v
    }

    /// ⭐⭐ **As arestas que o SINAL precisa nesta região** — as que atravessam a caixa.
    ///
    /// ⚠️ **É um conjunto diferente do da distância, e menor.** O enrolamento é um invariante de
    /// caminho: `w(p) = w(canto) + atravessamentos do caminho canto→p`, e esse caminho **não sai da
    /// caixa** ⇒ só uma aresta que a atravessa o pode cruzar. Uma aresta longe muda a distância e
    /// **não** pode mudar o sinal.
    #[must_use]
    pub fn crossing_edges(&self, lo: [f32; 2], hi: [f32; 2]) -> Vec<u32> {
        let mut v = Vec::new();
        for (i, e) in self.edges.iter().enumerate() {
            let elo = [e.a[0].min(e.b[0]), e.a[1].min(e.b[1])];
            let ehi = [e.a[0].max(e.b[0]), e.a[1].max(e.b[1])];
            if elo[0] <= hi[0] && ehi[0] >= lo[0] && elo[1] <= hi[1] && ehi[1] >= lo[1] {
                v.push(i as u32);
            }
        }
        v
    }

    /// ⭐⭐⭐ **Os ARCOS cuja meia-lua pode tocar esta caixa** — o terceiro conjunto da região.
    ///
    /// ⚠️ Diferente dos outros dois: o SINAL da região é o enrolamento das CORDAS (que só as cordas
    /// que atravessam a caixa mudam) mais a meia-lua de cada arco, e a meia-lua de um arco pode
    /// tocar a caixa sem a corda dele tocar — ela sai da corda até à flecha.
    #[must_use]
    pub fn sliver_edges(&self, lo: [f32; 2], hi: [f32; 2]) -> Vec<u32> {
        let mut v = Vec::new();
        for (i, e) in self.edges.iter().enumerate() {
            if e.arco.is_none() {
                continue;
            }
            let (elo, ehi) = e.caixa();
            if elo[0] <= hi[0] && ehi[0] >= lo[0] && elo[1] <= hi[1] && ehi[1] >= lo[1] {
                v.push(i as u32);
            }
        }
        v
    }
}
