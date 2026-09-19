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
//!
//! # ⭐⭐⭐ As DUAS CAMADAS, e porque uma peça grande inflava a grelha de TODAS
//!
//! O lado sai de `2 · alcance_max`, e o `alcance_max` é o **MÁXIMO GLOBAL**. ⇒ uma única peça `k ×`
//! maior que as outras faz a célula de **toda** peça crescer `k ×`, e o número de candidatos de
//! **toda** peça crescer `~k²` — *sem que um único toque a mais aconteça*. Medido
//! ([`crate::custo_probe::contagens::uma_peca_grande_infla_a_grelha_de_todas`], `1000` discos):
//!
//! | raio da peça 0 | candidatos/peça | tocam/peça |
//! |---|---|---|
//! | `1 × R` | `12,1` | `5,74` |
//! | `4 × R` | **`159,4`** | `5,76` |
//! | `16 × R` | `958,9` | `5,91` |
//!
//! ⚠️⚠️ **E o `4 × R` é a cena do dono:** o perfilador dele imprimiu `132`–`156` vizinhos por peça
//! numa nuvem de `1000` objectos, e a fixtura uniforme desta crate lê `12`. *A fixtura não continha
//! o fenómeno, e a diferença entre as duas é uma dispersão de tamanhos de ~`4 ×`.*
//!
//! ⇒ a grelha passa a ter **duas camadas**: as peças PEQUENAS numa grelha fina de lado
//! `2 · corte`, e as GRANDES numa lista à parte. Uma pequena vê as `3 × 3` da grelha fina **mais
//! todas as grandes**; uma grande vê a nuvem activa inteira.
//!
//! ⭐ **A promessa do SUPERCONJUNTO fica intacta, caso a caso:** pequena × pequena cabe na malha
//! fina (`d ≤ alc_i + alc_j ≤ 2 · corte`); pequena × grande está na lista; grande × qualquer está
//! na nuvem inteira. ⇒ a saída continua **bit-idêntica** a todos-os-pares, e é o mesmo gate que o
//! prova.
//!
//! ⚠️ **O corte não é um número escolhido — é uma MINIMIZAÇÃO** ([`Grelha::planeia`]), e o modelo
//! dela foi validado contra a tabela medida ([`crate::MARGEM_DO_CORTE`]). ⛔ E se o modelo errar, o
//! que se perde é **relógio e nunca resposta**: um plano mau dá uma grelha pior, não uma grelha
//! errada.

use super::{CELULAS_POR_CANDIDATO, MARGEM_DO_CORTE, celula};

/// ⛔⛔⛔ **O CORTE EM DUAS CAMADAS SHIPA DESLIGADO** (`PH2D_CONTACT_DUAS_CAMADAS=1` liga-o).
///
/// ⚠️⚠️ **A porta nasceu ao contrário e foi INVERTIDA por auditoria** (doc 115 §31). Ela era
/// `PH2D_CONTACT_UMA_CAMADA` — o corte LIGADO por omissão e o motor de antes atrás de uma bandeira
/// —, e isso violava a lei desta casa: *tudo o que é novo shipa desligado até o dono o aprovar.*
/// Eu liguei-o por omissão com **todas** as medições tiradas numa forma de cena que a dele não tem
/// (`12` vizinhos por peça contra `124`), e o dono perdeu quadros **três vezes seguidas**.
///
/// ⛔ *Quando a minha medição e o report do dono discordam e eu não consigo fechar a distância, o
/// caminho de omissão é o que ele APROVOU* — o ónus da prova é de quem mudou.
///
/// ⚠️ Ela é lida **uma vez** e **só no [`Grelha::planeia`]**, que é a porta do PRODUTO: o
/// [`Grelha::planeia_com_margem`] e o [`Grelha::planeia_numa_camada`] ficam de fora de propósito,
/// para que um gate continue a medir a lei e não o ambiente. *Uma bandeira global lida no fundo da
/// pilha é uma corrida escrita à mão, e esta casa já a pagou.*
fn duas_camadas_por_ordem() -> bool {
    static ORDEM: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *ORDEM.get_or_init(|| ordem_de(std::env::var("PH2D_CONTACT_DUAS_CAMADAS").ok().as_deref()))
}

