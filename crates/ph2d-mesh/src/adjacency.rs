//! Adjacência em **CSR** — vértice → faces e vértice → vértices.
//!
//! Adaptado de `reference/sculptgl/src/mesh/Mesh.js` (`initFaceRings`,
//! `initVertexRings`), MIT — ver `LICENSES/sculptgl-MIT.txt`.
//!
//! **Duas divergências deliberadas do original**, as duas para tirar caso
//! especial em vez de acrescentar:
//!
//! 1. O SculptGL guarda pares `(start, count)` intercalados; aqui é o CSR
//!    canônico — um `offsets` de `n+1` entradas, onde `count = off[i+1] − off[i]`.
//!    A mesma informação num array em vez de dois, e o `n+1` remove o ramo de
//!    "último vértice".
//! 2. O anel de vértices do original decide os vizinhos por uma cadeia de
//!    comparações contra o índice do próprio vértice. Aqui é o **ciclo da face**:
//!    os vizinhos de `v` numa face são o anterior e o seguinte na ordem dela.
//!    É a mesma resposta para tri e quad, escrita uma vez.
//!
//! ⚠️ **Custo, e é ele que a `measure_memory` reporta.** Numa malha fechada de
//! triângulos com `V` vértices há `≈2V` faces e `≈3V` arestas, então
//! `vert_face ≈ 6V` índices e `vert_vert ≈ 6V` — juntos ~56 B por vértice, mais
//! que os 24 B de posição+normal. O anel de VÉRTICES só é lido por Smooth/Relax
//! e pela topologia; se o número apertar, é ele que vira preguiçoso (o padrão
//! dos planos do impasto), e a decisão sai da sonda, não daqui.

use crate::face::Face;

/// Uma lista de adjacência em CSR **editável**. `values[starts[i]..][..lens[i]]`
/// são os vizinhos do item `i`.
///
/// ⚠️ **`starts` NÃO é mais uma soma de prefixos, e a mudança é o que torna uma
/// mudança de topologia uma EDIÇÃO em vez de uma malha nova.** No CSR canônico o
/// começo da linha `i + 1` é o fim da linha `i`, então crescer uma linha
/// significa deslocar todas as seguintes — `O(malha)` para acrescentar UM
/// vizinho. Aqui cada linha carrega o próprio começo e o próprio comprimento, e
/// uma linha que cresce **se muda para o fim** do vetor (a cópia é de meia dúzia
/// de índices, porque uma valência é meia dúzia). O rastro que ela deixa é
/// contado em [`Csr::dead`] e recolhido por [`Csr::compact_if_needed`].
///
/// ⚠️ **As linhas ficam em ORDEM CRESCENTE, e isso não é estética.** Uma linha
/// editada tem de ser *exatamente* a que uma reconstrução produziria: a soma das
/// normais de vértice percorre esta lista, e somar os mesmos números noutra
/// ordem dá outro último bit. Um incremental que "só" reordena o anel é um
/// incremental que diverge do `rebuild` no dígito que ninguém confere.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Csr {
    starts: Vec<u32>,
    lens: Vec<u32>,
    values: Vec<u32>,
    /// Entradas que nenhuma linha usa mais — o rastro das mudanças.
    dead: u32,
}

