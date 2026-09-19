//! ⭐⭐⭐ **A MÁSCARA DE ALCANCE** — o carimbo deixa de saltar para o que a
//! superfície não liga.
//!
//! # O defeito, medido
//!
//! Um dab junta os vértices dentro de uma **esfera** e pesa cada um pela
//! distância **pelo ar**. Numa peça com duas partes vizinhas — dois dedos, dois
//! vincos, uma dobra — a esfera alcança o outro lado. Medido pela
//! [`sonda_do_falloff_pela_superficie`](../tests/sonda_do_falloff_pela_superficie.rs)
//! em duas esferas com folga `0,05`: **`43,6 %` a `47,9 %` do peso do carimbo
//! cai no dedo ERRADO**. O controlo convexo lê `0,00 %`.
//!
//! # ⭐⭐ O que este módulo NÃO é, e é isso que o torna seguro
//!
//! ⛔ **Ele não troca a distância do falloff.** O peso continua a ser
//! `falloff(|p − c| / R)`, **ao bit**, e por isso toda a paridade com os
//! oráculos (a do SculptGL, a do tecido) fica intacta **por construção** — não
//! por promessa.
//!
//! A razão é que a pergunta que decide não é *«que distância?»* mas *«a
//! superfície liga isto?»*, e alcance não é distância.
//!
//! # ⭐⭐⭐⭐ A PAREDE FINA deixou de ficar de fora — e a premissa que a excluía
//! está MORTA
//!
//! Este cabeçalho dizia, até 2026-09-19:
//!
//! > *«Uma parede fina tem razão `superfície/ar = π/2 ≈ 1,57`, que é **menor**
//! > que o viés `√2` deste passeio ⇒ ele não a mede. Curá-la pede geodésica a
//! > sério (método do calor, ou MMP), e é outra wave com espec própria.»*
//!
//! As duas metades caíram por medição, e nenhuma da maneira que se esperava:
//!
//! * ⛔ **A razão `π/2` é do TUBO e não da parede fina.** Num tubo de raio `m` a
//!   superfície mede `π m` e o ar `2 m`, logo a razão é **fixa**; numa CHAPA o ar
//!   é a espessura `t` e a superfície é *ir à beira e voltar* (`2d + t`), logo a
//!   razão é **livre** e cresce sem limite à medida que a peça afina. Medido numa
//!   barbatana de `0,06` com o carimbo a `0,30` da beira: a superfície mede
//!   `1,65 × R` e a razão **`11,0`**, com **`98,8 %`** do que a face da frente
//!   andou a aparecer na de trás.
//! * ⛔⛔ **E não foi preciso geodésica nenhuma.** A marcha de Kimmel–Sethian foi
//!   construída ([`ph2d_mesh::Geodesica`], hoje atrás da `test-support`) e a
//!   medição mandou-a embora: nesta lei ela dá a **mesma tabela** que o passeio
//!   por arestas em toda peça aprovada (`0,00 pp`), corta **menos** o defeito e
//!   custa **`4,3 ×`**. *O que faltava não era precisão — era a PERGUNTA.*
//!
//! # ⭐⭐⭐ A pergunta certa é a RAZÃO, e não uma distância
//!
//! A régua de sempre é *«a superfície alcança isto dentro de `k × R`?»*, e ela
//! mede **a mesma grandeza que o raio do pincel** — logo apertá-la come a ORLA
//! da pegada antes de chegar ao defeito. Medido: baixar o [`ALCANCE_TECTO`] de
//! `2,00` para `1,50` corta só vértices com `ar/R` entre `0,80` e `1,00`, e o
//! placar de oráculo do `Scene Project` desce de `13` para `5`.
//!
//! A régua que separa é `superfície / ar` ([`RAZAO_MAXIMA`]), e o vão é de uma
//! ordem de grandeza: **`11,0`** nas costas de uma barbatana contra **`~1,55`**
//! na orla da pegada de uma cratera. ⭐ A classe já estava escrita na sonda de
//! 2026-09-10 com o nome `LONGE`; o produto é que nunca a usara.
//!
//! # ⛔⛔ O TECIDO não passa por aqui, e é estrutural
//!
//! O [`crate::Verb::Cloth`] **desvia antes do `dab_core`**
//! ([`crate::stroke_symmetry`]: ele é dono da própria expansão de simetria) ⇒ os
//! `86` traços do corpus do tecido não podem ser tocados por esta máscara. Não é
//! uma cerca que alguém tem de lembrar — é o roteamento.

