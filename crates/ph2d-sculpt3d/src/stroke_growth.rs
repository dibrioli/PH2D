//! **A MALHA CRESCEU DEBAIXO DO TRAÇO** — a herança que a topologia dinâmica
//! exige, e os leitores do `pre` que só ela usa.
//!
//! Filho (`#[path]`) do [`super`], pelo mesmo corte do `stroke_growth_tests.rs`:
//! no pai mora *o que um dab faz*, aqui *o que acontece quando a malha muda de
//! tamanho no meio de um gesto*. O pai bateu no teto de LOC no dia em que isto
//! entrou, e o tamanho é o sintoma — o assunto é a razão.

use super::{Mesh, SculptStroke};
use ph2d_mesh::{Birth, DEFAULT_MASK};

/// O ponto médio — a única aritmética que a herança de um vértice novo faz.
fn mid3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        (a[0] + b[0]) * 0.5,
        (a[1] + b[1]) * 0.5,
        (a[2] + b[2]) * 0.5,
    ]
}

/// Normaliza, devolvendo a entrada quando ela é degenerada — a média de duas
/// normais quase opostas não tem direção, e inventar uma seria pior que herdar
/// uma que ninguém vai ler (o vértice está numa dobra, e todo verbo que lê
/// `base_nrm` ali já está a trabalhar sobre uma quina).
fn unit3(v: [f32; 3]) -> [f32; 3] {
    let l = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    if l > 1e-12 {
        [v[0] / l, v[1] / l, v[2] / l]
    } else {
        v
    }
}

impl SculptStroke {
    /// **A malha CRESCEU no meio do traço** — acomoda os vértices novos e faz
    /// cada um HERDAR o passado dos pais, sem tocar no que já foi congelado.
    ///
    /// (Ver o cabeçalho deste módulo para o porquê de ele viver aqui.)
    ///
    /// ⚠️ **É o que deixa a topologia dinâmica conviver com a lei do traço.** O
    /// caminho óbvio seria chamar [`Self::begin`] de novo depois de refinar, e
    /// ele está ERRADO pelo motivo que esta casa já pagou quatro vezes no
    /// relevo do Painter: re-começar joga fora o `pre`, e o dab seguinte passa a
    /// medir a partir do resultado do anterior — o traço vira um PRODUTO sobre
    /// a lista de dabs, e a força passa a depender de quantas vezes o refino
    /// disparou.
    ///
    /// Ela só é correta porque o refino **APENDA**: um vértice que já existia
    /// mantém o índice, então `slot`, `stamp`, `touched` e `base_pos` seguem
    /// descrevendo exatamente os mesmos vértices. Um refino que reordenasse
    /// índices tornaria isto silenciosamente errado — e é por isso que a frase
    /// está aqui, e não só no motor.
    ///
    /// # ⚠️ A HERANÇA, e a frase FALSA que esteve aqui
    ///
    /// A primeira versão desta função só redimensionava, e o doc dela afirmava
    /// que o `pre` de um vértice novo *"será a posição em que ele NASCEU … a
    /// única resposta que existe para «onde ele estava antes?»"*. **Há outra
    /// resposta, e ela é a certa:** o vértice nasce no ponto médio de dois pais
    /// que este mesmo traço **já deslocou**, então essa posição já contém o
    /// deslocamento — capturá-la como `pre` faz o dab somar o deslocamento uma
    /// SEGUNDA vez, enquanto os vizinhos o receberam uma. A diferença é uma
    /// AGULHA por vértice novo, com a altura do próprio traço, e foi o que o
    /// smoke de 2026-08-04 reprovou (*"o aspecto da escultura depois do P fica
    /// horrível"*). Medido pelo desvio de guarda-chuva, p99 **0,3088 sem refino
    /// contra 0,6104 com ele**.
    ///
    /// Onde ele estava antes é o **ponto médio dos `pre` dos pais** — eles
    /// estavam lá desde o pen-down, e o meio deles é onde a aresta passava.
    /// Herdam-se os cinco canais por-vértice, e cada um por um motivo:
    ///
    /// - `base_pos`/`base_nrm`/`base_mask` — o `pre` do vértice, que é o que o
    ///   alvo de todo verbo lê;
    /// - `accum` — ⚠️ **o mais importante depois do `base_pos`.** Herdar `0` faz
    ///   o primeiro dab fraco que o alcance escrevê-lo em `lerp(pre, alvo, 0,05)`
    ///   = praticamente o `pre`, ou seja **puxá-lo de volta para a superfície
    ///   original** enquanto os vizinhos ficam levantados: uma CRATERA, o defeito
    ///   espelhado da agulha. Com o envelope herdado, o early-out o deixa quieto
    ///   até um dab de fato mais forte chegar;
    /// - `base_mask` — um vértice que nasce dentro de uma região MASCARADA e
    ///   herda máscara zero recebe peso cheio enquanto os vizinhos dele estão
    ///   congelados: ele sobe sozinho. Medido, a mesma agulha por um terceiro
    ///   caminho (0,867 da aresta contra 0,108);
    /// - `target` — por coerência. Um registro meio preenchido é uma armadilha
    ///   para quem ler o próximo canal, e o custo é uma cópia de 12 bytes.
    ///
    /// ⚠️ **A herança do `base_nrm` é COERÊNCIA e não correção, e a mutação dela
    /// NÃO sangra — de propósito, documentado.** O único verbo que lê a normal
    /// congelada é o [`Verb::Inflate`], e um vértice que nasce SOBRE a superfície
    /// tem normal viva praticamente igual à média das normais congeladas dos
    /// pais (o deslocamento do próprio Inflate é ao longo da normal, que quase
    /// não a gira). Ela fica porque um registro em que quatro canais vêm dos pais
    /// e um vem da malha viva é o tipo de assimetria que a próxima pessoa
    /// "corrige" para o lado errado.
    ///
    /// **Um pai não capturado tem `pre == posição viva`** (o mesmo fato que o
    /// [`Self::base_pos_of`] usa), então uma aresta em território virgem herda
    /// exatamente a posição em que o vértice nasceu — o comportamento antigo,
    /// que ali sempre esteve certo. É por isso que o vértice novo só é capturado
    /// quando ALGUM pai já foi: sem isso, todo refino capturaria a pegada inteira
    /// e a janela do undo cresceria sem nada ter se movido.
    ///
    /// ⚠️ **A ORDEM de `births` é load-bearing** — um passe parte arestas cujo
    /// extremo pode ser um vértice do passe anterior, e percorrer para a frente é
    /// o que garante que o pai já foi resolvido quando o filho pergunta.
    ///
    /// # Pânico
    ///
    /// Se a lista de nascimentos não explicar o crescimento. Um vértice novo sem
    /// parentela não tem `pre` derivável, e o modo de falha silencioso dele é
    /// exatamente a agulha que esta função existe para não ter.
    pub fn grow_with(&mut self, mesh: &Mesh, births: &[Birth]) {
        let verts = mesh.vert_count();
        let grew = verts.saturating_sub(self.slot.len());
        assert_eq!(
            births.len(),
            grew,
            "todo vértice novo tem de declarar de onde veio: a malha cresceu {grew} \
             e chegaram {} nascimentos",
            births.len()
        );
        if grew == 0 {
            return;
        }
        self.slot.resize(verts, u32::MAX);
        self.stamp.resize(verts, 0);
        for b in births {
            self.inherit(mesh, b);
        }
    }

