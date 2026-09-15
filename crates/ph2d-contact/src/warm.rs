//! ⭐⭐⭐ **A MEMÓRIA DE UM CONTACTO** — a fatia 1 da obra encomendada pelo dono em 2026-09-15
//! (*«encomendas o motor de contacto novo»* · *«comece etapa 1»*), espec no
//! [doc 111](../../../docs/Motion%20Nodes/111_o_motor_de_contacto_com_memoria.md).
//!
//! ## Porque é que isto existe
//!
//! O [doc 109 §8.14](../../../docs/Motion%20Nodes/109_o_colisor_na_forma.md) fechou **oito** cadeias
//! de cura sobre o zumbido da pilha, e as oito trocavam o tremor pelo rodopio. A razão é de CLASSE:
//! *uma correcção recalculada do zero a cada tique não tem memória do impulso que aplicou no tique
//! anterior.* Este módulo é essa memória — e **só** ela: a lei que a consome é a fatia seguinte, e
//! enquanto não existir **nada aqui toca no produto**.
//!
//! ## ⭐ A morada foi MEDIDA, e é por isso que ela é uma COLUNA
//!
//! Um `λ` por PAR seria uma tabela lateral, e uma tabela lateral **não atravessa a fronteira do
//! dispositivo** — que é o defeito que a obra vem curar, um nível acima. Medido na `=114`
//! (`probe_quantos_encostos_por_peca`, doc 111 §5.4), sobre `1 375` peças-tique da janela assente:
//!
//! ```text
//!   encostos | peças-tique | acumulado
//!          0 |          19 |    1,38 %
//!          1 |         370 |   28,29 %
//!          2 |         475 |   62,84 %
//!          3 |         262 |   81,89 %
//!          4 |         191 |   95,78 %
//!          5 |          58 |  100,00 %
//! ```
//!
//! ⇒ o pior caso é **5**, e [`K`] é **6**: a memória é **por ELEMENTO e de largura fixa**, logo *é*
//! uma coluna. Ela viaja no laço do estado como o `age` e o `sim_t`, e um kernel lê-a sem substrato
//! novo — **a wave do dispositivo deixa de ser uma wave e passa a ser uma consequência.**
//!
//! ⚠️ **A folga é UMA ranhura, não dez**, e é deliberado: `K` multiplica a memória de toda peça de
//! toda cena (`K/2` colunas `Vec4` = `4·K` bytes por elemento). Uma pilha mais densa que a medida
//! transborda, e o transbordo tem lei — ver [`Memoria::guardar`].
//!
//! ## ⛔⛔ A CHAVE, e a cerca que ela traz
//!
//! A chave de um contacto é o par de identidades das duas peças. Medido na `=114`
//! (`probe_a_pilha_tem_identidade`): **a coluna `id` está AUSENTE** — aquela cena nasce de uma
//! grelha com carimbo, e quem cunha `id` é o `sim.spawn`. ⇒ [`chaves`] cai no **índice**, e daí sai
//! uma cerca que a lei consumidora tem de honrar:
//!
//! > ⛔ **Sem `id`, a memória só é válida enquanto a POPULAÇÃO não muda.** Um nascimento ou uma
//! > morte reindexa a corrente, e um `λ` guardado num índice que se deslocou aquece o **contacto
//! > errado** — que é PIOR que não aquecer (doc 111 §5 W2).
//!
//! ⚠️ E há uma segunda cerca, do TIPO: um `id` viaja numa coluna de `f32`, exacta até `2²⁴`. Acima
//! disso duas peças partilham chave — a mesma aritmética que o doc 91 mediu no `sim.spawn::rate`.

use ph2d_nodegraph::attr::{Column, Stream};

/// **Quantos encostos uma peça lembra.** Medido: o pior caso da `=114` é `5` (ver o cabeçalho).
pub const K: usize = 6;

/// As colunas onde a memória viaja — `K/2` colunas `Vec4`, cada uma com DOIS pares
/// `(parceiro, λ)`. ⚠️ Os nomes são formato de corrente: **append only**.
pub const COLUNAS: [&str; K / 2] = ["contact_warm_0", "contact_warm_1", "contact_warm_2"];

/// **Uma ranhura vazia.** ⚠️ O `0` de parceiro NÃO serve de vazio — o índice `0` é uma peça
/// legítima —, então o vazio é o `λ` a zero: *um contacto que não empurra não tem o que lembrar.*
const VAZIA: (u32, f32) = (0, 0.0);

/// **O que uma peça lembra dos encostos dela**: até [`K`] pares `(parceiro, λ)`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Memoria {
    ranhuras: [(u32, f32); K],
}

impl Default for Memoria {
    fn default() -> Self {
        Self {
            ranhuras: [VAZIA; K],
        }
    }
}

impl Memoria {
    /// O `λ` lembrado para `parceiro`, ou `0` — que é o arranque FRIO, a lei de hoje.
    #[must_use]
    pub fn lambda(&self, parceiro: u32) -> f32 {
        self.ranhuras
            .iter()
            .find(|(p, l)| *p == parceiro && *l != 0.0)
            .map_or(0.0, |(_, l)| *l)
    }

