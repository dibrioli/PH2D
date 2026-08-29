//! Gates da **duração desigual por quadro** — a lei, e o que a mantém idêntica nos dois
//! motores.

use super::*;

/// Quantas células a tira de teste tem.
const CELLS: u32 = 4;

/// A sequência de células que uma fase crescente mostra, amostrada fino.
fn walk(text: &str, samples: usize) -> Vec<i32> {
    let lut = table(text);
    (0..samples)
        .map(|s| {
            let k = s as f32 / samples as f32 * CELLS as f32;
            held_index(&lut, k, CELLS) as i32
        })
        .collect()
}

/// Quantas amostras cada célula ocupa, por ordem.
fn runs(seq: &[i32]) -> Vec<(i32, usize)> {
    let mut out: Vec<(i32, usize)> = Vec::new();
    for c in seq {
        match out.last_mut() {
            Some((v, n)) if v == c => *n += 1,
            _ => out.push((*c, 1)),
        }
    }
    out
}

/// ⚠️ **SEM pesos, o nó é o que sempre foi — e a régua é o `k` INTACTO.**
///
/// A sentinela devolve o `k` de entrada sem lhe tocar, então o `cell_xform` a jusante recebe
/// exactamente o mesmo número. Uma afirmação sobre a célula final seria mais fraca: ela
/// coincidiria por acaso em qualquer fase que caísse no meio de uma célula.
#[test]
fn without_holds_the_index_passes_through_untouched_bit_for_bit() {
    let lut = table("");
    for k in [0.0f32, 0.37, 1.999, 3.5, -2.25, 17.0] {
        assert_eq!(
            held_index(&lut, k, CELLS).to_bits(),
            k.to_bits(),
            "k = {k} tem de sair intacto"
        );
    }
    // E o mesmo para as formas de lista que NÃO são uma autoria: um número só não é um
    // ritmo, e uma lista de lixo não é uma lista.
    for junk in ["   ", "1", "abc", "0 0 0", "-1 -2"] {
        assert_eq!(
            held_index(&table(junk), 1.75, CELLS).to_bits(),
            1.75f32.to_bits(),
            "{junk:?} nao e' uma autoria"
        );
    }
}

/// ⭐ **A LEI: `1 1 3 1` faz a terceira célula durar o triplo.**
///
/// A régua é quanto do CICLO cada célula ocupa. As três de peso `1` têm de ficar iguais entre
/// si, e a de peso `3` tem de ficar ~3× maior — a menos da resolução da tabela.
#[test]
fn a_weight_of_three_makes_that_cell_last_three_times_as_long() {
    let r = runs(&walk("1 1 3 1", 600));
    assert_eq!(
        r.iter().map(|(c, _)| *c).collect::<Vec<_>>(),
        vec![0, 1, 2, 3],
        "as quatro celulas em ordem: {r:?}"
    );
    let n: Vec<f32> = r.iter().map(|(_, n)| *n as f32).collect();
    let unit = (n[0] + n[1] + n[3]) / 3.0;
    for (j, want) in [(0usize, 1.0f32), (1, 1.0), (2, 3.0), (3, 1.0)] {
        let got = n[j] / unit;
        assert!(
            (got - want).abs() < 0.15,
            "a celula {j} devia durar {want}x e durou {got:.2}x ({r:?})"
        );
    }
}

/// ⚠️ **O CICLO NÃO MUDA DE DURAÇÃO** — os pesos redistribuem, não aceleram.
///
/// É a propriedade que impede uma segunda resposta a *«quão rápido»*: o `Cells / Second`
/// continua a mandar no tempo total, qualquer que seja a lista. Sem isto, uma lista
/// `"1 1 3 1"` faria o ciclo durar `6/4` do que o slider diz e o artista veria o número
/// mentir.
#[test]
fn the_weights_redistribute_the_cycle_and_never_stretch_it() {
    for text in ["", "1 1 1 1", "1 1 3 1", "5 1 1 1", "2 7 1 4"] {
        let lut = table(text);
        // A fase `k` e `k + CELLS` são o MESMO ponto do ciclo, seja qual for a lista.
        for k in [0.1f32, 1.3, 2.75, 3.9] {
            let a = held_index(&lut, k, CELLS);
            let b = held_index(&lut, k + CELLS as f32, CELLS);
            assert_eq!(
                (a.rem_euclid(CELLS as f32)) as i32,
                (b.rem_euclid(CELLS as f32)) as i32,
                "{text:?}: o ciclo tem de fechar em {CELLS} celulas (k = {k})"
            );
        }
    }
}

/// **Toda célula da lista é alcançável, e nenhuma extra aparece.**
///
/// Uma busca de prefixo com o `<` do lado errado saltaria a primeira ou a última — e o
/// sintoma seria *"a minha animação não mostra o primeiro quadro"*, que ninguém liga aos
/// pesos.
#[test]
fn every_authored_cell_is_reached_and_no_stranger_appears() {
    for text in ["1 1 3 1", "9 1 1 1", "1 1 1 9", "1 2 4 8"] {
        let mut seen: Vec<i32> = walk(text, 2000);
        seen.sort_unstable();
        seen.dedup();
        assert_eq!(seen, vec![0, 1, 2, 3], "{text:?} mostrou {seen:?}");
    }
}