/// `true` quando o caminho do produto está a CORTAR a grelha em duas camadas. ⛔ `false` é o
/// caminho de omissão — ver a porta acima.
pub(super) fn duas_camadas_activas() -> bool {
    duas_camadas_por_ordem()
}

/// A leitura da porta acima, **separada do ambiente para poder ser gateada**.
///
/// ⚠️⚠️ **`env VAR=` DEFINE a variável, vazia** — esta casa já pagou o erro: um controlo escrito
/// assim corre a mesma lei que devia contradizer, e lê-se como se o interruptor não existisse.
pub(super) fn ordem_de(v: Option<&str>) -> bool {
    matches!(v, Some(x) if !x.is_empty() && x != "0")
}

/// O tecto de células da grelha densa — o recurso é **MEMÓRIA** (`4 B` por célula no `inicio`,
/// logo `256 KiB` no pior caso), e não um palpite: acima dele o lado dobra (ver o cabeçalho).
const CELULAS_MAX: usize = 1 << 16;

/// A grelha de uma varredura. ⚠️ **Construída no sítio** (`constroi`), para que as `varreduras`
/// passagens de um passe partilhem a mesma alocação.
#[derive(Default)]
pub(super) struct Grelha {
    /// O lado que o [`Grelha::planeia`] escolheu — `2 × corte`. O [`Grelha::constroi`] parte dele e
    /// pode DOBRÁ-LO para caber em [`CELULAS_MAX`] (ver o cabeçalho).
    lado_planeado: f32,
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
    /// A célula de cada peça, ou `u32::MAX` para quem não está activa **ou é GRANDE** (uma grande
    /// não entra na grelha fina — ver o cabeçalho).
    celula_de: Vec<u32>,
    /// O cursor da contagem, reaproveitado.
    cursor: Vec<u32>,
    /// As peças GRANDES, em ordem CRESCENTE — a cauda da lista de candidatos de toda pequena.
    grandes: Vec<u32>,
    /// `true` para quem o plano promoveu a GRANDE. Um vector de `n`, sempre.
    e_grande: Vec<bool>,
    /// Todas as peças activas, em ordem CRESCENTE — a lista de candidatos de uma GRANDE, que já
    /// nasce ordenada e por isso **não se ordena**.
    ativas: Vec<u32>,
}

impl Grelha {
    /// ⭐⭐⭐ **O PLANO: quem é GRANDE, e qual é o lado da célula fina.**
    ///
    /// Corre **uma vez por passe** e não por varredura, porque o que ele lê — os ALCANCES — não
    /// muda entre varreduras: `alcance()` é feito das meias extensões e do comprimento do desvio,
    /// e rodar a peça preserva os dois.
    ///
    /// ⚠️ **O corte é o `g` que MINIMIZA os candidatos previstos**, e o modelo é o mais simples que
    /// a tabela medida valida (ver [`MARGEM_DO_CORTE`]): uma peça pequena vê `9 · ρ · lado²`
    /// vizinhos, cada pequena vê ainda as `g` grandes, e cada grande vê a nuvem activa inteira —
    /// *é este último termo que faz a conta virar, e é ele que a cerca existe para nomear.*
    ///
    /// ⛔ **`g = 0` é o caminho de sempre, ao bit**: sem dispersão de tamanhos nada é promovido, e
    /// a grelha é exactamente a de antes desta wave.
    pub(super) fn planeia(&mut self, foto: &[[f32; 2]], ativo: &[bool], alcances: &[f32]) {
        // ⭐⭐ **O caminho de OMISSÃO sai daqui sem construir NADA** — nem uma grelha a mais, nem uma
        // contagem, nem uma ordenação. É o plano de antes desta wave, e o laço da separação
        // constrói-o uma vez por varredura como sempre fez.
        self.planeia_com(foto, ativo, alcances, duas_camadas_por_ordem());
    }

    /// O [`Grelha::planeia`] com o interruptor das duas camadas **por argumento** — ver
    /// [`crate::Cercas`].
    pub(super) fn planeia_com(
        &mut self,
        foto: &[[f32; 2]],
        ativo: &[bool],
        alcances: &[f32],
        duas: bool,
    ) {
        let alcance_max = alcances.iter().fold(0.0_f32, |a, b| a.max(*b));
        self.planeia_numa_camada(ativo, 2.0 * alcance_max);
        if duas {
            self.planeia_medindo(foto, ativo, alcances);
        }
    }

