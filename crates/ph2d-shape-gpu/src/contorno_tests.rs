use super::ao_oitavo_do_degrau;

/// doc 121 §9.12: a `=127` densa pede `107 520` células — `114 688` em vez das `131 072` da potência
/// de dois (`46 MB` em vez de `52 MB` a `400 B` por célula).
#[test]
fn a_capacidade_das_celulas_da_cena_densa_sobe_ao_oitavo_do_degrau() {
    assert_eq!(ao_oitavo_do_degrau(107_520), 114_688);
}

/// A capacidade cobre sempre o pedido, sobra-lhe MENOS de um oitavo dele (a potência de dois deixava
/// até o dobro) e não desce quando o pedido sobe — a capacidade só cresce.
#[test]
fn a_capacidade_cobre_o_pedido_com_menos_de_um_oitavo_de_folga() {
    let mut anterior = 0;
    for n in 1..=(1u64 << 18) {
        let cap = ao_oitavo_do_degrau(n);
        assert!(cap >= n, "{n} -> {cap}");
        assert!((cap - n) * 8 < n.max(8), "{n} -> {cap}");
        assert!(cap >= anterior, "{n} -> {cap} < {anterior}");
        anterior = cap;
    }
    // As potências de dois não ganham folga nenhuma.
    assert_eq!(ao_oitavo_do_degrau(1 << 20), 1 << 20);
}

/// O preço do arredondamento fino: numa cena que CRESCE uma célula de cada vez (o pior caso — cada
/// medição nova passa a capacidade), os buffers recriam-se no máximo `8` vezes por oitava, contra `1`
/// da potência de dois.
#[test]
fn numa_cena_que_cresce_as_celulas_recriam_se_no_maximo_oito_vezes_por_oitava() {
    let recriacoes = |arredonda: fn(u64) -> u64| {
        let (mut cap, mut vezes) = (0, 0);
        for pedido in 1..=107_520u64 {
            if pedido > cap {
                cap = arredonda(pedido);
                vezes += 1;
            }
        }
        vezes
    };
    let potencia = recriacoes(u64::next_power_of_two);
    let oitavo = recriacoes(ao_oitavo_do_degrau);
    assert_eq!(potencia, 18);
    assert!(
        oitavo <= 8 * potencia,
        "{oitavo} recriações contra {potencia}"
    );
}
