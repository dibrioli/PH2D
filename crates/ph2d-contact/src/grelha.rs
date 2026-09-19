//! **A GRELHA ESPACIAL, EM CSR E COM OS BUFFERS REUTILIZADOS** — onde uma varredura vai buscar os
//! candidatos de cada peça.
//!
//! # Porque ela existe (report do dono, 2026-09-18: *«com 1024 FPS cai para 7, usando Boids»*)
//!
//! A 1.ª redacção era um `BTreeMap<(i64,i64), Vec<usize>>` construído **por varredura**, e cada
//! peça fazia **nove** buscas na árvore mais um `Vec` novo para os vizinhos. Medido a `500` peças
//! (a sonda [`crate::custo_probe`]): `49 %` do relógio de uma varredura era **achar os pares**, com
//! uma média de `5,2` vizinhos por peça — *a escrituração custava tanto como a lei que ela serve*.
//!
//! ⇒ aqui a grelha é:
//! - **densa** sobre a caixa das peças activas (nenhuma busca: o índice é aritmética),
//! - em **CSR** (um `itens` contíguo + um `inicio` por célula), construída por contagem,
//! - com os buffers **reaproveitados** entre varreduras (`1024` varreduras deixam de ser `1024`
//!   mapas e `n × 1024` alocações de vizinhos).
//!
//! # ⭐⭐⭐ Porque o LADO pode CRESCER, e porque isso não muda um bit
//!
//! A promessa da grelha é entregar um **SUPERCONJUNTO** dos contactos, em ordem crescente: quem não
//! toca é descartado pelo `manifesto` e não entra em soma nenhuma (é isso que faz
//! `the_grid_gives_the_same_bits_as_all_pairs` ser uma igualdade **exacta**). Como um contacto pede
//! `d ≤ alcance_i + alcance_j ≤ 2·alcance_max`, **qualquer** lado `≥ 2·alcance_max` tem as 9
//! células a cobrir tudo o que pode tocar.
//!
//! ⇒ quando a caixa das peças pediria mais de [`CELULAS_MAX`] células — uma peça largada a um
//! milhão de unidades das outras —, o lado **DOBRA** até caber. A malha fica grosseira, os
//! candidatos aumentam e o resultado é o mesmo **ao bit**: *o preço de uma cena esticada é relógio,
//! nunca resposta errada*.

use super::celula;

/// O tecto de células da grelha densa — o recurso é **MEMÓRIA** (`4 B` por célula no `inicio`,
/// logo `256 KiB` no pior caso), e não um palpite: acima dele o lado dobra (ver o cabeçalho).
const CELULAS_MAX: usize = 1 << 16;

/// A grelha de uma varredura. ⚠️ **Construída no sítio** (`constroi`), para que as `varreduras`
/// passagens de um passe partilhem a mesma alocação.
#[derive(Default)]
pub(super) struct Grelha {
    lado: f32,
    min: (i64, i64),
    cols: usize,
    rows: usize,
    /// `cols · rows + 1` fronteiras — a célula `c` ocupa `itens[inicio[c]..inicio[c + 1]]`.
    inicio: Vec<u32>,
    /// As peças activas, agrupadas por célula.
    ///
    /// ⚠️⚠️ **A ordem DENTRO de uma célula não é load-bearing, e a 1.ª redacção deste comentário
    /// dizia que era** (*«a metade da promessa de ordem»*). Quem a prova é uma MUTAÇÃO: inverter o
    /// laço da contagem — que põe cada célula por ordem decrescente — **sobrevive** ao gate da
    /// igualdade ao bit, porque o [`Self::vizinhos_de`] ordena o que colheu. *A promessa inteira é
    /// do `sort`; o laço crescente é uma conveniência.*
    itens: Vec<u32>,
    /// A célula de cada peça, ou `u32::MAX` para quem não está activa.
    celula_de: Vec<u32>,
    /// O cursor da contagem, reaproveitado.
    cursor: Vec<u32>,
}

