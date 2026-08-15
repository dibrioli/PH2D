//! **O SPRAY — um ponto do caminho deixa `n` MARCAS** (plano 38 W5, pesquisa no
//! [doc 37 §3.4](../../../../docs/Painter/37_pesquisa_tracos_procedurais.md)).
//!
//! Krita *Spray*, Photoshop *Scattering*, Painter *Image Hose*: um dab vira `n` cópias espalhadas,
//! cada uma com o seu tamanho, o seu ângulo e a sua cor.
//!
//! ⚠️ **A metade que faltava era a CONTAGEM, e só ela.** O `jitter` (posição), o `jitter_scale` e o
//! `jitter_rotate` deste motor já são o *Spread*, o *Size Jitter* e o *Angle Jitter* de um spray —
//! o doc 37 diz a frase exata: *"deslocam **um** dab … é a diferença entre tremer e espalhar"*.
//! Dar-lhes gêmeos numa segunda casa seria a **segunda porta** para a mesma pergunta, e esta casa já
//! removeu um caso idêntico (o *Random Angle* por-slot de Shape+Grain saiu em 2026-07-19 porque o
//! Jitter Rotate do traço era o superset). Logo o spray traz **um** número:
//! [`crate::BrushSpec::spray_count`].
//!
//! ⚠️ **E é por isso que ele NÃO é um [`crate::LineKind`]**, embora o plano o listasse como o quinto
//! membro daquele dropdown. `Speed`, `Sketchy` e `Wire` são leis **da linha** e mutuamente
//! exclusivas — cada uma responde *que forma o traço tem*. Uma contagem de marcas é um **multiplicador
//! da emissão** e compõe com todas elas: um chicote sprayado e um rabisco sprayado são pedidos
//! legítimos, e pôr o spray naquele dropdown os tornaria **inexprimíveis**. É também o modelo do
//! Photoshop, onde *Scattering* é um painel que empilha com o resto do pincel, e não um modo.
//!
//! ## A porta
//!
//! [`Stroke::stamp_at`] é a **única** porta por onde um ponto do CAMINHO vira marca. Ela funde as
//! duas metades que sempre andaram juntas (`dab_at` seguido de `emit`, em cinco sítios) porque o
//! spray precisa das duas ao mesmo tempo: o arremesso, os fios e o vão do arremesso são fatos do
//! caminho e acontecem **uma vez**; o jitter e o carimbo são fatos da marca e acontecem `n` vezes.
//!
//! ⚠️ **A cópia 0 é o dab de sempre**, emitida pela porta de sempre ⇒ com `spray_count = 1` nada
//! aqui alcança um pixel e o traço é **byte-idêntico**. As cópias extra saem depois dela, logo o
//! dab do caminho fica por BAIXO da própria nuvem — a ordem entre irmãos de uma nuvem é arbitrária,
//! e esta é a que mantém o neutro trivialmente verificável.
//!
//! ⚠️ **Ele herda Symmetry e Tiling de graça** porque cada cópia sai por
//! [`crate::symmetry::push_symmetric`], exatamente como o dab base — o fan-out fica no mesmo
//! estrangulamento onde a Symmetry já se pendura, e o Tiling é a jusante (é do carimbo, no tool).
//! Se tivesse sido preciso um caminho próprio, o desenho estaria errado.

use super::{Dab, Stroke};

