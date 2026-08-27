//! ⭐⭐⭐ **O INSTRUMENTO que faltava** — `PH2D_INSTANCE_LOG=1` (ADR-0164 / F4.6, o §14 do handoff).
//!
//! > *«Não funcionou. Ao mudo o path, as instâncias não mudaram.»* (Enio, 2026-08-26.)
//!
//! Os gates da F4.6b são verdes e o app não faz. ⛔ **Uma sonda headless que imita a ordem do
//! quadro também passa** (`vec_entities::sync` antes, o passe depois; com o mestre a ser um objeto
//! vazio com a forma dentro **e** com o mestre a ser a própria forma). ⇒ o que falha está num
//! estado que o app monta e o teste não — e sem instrumento, todo report chega como *«não
//! funcionou»*, sem o meio caminho.
//!
//! # O que ele imprime, e por que é ISSO
//!
//! O passe responde uma pergunta de cada vez, e cada uma tem um sítio onde pode morrer em silêncio:
//!
//! 1. **quantas instâncias vivas** — `0` significa que o elo não resolve (o `InstanceOf` da raiz não
//!    aponta para um `MasterRoot`), e nada do resto chega a correr;
//! 2. **quantos pares de peça** — o par é o que o sync percorre; uma peça sem par nunca recebe nada;
//! 3. **quantos pares têm DOCUMENTO dos dois lados** — é a guarda do `sync_one`: se a cópia nasceu
//!    **sem** `VecPathRef` (a clonagem da F4.6a não correu), este número é `0` e a geometria nunca
//!    propaga, sem uma linha de erro em lado nenhum;
//! 4. **quantos deles DIFEREM** e **quantos viraram excepção** — um override capturado no 1.º
//!    quadro congela a peça contra a receita **para sempre**, e o sintoma é exactamente o relatado.
//!
//! ⚠️ **Uma linha por MUDANÇA, não por quadro.** Um log por quadro a 60 fps é ruído que ninguém lê,
//! e o que interessa é o instante em que o número muda.

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};

/// Ligado por `PH2D_INSTANCE_LOG=1`. Lido uma vez, como as outras envs de diagnóstico.
fn armed() -> bool {
    static ON: AtomicBool = AtomicBool::new(false);
    static READ: AtomicBool = AtomicBool::new(false);
    if !READ.swap(true, Ordering::Relaxed) {
        ON.store(
            std::env::var("PH2D_INSTANCE_LOG").is_ok_and(|v| v.trim() != "0"),
            Ordering::Relaxed,
        );
    }
    ON.load(Ordering::Relaxed)
}

/// **O retrato de um passe** — cinco números, na ordem em que o passe os produz.
#[derive(Default, Clone, Copy, PartialEq, Eq)]
pub(crate) struct PassDiag {
    /// Instâncias vivas (raiz cujo elo resolve num `MasterRoot`).
    pub(crate) instances: usize,
    /// Pares peça↔peça percorridos.
    pub(crate) pairs: usize,
    /// Pares em que **os dois lados** têm `VecPathRef` — a guarda do `sync_one`.
    pub(crate) doc_pairs: usize,
    /// Desses, quantos tinham conteúdo diferente do mestre.
    pub(crate) doc_diff: usize,
    /// Escritas de geometria feitas neste passe.
    pub(crate) doc_wrote: usize,
    /// Excepções registadas na instância, somadas.
    pub(crate) overrides: usize,
}

/// Imprime **só quando algum número muda** — ver o cabeçalho.
pub(crate) fn report(d: PassDiag) {
    if !armed() {
        return;
    }
    static LAST: AtomicU64 = AtomicU64::new(u64::MAX);
    let key = pack(d);
    if LAST.swap(key, Ordering::Relaxed) == key {
        return;
    }
    println!(
        "[instancia] instancias={} pares={} com-documento={} diferentes={} escritas={} excepcoes={}",
        d.instances, d.pairs, d.doc_pairs, d.doc_diff, d.doc_wrote, d.overrides
    );
    if d.instances > 0 && d.pairs > 0 && d.doc_pairs == 0 {
        println!(
            "[instancia] ⚠️ NENHUM par tem documento dos dois lados — ou a receita nao tem forma \
             vetorial, ou a copia nasceu sem `VecPathRef` (a clonagem nao correu)"
        );
    }
    if d.doc_diff > 0 && d.doc_wrote == 0 {
        println!(
            "[instancia] ⚠️ ha' {} par(es) com forma DIFERENTE e nenhuma escrita — a peca esta' \
             congelada por excepcao, ou o eco diz que o mestre nao se mexeu",
            d.doc_diff
        );
    }
}

/// Os seis números num `u64` — a chave de *«mudou alguma coisa?»*. Satura em 1023 por campo, que é
/// muito acima de qualquer cena real e mantém a comparação num inteiro só.
fn pack(d: PassDiag) -> u64 {
    let f = |v: usize| (v.min(1023)) as u64;
    f(d.instances)
        | f(d.pairs) << 10
        | f(d.doc_pairs) << 20
        | f(d.doc_diff) << 30
        | f(d.doc_wrote) << 40
        | f(d.overrides) << 50
}

#[cfg(test)]
mod tests {
    use super::{PassDiag, pack};

    /// ⚠️ **Dois retratos diferentes não podem ter a mesma chave** — senão o log cala-se
    /// exactamente no quadro em que o estado mudou, que é o único que interessa.
    ///
    /// (Mutação: deslocar dois campos para o mesmo bit ⇒ RED.)
    #[test]
    fn every_field_moves_the_key() {
        let base = PassDiag::default();
        let mut seen = std::collections::BTreeSet::new();
        seen.insert(pack(base));
        for (i, d) in [
            PassDiag {
                instances: 1,
                ..base
            },
            PassDiag { pairs: 1, ..base },
            PassDiag {
                doc_pairs: 1,
                ..base
            },
            PassDiag {
                doc_diff: 1,
                ..base
            },
            PassDiag {
                doc_wrote: 1,
                ..base
            },
            PassDiag {
                overrides: 1,
                ..base
            },
        ]
        .into_iter()
        .enumerate()
        {
            assert!(
                seen.insert(pack(d)),
                "o campo {i} nao move a chave — o log cala-se quando ele muda"
            );
        }
    }
}