impl Csr {
    #[must_use]
    pub fn len(&self) -> usize {
        self.starts.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Os vizinhos de `i`. Fora de alcance devolve vazio — um vértice que não
    /// existe não tem vizinhos, e isso é mais útil que um pânico no meio de um
    /// laço de pincel.
    #[must_use]
    pub fn neighbours(&self, i: usize) -> &[u32] {
        let Some(&s) = self.starts.get(i) else {
            return &[];
        };
        let s = s as usize;
        &self.values[s..s + self.lens[i] as usize]
    }

    /// Total de entradas ALOCADAS — o que a sonda de memória multiplica por 4
    /// bytes. ⚠️ Inclui o rastro: é a pergunta de MEMÓRIA, não a de conteúdo.
    #[must_use]
    pub fn entry_count(&self) -> usize {
        self.values.len()
    }

    /// **Acrescenta `n` linhas VAZIAS** — os vértices que uma mudança de
    /// topologia criou.
    ///
    /// ⚠️ Uma linha vazia é `(start = 0, len = 0)`, e isso é seguro por
    /// construção: [`Csr::neighbours`] devolve `values[0..0]`, que é vazio seja
    /// qual for o conteúdo do vetor. Não há sentinela a manter em dia.
    fn grow_rows(&mut self, rows: usize) {
        self.starts.resize(rows, 0);
        self.lens.resize(rows, 0);
    }

    /// **Acrescenta `v` à linha `i`, mantendo-a crescente.** No-op se já estiver
    /// lá — um vizinho repetido não é um anel, é um bug a jusante.
    fn insert_sorted(&mut self, i: usize, v: u32) {
        let row = self.neighbours(i);
        let Err(at) = row.binary_search(&v) else {
            return;
        };
        let (s, n) = (self.starts[i] as usize, self.lens[i] as usize);
        // ⚠️ **A linha que já está no FIM cresce no lugar**, e este caso não é
        // raro: uma face nova tem índice maior que todas, então ela entra no fim
        // da linha — e a segunda face nova do mesmo vértice encontra a linha já
        // mudada para cá pela primeira. Sem isto, dar seis faces novas a um
        // vértice custaria seis realocações da linha inteira.
        if at == n && s + n == self.values.len() {
            self.values.push(v);
            self.lens[i] = u32::try_from(n + 1).unwrap_or(u32::MAX);
            return;
        }
        // A linha se muda para o fim: o CSR canônico não tem folga por linha, e
        // inventar folga custaria um terceiro vetor de capacidades por vértice.
        // Copiar meia dúzia de índices é mais barato que 4 B/vértice para sempre.
        let dest = u32::try_from(self.values.len()).unwrap_or(u32::MAX);
        self.values.reserve(n + 1);
        for k in 0..at {
            self.values.push(self.values[s + k]);
        }
        self.values.push(v);
        for k in at..n {
            self.values.push(self.values[s + k]);
        }
        self.starts[i] = dest;
        self.lens[i] = u32::try_from(n + 1).unwrap_or(u32::MAX);
        self.dead = self.dead.saturating_add(u32::try_from(n).unwrap_or(0));
    }

    /// **Tira `v` da linha `i`**, preservando a ordem. No-op se não estiver lá.
    fn remove(&mut self, i: usize, v: u32) {
        let Ok(at) = self.neighbours(i).binary_search(&v) else {
            return;
        };
        let (s, n) = (self.starts[i] as usize, self.lens[i] as usize);
        self.values.copy_within(s + at + 1..s + n, s + at);
        self.lens[i] = u32::try_from(n - 1).unwrap_or(0);
        self.dead = self.dead.saturating_add(1);
    }

    /// **Reescreve a linha `i` inteira**, na ordem dada.
    ///
    /// ⚠️ Ela NÃO ordena: o anel de vértices tem a ordem em que as faces
    /// aparecem, e é essa ordem que uma reconstrução produz.
    fn set_row(&mut self, i: usize, row: &[u32]) {
        let n = self.lens[i] as usize;
        if row.len() <= n {
            let s = self.starts[i] as usize;
            self.values[s..s + row.len()].copy_from_slice(row);
            self.lens[i] = u32::try_from(row.len()).unwrap_or(0);
            self.dead = self
                .dead
                .saturating_add(u32::try_from(n - row.len()).unwrap_or(0));
            return;
        }
        let dest = u32::try_from(self.values.len()).unwrap_or(u32::MAX);
        self.values.extend_from_slice(row);
        self.starts[i] = dest;
        self.lens[i] = u32::try_from(row.len()).unwrap_or(u32::MAX);
        self.dead = self.dead.saturating_add(u32::try_from(n).unwrap_or(0));
    }

    /// **Move a linha `from` para o slot `to`** — o passo de uma troca-com-o-
    /// último. O conteúdo não é copiado: a linha inteira muda de dono trocando
    /// dois números, e o que a linha `to` guardava vira rastro.
    fn move_row(&mut self, to: usize, from: usize) {
        self.dead = self.dead.saturating_add(self.lens[to]);
        self.starts[to] = self.starts[from];
        self.lens[to] = self.lens[from];
    }

    /// **Encolhe para `rows` linhas.** O que sobra vira rastro — as entradas
    /// continuam no vetor de valores até a próxima compactação.
    fn truncate_rows(&mut self, rows: usize) {
        for i in rows..self.starts.len() {
            self.dead = self.dead.saturating_add(self.lens[i]);
        }
        self.starts.truncate(rows);
        self.lens.truncate(rows);
    }

    /// **Recolhe o rastro quando ele passa da metade.**
    ///
    /// ⚠️ O gatilho é uma FRAÇÃO e não uma contagem: um limiar absoluto compacta
    /// demais numa malha pequena e nunca numa grande, que é o multiplicador
    /// disfarçado de teto que o ADR-0117 nomeia.
    fn compact_if_needed(&mut self) {
        if usize::try_from(self.dead).unwrap_or(0) * 2 <= self.values.len() {
            return;
        }
        let mut packed = Vec::with_capacity(self.values.len() - self.dead as usize);
        for i in 0..self.starts.len() {
            let (s, n) = (self.starts[i] as usize, self.lens[i] as usize);
            self.starts[i] = u32::try_from(packed.len()).unwrap_or(u32::MAX);
            packed.extend_from_slice(&self.values[s..s + n]);
        }
        self.values = packed;
        self.dead = 0;
    }
}

/// Os dois anéis que a malha carrega.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Adjacency {
    /// vértice → faces que o contêm.
    pub vert_faces: Csr,
    /// vértice → vértices ligados por uma aresta.
    pub vert_verts: Csr,
}

