//! **Copiar um plano canvas-shaped, em paralelo** — o primitivo, num lugar só.
//!
//! Duas partes do Painter copiam planos inteiros e as duas custam milissegundos a 4096²: a **porta de
//! fork** ([`crate::tool::paint`], que dá acesso exclusivo a um `Arc` compartilhado) e o **motor de
//! delta** do histórico ([`crate::undo_delta`], cuja materialização de um `Patch` começa clonando o plano
//! do cursor). São perguntas diferentes — *"posso escrever nisto?"* e *"me dê este estado"* — mas a
//! operação embaixo das duas é a mesma, e o limiar embaixo dela também.
//!
//! ⚠️ **É uma CÓPIA, e uma cópia tem uma resposta certa só.** Paralelizar muda qual thread copia qual
//! pedaço e nada mais — byte-idêntico por construção, do mesmo jeito que o fold da luz e os dois passes
//! de sculpt (ADR-0109: linhas disjuntas, leitura pura).

use rayon::prelude::*;

/// **O limiar é em BYTES, não em elementos** — e a diferença foi medida, não escolhida.
///
/// Um plano de `u8` e um de `[u8; 7]` com a mesma CONTAGEM carregam sete vezes a memória, e o que decide
/// se vale espalhar a cópia por threads é quanta memória ela move. Com o limiar em elementos a mesma
/// tela mandava o canvas para o caminho paralelo e o de material junto, e a 1024² isso **dobrou** o
/// custo de um Ctrl+Z (0,42 -> 0,86 ms) porque o fork do rayon passou a dominar quatro cópias pequenas.
///
/// Medido, um Ctrl+Z (que copia os quatro planos do cursor):
///
/// ```text
///   1024²  (planos de 1 a 7 MB)     serial  0,42 ms   paralelo  0,86 ms
///   2048²  (planos de 4 a 29 MB)    serial  3,12 ms   paralelo  3,47 ms
///   4096²  (planos de 17 a 117 MB)  serial 46,56 ms   paralelo 21,86 ms   2,1x
/// ```
///
/// A virada está entre 29 e 67 MB, então o limiar fica em **32 MB**: a 2048² tudo segue serial (onde o
/// serial ganha) e a 4096² os três planos grandes vão para o paralelo (onde ele ganha 2×). ⚠️ O número
/// grande do serial a 4096² **não é largura de banda** (5,8 GB/s é lento demais para isso): é o
/// *first-touch* de 67-117 MB recém-alocados, uma falha de página por vez — e é exatamente isso que
/// espalhar por threads conserta.
pub(crate) const PAR_MIN_BYTES: usize = 32 << 20;

/// Vale paralelizar uma cópia de `len` elementos de `T`? A pergunta é feita aqui e só aqui — a porta de
/// fork DECIDE por ela e o primitivo abaixo EXECUTA por ela, então as duas não podem divergir.
pub(crate) const fn worth_parallel<T>(len: usize) -> bool {
    len.saturating_mul(size_of::<T>()) >= PAR_MIN_BYTES
}

/// **O limiar do PREENCHIMENTO, e ele NÃO é o da cópia** — medido, não herdado.
///
/// ⚠️ Uma cópia LÊ e ESCREVE; um preenchimento só ESCREVE, e sobre memória **recém-alocada**, onde o
/// custo é o *first-touch* de página em vez de largura de banda. Herdar os 32 MB do [`PAR_MIN_BYTES`]
/// deixava o plano de MATERIAL a 2048² — **29,4 MB**, a um fio do limiar — na rota serial, que é
/// precisamente o caso que esta wave existe para curar.
///
/// Medido a frio (alocações SEGURADAS, uma página nova por amostra), `[u8; 7]`:
///
/// ```text
///     4 MB   serial 0,60   paralelo 0,53   1,11x
///     8 MB          1,35            0,44   3,08x
///    24 MB          3,86            0,92   4,21x
///    29 MB          5,11            1,12   4,57x   <- o plano de material a 2048²
///   117 MB         19,37            3,76   5,15x   <- o plano de material a 4096²
/// ```
///
/// O serial fica preso em **~6,2 GB/s** (a taxa de falta de página) e o paralelo em ~31 GB/s, então a
/// razão é ~4-5× em toda a faixa que importa. A virada está entre 4 e 8 MB; **o limiar fica em 8**,
/// onde o ganho já é inequívoco (3,08×) e abaixo do qual o serial custa no máximo 1,35 ms.
pub(crate) const FILL_PAR_MIN_BYTES: usize = 8 << 20;

