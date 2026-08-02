//! **O LADO `before` QUE VEM DO JOURNAL** — filho de [`super`] (`#[path]`, então ele enxerga os
//! campos privados da janela), o degrau 2 do S3 (doc 28 §5.58.2).
//!
//! O [`super::StoredPlane::split`] precisa dos DOIS endpoints materializados, e é isso que obriga o
//! `stroke_undo` e o `cursor` a segurarem um `Arc` de cada plano — os donos extras que fazem toda
//! escrita de gesto pagar `Arc::make_mut` sobre o documento inteiro. Aqui o lado `before` **não é um
//! buffer**: são os bytes velhos que o journal capturou na hora da escrita.
//!
//! ⚠️ **Isto SHIPA em release** — o degrau 4 promoveu o journal do RELEVO junto com a elisão do
//! `before`, e os dois têm de shipar juntos (elidir sem journal derruba a história a cada traço;
//! promover o journal sozinho paga captura *e* fork até o fork morrer — doc 28 §5.58.1). **Só o
//! journal do CANVAS continua `cfg(any(test, debug_assertions))`** (`WriteState::capture_canvas` tem
//! um no-op de release ao lado).
//!
//! ⚠️ **Este parágrafo dizia o contrário até 2026-08-02, e a mentira custou um diagnóstico:** ele
//! afirmava *"tudo aqui é `cfg(any(test, debug_assertions))`"* — verdade no degrau 2, falsa depois da
//! promoção —, e uma investigação da posse concluiu (e publicou) que a sonda não via o caminho do
//! produto. Não há `#[cfg]` nenhum neste arquivo; *grepe o atributo, não leia a prosa* (§5.67).

use std::collections::BTreeMap;
use std::sync::Arc;

use super::{PlaneWindow, StoredEntry, StoredMap, StoredPlane, fits};
use crate::layers::LayerId as RtLayerId;

impl PlaneWindow {
    /// **A janela de uma caixa de TILES do journal** — ver
    /// [`TileJournal::window`](crate::undo::journal::TileJournal::window), o único chamador.
    ///
    /// Ela chega já ancorada (o journal conhece o plano de que capturou), então não passa pelo
    /// [`Self::fit_to`]; o que este construtor faz é recusar o degenerado, que é o mesmo `None` que o
    /// `fit_to` devolve — *lento nunca, errado jamais*.
    pub(crate) const fn tiles(
        row: usize,
        rows: usize,
        col: usize,
        cols: usize,
        stride: usize,
        plane_len: usize,
    ) -> Option<Self> {
        if rows == 0 || cols == 0 || !fits(plane_len, stride) {
            return None;
        }
        Some(Self {
            row,
            rows,
            col,
            cols,
            stride,
            plane_len,
        })
    }

    /// **A interseção de duas janelas do MESMO plano** — `None` quando elas não se cruzam.
    ///
    /// ⚠️ **Cruzar dois SUPERCONJUNTOS ainda contém o escrito**, e é isso que a torna segura: a janela
    /// DECLARADA contém o que o passo escreveu (o contrato de [`crate::undo::window`]) e a caixa de
    /// tiles do journal também (ela é a caixa dos tiles que a área declarada tocou). Cruzá-las aperta a
    /// janela sem cortar um texel que mudou.
    ///
    /// ⚠️ **E ela existe por MEDIÇÃO, não por elegância.** Um tile mede 128 elementos de lado, então a
    /// caixa do journal arredonda o traço para fora em até 127 de cada lado: com ela sozinha o passo
    /// típico saltou de **2,51 para 8,23 MB a 1024²** e o `measure_undo_capacity` reprovou na hora — o
    /// delta comprava 3,9× mais passos que um documento por endpoint, contra os ~13× que a §5.28 mediu.
    fn intersect(self, other: Self) -> Option<Self> {
        if self.stride != other.stride || self.plane_len != other.plane_len {
            return None;
        }
        let row = self.row.max(other.row);
        let col = self.col.max(other.col);
        let row_end = (self.row + self.rows).min(other.row + other.rows);
        let col_end = (self.col + self.cols).min(other.col + other.cols);
        if row >= row_end || col >= col_end {
            return None;
        }
        Some(Self {
            row,
            rows: row_end - row,
            col,
            cols: col_end - col,
            stride: self.stride,
            plane_len: self.plane_len,
        })
    }

