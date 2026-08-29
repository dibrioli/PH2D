//! ⭐⭐⭐ **O ALVO DA GRADE, e as DUAS portas medidas-e-recusadas que ele destravou.**
//!
//! Irmão de [`super::retopo_extract`] por RESPONSABILIDADE: aquele responde *«o que o
//! botão FAZ?»*, e este responde a pergunta que vem antes — *«qual é o alvo, e onde ele
//! é mais fino?»*.
//!
//! ⚠️⚠️ **As duas portas aqui nascem DESLIGADAS, cada uma com a tabela da rejeição no
//! seu doc.** *Uma fase medida sozinha pode melhorar e piorar o produto* — a lei que esta
//! linha pagou duas vezes, e que aponta as duas para a mesma obra: o **factor de escala
//! conforme por construção**.

use ph2d_mesh::Mesh;

/// ⭐⭐⭐ **A FASE ZERO REMALHA PARA O ALVO, ou para o `ALPHA` fixo?**
///
/// ⛔⛔ **O report de 2026-08-29 (duas fotos, «o remesh amputou pontas»)**: a peça do artista
/// tem espinhos cujo **raio local** cai para `0,037`, e o F1 remalha com
/// `ALPHA × diagonal = 0,089` — **2,4× a espessura da ponta**. *A remalha isotrópica destrói
/// o espinho antes de a cadeia começar, e tudo a jusante trabalha sobre uma peça já
/// amputada.*
///
/// ⚠️ **A `ph2d-quadchain` levou esta correcção em 2026-08-25 e este caminho não** — o doc
/// do `phase_zero` diz-o com todas as letras: *«um parâmetro que metade da função ignora só
/// mente para o SEGUNDO chamador»*, e o segundo chamador é este botão.
///
/// # ⛔⛔⛔ E A HIPÓTESE FOI REFUTADA PELA MEDIÇÃO — por isso ela nasce DESLIGADA
///
/// Medido 2026-08-29 na fixtura de espinhos (`espinhos:6`), o mesmo alvo dos dois lados:
///
/// | | `Detail 0,50` | `Detail 0,85` |
/// |---|---|---|
/// | ⭐ `ALPHA` fixo (o que shipa) | `χ = 2` · `0` bordo · envies. `4,6°` · `21` dobras | `χ = 2` · `0` bordo · `4,0°` · `29` dobras |
/// | ⛔ segue o alvo | `χ = 1` · **`4` bordo** · `10,1°` · ⛔ **`123` dobras** | (não fechou a tempo) |
///
/// ⭐ **É a MESMA direcção que o varrimento de densidade da `ph2d-quadchain` deu** (§8-ter):
/// uma malha de trabalho mais fina não é mais informação — é onde a topologia se perde.
/// *A remalha grosseira é o filtro que faz o campo cruzado ver a forma e não o ruído.*
pub(super) fn f1_follows_target() -> bool {
    std::env::var("PH2D_F1_TARGET").as_deref() == Ok("1")
}