    /// Guarda `λ` para `parceiro`.
    ///
    /// ⚠️ **A lei do TRANSBORDO: quem sai é o `λ` MENOR.** Uma peça com mais de [`K`] encostos
    /// esquece o apoio mais FRACO, que é o que menos muda a resposta se voltar a arrancar frio —
    /// esquecer o mais forte tiraria a memória exactamente de onde ela sustenta a pilha.
    ///
    /// ⚠️ Um `λ` de `0` (ou não-finito) **APAGA** a ranhura em vez de a ocupar: *um contacto que
    /// deixou de empurrar não tem o que lembrar*, e guardá-lo gastaria uma ranhura das seis.
    pub fn guardar(&mut self, parceiro: u32, lambda: f32) {
        if !lambda.is_finite() || lambda <= 0.0 {
            if let Some(r) = self.ranhuras.iter_mut().find(|(p, _)| *p == parceiro) {
                *r = VAZIA;
            }
            return;
        }
        if let Some(r) = self
            .ranhuras
            .iter_mut()
            .find(|(p, l)| *p == parceiro && *l != 0.0)
        {
            *r = (parceiro, lambda);
            return;
        }
        // A ranhura mais fraca — uma vazia tem `λ = 0`, logo ela é sempre a primeira a sair.
        let Some(fraca) = self.ranhuras.iter_mut().min_by(|a, b| a.1.total_cmp(&b.1)) else {
            return;
        };
        if fraca.1 < lambda {
            *fraca = (parceiro, lambda);
        }
    }

    /// Quantas ranhuras estão ocupadas — para gates e censos.
    #[must_use]
    pub fn ocupadas(&self) -> usize {
        self.ranhuras.iter().filter(|(_, l)| *l != 0.0).count()
    }
}

/// **A CHAVE de cada peça**: o `id` quando a corrente o traz, senão o ÍNDICE — ver a cerca no
/// cabeçalho.
///
/// ⚠️ O `id` é lido com `max(0)` e arredondado: um `id` negativo ou não-finito cai em `0`, que é o
/// mesmo que a lei de hoje faz no `sim.collide`.
#[must_use]
pub fn chaves(s: &Stream, n: usize) -> Vec<u32> {
    match s.get("id") {
        Some(Column::Scalar(v)) if v.len() == n => v
            .iter()
            .map(|x| {
                #[expect(
                    clippy::cast_sign_loss,
                    clippy::cast_possible_truncation,
                    reason = "uma identidade e' um inteiro >= 0, e o `max(0)` garante-o"
                )]
                let k = x.max(0.0).round() as u32;
                k
            })
            .collect(),
        _ => {
            #[expect(
                clippy::cast_possible_truncation,
                reason = "um indice de corrente cabe em u32 muito antes de a memoria caber na RAM"
            )]
            let v = (0..n).map(|i| i as u32).collect();
            v
        }
    }
}

/// Lê a memória da corrente. Colunas ausentes ⇒ **tudo frio**, que é a lei de hoje **ao bit**.
#[must_use]
pub fn ler(s: &Stream, n: usize) -> Vec<Memoria> {
    let mut m = vec![Memoria::default(); n];
    for (c, nome) in COLUNAS.iter().enumerate() {
        let Some(Column::Vec4(v)) = s.get(nome) else {
            continue;
        };
        if v.len() != n {
            continue;
        }
        for (i, q) in v.iter().enumerate() {
            #[expect(
                clippy::cast_sign_loss,
                clippy::cast_possible_truncation,
                reason = "o parceiro foi escrito por esta mesma porta, de um u32"
            )]
            let (p0, p1) = (q[0].max(0.0) as u32, q[2].max(0.0) as u32);
            m[i].ranhuras[c * 2] = (p0, q[1]);
            m[i].ranhuras[c * 2 + 1] = (p1, q[3]);
        }
    }
    m
}

/// Escreve a memória na corrente de saída.
pub fn escrever(out: &mut Stream, m: &[Memoria]) {
    for (c, nome) in COLUNAS.iter().enumerate() {
        let v: Vec<[f32; 4]> = m
            .iter()
            .map(|x| {
                let (a, b) = (x.ranhuras[c * 2], x.ranhuras[c * 2 + 1]);
                // ⚠️ `u32 → f32` é EXACTO até `2²⁴` — a cerca do TIPO que o cabeçalho declara.
                // ⛔ Um `as u16` aqui truncaria toda cena com mais de 65 536 peças, em silêncio.
                #[expect(
                    clippy::cast_precision_loss,
                    reason = "exacto ate' 2^24; a cerca esta' declarada no cabecalho"
                )]
                let par = [a.0 as f32, a.1, b.0 as f32, b.1];
                par
            })
            .collect();
        out.set((*nome).to_string(), Column::Vec4(v));
    }
}

#[cfg(test)]
#[path = "warm_tests.rs"]
mod tests;
