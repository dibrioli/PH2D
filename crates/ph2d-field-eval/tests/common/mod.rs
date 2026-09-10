//! ⭐⭐⭐ **O QUE OS CENSOS DESTA CRATE PARTILHAM** — hoje, uma coisa só: correr nos núcleos todos.
//!
//! # ⛔⛔ O report que o obrigou (Enio, 2026-09-09)
//!
//! > *«porque um teste de 50 min? se os testes forem assim é impossível finalizar o app»*
//!
//! Ele tinha razão, e a causa era estrutural: o corredor de testes distribui **testes** pelos
//! núcleos, e os censos desta crate são **um** teste com um ciclo de centenas de casos lá dentro.
//! Com `32` núcleos na máquina, `31` ficavam a ver.
//!
//! | teste | em série | nos núcleos todos |
//! |---|---:|---:|
//! | `every_row_of_every_primitive_marches_safely_across_its_range` | `336,8 s` | **`41,6 s`** |
//!
//! ⚠️ **A causa de FUNDO era outra, e a W147 fechou-a** — esta nota dizia *«continua por curar»* e
//! envelheceu no dia em que alguém a curou. O `Field::at` deixou de ser o interpretador `f64` da
//! `fidget` e passou a ser uma **fita achatada** (`ph2d-field-eval/src/point_tape.rs`), com o
//! `gradient_norm` a mandar as seis amostras numa passagem só: **`~3,6×`** no valor e **`~6,3×`** no
//! gradiente, **bit-a-bit a mesma resposta** (há gate sobre as duas).
//!
//! ⛔⛔ **E a cura que esta nota PRESCREVIA está RECUSADA com número.** Ela dizia que o problema do
//! caminho em lote era a `f32` perder dígitos; o problema real é outro e é **estrutural**: num
//! **vinco** a derivada não existe, e a diferença central (que devolve a MÉDIA dos dois gradientes
//! laterais) e o gradiente analítico (que escolhe UM ramo) são **grandezas diferentes** — medido
//! sobre pontos postos em cima de uma aresta, discordam **`1,876e-1`** contra uma folga de módulo de
//! `2,0e-2`, ou seja **`9,4×`** a folga inteira. Passar a `f64` não cura nada disso.
//!
//! ⇒ o `143×` do caminho em lote **continua inalcançável para esta régua**, e agora pela razão certa.
//! Tabela, o mecanismo e a armadilha de amostragem que quase inverteu o veredito:
//! `docs/3DModeling/11_a_avaliacao_ponto_a_ponto.md`.
//!
//! ⚠️ **Um ficheiro em `tests/common/` NÃO é um alvo de teste** — é por isso que ele pode ser
//! partilhado por dois binários de teste sem nascer um terceiro, vazio.
#![allow(dead_code)]

/// ⭐⭐ **Um mapa que corre nos núcleos todos, e devolve NA ORDEM DA ENTRADA.**
///
/// ⚠️ **Roubo de trabalho por contador atómico**, e não fatias fixas: os casos destes censos custam
/// entre si ordens de grandeza (uma esfera contra um nó de toro), e uma partição estática deixaria
/// `31` núcleos à espera do pior.
///
/// ⚠️ **A ordem da SAÍDA é a da entrada, e não a de chegada** — uma mensagem de erro que muda de
/// ordem entre corridas é a diferença entre um diff legível e um diff que ninguém compara.
///
/// ⚠️ **`PH2D_CENSO_SERIE=1` volta a um núcleo**: sem esse interruptor o ganho seria uma alegação, e
/// o número de comparação teria de vir de uma corrida antiga feita noutra carga de máquina. *Um
/// interruptor torna a medição repetível na máquina de quem duvida.*
pub fn em_paralelo<T: Sync, R: Send>(itens: &[T], f: impl Fn(&T) -> R + Sync) -> Vec<R> {
    use std::sync::atomic::{AtomicUsize, Ordering};
    if itens.is_empty() {
        return Vec::new();
    }
    let proxima = AtomicUsize::new(0);
    let saida = std::sync::Mutex::new(Vec::with_capacity(itens.len()));
    let n = if std::env::var_os("PH2D_CENSO_SERIE").is_some() {
        1
    } else {
        std::thread::available_parallelism()
            .map_or(1, std::num::NonZeroUsize::get)
            .min(itens.len())
    };
    std::thread::scope(|escopo| {
        for _ in 0..n {
            escopo.spawn(|| {
                loop {
                    let i = proxima.fetch_add(1, Ordering::Relaxed);
                    let Some(item) = itens.get(i) else { break };
                    let v = f(item);
                    saida.lock().expect("o mutex do censo").push((i, v));
                }
            });
        }
    });
    let mut v = saida.into_inner().expect("o mutex do censo");
    v.sort_by_key(|(i, _)| *i);
    v.into_iter().map(|(_, r)| r).collect()
}