/// **Teto do `Count` — MEDIDO pela porta do produto, com a tabela ao lado.**
///
/// O recurso é o tempo de carimbo por evento, contra o kill de 8 ms desta casa: uma cópia é um dab
/// inteiro (silhueta, falloff, Shape, Grain, blend), então o custo é **linear na contagem** e o
/// multiplicador cai sobre o pincel inteiro. Medido por `measure_the_spray_budget_per_event`
/// (espiral de 8 voltas, 2048², pela porta `on_canvas_pointer`, com as três randomizações armadas —
/// sem elas as cópias caem umas sobre as outras, tocam os MESMOS texels e o custo mente para baixo):
///
/// | count | r=24 pior | r=200 pior | r=512 pior |
/// |---:|---:|---:|---:|
/// | 1 | 0,630 | 2,566 | 2,582 |
/// | 2 | 0,237 | 1,373 | 3,033 |
/// | 4 | 0,281 | 2,808 | 4,703 |
/// | 8 | 0,425 | 3,060 | **10,205** ⇠ estoura |
/// | **16** | **0,846** | **5,073** | 18,458 |
/// | 32 | 1,206 | **10,329** ⇠ estoura | 38,071 |
/// | 64 | 2,060 | 12,905 | 64,421 |
/// | 128 | 3,554 | 27,076 | 112,538 |
/// | 256 | 6,154 | 47,351 | 223,207 |
///
/// **`16` é a maior contagem cujo PIOR evento cabe no kill num pincel GRANDE de verdade (r=200).**
/// O número é do pior evento, nunca da média — um traço de artista concentra dabs na curva, e é lá
/// que o quadro cai.
///
/// ⚠️ **A ESCOLHA DO CANTO é a decisão desta constante, e ela tem um motivo que a tabela mostra:**
/// o custo é um PRODUTO (`contagem × área do pincel`), e um teto escalar tem de escolher onde medir.
/// No canto que os dois sliders alcançam **juntos** — `r = BRUSH_SIZE_MAX_PX` — o teto seria **4**,
/// e a razão é que **um único dab ali já custa 2,582 ms**, um terço do orçamento inteiro, *antes de
/// existir spray nenhum*. Derivar o teto dali seria deixar um slider que **nunca foi capado por
/// tempo** (o tamanho do pincel) ditar o teto de um que é — a forma invertida do §0. E no outro
/// extremo, a referência das outras tabelas deste plano (r=24) comporta **mais de 256**, o que daria
/// uma pista onde a faixa útil vive nos primeiros 8%: um teto legítimo pelo relógio e ilegítimo pelo
/// controle.
///
/// ⚠️ **O preço deste número ser UM está nomeado, não escondido:** no pincel de referência a mesma
/// contagem sobra **16× de margem**, e num pincel de 512 px o artista estoura o quadro com 8 marcas.
/// É o mesmo custo honesto que o [`crate::line_kind::SKETCHY_DENSITY_MAX`] carrega — um escalar a
/// governar um produto de dois — e a cura (o teto passar a ser função do raio) trocaria isto por um
/// slider cujo significado depende de OUTRO slider, que é a ergonomia que o Conserve já matou.
///
/// ⚠️ **A tabela inteira é de UMA corrida, e isto é parte do número.** Uma segunda sonda mediu o
/// mesmo `r=200, count 16` noutra corrida e deu **7,074** contra os 5,073 acima — ~40% de deriva
/// desta workstation compartilhada, a mesma que o doc 28 §5.46 documenta. As colunas só são
/// comparáveis entre si porque nasceram juntas; a irmã foi **removida** por isso.
pub const SPRAY_COUNT_MAX: u32 = 16;

/// **O ESPALHAMENTO que a primeira nuvem ganha** — uma fração do diâmetro, o mesmo número do
/// `Jitter → Position`.
///
/// ⚠️ **Sem ele o `Count` é um controle que não faz NADA no pincel de fábrica**, e isto está medido,
/// não suposto: o default é `strength: 1.0` / `flow: 1.0`, então `n` marcas idênticas empilhadas
/// pintam **exatamente** o que uma pinta — o artista sobe a contagem, a tela não muda, e conclui que
/// o slider está quebrado. É a falha que a DIRETIVA §2 recusa.
///
/// ⚠️ **A cura é o padrão `toggle_brush_impasto` desta casa, não uma segunda porta:** ao pedir a
/// PRIMEIRA marca extra (`1 → n`), o tool arma este espalhamento **só se o `Jitter → Position` ainda
/// estiver no zero de fábrica** — um default, nunca uma política. O artista vê o slider vizinho sair
/// do zero (visível, editável, e a mesma row que ele usaria à mão), e uma escolha deliberada de
/// espalhamento nunca é sobrescrita.
///
/// ⚠️ **É decisão de LOOK, e o SMOKE é quem a julga** — não há recurso a medir aqui. Meio diâmetro
/// põe cada marca a até um raio do caminho: a nuvem lê como spray e ainda como UM traço. Movê-la é
/// uma linha; o que não se pode é deixar a primeira nuvem invisível.
pub const SPRAY_DEFAULT_SPREAD: f32 = 0.5;