/// Vale paralelizar o preenchimento de `len` elementos de `T`? Irmã da [`worth_parallel`], com o
/// limiar DELA — ver [`FILL_PAR_MIN_BYTES`] para o porquê de não ser o mesmo número.
pub(crate) const fn worth_parallel_fill<T>(len: usize) -> bool {
    len.saturating_mul(size_of::<T>()) >= FILL_PAR_MIN_BYTES
}

/// Clona um plano, em paralelo quando [`worth_parallel`] diz que vale.
pub(crate) fn par_clone<T>(src: &[T]) -> Vec<T>
where
    T: Copy + Send + Sync,
{
    if !worth_parallel::<T>(src.len()) {
        return src.to_vec();
    }
    src.par_iter().copied().collect()
}

/// **Materializa `n` elementos no valor `value`, em paralelo quando vale** — a irmã exata do
/// [`par_clone`], e pelo MESMO motivo que ele existe.
///
/// ⚠️ **É um preenchimento, e um preenchimento tem uma resposta certa só**: todo elemento recebe o
/// mesmo valor, então qual thread escreve qual pedaço não é observável — byte-idêntico **por
/// construção**, a mesma forma do ADR-0109 que a cópia já usa (fatias disjuntas, nenhuma leitura).
///
/// Ela ganha do `resize` pela razão que o doc do [`PAR_MIN_BYTES`] já tinha medido para a CÓPIA e que
/// ninguém tinha aplicado aqui: o custo não é o laço, é o *first-touch* das páginas — **uma falha de
/// página por vez** numa thread só —, e espalhar por threads é precisamente o que o conserta, porque
/// cada núcleo falta as suas.
fn par_fill<T>(n: usize, value: T) -> Vec<T>
where
    T: Copy + Send + Sync,
{
    #[cfg(test)]
    PAR_FILLS.with(|c| c.set(c.get() + 1));
    rayon::iter::repeat_n(value, n).collect()
}