    /// Semeia UM vértice novo a partir dos pais. Ver [`Self::grow_with`].
    fn inherit(&mut self, mesh: &Mesh, b: &Birth) {
        let (pa, pb) = (self.slot_of(b.a), self.slot_of(b.b));
        // Nenhum pai tocado ⇒ nada se moveu ali ⇒ a posição em que ele nasceu
        // JÁ é o `pre` dele, e capturá-lo agora só engordaria a janela do undo.
        if pa.is_none() && pb.is_none() {
            return;
        }
        let vi = b.vert as usize;
        let pos = mid3(self.base_pos_of(mesh, b.a), self.base_pos_of(mesh, b.b));
        let nrm = unit3(mid3(
            self.base_nrm_of(mesh, b.a),
            self.base_nrm_of(mesh, b.b),
        ));
        let mask = 0.5 * (self.base_mask_of(mesh, b.a) + self.base_mask_of(mesh, b.b));
        let accum = 0.5 * (pa.map_or(0.0, |s| self.accum[s]) + pb.map_or(0.0, |s| self.accum[s]));
        let target = mid3(
            pa.map_or(mesh.positions()[b.a as usize], |s| self.target[s]),
            pb.map_or(mesh.positions()[b.b as usize], |s| self.target[s]),
        );
        self.stamp[vi] = self.epoch;
        self.slot[vi] = self.touched.len() as u32;
        self.touched.push(b.vert);
        self.base_pos.push(pos);
        self.base_nrm.push(nrm);
        self.base_mask.push(mask);
        self.accum.push(accum);
        self.target.push(target);
    }

    /// O slot deste vértice neste traço, ou `None` se ele nunca foi capturado.
    fn slot_of(&self, v: u32) -> Option<usize> {
        let vi = v as usize;
        (self.stamp[vi] == self.epoch).then(|| self.slot[vi] as usize)
    }

