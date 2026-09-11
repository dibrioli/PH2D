//! ⭐⭐⭐ **O SECTOR DO LEQUE, ISOLADO EM 2D PURO** — sem malha, sem campo, sem
//! cadeia. A pergunta é: *uma grade construída dentro de um sector de leque nasce
//! enviesada?*
//!
//! # Porque isto existe
//!
//! Sete hipóteses para o enviesamento morreram medidas (`PLAN.md`
//! §4-quinquiestricies..§4-duodequadragies), e a reprodução ficou numa **esfera
//! lisa**: `18°` de enviesamento mediano contra `6°` do oráculo, com as células a
//! terem as proporções certas.
//!
//! ⚠️ **Dos 16 patches dessa esfera, 8 são triângulos e 3 são pentágonos** — onze
//! passam pelo LEQUE. E o leque tem uma propriedade que se pode calcular sem medir
//! nada: o canto que cada sector faz **no centro** vale `2π/n`.
//!
//! | `n` | canto no centro | desvio de 90° |
//! |---|---|---|
//! | 3 | `120°` | **`30°`** |
//! | 4 | `90°` | `0°` |
//! | 5 | `72°` | **`18°`** |
//! | 6 | `60°` | **`30°`** |
//!
//! ⭐ **O `18°` do pentágono é exactamente a mediana que a esfera lisa mede.** Isso é
//! coincidência ou mecanismo — e a diferença entre as duas é este ficheiro.
//!
//! ⛔ **O que ele NÃO responde:** se a cura é mudar o leque. Ele responde *quanto do
//! enviesamento é obrigatório pela construção*, que é o número que faltava para
//! saber se vale a pena mexer nela.
//!
//! ⚠️ **Ele usa a MESMA [`ph2d_quadfill::fan::coons`] que o produto** — reescrever a
//! fórmula aqui mediria uma lei que não é a que shipa.

use ph2d_quadfill::fan::coons;

/// ⭐⭐⭐ **O MAPA DE POTÊNCIA que ENDIREITA o sector.**
///
/// O sector de um `n`-gono abre `2π/n` no centro; um quadrado abre `π/2`. A potência
/// complexa `w = z^(n/4)` leva um ao outro — `θ ↦ θ·n/4` e `r ↦ r^(n/4)` —, e é
/// **holomorfa fora da origem**, logo **conforme**: ela preserva ângulos em toda a
/// parte menos no centro.
///
/// ⭐ *É exactamente onde o defeito angular pode ficar*: o centro de um leque é um
/// vértice irregular por construção, e tem de o ser. A construção de hoje espalha
/// esse defeito pelo sector inteiro; esta concentra-o num ponto.
fn to_w(z: [f32; 2], n: usize) -> [f32; 2] {
    #[allow(clippy::cast_precision_loss)]
    let k = n as f32 / 4.0;
    let (r, a) = (z[0].hypot(z[1]), z[1].atan2(z[0]));
    let rk = r.powf(k);
    [rk * (a * k).cos(), rk * (a * k).sin()]
}

/// A inversa de [`to_w`].
fn to_z(w: [f32; 2], n: usize) -> [f32; 2] {
    #[allow(clippy::cast_precision_loss)]
    let k = 4.0 / n as f32;
    let (r, a) = (w[0].hypot(w[1]), w[1].atan2(w[0]));
    let rk = r.powf(k);
    [rk * (a * k).cos(), rk * (a * k).sin()]
}

/// O canto de um `n`-gono regular inscrito no círculo unitário — a mesma lei do
/// `param::corners_for`.
fn corner(n: usize, i: usize) -> [f32; 2] {
    #[allow(clippy::cast_precision_loss)]
    let a = std::f32::consts::TAU * i as f32 / n as f32;
    [a.cos(), a.sin()]
}

fn lerp(a: [f32; 2], b: [f32; 2], t: f32) -> [f32; 2] {
    [
        (1.0 - t).mul_add(a[0], t * b[0]),
        (1.0 - t).mul_add(a[1], t * b[1]),
    ]
}

/// Uma polilinha de `k` segmentos iguais entre dois pontos.
fn line(a: [f32; 2], b: [f32; 2], k: usize) -> Vec<[f32; 2]> {
    #[allow(clippy::cast_precision_loss)]
    (0..=k).map(|i| lerp(a, b, i as f32 / k as f32)).collect()
}

