//! ⭐⭐⭐ **QUANTO É QUE A LEI NOVA MOVE O DESENHO** — o gate do TAMANHO e a imagem.
//!
//! ⛔⛔⛔ **Este ficheiro existe por um erro meu.** O irmão [`super::lei_tests`] mede a aresta de
//! dentro do cotovelo e lê `5,1×` a `90°`; eu reportei esse número ao dono como se fosse o tamanho
//! da mudança. Ele é um **MÍNIMO sobre o contorno** — um extremo LOCAL —, e desenhadas lado a lado
//! as duas leis ficam **quase uma em cima da outra**: o desenho move-se `4,3 %` da espessura, com
//! metade do contorno a não mexer um fio.
//!
//! ⇒ *uma régua LOCAL não diz o TAMANHO do que se vê*, e quem cita o gate do irmão cita este.
//! ⚠️ E foi a IMAGEM que me corrigiu, não a tabela — é por isso que a sonda do SVG mora aqui.

use super::ouro_reguas_tests::*;

/// ⭐⭐⭐ **GATE — A LEI NOVA É LOCAL: ela move o COTOVELO, não a forma.**
///
/// ⛔⛔ **Ele é o par obrigatório do gate acima**, e existe porque eu li um `5,1×` local e reportei-o
/// ao dono como se fosse o tamanho da mudança. Medido ponto a ponto entre as duas leis, em unidades
/// da ESPESSURA da barra (`1,0`, que é a feição onde o defeito vive):
///
/// | `45°` | `70°` | `90°` |
/// |---:|---:|---:|
/// | `0,5 %` | `1,9 %` | `4,3 %` |
///
/// ⚠️ **As duas metades, e a segunda é a que me faltava:** a lei **muda** alguma coisa (senão a
/// porta seria inerte) **e** o que ela muda cabe numa fracção pequena da peça, com **metade do
/// contorno a não se mexer um bit**. *Uma régua LOCAL não diz o tamanho do que se vê.*
#[test]
fn a_lei_desdobrada_e_local_e_nao_refaz_a_forma() {
    use ph2d_skeleton::MisturaDoAngulo;
    const ESPESSURA: f64 = 1.0;
    let mut p = b_palco(true);
    // ⛔⛔ **A amostra é uniforme em ARCO e não por segmento, e a diferença é o gate inteiro.**
    // A [`b_amostra`] põe `32` pontos em CADA segmento, logo a mediana que ela alimenta é pesada
    // pela contagem de nós — e ela mudou quando o `DIVISOES_POR_OSSO` subiu (`34 → 54`), pondo
    // este gate a `5,6 %` contra a barra de `5 %` **sem uma linha de lei se mexer**. ⚠️ A frase
    // que o gate afirma é *«metade do CONTORNO não se mexe»*, e um contorno mede-se em
    // comprimento: ver [`super::ondulacao_regua_tests`].
    let rest = {
        let denso = b_amostra_com(&p.fonte, 256);
        super::ondulacao_regua_tests::b_no_passo(&denso, &denso, 0.01).0
    };
    for (graus, tecto) in [(45.0_f32, 0.015), (70.0, 0.035), (90.0, 0.070)] {
        p.dobra(graus);
        let pele = p.pele();
        let saida = |lei: MisturaDoAngulo| -> Vec<[f64; 2]> {
            rest.iter()
                .map(|&x| {
                    let mut w = pele.scratch();
                    let linha = p
                        .campo
                        .linha(x)
                        .unwrap_or_else(|| b_mais_proximo(&p.campo, x));
                    pele.weights_corrected(x, Some(&linha), &mut w, &p.correcoes);
                    pele.blend_com(x, &w, lei)
                })
                .collect()
        };
        let (a, b) = (
            saida(MisturaDoAngulo::Circulo),
            saida(MisturaDoAngulo::Desdobrado),
        );
        let mut d: Vec<f64> = a
            .iter()
            .zip(&b)
            .map(|(x, y)| (x[0] - y[0]).hypot(x[1] - y[1]))
            .collect();
        let (p50, _, max) = b_pct(&mut d);
        assert!(
            max > 1e-4,
            "a {graus}° as duas leis deram o MESMO desenho — ou a porta está inerte, \
             ou esta fixtura não as distingue"
        );
        assert!(
            max <= tecto * ESPESSURA,
            "a {graus}° a lei nova moveu o desenho {max:.4} ({:.1} % da espessura), \
             acima do tecto medido de {:.1} %",
            max / ESPESSURA * 100.0,
            tecto * 100.0
        );
        // ⚠️ **A concentração, e ela é uma RAZÃO e não um zero:** a sonda imprime `p50 0,0000` a
        // quatro casas e o número não é zero exacto — metade do contorno mexe-se um fio. O que a
        // lei afirma é que esse fio é uma fracção desprezável do pior ponto.
        assert!(
            p50 <= max * 0.05,
            "a {graus}° a mudança não está concentrada no cotovelo: p50 {p50:.6} contra um \
             máximo de {max:.6} ({:.1} % dele)",
            p50 / max * 100.0
        );
    }
}