use ph2d_mesh::Mesh;

/// **Quantos raios de pincel a superfície tem de andar antes de se chamar
/// «não alcança»** — `1,50 × R`.
///
/// ⭐⭐ **DERIVADO de um planalto MEDIDO com a lei que shipa**, e a tabela é a
/// da sonda `sonda_da_parede_fina::com_que_tecto_o_corte_nao_toca_o_que_a_superficie_alcanca`
/// (quanto do PESO do carimbo o corte leva):
///
/// | factor | esfera (APROVADO) | tubo `0,10` (APROVADO) | esfera rugosa (APROVADO) | dois dedos | **a BARBATANA** |
/// |---|---|---|---|---|---|
/// | `1,00×` | `0,00 %` | `0,46 %` ⛔ | `1,68 %` ⛔ | `47,1 %` | `46,96 %` |
/// | `1,20×` | `0,00 %` | `0,02 %` ⛔ | `0,14 %` ⛔ | `47,1 %` | `43,31 %` |
/// | `1,30×` | `0,00 %` | `0,01 %` ⛔ | `0,00 %` | `47,1 %` | `40,06 %` |
/// | **`1,50×`** | **`0,00 %`** | **`0,00 %`** | **`0,00 %`** | **`47,1 %`** | **`31,36 %`** |
/// | `2,00×` (o de antes) | `0,00 %` | `0,00 %` | `0,00 %` | `47,1 %` | `7,54 %` ⛔ |
/// | `3,00×` | `0,00 %` | `0,00 %` | `0,00 %` | `47,1 %` | `0,00 %` ⛔ |
///
/// ⇒ `1,50` é a **borda esquerda do planalto**: o menor factor que corta
/// **exactamente zero** peso de todas as linhas aprovadas.
///
/// ⭐ **E os `31,36 %` da barbatana batem uma régua INDEPENDENTE:** a geodésica
/// exacta por desdobramento daquela chapa diz que **`31,9 %`** do movimento do
/// carimbo cai onde a superfície não alcança (sonda
/// `quanto_as_costas_andam_quando_a_frente_e_esculpida`). *Duas leis diferentes
/// a chegar ao mesmo número é o que torna o tecto uma medição e não uma escolha.*
///
/// ⛔⛔ **A tabela de 2026-09-10 dizia `2,00` e não estava errada — a LEI por
/// baixo dela é que mudou.** Aquele número existia para caber o viés `√2` do
/// **passeio por arestas** (ali a esfera convexa lia `1,94 %`–`5,09 %` de corte
/// a `1,00×`, e era o viés que ela media, não a peça). Com a marcha de
/// [`ph2d_mesh::Geodesica`] o viés desaparece — a esfera lê `0,00 %` **já a
/// `1,00×`** —, e o tecto pôde descer até onde o defeito real mora.
/// *Uma constante derivada de uma lei não sobrevive à troca dessa lei.*
///
/// ⚠️ **Porque não `1,00×`, que seria a definição limpa de «o pincel alcança»:**
/// a marcha é de **primeira ordem** e sobrestima até `1,12` numa malha
/// anisotrópica (ver [`ph2d_mesh::geodesica`]), e uma peça esculpida tem arestas
/// irregulares — as duas coisas juntas fazem o corte comer a orla da pegada, que
/// é o que a coluna `1,00×` mede na esfera rugosa. *O tecto tem de absorver o
/// erro do instrumento, e `1,50` é onde a medição diz que ele acaba.*
pub const ALCANCE_TECTO: f32 = 2.0;