/// ⭐⭐⭐ **O PASSO DA GRADE POR VÉRTICE — o `Follow Curvature` deixa de ser um knob morto.**
///
/// ⛔⛔ **Report do artista (2026-08-28):** *«as pontas finas, que deveriam ser relativamente
/// mais densas que as áreas lisas, têm menos densidade de faces e perdem detalhes»*. E a
/// medição confirma-o: na saída dele o expoente de `aresta ∼ curvatura^n` é **`−0,003`**
/// sobre uma faixa de curvatura de **`9,4×`** — *a grade é rigorosamente uniforme.*
///
/// ⚠️ **A lei já existia e não tinha consumidor nesta cadeia:**
/// [`ph2d_quadflow::ScaleField::adaptive`] dá o lado do quad por vértice a partir da
/// curvatura, com a gradação limitada pela [`ph2d_quadflow::MAX_ADAPTIVE_RATIO`] (a cerca
/// que impede a grade de rasgar em vez de transitar). Até hoje ela só era lida pelo motor
/// **local**; o de omissão fazia `let _ = adaptive;`.
///
/// # ⭐⭐ A NORMALIZAÇÃO, e por que ela não é opcional
///
/// O slider passou a pedir uma **contagem** ([`ph2d_quadflow::MAX_QUADS`]). Redistribuir os
/// quads sem renormalizar mudaria a contagem junto com a distribuição, e o slider voltava a
/// mentir. ⇒ o campo é escalado por `√(N_previsto / N_pedido)`, com
/// `N = Σ_face área/h²`. *A adaptação move os quads; ela não os cria.*
///
/// ⚠️ **Com `adaptive == 0` o campo é VAZIO** — a saída é a de sempre, e há gate.
///
/// # ⛔⛔⛔ MEDIDO E NÃO ADOPTADO — o passo no alvo do gradiente é LAVADO pela projecção
///
/// Medido 2026-08-28 na peça do artista (`Detail` fino, alvo `0,0324`, `13 289` quads):
///
/// | `Follow Curvature` | campo entregue | expoente da SAÍDA | apertada / chapada | quads | `>60°` |
/// |---|---|---|---|---|---|
/// | `0` | — | `+0,047` | `1,167` | `13 289` | `3` |
/// | `0,5` | `0,0243..0,0486` (`2×`) | `+0,024` | `1,133` | `11 963` | `3` |
/// | `1,0` | `0,0162..0,0648` (**`4×`**) | `+0,014` | `1,090` | ⚠️ `11 302` | ⛔ `6` |
///
/// ⭐⭐⭐ **Pede-se `400 %` e a saída move-se `7 %`** — e paga `15 %` da contagem e o dobro
/// das faces com canto pior que `60°`.
///
/// ⚠️ **O MECANISMO, e ele não é um defeito desta função:** o G3 resolve um mapa **escalar
/// por patch** cujo gradiente se aproxima do alvo `direcção / h`. Com `h` constante esse
/// campo alvo é integrável; **com `h` a variar ele deixa de o ser** (o rotacional deixa de
/// ser nulo), e a projecção de mínimos quadrados fica com a parte integrável — que é, quase
/// exactamente, o campo uniforme. *A adaptação não é ignorada: ela é projectada fora.*
///
/// ⭐ **A cura publicada tem nome e é outra maquinaria:** o factor de escala tem de ser
/// **conforme por construção** — resolver `Δ log h` contra a curvatura de Gauss e usar
/// `h = h₀·e^{−s}`, que é integrável por definição. É a família *«integer-grid maps with
/// prescribed sizing»*, e é uma wave com espec própria.
///
/// ⇒ **O `Follow Curvature` continua a nascer em `0`** e o caminho de omissão é
/// **byte-idêntico**. O que esta wave deixa é o **substrato** (o passo do mapa deixou de ser
/// um número — [`ph2d_gridmap::Step`]) e a medição que diz o que falta.
pub(super) fn sizing_field(work: &Mesh, target: f32, adaptive: f32) -> Vec<f32> {
    if adaptive <= 0.0 {
        return Vec::new();
    }
    // ⛔⛔ **`adaptive_graded` e NÃO `adaptive_with`** — ver o doc dela. O piso da irmã é a
    // aresta média da malha de TRABALHO, que é a cerca do motor local; emprestada aqui ela
    // colapsa os dois extremos da banda no mesmo número e o campo sai constante ao bit.
    let field = ph2d_quadflow::ScaleField::adaptive_graded(work, target, adaptive);
    let mut per_vertex: Vec<f32> = (0..work.vert_count()).map(|v| field.at(v)).collect();
    // ⭐ A contagem que o campo prevê, sobre a mesma área que o alvo escalar prevê.
    let pos = work.positions();
    let (mut pred, mut area) = (0.0f64, 0.0f64);
    for f in work.faces() {
        let v = f.verts();
        for k in 1..v.len() - 1 {
            let (a, b, c) = (
                pos[v[0] as usize],
                pos[v[k] as usize],
                pos[v[k + 1] as usize],
            );
            let (u, w) = (
                [b[0] - a[0], b[1] - a[1], b[2] - a[2]],
                [c[0] - a[0], c[1] - a[1], c[2] - a[2]],
            );
            let n = [
                u[1].mul_add(w[2], -(u[2] * w[1])),
                u[2].mul_add(w[0], -(u[0] * w[2])),
                u[0].mul_add(w[1], -(u[1] * w[0])),
            ];
            let tri = f64::from(n[0].mul_add(n[0], n[1].mul_add(n[1], n[2] * n[2])).sqrt()) * 0.5;
            let h = f64::from(
                (per_vertex[v[0] as usize]
                    + per_vertex[v[k] as usize]
                    + per_vertex[v[k + 1] as usize])
                    / 3.0,
            )
            .max(1.0e-9);
            pred += tri / (h * h);
            area += tri;
        }
    }
    let want = area / f64::from(target.max(1.0e-9)).powi(2);
    // ⚠️ **A linha existe porque a 1.ª medição desta wave não distinguia «o campo é
    // constante» de «o campo não chegou»** — as três corridas do knob deram saída
    // byte-idêntica, e sem estes números não havia como dizer qual das duas era.
    {
        let mut v = per_vertex.clone();
        v.sort_by(f32::total_cmp);
        eprintln!(
            "[sculpt3d] densidade adaptativa {adaptive:.2}: passo {:.5}..{:.5} (mediana {:.5}, \
             alvo {target:.5}), previstos {pred:.0} para {want:.0} pedidos",
            v.first().copied().unwrap_or(0.0),
            v.last().copied().unwrap_or(0.0),
            v.get(v.len() / 2).copied().unwrap_or(0.0),
        );
    }
    if pred > 0.0 && want > 0.0 {
        #[allow(clippy::cast_possible_truncation)]
        let k = (pred / want).sqrt() as f32;
        if k.is_finite() && k > 0.0 {
            for h in &mut per_vertex {
                *h *= k;
            }
        }
    }
    per_vertex
}