impl Adjacency {
    /// **Este vértice está na BORDA de uma malha aberta?**
    ///
    /// A identidade é a do `Mesh.js` do SculptGL (`vertOnEdge`, dentro do build
    /// do anel): *nº de faces ≠ nº de vizinhos únicos*. Num vértice interior o
    /// anel FECHA, e cada aresta é compartilhada por duas faces, então os dois
    /// números batem; na beira o anel é aberto e sobra um vizinho.
    ///
    /// ⚠️ **Não há campo novo nem memória nova** — o dado já estava nos dois
    /// CSRs, e a pergunta é `O(1)`. Materializá-lo seria guardar duas vezes um
    /// fato que a estrutura já contém, e que teria de ser reconstruído a cada
    /// mudança de topologia.
    ///
    /// ⚠️ **Vale para triângulo e para quad**, porque a conta é sobre o ANEL e
    /// não sobre a forma da face: um vértice interior de grade de quads tem
    /// quatro faces e quatro vizinhos; na beira, duas faces e três vizinhos.
    #[must_use]
    pub fn is_border(&self, v: usize) -> bool {
        self.vert_faces.neighbours(v).len() != self.vert_verts.neighbours(v).len()
    }

    /// Quantos vizinhos este vértice tem — a **valência**.
    ///
    /// Abaixo de três não há superfície em volta dele (é a ponta de uma tira ou
    /// um vértice colado), e é isso que o Smooth precisa saber para não o
    /// escorregar para a corda entre os dois únicos vizinhos.
    #[must_use]
    pub fn valence(&self, v: usize) -> usize {
        self.vert_verts.neighbours(v).len()
    }
}

impl Adjacency {
    /// Constrói os dois anéis. `vert_count` é autoritativo: um vértice sem
    /// nenhuma face (solto num OBJ) existe e tem anel vazio, em vez de
    /// desaparecer e deslocar todos os índices depois dele.
    #[must_use]
    pub fn build(vert_count: usize, faces: &[Face]) -> Self {
        let vert_faces = build_vert_faces(vert_count, faces);
        let vert_verts = build_vert_verts(vert_count, faces, &vert_faces);
        Self {
            vert_faces,
            vert_verts,
        }
    }