/// ⭐⭐⭐⭐ **Quantas vezes o caminho pela SUPERFÍCIE pode ser mais longo que o
/// caminho pelo AR antes de o vértice estar «do outro lado» de alguma coisa** —
/// `3,50`.
///
/// # De onde sai o número
///
/// Do planalto medido (sonda `com_que_razao_o_corte_separa_a_parede_da_orla`),
/// **com a rugosidade da peça varrida até deixar de importar** — a borda
/// move-se com ela, e uma constante posta na borda de UMA rugosidade mede essa
/// rugosidade:
///
/// | peça aprovada | factor em que ela lê `0,00 %` de corte |
/// |---|---|
/// | esfera lisa · tubo · `sculpt_sphere` · cratera `0,25` e `0,50` | `≤ 2,00` |
/// | esfera rugosa `amp 0,08` | `2,00` |
/// | esfera rugosa `amp 0,12` | `2,50` |
/// | esfera rugosa `amp 0,16` | `3,00` |
/// | esfera rugosa `amp 0,20` | **`3,50`** |
/// | esfera rugosa `amp 0,24` | **`3,50`** ← pára de se mexer |
///
/// e a barbatana ainda lê **`34,82 %`** de peso cortado a `3,50×`, contra os
/// `9,86 %` que o tecto absoluto sozinho corta.
///
/// ⚠️⚠️ **Ele foi derivado DUAS vezes**, e a segunda é a que vale: a primeira
/// varredura correu com a marcha de Kimmel–Sethian e deu `3,50`; quando a
/// medição mostrou que a marcha não se paga, a constante foi re-derivada **com o
/// passeio por arestas, que é a lei que shipa** — e deu `3,50` outra vez.
/// *Uma constante derivada de uma lei não sobrevive à troca dessa lei, e esta
/// sobreviveu porque foi re-medida, não porque se acreditou nela.*
///
/// # ⛔ O PISO que não existe, e a medição que o apagou
///
/// A primeira redacção trazia um piso (*«só perguntar a razão a quem está a mais
/// de `0,15 × R` do cursor»*) para a razão não explodir onde o `ar` tende a
/// zero. Varrida com piso `0,15` e com piso `0`, a tabela sai **idêntica célula
/// a célula** ⇒ ele é **inerte por geometria**: à escala de uma aresta a
/// superfície é plana, logo `sup ≈ ar` e a razão nunca chega perto de `3,5`
/// junto do cursor. *Um knob que nenhuma fixtura pode acordar é peso morto* — a
/// mesma lei que apagou o `SEMENTE_RECUO` deste ficheiro.
///
/// # ⚠️ O que ela NÃO faz
///
/// Ela não dá ao pincel um alcance de `R` **pela superfície**: um vértice a
/// `1,5 × R` de caminho mas com o ar grande (deslocado DE LADO, junto de uma
/// beira) continua a entrar, porque a razão dele é baixa. Na barbatana isso é a
/// orla que sobra na figura — `26 %` do carimbo contra os `43 %` de antes.
/// ⭐ **A causa de não se poder apertar está diagnosticada:** a máscara trima a
/// PEGADA, e a pegada alimenta o ajuste de plano de verbos como o
/// `Scene Project`. Separar *quem se MOVE* de *qual é a superfície local* é wave
/// própria.
pub const RAZAO_MAXIMA: f32 = 3.5;