/// ⭐⭐⭐ **SONDA B9 — O DESENHO das duas leis, lado a lado, num SVG.**
///
/// ⛔⛔ **Uma tabela não decide sozinha nesta família.** A [`a_lei_desdobrada_empurra_o_bico_e_a_dobra`]
/// diz que o bico vai de `93°` para `121°`; o que isso é na ARTE só a imagem mostra — e esta linha
/// já leu duas tabelas ao contrário por não ter posto o desenho ao lado.
///
/// Escreve `PH2D_LEI_SVG=<ficheiro>`; sem a variável **não faz nada** (uma sonda que escreve no
/// disco a cada corrida da suíte é um efeito colateral, não um instrumento).
#[test]
fn diag_b_desenha_as_duas_leis() {
    use ph2d_skeleton::MisturaDoAngulo;
    let Some(saida) = std::env::var_os("PH2D_LEI_SVG") else {
        return;
    };
    let mut p = b_palco(true);
    let rest = b_amostra(&p.fonte);
    let mut svg = String::from(
        "<svg xmlns='http://www.w3.org/2000/svg' viewBox='-11 -8 22 22' width='1100' height='1100'>\n\
         <rect x='-11' y='-8' width='22' height='22' fill='#1c1c1e'/>\n",
    );
    let poli = |v: &[[f64; 2]]| {
        let mut s = String::new();
        for (i, q) in v.iter().enumerate() {
            s.push_str(&format!(
                "{}{:.4},{:.4} ",
                if i == 0 { "M " } else { "L " },
                q[0],
                -q[1]
            ));
        }
        s.push('Z');
        s
    };
    for (k, graus) in [90.0_f32, 100.0, 105.0].into_iter().enumerate() {
        #[expect(clippy::cast_precision_loss, reason = "k <= 2")]
        let dy = k as f64 * 7.0;
        p.dobra(graus);
        let pele = p.pele();
        svg.push_str(&format!(
            "<g transform='translate(0,{dy})'>\
             <text x='-10.5' y='-5.6' font-size='0.6' fill='#888'>{graus:.0}°</text>\n"
        ));
        let desenha = |lei: MisturaDoAngulo| -> Vec<[f64; 2]> {
            rest.iter()
                .map(|&x| {
                    let mut w = pele.scratch();
                    let linha = p
                        .campo
                        .linha(x)
                        .unwrap_or_else(|| b_mais_proximo(&p.campo, x));
                    pele.weights_corrected(x, Some(&linha), &mut w, &p.correcoes);
                    pele.blend_com(x, &w, lei)
                })
                .collect()
        };
        for (cor, rot, v) in [
            ("#3a3a3e", "repouso", rest.clone()),
            (
                "#e05a4a",
                "CÍRCULO (hoje)",
                desenha(MisturaDoAngulo::Circulo),
            ),
            (
                "#4ab3e0",
                "DESDOBRADO",
                desenha(MisturaDoAngulo::Desdobrado),
            ),
        ] {
            svg.push_str(&format!(
                "<path d='{}' fill='none' stroke='{cor}' stroke-width='0.05'/>\n",
                poli(&v)
            ));
            let _ = rot;
        }
        svg.push_str("</g>\n");
    }
    svg.push_str(
        "<text x='-10.5' y='13.4' font-size='0.5' fill='#e05a4a'>CÍRCULO (o que ship)</text>\n\
         <text x='-10.5' y='14.1' font-size='0.5' fill='#4ab3e0'>DESDOBRADO (a cura)</text>\n\
         <text x='-10.5' y='14.8' font-size='0.5' fill='#3a3a3e'>repouso</text>\n</svg>\n",
    );
    std::fs::write(&saida, svg).expect("escreve o svg");
    println!("svg: {}", saida.to_string_lossy());
}