    /// A normal de `v` ANTES do traço — o irmão exato do [`Self::base_pos_of`],
    /// e pelo mesmo fato: quem não foi capturado nunca foi escrito.
    fn base_nrm_of(&self, mesh: &Mesh, v: u32) -> [f32; 3] {
        let vi = v as usize;
        if self.stamp[vi] == self.epoch {
            self.base_nrm[self.slot[vi] as usize]
        } else {
            mesh.normals()[vi]
        }
    }

    /// A máscara de `v` ANTES do traço.
    fn base_mask_of(&self, mesh: &Mesh, v: u32) -> f32 {
        let vi = v as usize;
        if self.stamp[vi] == self.epoch {
            self.base_mask[self.slot[vi] as usize]
        } else {
            mesh.masks().map_or(DEFAULT_MASK, |m| m[vi])
        }
    }
}

impl SculptStroke {
    /// **A malha ENCOLHEU no meio do traço** — o colapso apagou vértices, e os
    /// que sobraram mudaram de índice.
    ///
    /// É a irmã exata do [`Self::grow_with`], e ela existe pelo mesmo motivo:
    /// re-chamar [`Self::begin`] depois de mexer na topologia joga fora o `pre` e
    /// faz o dab seguinte medir a partir do resultado do anterior. O que muda é
    /// que aqui **não há nada a herdar** — um vértice que some não deixa
    /// pergunta em aberto; o que há é uma renumeração a aplicar.
    ///
    /// ⚠️ **Sem ela o produto PANICA**, e é a forma boa de falhar: o
    /// [`Self::dab`] afirma `slot.len() == mesh.vert_count()`, então uma malha
    /// que encolhe sem avisar o traço bate nesse `assert` em vez de esculpir com
    /// o `pre` de outro vértice.
    ///
    /// # As DUAS compactações, e por que a segunda existe
    ///
    /// O traço tem dois eixos: um vetor **por VÉRTICE** (`slot`/`stamp`) e um
    /// vetor **por SLOT** (`touched` e os cinco planos do `pre`), ligados nos
    /// dois sentidos — `touched[slot[v]] == v`. Compactar só o primeiro deixaria
    /// os planos do `pre` a crescer para sempre ao longo de um traço que colapsa;
    /// compactar só o segundo deixaria `slot` a apontar para o `pre` de outro
    /// vértice. As duas ou nenhuma.
    ///
    /// ⚠️ **`stamp[from] = 0` é load-bearing**: depois de a captura se mudar para
    /// `to`, o slot dela tem UM dono, e sem apagar a marca de origem a varredura
    /// da cauda mataria a captura que acabou de sobreviver.
    pub fn shrink_with(&mut self, remap: &ph2d_mesh::Remap) {
        if remap.verts == self.slot.len() {
            return;
        }
        let mut dead_slots: Vec<u32> = Vec::new();
        for &(from, to) in &remap.vert_moves {
            let (from, to) = (from as usize, to as usize);
            if self.stamp[to] == self.epoch {
                dead_slots.push(self.slot[to]);
            }
            self.slot[to] = self.slot[from];
            self.stamp[to] = self.stamp[from];
            if self.stamp[to] == self.epoch {
                self.touched[self.slot[to] as usize] = u32::try_from(to).unwrap_or(u32::MAX);
            }
            self.stamp[from] = 0;
        }
        for v in remap.verts..self.slot.len() {
            if self.stamp[v] == self.epoch {
                dead_slots.push(self.slot[v]);
            }
        }
        self.slot.truncate(remap.verts);
        self.stamp.truncate(remap.verts);

        if dead_slots.is_empty() {
            return;
        }
        dead_slots.sort_unstable();
        dead_slots.dedup();
        // ⚠️ A MESMA aritmética que a malha usa, pela porta dela — uma segunda
        // cópia é como os dois lados passam a discordar sobre quem sobrou.
        let (moves, keep) = ph2d_mesh::compaction(&dead_slots, self.touched.len());
        for &(from, to) in &moves {
            let (from, to) = (from as usize, to as usize);
            self.touched[to] = self.touched[from];
            self.base_pos[to] = self.base_pos[from];
            self.base_nrm[to] = self.base_nrm[from];
            self.base_mask[to] = self.base_mask[from];
            self.accum[to] = self.accum[from];
            self.target[to] = self.target[from];
            self.slot[self.touched[to] as usize] = u32::try_from(to).unwrap_or(u32::MAX);
        }
        self.touched.truncate(keep);
        self.base_pos.truncate(keep);
        self.base_nrm.truncate(keep);
        self.base_mask.truncate(keep);
        self.accum.truncate(keep);
        self.target.truncate(keep);
    }
}