/// ⭐⭐⭐⭐ **A TERCEIRA condição: a folha que o artista ESTÁ A VER.**
///
/// Um vértice cuja normal aponta para **longe do olho** por mais do que isto não
/// é da folha que o cursor tocou, e sai da pegada. `n · olho > LIMIAR` corta
/// (o `olho` vai do olho PARA a superfície, logo a folha da frente lê `≈ −1`).
///
/// # ⛔⛔⛔ Porque a [`RAZAO_MAXIMA`] não bastava — e nenhum valor dela bastaria
///
/// Report do dono (2026-09-19): *«ainda não ficou bom»* e, a seguir, *«funciona
/// para tamanho menor do pincel»*. Reproduzido na barbatana da cena `=50`, as
/// costas movem-se **isto**, em fracção do que a frente moveu:
///
/// | `R` | `d = 0,10` | `d = 0,25` | `d = 0,50` |
/// |---|---|---|---|
/// | `0,20` | `79,4 %` | `1,3 %` | `0,0 %` |
/// | `0,40` | `106,1 %` | `113,1 %` | `145,1 %` |
/// | `0,65` (o da foto) | `104,6 %` | `103,2 %` | `104,3 %` |
/// | `0,90` | `103,6 %` | `103,8 %` | `104,2 %` |
///
/// ⭐ **O mecanismo é aritmético e não uma afinação.** Para um ponto de trás a
/// `L` de lado do cursor, com o cursor a `d` da beira de uma chapa de espessura
/// `t`: o **ar** mede `√(L² + t²)` e a **superfície** mede `2d + t + L` ⇒
///
/// > com `L ≫ t` a razão tende para **`1`** — que é exactamente o valor que um
/// > ponto da FRENTE a `L` de lado também tem.
///
/// Medido (`R = 0,65`, `d = 0,25`, um dab), a razão dos vértices de trás que
/// passam cai de `2,83` a `1,82` conforme o lateral cresce, **toda ela abaixo de
/// `3,5`** — e legitimamente: a superfície de facto os alcança. ⇒ *a razão
/// responde «a superfície alcança?» e a pergunta do artista é «é a folha que eu
/// estou a ver?»*. **Um pincel grande derrota a razão por construção.**
///
/// # De onde sai o `0,30`
///
/// Do vale MEDIDO, com o lado do DEFEITO e o lado APROVADO na mesma tabela —
/// `%` do que a lei de hoje mantém que esta condição cortaria, `R = 0,65`:
///
/// | peça | `0,00` | `0,20` | **`0,30`** | `0,45` | `0,60` |
/// |---|---|---|---|---|---|
/// | ⛔ barbatana (a cena `=50`) | `42,3` | `42,3` | **`42,3`** | `42,3` | `42,3` |
/// | ⛔ casca fina, dab na beira | `49,0` | `49,0` | **`49,0`** | `43,4` | `32,1` |
/// | ✅ esfera lisa · `sculpt_sphere` · cratera `0,50` · rugosa `0,16` · cilindro | `0,0` | `0,0` | **`0,0`** | `0,0` | `0,0` |
/// | ✅ rugosa `0,24` | `1,5` | `0,0` | **`0,0`** | `0,0` | `0,0` |
///
/// ⚠️ **O `1,5 %` da rugosa `0,24` é o que a barra existe para poupar:** são `3`
/// vértices de `197`, com `dot` máximo de `0,117` — o lábio de uma ruga funda,
/// um fio para lá do horizonte. Abaixo de `0,20` eles são comidos; a partir de
/// `0,60` o lado do defeito começa a perder. **O `0,30` é o meio do planalto.**
///
/// ⚠️⚠️ **E a fixtura que decide a barra tem de ser CURVA:** numa chapa a normal
/// das costas é `+1` EXACTO, logo toda barra abaixo de `1` a apanha e a coluna
/// não diz nada. Quem mede a folga é a casca. ⛔ *E a 1.ª redacção desta tabela
/// pôs o dab no POLO da casca, onde a beira fica a `~2,4` de superfície e o
/// [`ALCANCE_TECTO`] já corta tudo: as duas cascas liam `0,0 %` e a tabela media
/// o nada* — a terceira vez que esta wave paga a mesma armadilha.
pub const NORMAL_LIMIAR: f32 = 0.30;