    /// **Uma face TROCOU de vértices** — atualiza os dois anéis sem reconstruir
    /// coisa alguma. Devolve os vértices afetados, em ordem crescente.
    ///
    /// Cada `edit` é `(índice, como era, como ficou)`, e `after` é a lista de
    /// faces **já reescrita**. ⚠️ **As duas metades leem lados diferentes do
    /// tempo** — a de faces remove pelos vértices ANTIGOS, a de anéis re-deriva
    /// pelos NOVOS —, e é por isso que o "antes" vem por parâmetro em vez de a
    /// função o ler: guardar a lista inteira de faces para tê-lo seria `O(malha)`
    /// dentro da porta que existe para não ser.
    ///
    /// ⚠️ **O anel de vértices é RE-DERIVADO, não remendado**, e a assimetria é
    /// deliberada: quem está no anel de `v` é função de TODAS as faces de `v`, e
    /// um remendo por aresta teria de saber que a aresta `a—b` só morre se
    /// NENHUMA outra face a compartilha. Re-derivar meia dúzia de vizinhos a
    /// partir de meia dúzia de faces é mais barato que essa contabilidade, e não
    /// tem como sair de sincronia.
    pub fn relink(&mut self, edits: &[(u32, Face, Face)], after: &[Face]) -> Vec<u32> {
        self.splice(edits, &[], after, self.vert_faces.len())
    }

    /// **A TOPOLOGIA MUDOU** — faces trocaram de vértices, faces nasceram,
    /// vértices nasceram. Devolve os vértices afetados, em ordem crescente.
    ///
    /// É a generalização do [`Self::relink`] que um CORTE precisa: ele edita as
    /// faces que partiu, **apenda** as filhas e **apenda** os pontos médios.
    ///
    /// ⚠️ **`added` são índices de faces que ainda não estavam nos anéis** — não
    /// confundir com o lado "novo" de um `edit`, que é uma face que já existia e
    /// trocou de cantos. Passar uma face nova como edit faria o `remove` procurar
    /// um índice que nunca esteve na linha (no-op silencioso) e o resultado
    /// pareceria certo até alguém contar os vizinhos.
    ///
    /// ⚠️ **Os vértices novos entram ANTES das faces**, porque uma face nova pode
    /// citar um deles — e inserir numa linha que não existe é um `panic` de
    /// índice, não um erro tratável.
    pub fn splice(
        &mut self,
        edits: &[(u32, Face, Face)],
        added: &[u32],
        after: &[Face],
        vert_count: usize,
    ) -> Vec<u32> {
        if vert_count > self.vert_faces.len() {
            self.vert_faces.grow_rows(vert_count);
            self.vert_verts.grow_rows(vert_count);
        }
        let mut affected: Vec<u32> = Vec::new();
        for &(i, old, new) in edits {
            for &v in old.verts() {
                if !new.verts().contains(&v) {
                    self.vert_faces.remove(v as usize, i);
                }
                affected.push(v);
            }
            for &v in new.verts() {
                if !old.verts().contains(&v) {
                    self.vert_faces.insert_sorted(v as usize, i);
                }
                affected.push(v);
            }
        }
        for &i in added {
            for &v in after[i as usize].verts() {
                self.vert_faces.insert_sorted(v as usize, i);
                affected.push(v);
            }
        }
        affected.sort_unstable();
        affected.dedup();

        let mut ring = Vec::new();
        for &v in &affected {
            ring_of(v as usize, after, &self.vert_faces, &mut ring);
            self.vert_verts.set_row(v as usize, &ring);
        }
        self.vert_faces.compact_if_needed();
        self.vert_verts.compact_if_needed();
        affected
    }

