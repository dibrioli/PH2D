//! **A ordem do navegador que o ARTISTA arrastou** (23/10). A ordem por omissão é a da biblioteca (pasta,
//! série de lápis, nome — determinística); a manual é uma lista de caminhos relativos no `estado.json`, ao
//! lado dos favoritos. As duas convivem por [`mesclar`]: o que a manual conhece vem na ordem dela; um
//! pincel que ela não conhece (novo, importado, de fábrica numa versão nova) entra logo depois do vizinho
//! que o precede na ordem por omissão; um caminho que já não existe cai. Sem buracos nem duplicados.

/// A ordem que se vê: `manual` sobre `padrao` (ver o doc do módulo). `padrao` sem repetidos.
#[must_use]
pub fn mesclar(padrao: &[String], manual: &[String]) -> Vec<String> {
    let existe: std::collections::BTreeSet<&str> = padrao.iter().map(String::as_str).collect();
    let mut vistos = std::collections::BTreeSet::new();
    let mut out: Vec<String> = manual
        .iter()
        .filter(|r| existe.contains(r.as_str()) && vistos.insert(r.as_str()))
        .cloned()
        .collect();
    for (i, r) in padrao.iter().enumerate() {
        if vistos.contains(r.as_str()) {
            continue;
        }
        let pos = match i.checked_sub(1) {
            Some(j) => out
                .iter()
                .position(|x| *x == padrao[j])
                .map_or(0, |p| p + 1),
            None => 0,
        };
        out.insert(pos, r.clone());
        vistos.insert(r.as_str());
    }
    out
}

/// `lista` com `arrastado` posto antes (ou `depois`) de `alvo`. `None` quando um dos dois não está nela
/// ou são o mesmo — nada muda.
#[must_use]
pub fn mover(lista: &[String], arrastado: &str, alvo: &str, depois: bool) -> Option<Vec<String>> {
    if arrastado == alvo || !lista.iter().any(|r| r == alvo) {
        return None;
    }
    let de = lista.iter().position(|r| r == arrastado)?;
    let mut out = lista.to_vec();
    let item = out.remove(de);
    let a = out.iter().position(|r| r == alvo)?;
    out.insert(if depois { a + 1 } else { a }, item);
    Some(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn v(s: &[&str]) -> Vec<String> {
        s.iter().map(|x| (*x).to_owned()).collect()
    }

    #[test]
    fn sem_ordem_manual_e_a_por_omissao() {
        assert_eq!(mesclar(&v(&["a", "b", "c"]), &[]), v(&["a", "b", "c"]));
    }

    #[test]
    fn o_novo_entra_ao_lado_do_vizinho_e_o_apagado_cai() {
        // Manual: c antes de a. Novo «a 2» (depois de «a» na omissão) e «0» (o primeiro); «x» foi apagado.
        let padrao = v(&["0", "a", "a 2", "b", "c"]);
        let manual = v(&["c", "x", "a", "b", "c"]);
        assert_eq!(mesclar(&padrao, &manual), v(&["0", "c", "a", "a 2", "b"]));
    }

    #[test]
    fn mover_antes_e_depois() {
        let l = v(&["a", "b", "c"]);
        assert_eq!(mover(&l, "c", "a", false), Some(v(&["c", "a", "b"])));
        assert_eq!(mover(&l, "a", "c", true), Some(v(&["b", "c", "a"])));
        assert_eq!(mover(&l, "a", "a", true), None);
        assert_eq!(mover(&l, "z", "a", true), None);
    }
}