    /// **O plano de DUAS CAMADAS, decidido pela CONTAGEM REAL** — e **sem ler o ambiente**.
    ///
    /// ⚠️⚠️ **A separação entre esta porta e o [`Grelha::planeia`] é o que mantém os gates a medir a
    /// LEI e não a bandeira:** com o corte a shipar desligado, um gate que entrasse pela porta do
    /// produto passaria a medir o caminho de UMA camada e ficaria **verde a afirmar nada**.
    pub(super) fn planeia_medindo(&mut self, foto: &[[f32; 2]], ativo: &[bool], alcances: &[f32]) {
        let alcance_max = alcances.iter().fold(0.0_f32, |a, b| a.max(*b));
        let uma_camada = |g: &mut Self| {
            g.planeia_numa_camada(ativo, 2.0 * alcance_max);
            g.constroi(foto, ativo);
        };
        uma_camada(self);
        let uma = self.custo_medido();
        // O modelo PROPÕE (a margem é `1`: aqui ele só escolhe QUAL corte vale a pena tentar).
        self.planeia_com_margem(foto, ativo, alcances, 1.0);
        if self.grandes.is_empty() {
            uma_camada(self);
            return;
        }
        self.constroi(foto, ativo);
        // ⭐⭐⭐ **E a CONTAGEM REAL decide.** Ver [`MARGEM_DO_CORTE`].
        #[expect(clippy::cast_precision_loss, reason = "contagens de uma cena")]
        let (d, u) = (self.custo_medido() as f32, uma as f32);
        if d * MARGEM_DO_CORTE >= u {
            uma_camada(self);
        }
    }

    /// **Quantos candidatos esta grelha entregaria, sem os MATERIALIZAR** — três leituras do CSR por
    /// peça pequena, uma por grande.
    ///
    /// ⚠️ Ela é a régua que torna a decisão do plano uma MEDIÇÃO e não uma previsão, e por isso tem
    /// de contar **exactamente** o que o [`Self::vizinhos_de`] devolve — há gate a dobrar as duas.
    pub(super) fn candidatos_previstos(&self) -> usize {
        let mut soma = self.grandes.len() * self.ativas.len();
        if self.cols == 0 {
            return soma;
        }
        for (k, c) in self.celula_de.iter().enumerate() {
            if *c == u32::MAX {
                continue;
            }
            debug_assert!(!self.e_grande[k]);
            let (x, y) = (*c as usize % self.cols, *c as usize / self.cols);
            let (x0, x1) = (x.saturating_sub(1), (x + 1).min(self.cols - 1));
            let (y0, y1) = (y.saturating_sub(1), (y + 1).min(self.rows - 1));
            for yy in y0..=y1 {
                let linha = yy * self.cols;
                soma += (self.inicio[linha + x1 + 1] - self.inicio[linha + x0]) as usize;
            }
            soma += self.grandes.len();
        }
        soma
    }