    /// **FACES SUMIRAM E AS QUE SOBRARAM MUDARAM DE CASA** — a primeira das duas
    /// metades de um colapso.
    ///
    /// ⚠️ **A numeração de VÉRTICE não muda aqui, e essa separação é o desenho.**
    /// Um colapso renumera faces *e* vértices, e as duas renumerações entrelaçadas
    /// são a forma de errar: cada patch teria de saber em que numeração o vizinho
    /// já está. Em duas fases sequenciais, cada uma tem um só eixo — esta as
    /// faces, a [`Self::shrink_verts`] os vértices — e entre elas a estrutura está
    /// íntegra.
    ///
    /// `edits` e `dead` chegam na numeração ANTIGA (`dead` é *índice + como a
    /// face era*, porque depois de compactada ela não existe para ser lida);
    /// `after` é a lista de faces **já compactada**.
    ///
    /// ⚠️ **A renumeração de face MARCA os vértices dela, e a primeira versão
    /// disto estava ERRADA.** O doc que aqui esteve dizia que trocar o índice de
    /// uma face não afeta ninguém *"porque o anel guarda VÉRTICES"* — o CONJUNTO
    /// de vizinhos de facto não muda, mas a **ORDEM** muda: o anel é derivado
    /// percorrendo as faces em ordem CRESCENTE de índice, e a primeira aparição
    /// de cada vizinho decide o lugar dele. Renumerar uma face a move nessa
    /// ordem. O sintoma é um anel com o conjunto certo na ordem errada, que só o
    /// oráculo de bit vê — porque a normal de um vértice é a SOMA das normais do
    /// anel, e somar os mesmos números noutra ordem dá outro `f32`.
    pub fn shrink_faces(
        &mut self,
        edits: &[(u32, Face, Face)],
        dead: &[(u32, Face)],
        remap: &crate::Remap,
        after: &[Face],
    ) -> Vec<u32> {
        let mut affected: Vec<u32> = Vec::new();
        for &(i, old, new) in edits {
            for &v in old.verts() {
                if !new.verts().contains(&v) {
                    self.vert_faces.remove(v as usize, i);
                }
                affected.push(v);
            }
            for &v in new.verts() {
                if !old.verts().contains(&v) {
                    self.vert_faces.insert_sorted(v as usize, i);
                }
                affected.push(v);
            }
        }
        for &(f, old) in dead {
            for &v in old.verts() {
                self.vert_faces.remove(v as usize, f);
                affected.push(v);
            }
        }
        // ⚠️ **Os pares LÍQUIDOS, e não a sequência crua.** Um destino pode ser
        // uma casa de passagem que a lista compactada nem tem — ver
        // [`crate::Remap::net_face_moves`]. Aqui o que se lê é o CONTEÚDO da face
        // no destino FINAL, então a casa de passagem daria um índice fora da
        // lista (e, quando não desse, os vértices de outra face).
        for (from, to) in remap.net_face_moves(after.len() + dead.len()) {
            for &v in after[to as usize].verts() {
                self.vert_faces.remove(v as usize, from);
                self.vert_faces.insert_sorted(v as usize, to);
                affected.push(v);
            }
        }
        affected.sort_unstable();
        affected.dedup();
        let mut ring = Vec::new();
        for &v in &affected {
            ring_of(v as usize, after, &self.vert_faces, &mut ring);
            self.vert_verts.set_row(v as usize, &ring);
        }
        self.vert_faces.compact_if_needed();
        self.vert_verts.compact_if_needed();
        affected
    }

