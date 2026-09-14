//! ⛔⛔⛔ **A BANCADA DE PARIDADE DO ESFREGÃO NÃO PODE EXISTIR HOJE — e este
//! ficheiro é a MEDIÇÃO que o diz, com a catraca que a acorda.**
//!
//! A espec `SPEC_unblocked_brushes.md` §7 declara que o bloco `l` das fixturas
//! *«torna as famílias de multirresolução utilizáveis sem termos o §2.3
//! pronto»*, e o [`README`](../../../docs/3D/cleanroom/fixtures/unblocked/README.md)
//! repete-o. ⭐ **Isso é verdade para o APAGADOR e FALSO para o ESFREGÃO**, e a
//! diferença é a lei:
//!
//! | pincel | a lei lê | as fixturas trazem |
//! |---|---|---|
//! | *Erase* | **um ponto** (`p`, `R[v]`) | tudo o que ela precisa |
//! | *Smear* | **o ANEL** (`R[w]` de cada vizinho) | ⛔ **nenhuma conectividade** |
//!
//! As **12** fixturas da família `esfregao` trazem os blocos `r`, `l`, `s` e `c`
//! — e **nenhum bloco de faces**, apesar de a linha `blocos:` do cabeçalho delas
//! listar `f=faces` na legenda genérica. *Sem faces não há anel, e sem anel não
//! há lei para correr.*
//!
//! # ⛔ E a conectividade NÃO é recuperável — medido, não suposto
//!
//! A contagem bate exactamente com um cubo subdividido cinco vezes
//! (`8 → 26 → 98 → 386 → 1 538 → **6 146**`), e a caixa do bloco `l` é
//! perfeitamente simétrica (`±0,93268955` nos três eixos), o que é a assinatura
//! de uma superfície-limite de cubo. ⚠️ **Mas a bijecção POR POSIÇÃO falha:**
//! escalando a nossa superfície-limite pelo factor que iguala as caixas
//! (`k = 2,2220`), o emparelhamento por vizinho mais próximo dá **`528`
//! colisões** e pior distância **`3,04e-2`** — da ordem do espaçamento da
//! grelha (`~4,2e-2`). ⇒ *a malha do oráculo não é a nossa subdivisão
//! reescalada, e adivinhar a permutação produziria uma bancada que mede outra
//! malha.*
//!
//! # O que fica, e quem o paga
//!
//! ⏳ **Dívida NOMEADA, e é acto do E:** uma emenda às fixturas que emita o
//! bloco de faces do nível de topo. Com ele, esta bancada nasce em meia hora e o
//! pincel passa a ter o mesmo grau de prova que o de tecido (86 traços) e o de
//! pose (69 fixturas).
//!
//! ⚠️ **Até lá a lei é cobrada por gates de FORMA FECHADA** sobre uma fixtura
//! nossa em que ela se calcula à mão — `ph2d-sculpt3d/src/verb_smear_tests.rs`,
//! cuja tabela de quatro linhas é a espec §5.2 escrita em aritmética. *Uma
//! barra derivada da lei é mais fraca que o oráculo e mais forte que uma barra
//! calibrada sem o lado aprovado* (§8.3).

use std::path::PathBuf;

fn fixture_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../docs/3D/cleanroom/fixtures/unblocked/esfregao")
}

/// **DESCOMPRIME um `.gz`** — a gémea da do [`super::oraculo_dos_gestos_tangenciais`]:
/// o cabeçalho do gzip tem tamanho variável e o rabo são oito bytes.
fn inflar(caminho: &std::path::Path) -> String {
    let raw = std::fs::read(caminho).unwrap_or_else(|e| panic!("{caminho:?}: {e}"));
    assert!(
        raw.len() > 18 && raw[0] == 0x1f && raw[1] == 0x8b,
        "{caminho:?}: nao e' gzip"
    );
    let flg = raw[3];
    let mut off = 10usize;
    if flg & 0x04 != 0 {
        off += 2 + (usize::from(raw[off]) | (usize::from(raw[off + 1]) << 8));
    }
    for bit in [0x08u8, 0x10] {
        if flg & bit != 0 {
            while raw[off] != 0 {
                off += 1;
            }
            off += 1;
        }
    }
    if flg & 0x02 != 0 {
        off += 2;
    }
    let bytes = miniz_oxide::inflate::decompress_to_vec(&raw[off..raw.len() - 8])
        .unwrap_or_else(|e| panic!("{caminho:?}: nao inflou: {e:?}"));
    String::from_utf8(bytes).expect("utf-8")
}

/// ⛔⛔ **A CATRACA DA DÍVIDA: no dia em que uma fixtura do esfregão trouxer
/// FACES, este teste reprova e manda construir a bancada.**
///
/// ⚠️ **Ele tem as duas metades que uma catraca honesta exige:**
///
/// 1. o **piso de população** — `12` fixturas. *Um censo que varre zero fica
///    trivialmente verde, e foi assim que três gates deste repo morreram mudos
///    ao mudar de directório.*
/// 2. a **obsolescência** — a ausência do bloco `f`. O dia em que ele aparecer,
///    a dívida deixou de existir e a nota acima passa a MENTIR.
#[test]
fn as_fixturas_do_esfregao_ainda_nao_trazem_a_conectividade() {
    let mut vistas = 0usize;
    let mut com_faces = Vec::new();
    for e in std::fs::read_dir(fixture_dir()).expect("a pasta das fixturas") {
        let caminho = e.expect("entrada").path();
        if caminho.extension().is_none_or(|x| x != "gz") {
            continue;
        }
        vistas += 1;
        let texto = inflar(&caminho);
        let prefixos: std::collections::BTreeSet<&str> = texto
            .lines()
            .filter_map(|l| l.split_whitespace().next())
            .collect();
        assert!(
            prefixos.contains("l"),
            "{caminho:?}: sem o bloco `l`, a família não é a que este gate descreve"
        );
        if prefixos.contains("f") || prefixos.contains("fr") || prefixos.contains("fs") {
            com_faces.push(caminho);
        }
    }
    assert_eq!(
        vistas, 12,
        "o piso de população: a espec §7 conta 12 fixturas de `esfregao`"
    );
    assert!(
        com_faces.is_empty(),
        "⭐ A DÍVIDA FECHOU: {} fixtura(s) já trazem faces ({com_faces:?}).\n\
         Construa a bancada de paridade e APAGUE este gate — o cabeçalho deste \
         ficheiro passou a mentir.",
        com_faces.len()
    );
}
