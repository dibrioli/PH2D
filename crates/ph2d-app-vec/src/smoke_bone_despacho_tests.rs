//! ⚠️⚠️ **ESTES GATES VIVEM NUM FICHEIRO IRMÃO, e a razão é MEDIDA.**
//!
//! Eles leem o `smoke_bone.rs` por `include_str!` e CONTAM a agulha. ⛔ Escritos lá dentro, a agulha do
//! próprio teste entrava na conta — as duas primeiras redacções reprovaram exactamente assim
//! (`3` contra `2`, `2` contra `1`) —, e a cura barata (subir o número esperado) tornaria o gate
//! **inerte**: apagar a chamada de produto deixá-lo-ia verde.
//!
//! *Um `include_str!` que procura uma string escrita nele próprio não afirma nada.* Esta casa já
//! pagou a mesma lição no rótulo do painel do esqueleto.

/// ⭐⭐⭐ **E O DESPACHO TEM DE EXISTIR NOS DOIS TEMPOS** — sem esta metade a lei acima afirma
/// sobre um número que ninguém lê.
///
/// ⛔⛔ *Um roteador que devolve o nível certo e um corpo que nunca o consulta leem-se
/// exactamente igual num teste de unidade* — e o `=2` abriria a cena `=1`, em silêncio. ⚠️ Por
/// `include_str!` e não por `grep`: se o ficheiro mudar de sítio isto deixa de **compilar**, em
/// vez de passar a varrer zero e ficar verde.
#[test]
fn os_dois_tempos_despacham_o_nivel() {
    const FONTE: &str = include_str!("smoke_bone.rs");
    assert_eq!(
        FONTE.matches("crate::smoke_bone_envelope::").count(),
        2,
        "a cena do envelope deixou de ser alcancavel dos DOIS tempos: com um so', ela monta a \
         arte e nunca a prende (ou prende sem a montar)"
    );
}
