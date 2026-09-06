//! **SONDA — o que cada param do `motion.duplicator` PRECISA para se ver** (report do Enio,
//! 2026-09-05: *«em duplicator não vejo o efeito dos parâmetros: Pick, Point Scale e
//! Transfer»*).
//!
//! ⚠️ **A régua é a do PRODUTO** (a mesma da wave do painel do L-System): varrer o param pela
//! faixa do hint e contar quantas saídas **distintas ao bit** o nó emite. Um param que emite
//! **uma** saída em toda a faixa é, para o artista daquela cena, um controlo morto — mesmo que
//! o `eval` o leia.
//!
//! ⚠️ E a régua corre-se sobre TRÊS entradas, porque a resposta depende delas: é isso que a
//! sonda descobre.

use super::transfer::Transfer;
use super::*;

/// Uma assinatura ao bit da saída: contagem + toda coluna, nome e conteúdo.
fn assinatura(s: &Stream) -> String {
    let mut t = format!("n={}", s.count());
    for (name, col) in s.columns() {
        t.push_str(&format!("|{name}={col:?}"));
    }
    t
}

/// `ns` formas na origem, cada uma com o seu `texture_id` (é o que distingue uma da outra).
fn formas(ns: usize) -> Stream {
    Stream::new(ns)
        .with("P", Column::Vec2(vec![[0.0, 0.0]; ns]))
        .with(
            "texture_id",
            Column::Scalar((0..ns).map(|i| i as f32).collect()),
        )
}

/// Uma fila de `np` pontos — como um `motion.grid`/`scatter` a entrega: **só `P`**.
fn pontos_nus(np: usize) -> Stream {
    Stream::new(np).with(
        "P",
        Column::Vec2((0..np).map(|i| [i as f32, 0.0]).collect()),
    )
}

/// Formas que já trazem cor própria — é o que faz a coluna ser **DISPUTADA**.
fn formas_com_tint(ns: usize) -> Stream {
    formas(ns).with("tint", Column::Vec4(vec![[0.5, 0.5, 0.5, 1.0]; ns]))
}

/// A mesma fila, mas AUTORADA: com escala e cor próprias (um `motion.scale`/`color_ramp`
/// aplicado ao arranjo antes do carimbo).
fn pontos_autorados(np: usize) -> Stream {
    pontos_nus(np)
        .with(
            "size",
            Column::Vec2((0..np).map(|i| [1.0 + i as f32 * 0.5; 2]).collect()),
        )
        .with(
            "tint",
            Column::Vec4(
                (0..np)
                    .map(|i| [i as f32 / np as f32, 0.2, 0.8, 1.0])
                    .collect(),
            ),
        )
}

/// Quantas saídas distintas o param emite ao varrer a faixa inteira do hint.
fn distintas(shape: &Stream, points: &Stream, param: &str, valores: &[f32]) -> usize {
    let mut vistas = std::collections::BTreeSet::new();
    for &v in valores {
        let (pick, ps, tr) = match param {
            "pick" => (Pick::of(v), 0.0, Transfer::ShapeWins),
            POINT_SCALE => (Pick::Off, v, Transfer::ShapeWins),
            _ => (Pick::Off, 0.0, Transfer::of(v)),
        };
        let np = points_within_budget(pick, shape.count(), points.count(), 1 << 20);
        let out = duplicate(shape, points, np, pick, 0, ps, tr);
        vistas.insert(assinatura(&out));
    }
    vistas.len()
}

#[test]
#[ignore = "sonda de medição — corra à mão"]
fn measure_what_each_param_needs() {
    let casos: [(&str, Stream, Stream); 5] = [
        ("1 forma · pontos NUS", formas(1), pontos_nus(6)),
        ("3 formas · pontos NUS", formas(3), pontos_nus(6)),
        ("1 forma · pontos AUTORADOS", formas(1), pontos_autorados(6)),
        (
            "3 formas · pontos AUTORADOS",
            formas(3),
            pontos_autorados(6),
        ),
        (
            "3 formas COM TINT · pontos AUTORADOS",
            formas_com_tint(3),
            pontos_autorados(6),
        ),
    ];
    println!("\n  entrada                       | Pick | Point Scale | Transfer");
    println!("  ------------------------------|------|-------------|---------");
    for (nome, shape, points) in &casos {
        let pick = distintas(shape, points, "pick", &[0.0, 1.0, 2.0]);
        let ps = distintas(shape, points, POINT_SCALE, &[0.0, 0.25, 0.5, 0.75, 1.0]);
        let tr = distintas(shape, points, transfer::TRANSFER, &[0.0, 1.0, 2.0, 3.0]);
        println!("  {nome:<30}| {pick:^4} | {ps:^11} | {tr:^8}");
    }
    println!(
        "\n  (o número é quantas saídas DISTINTAS ao bit o param produz na sua faixa;\n   \
         1 = o artista não vê nada mexer)\n"
    );
}
