//! ⏱️ **A SONDA DA DOBRA QUE RASGA** — filho do [`super`] para herdar a fixtura do braço.
//!
//! ⛔⛔ **Ela existe porque duas medições da mesma grandeza discordam.** A tabela de recusas do
//! módulo diz *«a `2,08 ×` a meia-altura da arte são **zero** pontos invertidos até `150°`»* — e a
//! foto da cena `=2`, tirada a `25°` por junta (`50°` na ponta), mostra a arte **rasgada em duas**.
//!
//! ⭐⭐⭐ **A contradição resolve-se pela DATA, e é o §0.0 outra vez:** aquela recusa é de
//! **2026-09-14** e mede a lei de peso **derivada por distância**; o bind passou ao **padrão-ouro
//! (Bounded Biharmonic)** em **2026-09-15**. *Quem move o número que tornava algo inalcançável tem
//! de reconferir a nota* — e ninguém reconferiu: **a dobra sob a lei de HOJE nunca foi medida.**
//!
//! ⚠️ **A régua é o DETERMINANTE por triângulo, não o olho.** Um triângulo cuja área trocou de
//! sinal foi virado do avesso: ele desaparece do desenho (o recorte fica vazio) e o buraco lê-se
//! como um RASGO. *Contar quantos rasga é a única forma de separar «a lei está errada» de «a cena
//! pediu demais».*

use super::*;

/// Quantos triângulos da malha `m` estão **do avesso**, e o pior factor de área.
///
/// ⚠️ **O repouso é a UV** (a posição do vértice no quad, antes de a pele lhe tocar) e o deformado é
/// o `local` — as duas listas são do MESMO vértice, índice a índice, e é isso que faz a comparação
/// ser por triângulo e não por silhueta.
///
/// ⛔⛔ **A convenção de `v` da UV é invertida** (`½ − qy`), logo o sinal de TODA a malha em repouso
/// vem trocado — e a 1.ª redacção desta régua leu a discordância crua e acusou **`3 593` de `3 593`
/// triângulos do avesso com a corrente A ZERO GRAUS**. *Uma régua que acusa o repouso não mede
/// deformação nenhuma.* ⇒ o sinal do repouso é **desfeito** (`-area2`), nunca interpretado, e o
/// controlo da sonda é a linha dos `0°`: ali tem de ler `0` e `+1`.
fn do_avesso(m: &SpriteMesh) -> (usize, usize, f64) {
    fn area2(a: [f32; 2], b: [f32; 2], c: [f32; 2]) -> f64 {
        let (ax, ay) = (f64::from(a[0]), f64::from(a[1]));
        let (bx, by) = (f64::from(b[0]), f64::from(b[1]));
        let (cx, cy) = (f64::from(c[0]), f64::from(c[1]));
        (bx - ax) * (cy - ay) - (by - ay) * (cx - ax)
    }
    let mut avesso = 0;
    let mut pior = f64::INFINITY;
    for t in &m.tris {
        let [i, j, k] = t.map(|v| v as usize);
        let rest = -area2(m.uv[i], m.uv[j], m.uv[k]);
        let viva = area2(m.local[i], m.local[j], m.local[k]);
        // Um triângulo degenerado em repouso não tem sinal para trair — a régua salta-o em vez de
        // dividir por zero e inventar um extremo.
        if rest.abs() < 1e-12 {
            continue;
        }
        let razao = viva / rest;
        pior = pior.min(razao);
        if razao < 0.0 {
            avesso += 1;
        }
    }
    (avesso, m.tris.len(), pior)
}

/// ⭐⭐⭐ **A PELE NÃO VIRA UM TRIÂNGULO DO AVESSO ATÉ `75°` POR JUNTA** — a lei que a sonda mediu,
/// promovida a gate porque uma medição sem gate envelhece (e esta já envelheceu uma vez).
///
/// ⚠️ **Ela é uma propriedade da LEI DE PESO, não da cena** — por isso a barra é o ângulo em que a
/// medição mudou de resposta, e não o ângulo que um smoke por acaso arma hoje. *Um gate que espelha
/// uma constante de outra crate é um espelho a envelhecer; um que defende o vale medido, não.*
///
/// ⛔⛔ **O CONTROLO POSITIVO é metade do gate:** a `90°` por junta a corrente dobra-se por completo
/// sobre si e a régua TEM de ver triângulos virados. Sem ele, uma régua partida — a 1.ª redacção
/// desta lia `3 593` de `3 593` **com a corrente a zero graus** — passaria a metade de cima a dizer
/// *«zero invertidos»* sobre um produto que ela não estava a medir.
#[test]
fn a_pele_nao_vira_um_triangulo_ate_setenta_e_cinco_graus() {
    for graus in [0.0_f32, 13.0, 25.0, 50.0, 75.0] {
        let malhas = braco_desenhado(0, graus);
        let (m, _) = malhas.first().expect("a sprite simples desenha");
        let (avesso, total, pior) = do_avesso(m);
        assert_eq!(
            avesso, 0,
            "a {graus}° por junta a pele virou {avesso} de {total} triangulos do avesso \
             (pior factor de area {pior:.4}) — eles somem do desenho e a arte le-se RASGADA"
        );
        assert!(
            pior > 0.0,
            "a {graus}° o pior factor de area e' {pior:.4}: a regua nao concorda consigo mesma"
        );
    }
    // O CONTROLO: com a corrente dobrada por completo, a régua TEM de ver o fenómeno.
    let malhas = braco_desenhado(0, 90.0);
    let (m, _) = malhas.first().expect("a sprite simples desenha");
    let (avesso, total, pior) = do_avesso(m);
    assert!(
        avesso > 0 && pior < 0.0,
        "a 90° por junta (a corrente dobrada sobre si) a regua leu {avesso} de {total} do avesso e \
         pior factor {pior:.4} — ela deixou de ver o fenomeno que a metade de cima nega"
    );
}

/// ⏱️ **A DOBRA sob a lei de peso de HOJE, ângulo a ângulo** — a medição que a recusa de 14/09 já
/// não descreve.
///
/// Corre pelo filtro `sonda_da_dobra` com `--ignored --nocapture`.
#[test]
#[ignore = "sonda: imprime a tabela, nao afirma uma barra"]
fn sonda_da_dobra() {
    println!(
        "graus/junta | ponta | triangulos do avesso | pior factor de area (<0 = virado)\n\
         ------------+-------+----------------------+----------------------------------"
    );
    for graus in [
        0.0_f32, 5.0, 10.0, 13.0, 20.0, 25.0, 30.0, 40.0, 50.0, 60.0, 75.0, 90.0,
    ] {
        let malhas = braco_desenhado(0, graus);
        let (m, _) = malhas.first().expect("a sprite simples desenha");
        let (avesso, total, pior) = do_avesso(m);
        println!(
            "{graus:>11.0} | {:>5.0} | {avesso:>8} de {total:<9} | {pior:>10.4}",
            graus * 2.0,
        );
    }
}