    /// Copia a janela para fora de uma função de ELEMENTO, em vez de um slice.
    ///
    /// Existe porque o lado `before` do journal não é um buffer: ele é
    /// `journal.get(i).unwrap_or(vivo[i])`, elemento a elemento. Irmã do [`Self::extract`], e as duas
    /// percorrem a janela na MESMA ordem — o `Patch` guarda os dois lados e eles têm de se corresponder
    /// índice a índice.
    /// A rota ELEMENTO-A-ELEMENTO, **CONGELADA** como oráculo — é o código que shipava antes da
    /// §5.70, verbatim.
    ///
    /// ⚠️ Ela vive sob `cfg(test)` e não como um método sem chamador: um segundo caminho vivo é uma
    /// **segunda resposta** esperando alguém chamá-la (a lição do `warp_axis` e do `serial_side`).
    #[cfg(test)]
    fn extract_by<T>(&self, mut f: impl FnMut(usize) -> T) -> Box<[T]> {
        let mut out = Vec::with_capacity(self.elems());
        for r in 0..self.rows {
            let s = (self.row + r) * self.stride + self.col;
            out.extend((s..s + self.cols).map(&mut f));
        }
        out.into_boxed_slice()
    }

    /// **O lado `before` da janela, lido do journal POR CORRIDA de tile** (doc 28 §5.70).
    ///
    /// Mesma resposta que `extract_by(|i| j.get(i).unwrap_or(live[i]))`, sem repetir a aritmética de
    /// tile por elemento — ver [`TileJournal::read_row_into`](crate::undo::journal::TileJournal), que é
    /// quem sabe onde os tiles moram. A ORDEM de percurso é a mesma do [`Self::extract`], e ela é
    /// load-bearing: o `Patch` guarda os dois lados e eles se correspondem índice a índice.
    fn extract_journal<T: Copy + Send + Sync>(
        &self,
        j: &crate::undo::journal::TileJournal<T>,
        live: &[T],
    ) -> Box<[T]> {
        let mut out = Vec::with_capacity(self.elems());
        for r in 0..self.rows {
            j.read_row_into(self.row + r, self.col, self.cols, live, &mut out);
        }
        out.into_boxed_slice()
    }
}