/// ⭐⭐⭐ **SONDA B10 — QUANTO É QUE A LEI NOVA MOVE O DESENHO?**
///
/// ⛔⛔⛔ **Ela existe porque eu reportei ao dono um ganho que o DESENHO não mostra.** A
/// [`a_lei_desdobrada_empurra_o_bico_e_a_dobra`] mede `estic MIN` — o **MÍNIMO** sobre o contorno —
/// e ele vai de `0,0496` para `0,2524` a `90°`, `5,1×`. Desenhadas as duas (a
/// [`diag_b_desenha_as_duas_leis`]), elas ficam **quase uma em cima da outra**.
///
/// ⚠️⚠️ **As duas coisas são verdade ao mesmo tempo, e a lição é de RÉGUA:** um mínimo sobre o
/// contorno é um extremo LOCAL, e um segmento de `500` pode multiplicar-se por cinco sem a forma
/// mudar de aspecto. *É a mesma família do `edge_max` cego ao quad fino, agora do outro lado: uma
/// régua LOCAL não diz o tamanho do que se vê.*
///
/// ⇒ esta imprime o deslocamento PONTO A PONTO entre as duas leis, em unidades da **ESPESSURA** da
/// barra (`1,0`), que é a feição onde o defeito vive.
#[test]
fn diag_b_quanto_a_lei_nova_move_o_desenho() {
    use ph2d_skeleton::MisturaDoAngulo;
    const ESPESSURA: f64 = 1.0;
    let mut p = b_palco(true);
    let rest = b_amostra(&p.fonte);
    println!("\n{:=<92}", "");
    println!("SONDA B10 · QUANTO A LEI NOVA MOVE O DESENHO — barra da cena do dono");
    println!("{:=<92}", "");
    println!(
        "{:>6} | {:>9} {:>9} {:>9} | {:>11} | {:>9} {:>9}",
        "graus", "p50", "p90", "MÁX", "MÁX / esp.", "estic MIN", "→ novo"
    );
    for graus in [45.0_f32, 70.0, 90.0, 110.0, 130.0, 150.0] {
        p.dobra(graus);
        let pele = p.pele();
        let saida = |lei: MisturaDoAngulo| -> Vec<[f64; 2]> {
            rest.iter()
                .map(|&x| {
                    let mut w = pele.scratch();
                    let linha = p
                        .campo
                        .linha(x)
                        .unwrap_or_else(|| b_mais_proximo(&p.campo, x));
                    pele.weights_corrected(x, Some(&linha), &mut w, &p.correcoes);
                    pele.blend_com(x, &w, lei)
                })
                .collect()
        };
        let (a, b) = (
            saida(MisturaDoAngulo::Circulo),
            saida(MisturaDoAngulo::Desdobrado),
        );
        let mut d: Vec<f64> = a
            .iter()
            .zip(&b)
            .map(|(x, y)| (x[0] - y[0]).hypot(x[1] - y[1]))
            .collect();
        let cum = b_cum(&rest);
        let emin = |v: &[[f64; 2]]| {
            let mut m = f64::MAX;
            for i in 0..v.len() {
                let j = (i + 1) % v.len();
                let dr = cum[i + 1] - cum[i];
                if dr > 1e-9 {
                    m = m.min((v[i][0] - v[j][0]).hypot(v[i][1] - v[j][1]) / dr);
                }
            }
            m
        };
        let (p50, p90, max) = b_pct(&mut d);
        println!(
            "{graus:>6.0} | {p50:>9.4} {p90:>9.4} {max:>9.4} | {:>10.1}% | {:>9.4} {:>9.4}",
            max / ESPESSURA * 100.0,
            emin(&a),
            emin(&b)
        );
    }
    println!("{:=<92}", "");
}