/// ⭐⭐⭐⭐ **A MÁSCARA DECIDE QUEM ENTRA NO TRAÇO; ELA NUNCA DECIDE QUEM SAI.**
///
/// # Os dois reports que a escreveram (2026-09-19)
///
/// 1. *«Snake Hook se dá muito mal com `Connected Only`, deformando com má
///    remesh ou má topologia a face POSTERIOR do traço»* — a bossa puxada saía
///    com as costas pretas e um refino explodido.
/// 2. *«algumas vezes correto, algumas vezes bugado»* — depois da 1.ª cura.
///
/// # ⭐⭐⭐ O mecanismo, que é UM e não três
///
/// As três condições da máscara medem a malha **VIVA**, e um gancho **muda a
/// peça debaixo delas**: puxa um tubo (cuja face de trás se vira para longe do
/// olho **por construção**) e ESTICA a superfície (o que afasta o barro da
/// semente ao longo dela). ⇒ o veredito muda a meio do gesto, e *quem já andava
/// pára enquanto o vizinho continua* — que é um rasgo. O passe de topologia
/// depois parte a aresta longa que o rasgo abriu: o refino da foto.
///
/// # A atribuição, condição a condição
///
/// Contando quantos vértices **que o traço já capturava** cada condição retirou
/// ([`crate::stroke::gancho_sonda`], esfera de escultura, máscara ligada):
///
/// | regime | tecto | razão | normal |
/// |---|---|---|---|
/// | gancho tangencial ou a 45°, qualquer raio | `0` | `0` | `0` |
/// | ⛔ gancho **oblíquo**, `raio 0,12` | **`9`** | `0` | `0` |
/// | ⛔ gancho **oblíquo**, `raio 0,25` | **`47`** | `0` | `0` |
/// | ⛔ gancho **oblíquo**, `raio 0,45` | **`24`** | `0` | `0` |
///
/// ⚠️ **A 1.ª cura tratou só a NORMAL** (dava-lhe a normal congelada do
/// `capture`) e por isso o defeito ficou *«algumas vezes»*: o que sobrava era o
/// **TECTO DO PASSEIO**, que mede na superfície que o próprio gancho estica.
/// *Curar uma condição de cada vez deixa o report vivo com outra cara.*
///
/// # A lei
///
/// Um vértice que o traço **já capturou** não volta a ser julgado: a máscara
/// filtra **quem entra**. ⭐ A pergunta *«este barro já anda?»* já tem resposta
/// `O(1)` — é o carimbo de época do congelamento do UNDO (`stamp`/`epoch`) —, e
/// um vértice **nascido** no refino herda-a dos pais pelo canal `grow_with` que
/// já existe ⇒ *a lei atravessa a topologia dinâmica sem uma linha nova*.
///
/// ⚠️ **O lado do defeito de ontem fica intacto, e é estrutural:** numa parede
/// fina as costas **nunca são capturadas** (o 1.º dab já as corta), logo
/// continuam a ser julgadas dab após dab. *A cura muda quem já estava dentro,
/// nunca quem nunca entrou* — e um vértice só chega a ser capturado depois de
/// PASSAR pela máscara.
pub(crate) struct MemoriaDoTraco<'a> {
    /// `stamp[v] == epoca` ⇔ o traço já capturou `v` — ou seja, *este barro já
    /// anda*.
    pub stamp: &'a [u32],
    pub epoca: u32,
}

impl MemoriaDoTraco<'_> {
    /// **Este barro já anda?**
    fn anda(&self, v: u32) -> bool {
        let vi = v as usize;
        vi < self.stamp.len() && self.stamp[vi] == self.epoca
    }
}

