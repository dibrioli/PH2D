//! **A chave de ablação do laço de altura** — medição, nunca produto.
//!
//! O `measure_impasto_cost` fecha a acusação até o 2º nível (`DrawTo::Depth` isola a metade da
//! ALTURA: 111,6 ms a raio 100, com pigmento ZERO) e para ali com a pergunta escrita: *"o que eu NÃO
//! consegui estabelecer: **por que** esses 118 ms"*. Abaixo do `DrawTo` **não existe porta de produto**
//! — nenhum knob do artista separa a silhueta do AA do filme, nem o miolo do laço da cauda de
//! escritas —, e as duas saídas que restam são ruins:
//!
//! * uma sonda com **laço próprio** fica CEGA à porta (a lição do `warp_axis` e do `serial_side`:
//!   ela seguiria imprimindo o custo de um código que o produto parou de rodar);
//! * `#[cfg(test)]` nesta crate **não vale** quando quem roda o teste é a `ph2d-tool-painter` — o
//!   `cfg(test)` só liga na crate sob teste, então o flag ficaria morto exactamente na medição que
//!   precisa atravessar `on_canvas_pointer`.
//!
//! Daí um flag `pub` de verdade, lido **UMA vez por dab** (hoisted para fora do laço de texel, ao lado
//! do `film_lut_for`, que já é uma consulta thread-local por dab) e comparado por bit lá dentro. O
//! custo no produto é uma leitura de TLS por dab e um `and` por texel que a predição acerta sempre —
//! e ⚠️ **todas as configurações pagam o MESMO `and`**, o que torna as DIFERENÇAS honestas mesmo que
//! o absoluto carregue o preço da própria chave.
//!
//! ⚠️ **Ele é `0` em todo caminho de produto** e nada além das sondas o escreve — há gate afirmando
//! isso. Um flag de ablação que alguém arma e esquece é uma engine com duas leis.
//!
//! ## Por que a cauda é ablacionada PRIMEIRO
//!
//! Trocar a silhueta por um degrau muda o `m = w·coverage`, e `m` decide a **rejeição de envelope**
//! (`if m <= paint[i] { continue }`) — com `w = 1` o 1º dab satura o texel e os vizinhos passam a ser
//! rejeitados mais cedo. Comparar `full` contra `full|SILHOUETTE` mediria a silhueta **e** a mudança
//! na taxa de rejeição, e atribuiria as duas à silhueta. Por isso a decomposição compara sempre com a
//! CAUDA já desligada nos dois lados: aí a única diferença é a peça sob medição.

use std::cell::Cell;

thread_local! {
    static MASK: Cell<u32> = const { Cell::new(0) };
}

/// Troca `silhouette_at` por um degrau de MESMO SUPORTE (`t < 1`), removendo falloff, imagem de Shape
/// e máscara — sem mudar quais texels o laço visita.
pub const SILHOUETTE: u32 = 1 << 0;
/// Força o caminho `film_of(w)` de amostra única — que é o ramo `None` que o próprio kernel já tem
/// quando o brush não admite AA, e não uma segunda implementação escrita para a sonda.
pub const FILM_AA: u32 = 1 << 1;
/// Para logo depois do envelope do filme: sem grain, sem a mordida do bow wave, sem as quatro
/// escritas de plano, sem `derive_height`.
pub const TAIL: u32 = 1 << 2;
/// Força a rota **SERIAL** do laço de altura (uma banda só), sem tocar em nenhuma aritmética.
///
/// ⚠️ **É ablação de ROTA, não de peça** — as outras três removem trabalho e mudam o resultado; esta
/// não pode mudar um byte, e é isso que ela existe para provar. Dois consumidores:
/// * o gate de identidade, que compara os cinco planos da rota em banda contra os da serial;
/// * o A/B de relógio, que **tem de medir as duas rotas costas-com-costas dentro da MESMA corrida** —
///   nesta workstation o mesmo passo já mediu 14,5 e 30,2 ms sem uma linha mudar (doc 28 §5.46), e um
///   A/B entre corridas atribuiria a carga da máquina ao ganho.
pub const SERIAL: u32 = 1 << 3;

/// Arma a máscara. **Sondas apenas** — todo caminho de produto a deixa em `0`.
pub fn set(mask: u32) {
    MASK.with(|c| c.set(mask));
}

/// A máscara vigente. Lida UMA vez por dab.
#[must_use]
pub fn get() -> u32 {
    MASK.with(Cell::get)
}

/// Roda `f` com a máscara armada e a **devolve ao valor anterior**, aconteça o que acontecer.
///
/// Uma sonda que esquece de desarmar envenena todo teste que rode depois dela na mesma thread — e
/// `--test-threads=1`, que as sondas de relógio exigem, é justamente o modo em que isso acontece.
pub fn with<T>(mask: u32, f: impl FnOnce() -> T) -> T {
    struct Restore(u32);
    impl Drop for Restore {
        fn drop(&mut self) {
            set(self.0);
        }
    }
    let _guard = Restore(get());
    set(mask);
    f()
}