    /// **VÉRTICES SUMIRAM** — a segunda metade, e ela **reescreve as faces**.
    ///
    /// ⚠️ **As duas metades ou nenhuma, e é por isso que as faces entram por aqui
    /// em vez de o chamador as reescrever.** Renomear um vértice é *reescrever
    /// toda face que o cita* **e** *mudar a linha dele de casa*; separá-las daria
    /// uma porta que deixa a malha citando um índice que já é de outro vértice —
    /// e nada falha, porque o índice novo é válido. Quem sabe QUAIS faces citam
    /// `from` é o anel de faces, que mora aqui.
    ///
    /// `mark` chega do tamanho ANTIGO, já marcado com quem a fase das faces
    /// afetou, e sai truncado e LIMPO. ⚠️ **Ele é carregado pelas trocas** — um
    /// destino pode mudar de casa outra vez (com os mortos `[8, 3]` numa lista de
    /// 10, o slot 8 recebe o 9 e depois se muda para o 3), então uma lista de
    /// índices coletada antes descreveria a malha de duas trocas atrás.
    ///
    /// Devolve os afetados na numeração NOVA, prontos para o refresco de região.
    pub fn shrink_verts(
        &mut self,
        faces: &mut [Face],
        remap: &crate::Remap,
        mark: &mut [bool],
    ) -> Vec<u32> {
        for &(from, to) in &remap.vert_moves {
            // ⚠️ **A marca sai das FACES e não do anel de vértices**, e a
            // diferença mordeu: o anel só é re-derivado no fim, então durante o
            // laço ele ainda pode citar um vizinho que já mudou de casa — marcar
            // por ele marca o vértice ERRADO, e o sintoma é um anel com o
            // conjunto certo na ordem errada, que só o oráculo de bit vê. Os
            // cantos de uma face estão sempre em dia, porque é este laço que os
            // reescreve.
            for &fi in self.vert_faces.neighbours(from as usize) {
                let f = &mut faces[fi as usize];
                f.rename_vert(from, to);
                for &w in f.verts() {
                    mark[w as usize] = true;
                }
            }
            // ⚠️ Incondicional: um vértice sem face nenhuma não é alcançado pelo
            // laço acima, e o slot dele tem de ficar com um anel vazio em vez do
            // que o morto guardava.
            mark[to as usize] = true;
            self.vert_faces.move_row(to as usize, from as usize);
            self.vert_verts.move_row(to as usize, from as usize);
        }
        self.vert_faces.truncate_rows(remap.verts);
        self.vert_verts.truncate_rows(remap.verts);
        let mut affected: Vec<u32> = Vec::new();
        for (v, m) in mark.iter_mut().enumerate().take(remap.verts) {
            if *m {
                *m = false;
                affected.push(u32::try_from(v).unwrap_or(u32::MAX));
            }
        }
        let mut ring = Vec::new();
        for &v in &affected {
            ring_of(v as usize, faces, &self.vert_faces, &mut ring);
            self.vert_verts.set_row(v as usize, &ring);
        }
        self.vert_faces.compact_if_needed();
        self.vert_verts.compact_if_needed();
        affected
    }
}

/// Passe de contagem + passe de preenchimento — o CSR clássico, sem `Vec<Vec<_>>`
/// intermediário (que a 5 M faces seriam milhões de alocações).
fn build_vert_faces(vert_count: usize, faces: &[Face]) -> Csr {
    let mut starts = vec![0u32; vert_count + 1];
    for f in faces {
        for &v in f.verts() {
            starts[v as usize + 1] += 1;
        }
    }
    for i in 0..vert_count {
        starts[i + 1] += starts[i];
    }
    let total = starts[vert_count] as usize;
    let mut values = vec![0u32; total];
    // `cursor` anda dentro de cada faixa; no fim ele é o começo do vértice
    // seguinte, que é a invariante que o gate `the_freshly_built_csr_is_packed`
    // afirma. ⚠️ E porque as faces são percorridas em ordem, **cada linha nasce
    // CRESCENTE** — a propriedade de que a edição por inserção ordenada depende.
    let mut cursor: Vec<u32> = starts[..vert_count].to_vec();
    for (fi, f) in faces.iter().enumerate() {
        for &v in f.verts() {
            let slot = &mut cursor[v as usize];
            values[*slot as usize] = fi as u32;
            *slot += 1;
        }
    }
    let lens = (0..vert_count).map(|i| starts[i + 1] - starts[i]).collect();
    starts.truncate(vert_count);
    Csr {
        starts,
        lens,
        values,
        dead: 0,
    }
}