impl Grelha {
    /// Reconstrói a grelha para esta varredura. `lado_min` é `2 · alcance_max`.
    pub(super) fn constroi(&mut self, foto: &[[f32; 2]], ativo: &[bool], lado_min: f32) {
        let n = foto.len();
        self.celula_de.clear();
        self.celula_de.resize(n, u32::MAX);
        self.lado = lado_min;
        // A caixa das células, dobrando o lado até a grelha densa caber (ver o cabeçalho).
        let (cols, rows) = loop {
            let (mut x0, mut y0, mut x1, mut y1) = (i64::MAX, i64::MAX, i64::MIN, i64::MIN);
            for (i, q) in foto.iter().enumerate() {
                if !ativo[i] {
                    continue;
                }
                let (cx, cy) = celula(*q, self.lado);
                (x0, y0) = (x0.min(cx), y0.min(cy));
                (x1, y1) = (x1.max(cx), y1.max(cy));
            }
            if x0 > x1 {
                (self.cols, self.rows) = (0, 0); // nenhuma peça activa
                return;
            }
            // ⚠️ Tudo por `try_from` e por `checked_*`: uma caixa de células de uma cena esticada
            // estoura qualquer inteiro, e um `as` truncado daria uma grelha PEQUENA que se leria
            // como válida — a peça distante cairia numa célula que não é a dela.
            let cabe = usize::try_from(x1.saturating_sub(x0).saturating_add(1))
                .ok()
                .zip(usize::try_from(y1.saturating_sub(y0).saturating_add(1)).ok())
                .filter(|(c, r)| c.checked_mul(*r).is_some_and(|t| t <= CELULAS_MAX));
            if let Some((c, r)) = cabe {
                self.min = (x0, y0);
                break (c, r);
            }
            self.lado *= 2.0;
        };
        (self.cols, self.rows) = (cols, rows);
        // A contagem por célula, e daí as fronteiras.
        self.inicio.clear();
        self.inicio.resize(cols * rows + 1, 0);
        for (i, q) in foto.iter().enumerate() {
            if !ativo[i] {
                continue;
            }
            let c = self.indice(*q);
            self.celula_de[i] = c;
            self.inicio[c as usize + 1] += 1;
        }
        for c in 0..cols * rows {
            self.inicio[c + 1] += self.inicio[c];
        }
        // O laço percorre `i` a crescer, logo cada célula sai ordenada. ⚠️ Isto NÃO é a promessa
        // de ordem (ver o campo `itens`): quem a cumpre é o `sort` do `vizinhos_de`, e uma mutação
        // que inverte este laço sobrevive a todo gate desta crate.
        self.cursor.clear();
        self.cursor.extend_from_slice(&self.inicio[..cols * rows]);
        self.itens.clear();
        self.itens.resize(self.inicio[cols * rows] as usize, 0);
        for i in 0..n {
            let c = self.celula_de[i];
            if c == u32::MAX {
                continue;
            }
            let slot = &mut self.cursor[c as usize];
            #[expect(clippy::cast_possible_truncation, reason = "i < n, e n cabe num u32")]
            {
                self.itens[*slot as usize] = i as u32;
            }
            *slot += 1;
        }
    }

    /// O índice linear da célula de `q`. ⚠️ Só é chamado depois de a caixa estar fixada, logo
    /// `q` cai sempre dentro dela.
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "a celula de `q` esta' dentro da caixa que a construcao acabou de medir"
    )]
    fn indice(&self, q: [f32; 2]) -> u32 {
        let (cx, cy) = celula(q, self.lado);
        let (x, y) = ((cx - self.min.0) as usize, (cy - self.min.1) as usize);
        (y * self.cols + x) as u32
    }

    /// Os candidatos de `k` — as `3 × 3` células à volta dela —, **em ordem crescente** e escritos
    /// num buffer que o chamador reaproveita.
    ///
    /// ⭐ **Três fatias e não nove:** as células de uma linha são contíguas no CSR, logo uma linha
    /// inteira do vizinhado copia-se de uma vez.
    pub(super) fn vizinhos_de(&self, k: usize, out: &mut Vec<u32>) {
        out.clear();
        let c = self.celula_de[k];
        if c == u32::MAX {
            return;
        }
        let (x, y) = (c as usize % self.cols, c as usize / self.cols);
        let (x0, x1) = (x.saturating_sub(1), (x + 1).min(self.cols - 1));
        let (y0, y1) = (y.saturating_sub(1), (y + 1).min(self.rows - 1));
        for yy in y0..=y1 {
            let linha = yy * self.cols;
            let (a, b) = (self.inicio[linha + x0], self.inicio[linha + x1 + 1]);
            out.extend_from_slice(&self.itens[a as usize..b as usize]);
        }
        out.sort_unstable();
    }
}

/// **Quantos CANDIDATOS a grelha entrega, somados sobre as peças** — o multiplicador do custo de
/// uma varredura, e o número que separa *«muitas peças»* de *«uma pilha apertada»*.
///
/// ⚠️ Ele corre a MESMA grelha do produto, uma vez, sobre a configuração de entrada — não é uma
/// segunda resposta à pergunta *«quem está perto de quem?»*.
#[must_use]
pub fn candidatos(colisores: &[Option<super::Colisor>], p: &[[f32; 2]], _pesos: &[f32]) -> usize {
    let n = p.len();
    let ativo: Vec<bool> = (0..n)
        .map(|i| super::ativo(p[i], colisores[i].as_ref()))
        .collect();
    let alcance_max = (0..n)
        .filter(|&i| ativo[i])
        .filter_map(|i| colisores[i].map(|c| c.alcance()))
        .fold(0.0_f32, f32::max);
    if alcance_max <= 0.0 {
        return 0;
    }
    let mut grade = Grelha::default();
    grade.constroi(p, &ativo, 2.0 * alcance_max);
    let mut viz: Vec<u32> = Vec::new();
    let mut soma = 0usize;
    for k in 0..n {
        grade.vizinhos_de(k, &mut viz);
        soma += viz.len();
    }
    soma
}