impl<T: Copy + PartialEq + Send + Sync> StoredPlane<T> {
    /// **O delta de um plano cujo lado `before` vem do JOURNAL** — o degrau 2 do S3 (doc 28 §5.58.2).
    ///
    /// O `split` clássico precisa dos DOIS endpoints materializados, e é isso que obriga o
    /// `stroke_undo` e o `cursor` a segurarem um `Arc` de cada plano — os donos extras que fazem toda
    /// escrita de gesto pagar `Arc::make_mut` sobre o documento inteiro. Aqui o lado `before` **não é
    /// um buffer**: são os bytes velhos que o journal capturou na hora da escrita, e o lado `after` é o
    /// plano VIVO. Nenhum dos dois é uma cópia nova.
    ///
    /// ⚠️ **A lei do lado `before` é a identidade da §5.28**, e ela é o que torna a janela do journal
    /// (uma caixa de tiles, sempre um SUPERCONJUNTO do escrito) utilizável sem erro:
    ///
    /// ```text
    ///   before[i] == journal.get(i).unwrap_or(vivo[i])
    /// ```
    ///
    /// Dentro da caixa há tiles que ninguém tomou e elementos que ninguém escreveu; para todos eles o
    /// valor de antes **é** o valor de agora, então lê-los do vivo é exato, não uma aproximação.
    ///
    /// ⚠️ **`None` = "não sei descrever este plano", e o chamador cai no caminho de sempre.** É a mesma
    /// política do `undeclared` do S1 e do `journal_describes_step_at` do S2: um caso que o journal não
    /// cobre degrada o passo para *lento*, nunca para *errado*.
    ///
    /// ⛔ **O `Whole` POR LIMIAR foi REMOVIDO, e a medição é o motivo** (doc 28 §5.69). Este
    /// doc-comment dizia que ele *"só é tomado quando a janela cobre mais de metade do plano, e ali o
    /// `Whole` guardaria os dois planos inteiros de qualquer forma — o `split` clássico faz exatamente a
    /// mesma escolha, no mesmo limiar"*. A escolha era a mesma; ⚠️ **a PREMISSA não**: no
    /// [`super::StoredPlane::from_window`] o `Whole` **MOVE** os `Arc` que já existem (custo zero e
    /// nenhuma cópia), e aqui ele **COPIA** — `par_clone` do plano inteiro mais uma varredura
    /// `j.get(i)` de plano inteiro — **por cima de um `before`/`after` que as linhas acima já
    /// extraíram**. Era uma regra transplantada para o sítio onde o que a justificava é falso.
    ///
    /// Medido pela porta do produto, diagonal de canto a canto (§5.69): o commit cai de **272,5 para
    /// 151,6 ms a 4096²** e de 70,8 para 42,3 a 2048² — e ele **também perdia em BYTES**, que era o
    /// único eixo em que eu supunha que ele ganhava: `Whole` guarda os dois lados dos QUATRO planos
    /// inteiros (**8,00× um plano RGBA por passo**, exato nas duas telas) contra **7,66×** do `Patch`.
    /// Mais a posse: o `after: Arc::clone(live)` era um **segundo dono permanente** do plano vivo, que
    /// fazia a primeira escrita do gesto seguinte copiar o documento.
    ///
    /// ⚠️ **O `Whole` de CAPTURA-INTEIRA (logo abaixo) FICA** — ali o journal tem o plano completo e não
    /// há janela a extrair, então `w.to_vec()` é uma cópia contra as DUAS que um `Patch` de plano
    /// inteiro faria. A premissa que o limiar não tinha, este tem.
    pub(crate) fn from_journal(
        live: &Arc<Vec<T>>,
        j: &crate::undo::journal::TileJournal<T>,
        hint: Option<PlaneWindow>,
    ) -> Option<Self> {
        // A captura de plano INTEIRO (o sítio que não sabia onde ia escrever): o `before` já existe
        // completo, então não há janela a derivar.
        if let Some(w) = j.whole() {
            if w.len() != live.len() {
                return None; // o journal descreve um plano de outra forma
            }
            if w == live.as_slice() {
                return Some(Self::Unchanged);
            }
            return Some(Self::Whole {
                before: Arc::new(w.to_vec()),
                after: Arc::clone(live),
            });
        }
        let Some(jwin) = j.window() else {
            // Nada capturado: este passo não escreveu neste plano, logo os dois lados são o mesmo.
            return Some(Self::Unchanged);
        };
        if jwin.plane_len != live.len() {
            return None;
        }
        // A caixa de tiles APERTADA pela janela declarada — ver [`PlaneWindow::intersect`], que é quem
        // devolve ao passo o tamanho que a §5.28 mediu. Sem declaração, a caixa sozinha (correta, só
        // mais gorda), que é a mesma política do `hint` ausente no `split`.
        let win = match hint.and_then(|h| h.fit_to(live.len())) {
            Some(h) => jwin.intersect(h)?,
            None => jwin,
        };
        let before = win.extract_journal(j, live);
        let after = win.extract(live);
        if before == after {
            // Escreveu os mesmos bytes de volta — o `split` clássico chega aqui pelo `diff_window`, que
            // devolve `None`. As duas rotas têm de dar a MESMA forma, senão o gate de igualdade
            // reportaria uma diferença de memória como se fosse de conteúdo.
            return Some(Self::Unchanged);
        }
        // ⚠️ **Nenhum limiar aqui, e é a entrega da §5.69.** `before` e `after` já estão extraídos:
        // qualquer ramo que os descarte para materializar o plano inteiro paga uma cópia a mais, guarda
        // mais bytes, e ainda pina o plano vivo. Ver o ⛔ do doc acima.
        Some(Self::Patch { win, before, after })
    }
}