// ⛔⛔ **UMA CONSTANTE QUE FOI CONSTRUÍDA, MEDIDA E REMOVIDA — e o registo fica.**
//
// Havia aqui um `SEMENTE_RECUO = 0,25`: o ponto de semeadura era deslocado na
// direcção do olho antes de se procurar o vértice mais próximo, para defender
// da armadilha *«o mais próximo do centro do dab está na folha ERRADA quando as
// duas se tocam a menos de uma aresta»*.
//
// ⭐ **A mutação `0,25 → 0,0` SOBREVIVEU a tudo** — os quatro gates deste módulo
// e a suíte inteira da crate (`435` verdes, `exit 0`). Das três leituras de uma
// mutação sobrevivente, a que se aplica é a terceira: *nenhuma fixtura pode
// produzir o fenómeno*, e a razão é geométrica e não uma falha de imaginação:
//
// > O centro do dab está **SOBRE** a superfície que o raio atingiu. Para duas
// > folhas paralelas — placa fina, dois dedos, uma dobra — todo vértice da
// > folha de LÁ é um vértice da folha de CÁ mais um deslocamento na espessura
// > ⇒ `√(lateral² + espessura²) > √(lateral²)`. **A folha de cá ganha sempre**,
// > para qualquer espessura positiva.
//
// ⇒ a semente é o vértice mais próximo do centro, sem recuo nenhum. *Um knob
// que nenhuma fixtura pode acordar é peso morto, e §0.0 proíbe escrever um
// número que a medição não defende.*

/// Os buffers da marcha, reusados entre dabs.
///
/// ⚠️⚠️ **A marca de ÉPOCA é o que faz o custo ser `O(visitados)` e não
/// `O(malha)`** — ela vive hoje dentro da [`Geodesica`], pela mesma razão: um
/// `vec![∞; n]` por dab escreve a malha inteira a cada evento de ponteiro, e
/// numa peça de um milhão de vértices isso é ~`1 ms` **por dab** só para limpar.
#[derive(Clone, Debug, Default)]
pub(crate) struct Alcance {
    dist: Vec<f32>,
    marca: Vec<u32>,
    epoca: u32,
    fila: std::collections::BinaryHeap<Perto>,
    /// Quantos vértices a última varredura FIXOU.
    ///
    /// ⚠️⚠️ **Ela existe porque o [`ALCANCE_TECTO`] deixou de ter efeito na
    /// SAÍDA e passou a ter efeito só no TRABALHO** — desde que a lei é a
    /// [`RAZAO_MAXIMA`], apagar o tecto não muda um vértice do resultado (uma
    /// mutação sobreviveu a provar isso) e passa a varrer a MALHA INTEIRA por
    /// dab. *Um tecto que só governa custo precisa de uma régua de custo, e uma
    /// CONTAGEM é determinista onde um relógio é uma flake de carga.*
    #[cfg(test)]
    visitados: usize,
    /// ⚠️ **Quantos vértices que o traço JÁ CAPTURAVA cada condição retirou na
    /// última varredura** — `[tecto, razão, normal]`. Ela existe para responder
    /// à pergunta do report *«algumas vezes correto, algumas vezes bugado»*:
    /// *alguma condição ainda tira barro que já está a andar?* — porque **é
    /// isso, e só isso, que rasga**.
    #[cfg(test)]
    tirou_do_traco: [usize; 3],
}

/// Uma entrada da fila — `f32` não é `Ord`, e o `BinaryHeap` é max-heap.
#[derive(Clone, Debug, PartialEq)]
struct Perto(f32, u32);
impl Eq for Perto {}
impl Ord for Perto {
    fn cmp(&self, o: &Self) -> std::cmp::Ordering {
        o.0.partial_cmp(&self.0)
            .unwrap_or(std::cmp::Ordering::Equal)
    }
}
impl PartialOrd for Perto {
    fn partial_cmp(&self, o: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(o))
    }
}

fn dist2(a: [f32; 3], b: [f32; 3]) -> f32 {
    let d = [a[0] - b[0], a[1] - b[1], a[2] - b[2]];
    d[0] * d[0] + d[1] * d[1] + d[2] * d[2]
}

impl Alcance {
    /// ⚠️ **Só para o gate da volta da época** — ela é a única maneira de correr
    /// o wrap sem esperar quatro bilhões de dabs, e a cerca que ela testa é
    /// exactamente a que o [`ph2d_mesh::QueryScratch`] documenta.
    #[cfg(test)]
    pub(crate) fn forcar_epoca_para_teste(&mut self, e: u32) {
        self.epoca = e;
    }