impl Stroke {
    /// **A PORTA ÚNICA por onde um ponto do CAMINHO vira marca** — arremessa uma vez, constrói `n`.
    ///
    /// Ela existe porque `dab_at` e `emit` nunca foram duas decisões: em todos os cinco sítios que a
    /// chamavam, uma seguia a outra na linha de baixo. Fundi-las é o que dá ao spray um lugar onde
    /// *"quantas marcas este ponto deixa"* possa ser respondido **uma vez** — a alternativa era um
    /// canal de estado (`dab_at` guarda a posição pré-jitter, `emit` a lê), que é o *um fato, duas
    /// cópias* que este módulo já pagou caro.
    pub(super) fn stamp_at(
        &mut self,
        pos: [f32; 2],
        pressure: f32,
        overlap: f32,
        arc: f32,
        out: &mut Vec<Dab>,
    ) {
        // O arremesso é do CAMINHO: uma vez por ponto, e é ele que abre o vão que o `emit` preenche.
        let thrown = self.throw(pos, arc);
        // O desvio do `Rough` é do CAMINHO pela mesma razão, e vem DEPOIS do arremesso porque os dois
        // são translações e o arremesso é quem decide onde a tinta desta passada nasce.
        // ⚠️ A passada 0 é a de sempre, então com o tipo desarmado o `roughen` devolve `thrown` e o
        // traço é byte-idêntico.
        let base_pos = self.roughen(thrown, arc, 0);
        let base = self.dab_from(base_pos, pressure, overlap, arc);
        self.emit(base, out);
        self.rough_passes(thrown, pressure, overlap, arc, out);
        self.spray_copies(base_pos, pressure, overlap, arc, out);
    }

    /// As passadas EXTRA do `Rough` (`passes − 1`) — no-op com o tipo desarmado ou com uma passada.
    ///
    /// ⚠️ **Elas não passam pelo [`Stroke::emit`], e é o MESMO argumento do [`Self::spray_copies`]:**
    /// aquela porta alimenta a memória dos fios e preenche o vão do arremesso, e as duas coisas são
    /// propriedades do CAMINHO — que aconteceu **uma** vez. Uma segunda caminhada empurraria o traço
    /// para a própria memória e mandaria o preenchedor percorrer o vão duas vezes.
    ///
    /// ⚠️ **E ela é uma CAMINHADA, não uma cópia:** cada passada avalia o campo com a SUA semente, o
    /// que a faz divergir e cruzar a primeira ao longo do arco — o contorno duplo do Excalidraw. Uma
    /// cópia no mesmo lugar (o molde literal do spray) empilharia tinta e não desenharia nada.
    fn rough_passes(
        &mut self,
        thrown: [f32; 2],
        pressure: f32,
        overlap: f32,
        arc: f32,
        out: &mut Vec<Dab>,
    ) {
        let n = self.spec.rough_pass_count();
        for pass in 1..n {
            let p = self.roughen(thrown, arc, pass);
            let copy = self.dab_from(p, pressure, overlap, arc);
            crate::symmetry::push_symmetric(out, copy, &self.spec.symmetry);
        }
    }

    /// As cópias EXTRA de um ponto do caminho (`count − 1`) — no-op com o `count` neutro.
    ///
    /// ⚠️ **Elas não passam pelo [`Stroke::emit`], e é correção e não economia:** aquela porta
    /// também alimenta a memória dos fios e preenche o vão do arremesso, e as duas coisas são
    /// propriedades do CAMINHO. Uma nuvem de oito marcas empurraria oito pontos para a memória do
    /// Sketchy — o traço passaria a costurar-se ao próprio spray — e mandaria o preenchedor do
    /// arremesso percorrer oito vezes o mesmo vão.
    ///
    /// ⚠️ **O gate é o `allows_jitter()`, o MESMO do resto do sorteio por-dab.** Drag Dot, Anchored e
    /// Grid Stamp põem **uma** marca deliberada — o Anchored é literalmente um carimbo ancorado no
    /// ponto do clique, e o Grid Stamp promete que todo carimbo pousa no centro de uma célula.
    /// Espalhá-los contradiria a razão de existir de cada um, e é por isso que eles já não sorteiam
    /// nada. (Os três também não passam por esta porta: emitem por rotas próprias.)
    fn spray_copies(
        &mut self,
        thrown: [f32; 2],
        pressure: f32,
        overlap: f32,
        arc: f32,
        out: &mut Vec<Dab>,
    ) {
        if !self.spec.stroke_method.allows_jitter() {
            return;
        }
        let n = self.spec.spray_count.clamp(1, SPRAY_COUNT_MAX);
        for _ in 1..n {
            let copy = self.dab_from(thrown, pressure, overlap, arc);
            crate::symmetry::push_symmetric(out, copy, &self.spec.symmetry);
        }
    }
}