impl<T: Copy + PartialEq + Send + Sync> StoredMap<T> {
    /// **O mapa de uma família de relevo, com o lado `before` vindo do JOURNAL** — ver
    /// [`StoredPlane::from_journal`], que é o motor; aqui mora só *quais chaves*, que é a metade que o
    /// journal não sabe (ele guarda bytes de UM plano, não a forma de um mapa).
    ///
    /// ⚠️ **As OUTRAS camadas são `Unchanged` por LEI, não por comparação** — e a lei é o guard que
    /// autoriza esta rota: os journals só falam pelo passo quando **uma** camada foi capturada
    /// (`speaks_for` recusa `mixed`), e toda escrita de relevo passa por uma porta nomeada
    /// (`fork_heights`/`fork_covers`/`fork_mats`). Logo nenhuma outra camada foi escrita neste passo.
    /// Em DEBUG a premissa é conferida contra os dois lados, que o degrau 2 ainda carrega — é o mesmo
    /// molde da rede que o `split` mantém sobre a janela declarada.
    ///
    /// ⚠️ **`absent` com a chave presente no `before` é RECUSA, não `Unchanged`.** O journal marca um
    /// plano ausente quando ele foi escrito sem ter forma de canvas; se o `before` o tem assim mesmo, o
    /// plano existia com OUTRA forma, e um mapa que dissesse `Unchanged` perderia a troca em silêncio —
    /// que é exatamente o modo de falha que o [`diff_window`] documenta e teme.
    ///
    /// `None` = o journal não descreve este mapa; o chamador usa o [`Self::split`] de sempre.
    ///
    /// ⚠️ **Ele NÃO esvazia os mapas, ao contrário do [`Self::split`]** — e a assimetria é o que torna a
    /// rota tudo-ou-nada. São TRÊS famílias de relevo e qualquer uma pode recusar; esvaziar à medida
    /// que cada uma passa deixaria o `heights` drenado quando o `mats` recusasse, e o caminho de
    /// fallback partiria de mapas vazios. Quem esvazia é o chamador, **depois** de as três passarem.
    ///
    /// # O `before` ELIDIDO — o terceiro estado (degrau 4, doc 28 §5.60)
    ///
    /// `before_elided` são as camadas que o `before` **descreve sem segurar** (ver
    /// [`crate::undo::elide`]). Elas entram na união de chaves exatamente como as fortes, e é **isso**
    /// que impede o defeito medido: sem o terceiro estado uma chave elidida cai no braço
    /// `(None, Some(a))` = `OnlyAfter`, que SIGNIFICA *"não existia antes"* ⇒ desfazer remove a chave
    /// e **o undo apaga o relevo**.
    ///
    /// ⚠️ **E toda chave elidida passa pela TESTEMUNHA.** O lado `before` de um passo elidido é
    /// `journal.get(i).unwrap_or(vivo[i])`, e essa identidade só vale enquanto o plano vivo for o
    /// MESMO objeto que o snapshot descrevia. Um sítio que troca o plano por inteiro a quebra, e a
    /// resposta é recusar — reconstruir um `before` sobre um fundo estranho não falharia em lugar
    /// nenhum.
    pub(crate) fn from_journal(
        before: &BTreeMap<RtLayerId, Arc<Vec<T>>>,
        before_elided: &BTreeMap<RtLayerId, std::sync::Weak<Vec<T>>>,
        after: &BTreeMap<RtLayerId, Arc<Vec<T>>>,
        layer: RtLayerId,
        absent: bool,
        j: &crate::undo::journal::TileJournal<T>,
        hint: Option<PlaneWindow>,
    ) -> Option<Self> {
        let mut entries = BTreeMap::new();
        let keys: Vec<RtLayerId> = before
            .keys()
            .chain(before_elided.keys())
            .chain(after.keys())
            .copied()
            .collect();
        for k in keys {
            if entries.contains_key(&k) {
                continue;
            }
            // *O `before` TINHA relevo nesta camada?* — as duas metades do terceiro estado respondem
            // juntas, e é a única pergunta que este laço faz sobre o lado de antes.
            let had = before.contains_key(&k) || before_elided.contains_key(&k);
            let e = match (had, after.get(&k)) {
                (true, Some(_)) if k == layer && absent => return None,
                (true, Some(a)) => {
                    // ⚠️ **A TESTEMUNHA, e SÓ onde o journal é silencioso sobre este plano.**
                    //
                    // `Some(false)` diz *"o plano vivo não é o objeto que o snapshot descrevia"*. Com
                    // o journal calado isso é fatal: o `Unchanged` que sairia daqui afirmaria que nada
                    // mudou sobre bytes que ninguém olhou — o buraco exato das escritas que trocam o
                    // plano por inteiro sem passar por porta de captura.
                    //
                    // ⚠️ **Com o journal FALANDO ela é estrita demais, e isso está MEDIDO:** o próprio
                    // fold (`commit_stroke_height`) monta um plano novo e o INSERE, então o `Arc` muda
                    // de identidade em todo traço de impasto. Os bytes estão cobertos pela captura, e
                    // exigir `ptr_eq` ali reprovou **58 gates** do caminho normal.
                    if j.is_empty()
                        && crate::undo::elide::witness(before_elided, k, a) == Some(false)
                    {
                        return None;
                    }
                    if k == layer {
                        StoredEntry::Both(StoredPlane::from_journal(a, j, hint)?)
                    } else {
                        debug_assert!(
                            before
                                .get(&k)
                                .is_none_or(|b| Arc::ptr_eq(b, a) || **b == **a),
                            "o journal fala pela camada {layer:?} e outra camada MUDOU no mesmo passo \
                             — a lei do `speaks_for` (uma camada por passo) nao vale aqui, e o \
                             `Unchanged` abaixo perderia a edicao (doc 28 §5.58.2)"
                        );
                        StoredEntry::Both(StoredPlane::Unchanged)
                    }
                }
                // O passo REMOVEU o relevo desta camada. O journal descreve o que foi escrito, nunca o
                // que deixou de existir: a resposta honesta é não descrever.
                (true, None) => return None,
                (false, Some(a)) => StoredEntry::OnlyAfter(Arc::clone(a)),
                (false, None) => continue,
            };
            entries.insert(k, e);
        }
        Some(Self { entries })
    }
}