/// O maior desvio de 90° nos quatro cantos de um quad 2D, em graus.
fn skew(q: [[f32; 2]; 4]) -> f32 {
    let mut worst = 0.0f32;
    for k in 0..4 {
        let p = q[k];
        let a = [q[(k + 3) % 4][0] - p[0], q[(k + 3) % 4][1] - p[1]];
        let b = [q[(k + 1) % 4][0] - p[0], q[(k + 1) % 4][1] - p[1]];
        let (la, lb) = (a[0].hypot(a[1]).max(1.0e-12), b[0].hypot(b[1]).max(1.0e-12));
        let c = (a[0].mul_add(b[0], a[1] * b[1]) / (la * lb)).clamp(-1.0, 1.0);
        worst = worst.max((c.acos().to_degrees() - 90.0).abs());
    }
    worst
}

/// ⭐⭐⭐ **A MEDIÇÃO.** Para cada `n`, monta **um** sector do leque no domínio e mede
/// o enviesamento da grade que a lei do produto constrói lá dentro.
///
/// A geometria do sector, como o `stitch` a monta:
///
/// ```text
///     bottom : do CORTE do lado i até ao canto  (metade do lado i)
///     right  : do canto até ao CORTE do lado i+1 (metade do lado i+1)
///     left   : o RAIO, do centro ao corte do lado i
///     top    : o RAIO, do centro ao corte do lado i+1
/// ```
///
/// ⚠️ **Os cortes ficam a meio de cada lado e o centro na origem** — o caso
/// **simétrico**, que é o mais favorável possível. *Se a construção enviesa já aqui,
/// enviesa sempre.*
#[test]
fn how_much_skew_does_a_fan_sector_force() {
    for n in 3..=6usize {
        // O corte a meio do lado `i`, e o centro na origem (a simetria dá isso).
        let cut = |i: usize| lerp(corner(n, i), corner(n, (i + 1) % n), 0.5);
        let centre = [0.0f32, 0.0];
        let (i, j) = (0usize, 1usize % n);
        let k = 8; // segmentos por bordo
        // ⚠️⚠️ **O SENTIDO DE CADA BORDO É LOAD-BEARING**, e a primeira versão deste
        // ficheiro errou-o: passou `left` do centro para o corte, quando o produto o
        // passa **ao contrário** (`spoke[i].rev()`). O [`coons`] exige
        // `bottom[0] == left[0]`, e com os bordos trocados ele devolve *«uma grade
        // que parece plausível e tem os bordos torcidos»* — as palavras do doc dele.
        //
        // ⭐ **O que apanhou o erro foi o CONTROLO que já estava na tabela:** `n = 4`
        // TEM de dar `0°` (o canto no centro é recto) e dava **`45°`**. *Uma linha
        // cujo valor é conhecido de antemão é o que separa medir de imaginar.*
        let bottom = line(cut(i), corner(n, j), k);
        let right = line(corner(n, j), cut(j), k);
        let left = line(cut(i), centre, k);
        let top = line(centre, cut(j), k);
        assert_eq!(bottom[0], left[0], "n={n}: os bordos nao emendam no canto");
        assert_eq!(bottom[k], right[0], "n={n}: os bordos nao emendam no canto");
        assert_eq!(left[k], top[0], "n={n}: os bordos nao emendam no canto");
        assert_eq!(right[k], top[k], "n={n}: os bordos nao emendam no canto");
        let g = coons(&bottom, &top, &left, &right);

        // ⭐⭐⭐ **A CURA, medida AQUI antes de tocar no produto.** O mesmo sector,
        // com os quatro bordos levados ao espaço `w` — onde ele abre `π/2` —, a
        // grade construída lá, e cada ponto trazido de volta. ⚠️ *A medição é feita
        // em `z`*, que é o domínio onde o achatamento vive: medir em `w` diria que um
        // quadrado é um quadrado, o que é tautologia
        // ([[feedback_a_gate_that_asserts_what_construction_guarantees_is_a_tautology]]).
        let cured = {
            let m = |c: &[[f32; 2]]| -> Vec<[f32; 2]> { c.iter().map(|&q| to_w(q, n)).collect() };
            let gw = coons(&m(&bottom), &m(&top), &m(&left), &m(&right));
            gw.iter()
                .map(|row| row.iter().map(|&q| to_z(q, n)).collect::<Vec<_>>())
                .collect::<Vec<_>>()
        };
        let cured_p50 = {
            let mut cs: Vec<f32> = Vec::new();
            for a in 0..cured.len() - 1 {
                for b in 0..cured[a].len() - 1 {
                    cs.push(skew([
                        cured[a][b],
                        cured[a + 1][b],
                        cured[a + 1][b + 1],
                        cured[a][b + 1],
                    ]));
                }
            }
            cs.sort_by(f32::total_cmp);
            #[allow(clippy::cast_precision_loss, clippy::cast_sign_loss)]
            let q = |t: f32| cs[(((cs.len() - 1) as f32 * t).round() as usize).min(cs.len() - 1)];
            eprintln!(
                "        ⭐ com o mapa de POTENCIA: p50 {:>5.1}° p95 {:>5.1}° max {:>5.1}°",
                q(0.50),
                q(0.95),
                cs.last().copied().unwrap_or(0.0),
            );
            q(0.50)
        };

        let mut all: Vec<f32> = Vec::new();
        let mut centre_cell = 0.0f32;
        for a in 0..g.len() - 1 {
            for b in 0..g[a].len() - 1 {
                let s = skew([g[a][b], g[a + 1][b], g[a + 1][b + 1], g[a][b + 1]]);
                if a == 0 && b == 0 {
                    centre_cell = s;
                }
                all.push(s);
            }
        }
        all.sort_by(f32::total_cmp);
        let p = |q: f32| {
            #[allow(clippy::cast_precision_loss, clippy::cast_sign_loss)]
            let i = ((all.len() - 1) as f32 * q).round() as usize;
            all[i]
        };
        #[allow(clippy::cast_precision_loss)]
        let esperado = (360.0 / n as f32 - 90.0).abs();
        let max = all.last().copied().unwrap_or(0.0);
        eprintln!(
            "  n={n}: canto no centro {esperado:>4.0}° · celula do centro {centre_cell:>5.1}° \
             | ⭐ grade do sector: p50 {:>5.1}° p95 {:>5.1}° max {max:>5.1}°",
            p(0.50),
            p(0.95),
        );
        // ⭐⭐⭐ **A LEI, escrita como asserção:** o pior enviesamento que a grade de
        // um sector carrega é **exactamente** o defeito angular do centro,
        // `|360/n − 90|`. Não é uma tendência nem um limite: é a geometria do
        // domínio a chegar intacta à célula.
        assert!(
            (max - esperado).abs() <= 0.5,
            "n={n}: a grade do sector traz {max:.1}° e o canto do centro pede {esperado:.1}° \
             -- a lei mudou, ou os bordos foram passados trocados"
        );
        // ⛔⛔ **O MAPA DE POTÊNCIA foi MEDIDO E REJEITADO** (2026-08-23), e o custo
        // de o saber foram dez minutos neste arnês em vez de uma obra no produto.
        //
        // | `n` | leque | ⭐ potência |
        // |---|---|---|
        // | **3** | `14,4°` | ⛔ **`18,6°`** — PIOR |
        // | 4 | `0,0°` | `0,0°` |
        // | 5 | `7,6°` | `6,4°` |
        // | 6 | `14,4°` | `11,6°` |
        //
        // ⛔ **Ele piora justamente o caso dominante:** na esfera lisa, **8 dos 16**
        // patches são triângulos. ⚠️ **E o motivo é geométrico:** `z^(n/4)` é conforme
        // e endireita o ângulo do centro, mas os dois bordos NÃO-radiais do sector são
        // **rectas** do polígono — sob uma potência elas deixam de o ser, e a grade de
        // Coons herda a curvatura delas. *A cura de um defeito angular introduz um
        // defeito de bordo.*
        //
        // ⇒ **A saída não é uma mudança de coordenada no domínio partilhado.** Ela é
        // dar a cada sector o SEU domínio — ver o `PLAN.md` §4-undequadragies.
        assert!(
            (n != 3) || cured_p50 > p(0.50),
            "n=3: o mapa de potencia devia PIORAR (medido 14,4° -> 18,6°) e deu {cured_p50:.1}° \
             -- se ele passou a ajudar, a tabela da recusa envelheceu e tem de ser refeita"
        );
        // ⭐⭐ **O CONTROLO, e ele é o que separa medir de imaginar.** Um patch de
        // QUATRO lados tem o canto do centro recto, e a grade do sector dele tem de
        // sair **perfeita**. ⛔ A primeira versão deste ficheiro dava `45°` aqui —
        // era o sinal de que os bordos estavam trocados, não de que o leque enviesa.
        if n == 4 {
            assert!(
                max <= 1.0e-3,
                "n=4: o sector devia sair perfeito e traz {max:.2}°"
            );
        } else {
            // ⛔ E para todo `n ≠ 4` o enviesamento é **obrigatório**: metade das
            // células passa de `7,6°` (pentágono) ou `14,4°` (triângulo), num
            // domínio ideal, simétrico e plano. *Nenhuma escolha a jusante o
            // desfaz.*
            assert!(
                p(0.50) >= 5.0,
                "n={n}: o leque devia FORCAR enviesamento e a mediana e' {:.1}°",
                p(0.50)
            );
        }
    }
}