// Quantas vezes o [`par_fill`] disparou **nesta thread** — o contador do gate, e não uma estatística.
//
// ⚠️ Ele existe porque a identidade **não consegue** ver esta cura: as duas rotas devolvem o mesmo
// plano, então um gate de bytes fica VERDE com a rota paralela removida, e só um relógio as
// distinguiria — um relógio que numa máquina compartilhada seria silenciado em vez de acreditado.
// Contar é a resposta determinística, e é o precedente do ADR-0120 (*o gate que CONTA quantas vezes o
// caminho rápido dispara*).
//
// ⚠️ **POR THREAD, e isto é estrutural e não gosto.** O `par_fill` é chamado pela thread que chamou o
// [`size_to`] (o `collect` é que se espalha), então um contador global seria poluído por qualquer
// outro teste a materializar um plano em paralelo ao lado — que é *exactamente* a flake que a
// `ph2d-painter-brush` pagou com uma trava cuja lista de quem-a-segura apodreceu (13 sítios, 2
// seguravam). Por thread não há lista para esquecer.
#[cfg(test)]
thread_local! {
    static PAR_FILLS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

/// **Materializa um plano de `n` elementos no valor `value`** — e o caso do ZERO é de graça.
///
/// ⚠️ **`Vec::resize` num vetor VAZIO escreve elemento a elemento**, mesmo quando o valor é zero;
/// `vec![zero; n]` pede ao SO páginas **já zeradas** (a especialização `IsZero` da `std`) e não toca um
/// byte. A diferença é medida e não é pequena: o primeiro commit de um documento a 4096² paga **2,05 ms**
/// só para materializar o plano de COBERTURA, que são 16,8 MB de zeros escritos à mão.
///
/// ⚠️ **Mas o atalho do zero é do ALOCADOR, não uma garantia** — e o número está medido. O `calloc`
/// só pula o memset quando devolve páginas NOVAS do `mmap`; o `mmap_threshold` do glibc é **dinâmico**
/// e sobe até 32 MB à medida que blocos grandes são libertados, e a partir daí uma alocação de 16,8 MB
/// vem do heap reciclado, que ele tem de **zerar à mão** — medido, **2,02 ms contra 0,01** para o
/// MESMO plano, decidido apenas pelo que a sessão libertou antes (o controle está no doc do
/// `what_the_relief_commit_is_made_of`). Fica como está: no caminho comum o ramo do zero é de graça, e
/// trocá-lo pelo [`par_fill`] pagaria ~0,5 ms **sempre** para poupar 2 ms **às vezes**.
///
/// ⚠️ **E o valor NÃO-zero tem de ser escrito** — o material começa em `NEUTRAL`, não em zero (zero é
/// `roughness = 0`, que é ESPELHO), e nenhum atalho de alocador entrega um padrão de sete bytes. Aí a
/// pergunta deixa de ser *se* escrever e passa a ser **com quantos núcleos**, e a resposta é o
/// [`par_fill`]: medido sobre o plano de material, **19,37 -> 3,76 ms a 4096²** e **5,11 -> 1,12 a
/// 2048²**, com a máquina calma e a rota paralela conferida byte a byte contra a serial.
///
/// ⛔ **MEDIDO E REJEITADO, não refaça: preencher por DUPLICAÇÃO** (escrever um elemento e copiar o
/// prefixo sobre o dobro do espaço, fazendo de cada passo um `memcpy`). Foi construída e medida: **17,5
/// contra 18,7 ms**, 6% — porque as duas estão no MESMO teto. 117 MB em 17 ms são **6,9 GB/s**: o custo
/// é o *first-touch* das páginas, não o laço que as escreve.
///
/// ⚠️ **E é essa mesma frase que dizia por que a rota paralela não fora tentada — ela estava certa
/// sobre a pergunta errada.** *"Nenhuma esperteza sobre COMO escrever muda quantas páginas há para
/// tocar"* é verdade, e irrelevante: o que muda não é o número de páginas, é **quantos núcleos as
/// faltam**. O doc do [`PAR_MIN_BYTES`] já tinha medido exatamente isto do lado da CÓPIA (*"é
/// exatamente isso que espalhar por threads conserta"*, 2,1x) e a porta irmã nunca o recebeu. Uma
/// medição correta sobre um mecanismo arquivou a alternativa que o resolvia.
///
/// ⚠️ **Duas rotas foram avaliadas ANTES desta e as duas morreram numa leitura, não numa opinião:**
/// *plano ausente = neutro* (a convenção que a luz já implementa) **nunca dispararia**, porque o
/// pincel de fábrica tem `impasto_shine: 0.7` e deposita material não-neutro desde a primeira
/// pincelada; e *preencher só a janela* morre no `body` da luz, que toma
/// `paint_body(cover).max(form[3]).max(paper_body)` — com um papel a presença é **1 em toda parte**,
/// logo o chão do plano É observável fora da tinta.
pub(crate) fn size_to<T>(dst: &mut Vec<T>, n: usize, value: T)
where
    T: Copy + Default + PartialEq + Send + Sync,
{
    if dst.len() == n {
        return;
    }
    if !dst.is_empty() {
        dst.resize(n, value); // tela que mudou de tamanho: o conteúdo existente manda
        return;
    }
    if value == T::default() {
        // `alloc_zeroed`: o SO entrega as páginas prontas e ninguém escreve um byte.
        *dst = vec![T::default(); n];
        return;
    }
    if worth_parallel_fill::<T>(n) {
        *dst = par_fill(n, value);
        return;
    }
    dst.resize(n, value);
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **A cópia paralela é a cópia serial.** Sobre um comprimento que cruza o [`PAR_MIN_BYTES`] (senão o
    /// caminho paralelo não roda — a armadilha do ADR-0120: uma otimização que ninguém exercita é código
    /// verde que nunca executa) e outro que não.
    #[test]
    fn a_parallel_clone_is_the_serial_one() {
        let big = PAR_MIN_BYTES / size_of::<f32>() + 1_000;
        for n in [big, 64] {
            let src: Vec<f32> = (0..n).map(|i| (i as f32) * 0.25 - 3.0).collect();
            assert_eq!(par_clone(&src), src, "n = {n}");
        }
        // O tipo mais largo que o histórico guarda — 7 bytes por elemento, sem `memcpy` de fatia.
        let n7 = PAR_MIN_BYTES / 7 + 7;
        let m: Vec<[u8; 7]> = (0..n7).map(|i| [(i % 251) as u8; 7]).collect();
        assert_eq!(par_clone(&m), m);
    }

    /// **O limiar é em BYTES, e isso é o gate** — porque em ELEMENTOS ele mandava um plano de `u8` de
    /// 1 MB para o caminho paralelo junto com um de material de 7 MB, e a 1024² isso DOBROU o custo de um
    /// Ctrl+Z. A mesma CONTAGEM tem de decidir diferente conforme o tamanho do elemento.
    #[test]
    fn the_threshold_counts_bytes_not_elements() {
        let n = PAR_MIN_BYTES / 4; // 4 bytes por elemento se for `f32`, 1 se for `u8`
        assert!(!worth_parallel::<u8>(n), "u8: {n} elementos sao {n} bytes");
        assert!(worth_parallel::<f32>(n), "f32: {n} elementos sao 4x isso");
        // E o tipo mais largo do histórico cruza antes de todos.
        assert!(worth_parallel::<[u8; 7]>(PAR_MIN_BYTES / 7 + 1));
        assert!(!worth_parallel::<[u8; 7]>(PAR_MIN_BYTES / 7 - 1_000));
    }

    /// **COMO se materializa um plano de 117 MB** — a sonda que decide se o ramo do valor fica serial.
    ///
    /// Rodar: `cargo test -p ph2d-tool-painter --release measure_how_a_plane_is_materialised -- --ignored --nocapture`
    #[test]
    #[ignore = "medição, não gate"]
    fn measure_how_a_plane_is_materialised() {
        const N: usize = 4096 * 4096; // o plano de MATERIAL a 4096²: 117 MB
        let neutral = [0u8, 128, 0, 0, 255, 255, 255];
        println!("\n=== MATERIALIZAR 117 MB de [u8;7] (o plano de MATERIAL a 4096^2) ===");
        for k in 0..3 {
            let t0 = std::time::Instant::now();
            let mut a = Vec::<[u8; 7]>::new();
            a.resize(N, neutral);
            let ser = t0.elapsed().as_secs_f64() * 1e3;
            let t1 = std::time::Instant::now();
            let b: Vec<[u8; 7]> = rayon::iter::repeat_n(neutral, N).collect();
            let par = t1.elapsed().as_secs_f64() * 1e3;
            assert_eq!(a, b, "as duas rotas nao dao o mesmo plano");
            println!(
                "[fill {k}] serial {ser:>7.2} ms | paralelo {par:>7.2} ms | {:.2}x",
                ser / par
            );
        }
        // E o ramo do ZERO, que já é de graça — para a sonda dizer o piso ao lado do teto.
        let t2 = std::time::Instant::now();
        let z = vec![[0u8; 7]; N];
        println!(
            "[fill  z] alloc_zeroed {:>7.2} ms (len {})\n",
            t2.elapsed().as_secs_f64() * 1e3,
            z.len()
        );

        // ⚠️ **O LIMIAR DA CÓPIA NÃO É O LIMIAR DO PREENCHIMENTO** — herdá-lo seria o erro que o
        // §0 nomeia. Uma cópia LÊ e ESCREVE (dois toques de memória por elemento); um preenchimento
        // só ESCREVE. A virada não tem por que cair no mesmo byte, e o plano de MATERIAL a 2048²
        // mede 29,4 MB — a um fio dos 32 da irmã, e portanto exatamente onde a herança decide.
        println!("=== ONDE O PREENCHIMENTO PARALELO PASSA A GANHAR ([u8;7]) ===");
        println!(
            "{:>8} {:>10} {:>11} {:>8}",
            "MB", "serial", "paralelo", "razao"
        );
        // ⚠️ **As alocações são SEGURADAS, e é isso que torna a sonda honesta.** A 1ª versão
        // liberava e re-alocava o mesmo tamanho e tomava o MÍNIMO de três — da 2ª volta em diante o
        // alocador devolve as MESMAS páginas, já mapeadas, e o número medido é o do cache e não o
        // do produto (a armadilha do doc 28 §5.16). Com os vetores vivos, cada alocação recebe
        // páginas NOVAS, que é o que um documento recém-aberto de facto encontra.
        let mut keep: Vec<Vec<[u8; 7]>> = Vec::new();
        for mb in [4usize, 8, 16, 24, 29, 32, 48, 64, 117] {
            let n = mb * 1024 * 1024 / 7;
            let t = std::time::Instant::now();
            let mut a = Vec::<[u8; 7]>::new();
            a.resize(n, neutral);
            let ser = t.elapsed().as_secs_f64() * 1e3;
            let t = std::time::Instant::now();
            let b: Vec<[u8; 7]> = rayon::iter::repeat_n(neutral, n).collect();
            let par = t.elapsed().as_secs_f64() * 1e3;
            assert_eq!(a.len(), b.len());
            keep.push(a);
            keep.push(b);
            println!("{mb:>8} {ser:>10.2} {par:>11.2} {:>8.2}x", ser / par);
        }
        drop(keep);
        println!();
    }

    /// **A porta devolve o mesmo plano que o `resize` devolvia** — nos dois ramos, e num vetor que já
    /// tem conteúdo (o caso da tela que mudou de tamanho, onde o atalho do zero não pode disparar).
    #[test]
    fn sizing_a_plane_gives_what_resize_gave() {
        for n in [4usize, 1_000] {
            let (mut a, mut b) = (Vec::<u8>::new(), Vec::<u8>::new());
            size_to(&mut a, n, 0);
            b.resize(n, 0);
            assert_eq!(a, b, "o ramo do zero, n = {n}");

            let neutral = [3u8, 1, 4, 1, 5, 9, 2];
            let (mut c, mut d) = (Vec::<[u8; 7]>::new(), Vec::<[u8; 7]>::new());
            size_to(&mut c, n, neutral);
            d.resize(n, neutral);
            assert_eq!(c, d, "o ramo do valor, n = {n}");

            // Já com conteúdo: o atalho do zero NÃO pode disparar, senão apagaria o que lá está.
            let mut e = vec![7u8; 3];
            size_to(&mut e, n.max(3), 0);
            assert_eq!(
                &e[..3],
                &[7, 7, 7],
                "o atalho do zero apagou conteudo existente"
            );
        }
    }

    /// **O preenchimento PARALELO dá o que o `resize` dava, e ele DISPAROU** — as duas metades.
    ///
    /// ⚠️ **A segunda metade é a que torna o gate capaz de falhar.** As duas rotas devolvem o mesmo
    /// plano por construção (todo elemento recebe o mesmo valor), então a igualdade fica VERDE com a
    /// rota paralela arrancada — é a armadilha que o doc do [`super::par_clone`] nomeia (*"uma
    /// otimização que ninguém exercita é código verde que nunca executa"*), e ela morde aqui com força
    /// dobrada, porque nem o comprimento do teste antigo (`n = 4` e `1_000`) cruzava o limiar.
    ///
    /// **Mutação que tem de sangrar:** remover o ramo `worth_parallel` do [`size_to`] — a igualdade
    /// segue verde e o contador cai para 0.
    #[test]
    fn the_big_fill_takes_the_parallel_road_and_lands_on_the_same_bytes() {
        let neutral = [0u8, 128, 0, 0, 255, 255, 255]; // `Material::NEUTRAL.to_bytes()`
        let big = FILL_PAR_MIN_BYTES / 7 + 7;
        // CONTROLE: a fixture TEM de cruzar o limiar, senão o gate compara serial com serial.
        assert!(
            worth_parallel_fill::<[u8; 7]>(big),
            "a fixture nao cruza o piso"
        );

        PAR_FILLS.with(|c| c.set(0));
        let mut par = Vec::<[u8; 7]>::new();
        size_to(&mut par, big, neutral);
        let fired = PAR_FILLS.with(std::cell::Cell::get);
        let mut ser = Vec::<[u8; 7]>::new();
        ser.resize(big, neutral);
        assert_eq!(par, ser, "a rota paralela nao da o plano que o resize dava");
        assert_eq!(
            fired, 1,
            "o preenchimento grande NAO foi pela rota paralela"
        );

        // E a AUSÊNCIA: abaixo do limiar ela não pode disparar — um fork de rayon por plano pequeno é
        // o que dobrou o custo de um Ctrl+Z a 1024² quando o limiar era em elementos.
        PAR_FILLS.with(|c| c.set(0));
        let mut small = Vec::<[u8; 7]>::new();
        size_to(&mut small, 64, neutral);
        assert_eq!(
            PAR_FILLS.with(std::cell::Cell::get),
            0,
            "um plano pequeno foi para o rayon"
        );
    }

    /// **O plano de MATERIAL de uma tela de 2048 CRUZA o limiar do preenchimento** — e este gate
    /// existe porque a resposta muda conforme QUAL limiar se herda.
    ///
    /// ⚠️ Ele afirma a propriedade em DIMENSÕES de tela, não na constante: `2048² × 7 B = 29,4 MB`
    /// fica **abaixo** dos 32 MB da cópia e **acima** dos 8 do preenchimento, então unificar os dois
    /// números — a simplificação que salta aos olhos — devolve aquela tela à rota serial e custa
    /// **5,11 ms contra 1,12** no primeiro traço de todo documento, em silêncio.
    ///
    /// **Mutação que tem de sangrar:** trocar `worth_parallel_fill` por `worth_parallel` no
    /// [`size_to`].
    #[test]
    fn a_2048_material_plane_is_worth_filling_in_parallel() {
        const N: usize = 2048 * 2048;
        assert!(
            worth_parallel_fill::<[u8; 7]>(N),
            "o plano de material a 2048^2 ({} MB) ficou abaixo do limiar de preenchimento",
            N * 7 / (1 << 20)
        );
        // E o CONTROLE que nomeia a diferença: pelo limiar da CÓPIA ele NÃO cruzaria.
        assert!(
            !worth_parallel::<[u8; 7]>(N),
            "os dois limiares convergiram — este gate deixou de dizer alguma coisa"
        );
    }
}