/// ⚠️ **Uma fase NEGATIVA conta do fim** — a mesma lei do `rem_euclid` do [`cell_xform`], e
/// pelo mesmo motivo: um `stagger` para trás põe elementos em `k < 0`.
#[test]
fn a_negative_phase_counts_from_the_end_of_the_cycle() {
    let lut = table("1 1 3 1");
    for k in [0.25f32, 1.5, 2.9, 3.1] {
        let a = held_index(&lut, k, CELLS);
        let b = held_index(&lut, k - CELLS as f32, CELLS);
        assert_eq!(a as i32, b as i32, "k = {k} contra k - {CELLS}");
    }
}

/// **O amostrador é o do codegen, letra a letra** — as pontas, o passo, e a mistura.
///
/// ⚠️ Ele é uma cópia deliberada (o original vive no gerador do WGSL, que uma crate de nó não
/// alcança), então o que o mantém honesto é este gate mais o de paridade de GPU. Aqui
/// fixam-se as três propriedades que o texto do gerador promete: **grampa** fora de `[0,1]`,
/// **acerta** nas entradas, e **mistura** linearmente entre vizinhas.
#[test]
fn the_sampler_matches_the_generated_accessors_three_promises() {
    let lut = [0.0f32, 1.0, 2.0, 3.0];
    assert_eq!(sample(&lut, -5.0), 0.0, "grampa em baixo");
    assert_eq!(sample(&lut, 5.0), 3.0, "grampa em cima");
    for (k, want) in [(0usize, 0.0f32), (1, 1.0), (2, 2.0), (3, 3.0)] {
        let t = k as f32 / 3.0;
        assert!(
            (sample(&lut, t) - want).abs() < 1e-5,
            "acerta na entrada {k}"
        );
    }
    assert!(
        (sample(&lut, 0.5 / 3.0) - 0.5).abs() < 1e-5,
        "mistura a meio caminho"
    );
    assert_eq!(
        sample(&[], 0.5),
        NO_HOLDS,
        "uma tabela vazia e' `sem holds`"
    );
}

/// **A tabela é o degrau, e a sentinela cobre-a inteira quando nada foi autorado.**
#[test]
fn the_table_is_a_staircase_when_authored_and_a_sentinel_when_not() {
    let empty = table("");
    assert!(
        empty.iter().all(|v| *v == NO_HOLDS),
        "a sentinela tem de cobrir a tabela toda — senao a interpolacao produz um valor \
         legitimo perto da borda e um dos motores toma o outro ramo"
    );
    let held = table("1 1 3 1");
    assert_eq!(held.len(), HOLD_LUT_RESOLUTION as usize);
    assert!(
        held.iter().all(|v| (0.0..1.0).contains(v)),
        "a saida vive em [0,1)"
    );
    // Monótona: uma fase que anda para a frente nunca mostra uma célula anterior.
    for w in held.windows(2) {
        assert!(w[1] >= w[0] - 1e-6, "a escada tem de subir: {w:?}");
    }
}

/// Os pesos só contam a partir de DOIS — um número só não é um ritmo, e um ritmo de uma
/// célula é a própria ausência de ritmo.
#[test]
fn the_weights_need_at_least_two_cells_to_mean_anything() {
    assert!(weights("").is_none());
    assert!(weights("3").is_none());
    assert_eq!(weights("1 2").map(|w| w.len()), Some(2));
    // Separadores: os que a mão usa.
    assert_eq!(weights("1,2;3\t4").map(|w| w.len()), Some(4));
    // Um peso não-positivo é DESCARTADO, não coagido — ver a nota de [`weights`].
    assert_eq!(weights("1 0 2 -3").map(|w| w.len()), Some(2));
}

/// ⚠️ **UMA FRONTEIRA É FECHADA À ESQUERDA** — em `t` exactamente igual ao início de uma
/// célula, é ELA que já está no ecrã, não a anterior.
///
/// ⚠️ **Este gate nasceu de uma mutação que SOBREVIVEU**: trocar o `<` da busca de prefixo
/// por `<=` não matou nenhum dos oito gates irmãos, porque a amostragem uniforme da tabela
/// quase nunca cai exactamente numa fronteira — `6k/511` não é inteiro para `k` nenhum. A
/// igualdade é um conjunto de medida nula *na tabela*, e uma lei na mesma: é ela que faz os
/// intervalos das células **compor** em vez de se sobreporem por um ponto.
///
/// ⭐ Os pesos são `1 1 2 4` de propósito: o total é `8`, então cada fronteira é uma potência
/// de dois e `t · total` é **exacto em `f32`**. Com `1 1 3 1` a fronteira cairia em `1/6`, que
/// não é representável, e o gate mediria o arredondamento em vez da lei.
#[test]
fn a_cell_boundary_belongs_to_the_cell_that_starts_there() {
    let w = [1.0f32, 1.0, 2.0, 4.0];
    let cell = |t: f32| (cell_fraction(&w, t) * w.len() as f32).floor() as i32;
    for (t, want) in [(0.0f32, 0), (0.125, 1), (0.25, 2), (0.5, 3)] {
        assert_eq!(cell(t), want, "em t = {t} ja' tem de estar a celula {want}");
    }
    // E o CONTROLE do outro lado: um epsilon ANTES da fronteira ainda é a célula anterior.
    for (t, want) in [(0.125f32 - 1e-4, 0), (0.25 - 1e-4, 1), (0.5 - 1e-4, 2)] {
        assert_eq!(cell(t), want, "logo antes de {t} ainda e' a celula {want}");
    }
}