/// O anel de vértices sai do anel de faces: para cada face que toca `v`, os
/// vizinhos são o anterior e o seguinte no **ciclo** dela.
///
/// A deduplicação usa um carimbo (`stamp`) por vértice em vez de um `HashSet`:
/// zero alocação por vértice, e determinístico — a ordem de saída é a ordem em
/// que as faces aparecem, nunca a ordem de iteração de um mapa.
fn build_vert_verts(vert_count: usize, faces: &[Face], vf: &Csr) -> Csr {
    let mut stamp = vec![u32::MAX; vert_count];
    let mut starts = vec![0u32; vert_count + 1];

    // Passe 1 — conta os vizinhos únicos.
    for v in 0..vert_count {
        let mut n = 0u32;
        for_each_ring_neighbour(v, faces, vf, &mut stamp, v as u32, |_| n += 1);
        starts[v + 1] = n;
    }
    for i in 0..vert_count {
        starts[i + 1] += starts[i];
    }

    // Passe 2 — preenche. O carimbo é reiniciado porque o passe 1 o gastou; um
    // segundo vetor seria memória à toa.
    stamp.fill(u32::MAX);
    let total = starts[vert_count] as usize;
    let mut values = vec![0u32; total];
    for v in 0..vert_count {
        let mut w = starts[v] as usize;
        for_each_ring_neighbour(v, faces, vf, &mut stamp, v as u32, |nb| {
            values[w] = nb;
            w += 1;
        });
        debug_assert_eq!(w, starts[v + 1] as usize, "os dois passes discordaram");
    }

    let lens = (0..vert_count).map(|i| starts[i + 1] - starts[i]).collect();
    starts.truncate(vert_count);
    Csr {
        starts,
        lens,
        values,
        dead: 0,
    }
}

/// O anel de vértices de UM vértice, na mesma ordem que o
/// [`build_vert_verts`] produziria — a metade que a edição incremental precisa
/// re-derivar.
///
/// ⚠️ **A deduplicação é uma varredura linear e não um carimbo**, e é de
/// propósito: um carimbo é um vetor do tamanho da MALHA, e esta função existe
/// justamente para não pagar isso. Um anel tem meia dúzia de vizinhos.
fn ring_of(v: usize, faces: &[Face], vf: &Csr, out: &mut Vec<u32>) {
    out.clear();
    for &fi in vf.neighbours(v) {
        let vs = faces[fi as usize].verts();
        let Some(pos) = vs.iter().position(|&x| x as usize == v) else {
            continue;
        };
        let n = vs.len();
        for nb in [vs[(pos + n - 1) % n], vs[(pos + 1) % n]] {
            if nb as usize != v && !out.contains(&nb) {
                out.push(nb);
            }
        }
    }
}

/// Chama `f` uma vez por vizinho ÚNICO de `v`, na ordem das faces.
///
/// `stamp[u] == epoch` significa "já visto nesta volta". `epoch` é o próprio
/// índice do vértice, que é único por definição — sem contador global e sem
/// risco de dois passes se confundirem.
fn for_each_ring_neighbour(
    v: usize,
    faces: &[Face],
    vf: &Csr,
    stamp: &mut [u32],
    epoch: u32,
    mut f: impl FnMut(u32),
) {
    // O próprio vértice nunca é vizinho de si.
    stamp[v] = epoch;
    for &fi in vf.neighbours(v) {
        let face = faces[fi as usize];
        let vs = face.verts();
        let Some(pos) = vs.iter().position(|&x| x as usize == v) else {
            continue;
        };
        let n = vs.len();
        for nb in [vs[(pos + n - 1) % n], vs[(pos + 1) % n]] {
            if stamp[nb as usize] != epoch {
                stamp[nb as usize] = epoch;
                f(nb);
            }
        }
    }
}

#[cfg(test)]
#[path = "adjacency_tests.rs"]
mod tests;