    /// O [`Grelha::planeia`] com a margem **por argumento**.
    ///
    /// ⚠️⚠️ **Ela é parâmetro e não uma const lida aqui para que o gate da margem possa correr o
    /// CONTROLO dentro de si mesmo** — o mesmo plano com a margem desarmada. *Sem isso, provar que
    /// esta fixtura está no regime em que a margem decide dependia de uma mutação, e um gate cuja
    /// não-vacuidade vive fora dele mede o nada no dia em que a fixtura mudar.*
    pub(super) fn planeia_com_margem(
        &mut self,
        foto: &[[f32; 2]],
        ativo: &[bool],
        alcances: &[f32],
        margem: f32,
    ) {
        let n = foto.len();
        self.e_grande.clear();
        self.e_grande.resize(n, false);
        self.grandes.clear();
        self.ativas.clear();
        #[expect(clippy::cast_possible_truncation, reason = "i < n, e n cabe num u32")]
        for (i, vivo) in ativo.iter().enumerate() {
            if *vivo {
                self.ativas.push(i as u32);
            }
        }
        // Os alcances activos, do maior para o menor — a escada por onde o corte desce.
        let mut ord: Vec<f32> = self
            .ativas
            .iter()
            .map(|&i| alcances[i as usize])
            .collect::<Vec<_>>();
        ord.sort_by(|a, b| b.total_cmp(a));
        self.lado_planeado = 2.0 * ord.first().copied().unwrap_or(0.0);
        let m = ord.len();
        if m < 4 || self.lado_planeado <= 0.0 {
            return; // com meia dúzia de peças não há grelha que pague um corte
        }
        // A densidade da nuvem — o único termo do modelo que vem das POSIÇÕES.
        let (mut x0, mut y0, mut x1, mut y1) = (f32::MAX, f32::MAX, f32::MIN, f32::MIN);
        for (i, q) in foto.iter().enumerate() {
            if ativo[i] {
                (x0, y0) = (x0.min(q[0]), y0.min(q[1]));
                (x1, y1) = (x1.max(q[0]), y1.max(q[1]));
            }
        }
        #[expect(
            clippy::cast_precision_loss,
            reason = "uma contagem de pecas de uma cena"
        )]
        let mf = m as f32;
        let area = (x1 - x0).max(self.lado_planeado) * (y1 - y0).max(self.lado_planeado);
        let densidade = mf / area;
        // Os candidatos que uma peça de alcance `a` vê na malha de lado `2a` — nunca mais do que a
        // nuvem inteira, que é o tecto que a própria grelha já impõe.
        let por_peca = |a: f32| (9.0 * densidade * (2.0 * a) * (2.0 * a)).min(mf);
        let custo = |g: usize| {
            #[expect(clippy::cast_precision_loss, reason = "contagens de uma cena")]
            let (gf, pf) = (g as f32, (m - g) as f32);
            pf * (por_peca(ord[g]) + gf) + gf * mf
        };
        let base = custo(0);
        let (mut melhor_g, mut melhor) = (0usize, base);
        for (g, a) in ord.iter().enumerate().take(m / 2 + 1).skip(1) {
            if *a <= 0.0 {
                break;
            }
            let c = custo(g);
            if c < melhor {
                (melhor_g, melhor) = (g, c);
            }
        }
        // ⚠️ A MARGEM: o modelo só decide quando o ganho previsto é grande — ver [`MARGEM_DO_CORTE`].
        if melhor_g == 0 || melhor * margem >= base {
            return;
        }
        let corte = ord[melhor_g];
        self.lado_planeado = 2.0 * corte;
        #[expect(clippy::cast_possible_truncation, reason = "i < n, e n cabe num u32")]
        for (i, vivo) in ativo.iter().enumerate() {
            if *vivo && alcances[i] > corte {
                self.e_grande[i] = true;
                self.grandes.push(i as u32);
            }
        }
    }

    /// **O plano de UMA CAMADA, de lado dado** — o caminho de antes desta wave, e o **CONTROLO** de
    /// todo gate e de toda sonda que meça o que as duas camadas compram.
    ///
    /// ⚠️ Ele existe para que a comparação seja entre dois PLANOS e não entre duas versões do
    /// ficheiro: *uma medição contra um binário antigo não é reproduzível por quem vier a seguir*.
    ///
    /// ⭐ **E ele deixou de ser `#[cfg(test)]` quando passou a ter consumidor de PRODUTO:** o
    /// [`Grelha::planeia`] mede-o contra o corte e fica com o vencedor, e a porta de bissecção
    /// devolve-o inteiro.
    pub(super) fn planeia_numa_camada(&mut self, ativo: &[bool], lado: f32) {
        let n = ativo.len();
        self.e_grande.clear();
        self.e_grande.resize(n, false);
        self.grandes.clear();
        self.ativas.clear();
        #[expect(clippy::cast_possible_truncation, reason = "i < n, e n cabe num u32")]
        for (i, vivo) in ativo.iter().enumerate() {
            if *vivo {
                self.ativas.push(i as u32);
            }
        }
        self.lado_planeado = lado;
    }

    /// Reconstrói a grelha FINA para esta varredura. ⚠️ **Exige um [`Grelha::planeia`] antes** — é
    /// ele que decide o lado da célula e quem fica de fora.
    pub(super) fn constroi(&mut self, foto: &[[f32; 2]], ativo: &[bool]) {
        let n = foto.len();
        debug_assert_eq!(self.e_grande.len(), n, "constroi sem planeia");
        self.celula_de.clear();
        self.celula_de.resize(n, u32::MAX);
        self.lado = self.lado_planeado;
        if self.lado <= 0.0 {
            (self.cols, self.rows) = (0, 0);
            return;
        }
        // A caixa das células, dobrando o lado até a grelha densa caber (ver o cabeçalho).
        let (cols, rows) = loop {
            let (mut x0, mut y0, mut x1, mut y1) = (i64::MAX, i64::MAX, i64::MIN, i64::MIN);
            for (i, q) in foto.iter().enumerate() {
                if !ativo[i] || self.e_grande[i] {
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
            if !ativo[i] || self.e_grande[i] {
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
        // ⭐ Uma peça GRANDE pode tocar qualquer outra — a lista dela é a nuvem activa inteira, e
        // ela **já vem crescente**, logo não há o que ordenar.
        if self.e_grande[k] {
            out.extend_from_slice(&self.ativas);
            return;
        }
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
        // ⚠️ E as GRANDES, que não vivem na malha fina. Sem duplicados **por construção** (a
        // partição é exclusiva), logo a soma de `k` não conta ninguém duas vezes.
        out.extend_from_slice(&self.grandes);
        out.sort_unstable();
    }

    /// Quantas CÉLULAS a malha fina tem — o trabalho `O(células)` que o [`Grelha::constroi`] paga
    /// **por varredura** (zerar o `inicio` e correr a soma acumulada).
    pub(super) fn celulas(&self) -> usize {
        self.cols * self.rows
    }

    /// **O CUSTO MEDIDO de uma varredura com esta grelha**, nas duas moedas dela: os candidatos que
    /// ela entrega mais as células que ela obriga a varrer.
    ///
    /// ⚠️⚠️ **A 1.ª redacção da decisão contava só a primeira**, e o report do dono de 19/09 obrigou
    /// a medir a segunda: a `1 000` candidatos parados, quadruplicar as células **dobra o relógio**
    /// de uma varredura. Ver [`CELULAS_POR_CANDIDATO`].
    pub(super) fn custo_medido(&self) -> usize {
        self.candidatos_previstos() + self.celulas() / CELULAS_POR_CANDIDATO
    }

    /// Quantas peças o plano promoveu a GRANDE — o número que NOMEIA a causa de uma cena cara.
    pub(super) fn grandes(&self) -> usize {
        self.grandes.len()
    }
}

/// **Quantos CANDIDATOS a grelha entrega, somados sobre as peças** — o multiplicador do custo de
/// uma varredura, e o número que separa *«muitas peças»* de *«uma pilha apertada»*.
///
/// ⚠️ Ele corre a MESMA grelha do produto, uma vez, sobre a configuração de entrada — não é uma
/// segunda resposta à pergunta *«quem está perto de quem?»*.
#[must_use]
pub fn candidatos(colisores: &[Option<super::Colisor>], p: &[[f32; 2]], pesos: &[f32]) -> usize {
    candidatos_e_grandes(colisores, p, pesos).0
}

/// **O ALCANCE de cada peça, ou `0` para quem não entra** — a porta única de que o plano da grelha
/// e o laço de separação leem a mesma escada.
pub(super) fn alcances_de(colisores: &[Option<super::Colisor>], ativo: &[bool]) -> Vec<f32> {
    (0..ativo.len())
        .map(|i| {
            if ativo[i] {
                colisores[i].map_or(0.0, |c| c.alcance())
            } else {
                0.0
            }
        })
        .collect()
}

/// Como o [`candidatos`], e ainda **quantas peças o plano promoveu a GRANDE** — o número que separa
/// *«uma pilha apertada»* de *«uma peça grande a inflar a grelha de todas»* (ver o cabeçalho).
#[must_use]
pub fn candidatos_e_grandes(
    colisores: &[Option<super::Colisor>],
    p: &[[f32; 2]],
    _pesos: &[f32],
) -> (usize, usize) {
    let n = p.len();
    let ativo: Vec<bool> = (0..n)
        .map(|i| super::ativo(p[i], colisores[i].as_ref()))
        .collect();
    let alcances = alcances_de(colisores, &ativo);
    if alcances.iter().fold(0.0_f32, |a, b| a.max(*b)) <= 0.0 {
        return (0, 0);
    }
    let mut grade = Grelha::default();
    grade.planeia(p, &ativo, &alcances);
    grade.constroi(p, &ativo);
    let mut viz: Vec<u32> = Vec::new();
    let mut soma = 0usize;
    for k in 0..n {
        grade.vizinhos_de(k, &mut viz);
        soma += viz.len();
    }
    (soma, grade.grandes())
}