    /// Quantos vértices a última varredura fixou — ver [`Self::visitados`].
    #[cfg(test)]
    pub(crate) fn visitados_no_teste(&self) -> usize {
        self.visitados
    }

    /// O que a última varredura tirou a quem já andava — ver
    /// [`Self::tirou_do_traco`].
    #[cfg(test)]
    pub(crate) fn tirou_do_traco_no_teste(&self) -> [usize; 3] {
        self.tirou_do_traco
    }

    /// **CORTA da `pegada` quem a superfície não alcança** dentro de
    /// `tecto = ALCANCE_TECTO × raio`, e devolve quantos saíram.
    ///
    /// ⚠️ **A `pegada` é a lista VIVA do dab**, e a ordem dos que ficam é
    /// preservada (`retain`): o `dab_core` percorre-a e a ordem de visita é lei
    /// noutros sítios desta crate.
    pub(crate) fn corta(
        &mut self,
        mesh: &Mesh,
        centro: [f32; 3],
        olho: [f32; 3],
        raio: f32,
        pegada: &mut Vec<u32>,
        memoria: Option<&MemoriaDoTraco<'_>>,
    ) -> usize {
        if pegada.len() < 2 || raio <= 0.0 {
            return 0;
        }
        let pos = mesh.positions();
        if self.marca.len() != pos.len() {
            self.marca = vec![0; pos.len()];
            self.dist = vec![0.0; pos.len()];
            self.epoca = 0;
        }
        self.epoca = self.epoca.wrapping_add(1);
        if self.epoca == 0 {
            self.epoca = 1;
            self.marca.fill(0);
        }

        // A semente: o vértice da pegada mais próximo do centro. ⚠️ **Da PEGADA e
        // não da malha** — o centro está sobre a superfície, logo o mais próximo
        // está na pegada por construção, e varrer a malha custaria `O(n)` por
        // dab. Ver a nota apagada acima sobre o recuo que não era preciso.
        let mut semente = pegada[0];
        let mut melhor = f32::INFINITY;
        for &v in pegada.iter() {
            let d = dist2(pos[v as usize], centro);
            if d < melhor {
                melhor = d;
                semente = v;
            }
        }

        let tecto = ALCANCE_TECTO * raio;
        self.fila.clear();
        self.marca[semente as usize] = self.epoca;
        self.dist[semente as usize] = 0.0;
        self.fila.push(Perto(0.0, semente));
        let viz = &mesh.adjacency().vert_verts;
        #[cfg(test)]
        {
            self.visitados = 0;
        }
        while let Some(Perto(du, u)) = self.fila.pop() {
            if du > self.dist[u as usize] {
                continue;
            }
            #[cfg(test)]
            {
                self.visitados += 1;
            }
            for &v in viz.neighbours(u as usize) {
                let nd = du + dist2(pos[u as usize], pos[v as usize]).sqrt();
                if nd > tecto {
                    continue;
                }
                let vi = v as usize;
                if self.marca[vi] != self.epoca || nd < self.dist[vi] {
                    self.marca[vi] = self.epoca;
                    self.dist[vi] = nd;
                    self.fila.push(Perto(nd, v));
                }
            }
        }

        // ⚠️ **O olho pode chegar degenerado** (um `Dab` construído por uma
        // fixtura antiga, um raio sem direcção) — ali a terceira condição
        // **desliga-se** em vez de cortar ao acaso, que é o valor conservador.
        let le = (olho[0] * olho[0] + olho[1] * olho[1] + olho[2] * olho[2]).sqrt();
        let olho_bom = le.is_finite() && le > 1e-6;
        let olho_u = if olho_bom {
            [olho[0] / le, olho[1] / le, olho[2] / le]
        } else {
            [0.0, 0.0, 0.0]
        };
        let nrm = mesh.normals();
        let tem_normais = nrm.len() == pos.len();

        // ⛔⛔⛔ **SE O CORTE ESVAZIA A PEGADA, ELE NÃO CORRE — e isto não é um
        // remendo, é a lei a dizer o que ela é.**
        //
        // A terceira condição escolhe entre DUAS folhas. Quando *toda* a pegada
        // aponta para longe do olho não há duas — há uma, e o olho discorda dela
        // (uma fixtura que carimba o polo de baixo com o olho de cima, o passe do
        // filtro, um verbo que pega a peça pelas costas de propósito). Cortar ali
        // entrega um pincel que **não faz nada**, que é o defeito que esta linha
        // curou três vezes noutros sítios.
        //
        // ⚠️ **Quem escreveu esta cerca foram SEIS gates vermelhos**, todos com a
        // mesma mensagem (*«o dab não moveu nada»*) — entre eles o
        // `a_footprint_entirely_facing_away_still_fits_a_sane_plane`, que é
        // exactamente este caso com o nome dele.
        // ⛔⛔⛔ **A CERCA JULGA A PEGADA INTEIRA — e a versão que julgava só os
        // CANDIDATOS foi escrita, MEDIDA e revertida no mesmo dia.**
        //
        // Ela parecia a leitura conservadora (*«quem já anda está fora do
        // alcance do corte, logo não devia votar»*) e abria um buraco: num traço
        // **PARADO** sobre uma parede fina, ao 2.º dab a frente já está toda
        // capturada, logo os únicos candidatos são as COSTAS — todas viradas ao
        // contrário — e a cerca deixava de armar. Medido: as costas andavam
        // `0,0442` a partir do segundo dab. ⇒ gate
        // `um_traco_que_para_nao_deixa_a_parede_fina_entrar`, e a mutação que a
        // reverte sangra nele.
        //
        // *A memória diz quem não pode ser CORTADO; ela não apaga o que a pegada
        // SABE* — e o barro que já anda é a prova de que existe uma folha
        // virada ao artista.
        let dot = |v: u32| -> f32 {
            let n = nrm[v as usize];
            n[0] * olho_u[0] + n[1] * olho_u[1] + n[2] * olho_u[2]
        };
        let corta_normal =
            olho_bom && tem_normais && pegada.iter().any(|&v| dot(v) <= NORMAL_LIMIAR);

        let antes = pegada.len();
        #[cfg(test)]
        let mut tirou = [0usize; 3];
        let (marca, dd, epoca) = (&self.marca, &self.dist, self.epoca);
        pegada.retain(|&v| {
            let vi = v as usize;
            let anda = memoria.is_some_and(|m| m.anda(v));
            // ⭐⭐⭐⭐ **A MÁSCARA DECIDE QUEM ENTRA NO TRAÇO; ELA NUNCA DECIDE
            // QUEM SAI** — ver [`MemoriaDoTraco`].
            if anda {
                return true;
            }
            if marca[vi] != epoca {
                #[cfg(test)]
                if anda {
                    tirou[0] += 1;
                }
                return false;
            }
            if dd[vi] > RAZAO_MAXIMA * dist2(pos[vi], centro).sqrt() {
                #[cfg(test)]
                if anda {
                    tirou[1] += 1;
                }
                return false;
            }
            // ⭐⭐⭐ **E a folha que o artista vê** — ver [`NORMAL_LIMIAR`] e,
            // para quem ela pode julgar, [`MemoriaDoTraco`].
            if corta_normal && dot(v) > NORMAL_LIMIAR {
                #[cfg(test)]
                if anda {
                    tirou[2] += 1;
                }
                return false;
            }
            true
        });
        #[cfg(test)]
        {
            self.tirou_do_traco = tirou;
        }
        antes - pegada.len()
    }
}

#[cfg(test)]
#[path = "dab_alcance_tests.rs"]
mod tests;

/// **Quem decide a folha que o artista vê** — o irmão (`#[path]`) dos gates
/// acima, cortado por responsabilidade quando o ficheiro cruzou o tecto de LOC.
/// Ver [`olho`].
#[cfg(test)]
#[path = "dab_alcance_olho_tests.rs"]
mod olho;
