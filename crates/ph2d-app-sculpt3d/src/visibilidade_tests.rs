//! ⭐⭐⭐ **O OLHO DA HIERARQUIA CHEGA AO PINCEL** — os gates da regra *«um alvo
//! ESCONDIDO não conta»* (`SPEC_unblocked_brushes.md` §6.1).
//!
//! ⚠️ **Puros, sem device e sem mundo**, e é essa a razão de a lei viver numa
//! função livre ([`crate::space::aparece`]) em vez de num método da cena: a
//! `Sculpt3dScene` pede um `wgpu::Device`, logo um gate sobre ela nasceria
//! `#[ignore]` e **o CI nunca o correria**. *A mesma lição que a
//! [`crate::recusa`] pagou.*

use std::collections::BTreeSet;

use crate::ObjectId;
use crate::space::aparece;

fn conjunto(ids: &[u32]) -> BTreeSet<ObjectId> {
    ids.iter().copied().map(ObjectId).collect()
}

/// ⭐⭐⭐ **AS DUAS METADES DE «ESTA PEÇA APARECE?», e elas são independentes.**
///
/// ⚠️ **A metade NEGATIVA de cada uma é metade do valor:** sem ela, uma lei que
/// respondesse `false` sempre passaria as afirmações positivas — e o pincel
/// ficaria inerte com a cena inteira à vista.
#[test]
fn o_isolamento_e_o_olho_escondem_os_dois_e_independentemente() {
    let (a, b) = (ObjectId(1), ObjectId(2));
    let nenhuma = conjunto(&[]);

    // (1) Sem isolamento e sem olho fechado, aparecem as duas.
    assert!(aparece(a, None, &nenhuma) && aparece(b, None, &nenhuma));

    // (2) O ISOLAMENTO: só a isolada aparece.
    assert!(aparece(a, Some(a), &nenhuma), "a isolada tem de aparecer");
    assert!(!aparece(b, Some(a), &nenhuma), "a outra tem de sumir");

    // (3) O OLHO: independente do isolamento.
    assert!(!aparece(b, None, &conjunto(&[2])), "o olho fechado esconde");
    assert!(aparece(a, None, &conjunto(&[2])), "e só a que ele nomeia");

    // (4) ⭐ E as duas COMPÕEM: a própria isolada pode estar escondida.
    assert!(
        !aparece(a, Some(a), &conjunto(&[1])),
        "isolar não reabre o olho — são duas perguntas, e as duas têm de dizer \
         sim"
    );
}

/// ⛔⛔⛔ **E A LEI TEM CHAMADOR — as três rotas** que a espec §6.1 toca.
///
/// ⚠️⚠️ *Uma porta com a lei certa e zero chamadores produz o MESMO app que uma
/// lei ausente*, e esta crate já pagou essa forma. Os dois pen-downs (o do
/// traço e o do filtro de tecido) tinham o laço escrito **letra a letra**, e uma
/// cláusula nova entraria só num deles.
///
/// ⚠️ **`include_str!` e não `read_to_string`:** se um destes ficheiros mudar de
/// nome isto **deixa de compilar**, em vez de ficar verde a medir nada.
#[test]
fn as_tres_rotas_perguntam_a_mesma_porta() {
    const ROTAS: [(&str, &str, &str); 3] = [
        (
            "o pen-down do traço",
            include_str!("input_down.rs"),
            "alvos_visiveis()",
        ),
        (
            "o pen-down do filtro",
            include_str!("filter.rs"),
            "alvos_visiveis()",
        ),
        (
            "a recusa em voz alta",
            include_str!("recusa.rs"),
            "alvos_visiveis()",
        ),
    ];
    for (quem, texto, agulha) in ROTAS {
        let corpo: String = texto
            .lines()
            .map(str::trim)
            .filter(|l| !l.starts_with("//"))
            .collect::<Vec<_>>()
            .join("\n");
        assert!(
            corpo.contains(agulha),
            "{quem} não pergunta à porta `{agulha}` — ela vai divergir da \
             cláusula do «escondido» no dia seguinte"
        );
        // ⛔ E a metade que impede a recaída: o laço à mão que a porta
        // substituiu não pode voltar.
        assert!(
            !corpo.contains("if i != activo"),
            "{quem} voltou a filtrar o activo à mão — era assim que a cláusula \
             do «escondido» ficava só num dos dois"
        );
    }
    // ⭐ E o ESPELHO: sem quem o encha, o conjunto é sempre vazio e as três
    // rotas acima ficam verdes sobre uma lei inerte.
    let sync: String = include_str!("entities.rs")
        .lines()
        .map(str::trim)
        .filter(|l| !l.starts_with("//"))
        .collect::<Vec<_>>()
        .join("\n");
    assert!(
        sync.contains("scene.escondidas = escondidas_do_mundo(sim)"),
        "ninguém enche o espelho do olho da Hierarquia — o conjunto fica vazio \
         e a regra §6.1 é inalcançável"
    );
}