#[cfg(test)]
mod extract_tests {
    //! **A leitura POR CORRIDA é a leitura POR ELEMENTO, ao bit** (doc 28 §5.70).
    //!
    //! O oráculo é a rota congelada [`PlaneWindow::extract_by`] — *o código que shipava* — e não uma
    //! re-derivação: as duas têm de percorrer a janela na MESMA ordem, senão o `Patch` guarda dois
    //! lados que não se correspondem índice a índice e o undo instala bytes trocados de lugar.

    use super::PlaneWindow;
    use crate::undo::journal::TileJournal;

    /// Um plano ESTRUTURADO — um campo chato faria qualquer leitura concordar, e o gate seria verde
    /// por vácuo.
    fn plane(stride: usize, rows: usize) -> Vec<u8> {
        (0..stride * rows)
            .map(|i| u8::try_from((i * 37 + i / stride * 11) % 251).expect("cabe em u8"))
            .collect()
    }

    /// ⚠️ **A fixture tem de conter tile CAPTURADO e tile NÃO capturado**, e janelas que ATRAVESSAM a
    /// fronteira de 128: dentro de um tile só, as duas rotas leem o mesmo bloco e a corrida nunca é
    /// exercitada — que é a forma de passar sem julgar nada.
    #[test]
    fn the_run_walk_reads_what_the_element_walk_reads() {
        let mut from_journal = 0usize;
        let (stride, rows) = (400usize, 300usize);
        let before = plane(stride, rows);
        let mut live = before.clone();
        for (i, v) in live.iter_mut().enumerate() {
            *v = v
                .wrapping_add(u8::try_from(i % 7).expect("cabe"))
                .wrapping_add(1);
        }

        for area in [
            Some((10usize, 5usize, 260usize, 200usize)), // atravessa 128 nos dois eixos
            Some((0, 0, 128, 128)),                      // um tile exato
            Some((130, 130, 140, 140)),                  // um pedaço de UM tile interior
            None,                                        // captura de plano inteiro
        ] {
            let mut j = TileJournal::default();
            j.capture(&before, stride, area);

            for win in [
                PlaneWindow::tiles(0, rows, 0, stride, stride, before.len()),
                PlaneWindow::tiles(3, 210, 7, 300, stride, before.len()),
                PlaneWindow::tiles(129, 40, 127, 5, stride, before.len()),
            ]
            .into_iter()
            .flatten()
            {
                let fast = win.extract_journal(&j, &live);
                let slow = win.extract_by(|i| j.get(i).unwrap_or(live[i]));
                assert_eq!(
                    fast.len(),
                    slow.len(),
                    "as duas rotas devolvem tamanhos diferentes (area {area:?})"
                );
                let bad = fast.iter().zip(&slow).filter(|(a, b)| a != b).count();
                assert_eq!(
                    bad, 0,
                    "{bad} elementos divergiram da leitura por-elemento (area {area:?})"
                );
                // ⚠️ Controle de COBERTURA, e ele é do CONJUNTO e não de cada par: uma janela que
                // não cruza a área capturada lê tudo do plano vivo — legítimo, e é justamente um dos
                // casos a testar. O que não pode é NENHUM par tocar o journal, porque aí a igualdade
                // seria sobre o plano vivo e não julgaria resolução de tile nenhuma.
                if slow
                    .iter()
                    .zip(win_indices(&win))
                    .any(|(v, i)| *v != live[i])
                {
                    from_journal += 1;
                }
            }
        }
        assert!(
            from_journal >= 4,
            "controle: so' {from_journal} par(es) leram bytes do JOURNAL — a igualdade estaria \
             sendo medida sobre o plano vivo, que as duas rotas leem igual por construcao"
        );
    }

    /// Os índices globais que a janela percorre, na ordem em que as duas rotas os percorrem.
    fn win_indices(w: &PlaneWindow) -> Vec<usize> {
        let mut v = Vec::new();
        for r in 0..w.rows {
            let s = (w.row + r) * w.stride + w.col;
            v.extend(s..s + w.cols);
        }
        v
    }
}
