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

/// ⭐⭐⭐ **O QUE O `POINT SCALE` FAZ QUE O NÓ `SCALE` NÃO FAZ** — pergunta do Enio, 2026-09-06:
/// *«point scale faz exatamente o que o nó scale faz?»*
///
/// ⚠️ **Não são a mesma pergunta**, e a diferença mede-se: o `motion.scale` é *«fique N vezes
/// maior»* — UM número, o mesmo para todos. O `point_scale` é *«o arranjo já traz um tamanho por
/// ponto: quanto dele eu obedeço?»* — a variação **vem dos pontos**, não do knob.
///
/// ⛔ **E a diferença não é de conveniência: é de INFORMAÇÃO PERDIDA.** Com `point_scale = 0` o
/// carimbo **deita fora** a coluna `size` do ponto — ela não chega à saída, e nenhum nó a
/// jusante a pode recuperar, porque já não existe. Um `motion.scale` depois do carimbo multiplica
/// o que sobrou; ele não sabe o que foi deitado fora.
#[test]
#[ignore = "sonda de medição — corra à mão"]
fn measure_what_point_scale_does_that_scale_cannot() {
    let shape = formas(1); // sem `size` autorado ⇒ a identidade, 1
    // Um arranjo que TRAZ tamanho próprio, diferente em cada ponto (um `motion.scale` com
    // falloff, um `motion.drive(Size)`, um scatter que varia — o gesto é comum).
    let np = 4;
    let pontos = pontos_nus(np).with(
        "size",
        Column::Vec2((0..np).map(|i| [1.0 + i as f32, 1.0 + i as f32]).collect()),
    );
    println!("\n  o ARRANJO traz size = [1, 2, 3, 4] por ponto; a FORMA não traz nenhum (= 1)\n");
    println!("  point_scale | o `size` de cada cópia na saída");
    println!("  ------------|--------------------------------");
    for t in [0.0_f32, 0.5, 1.0] {
        let out = duplicate(&shape, &pontos, np, Pick::Off, 0, t, Transfer::ShapeWins);
        let s = match out.get("size") {
            Some(Column::Scalar(v)) => format!("{v:?}"),
            Some(Column::Vec2(v)) => format!("{:?}", v.iter().map(|s| s[0]).collect::<Vec<_>>()),
            _ => "(a coluna NÃO EXISTE na saída)".to_string(),
        };
        println!("     {t:>4}     | {s}");
    }
    // ⚠️ **A METADE QUE PROVA QUE NÃO É UM `scale`:** com `t = 0` a saída é a MESMA por mais que
    // se mexa nos tamanhos do arranjo — a informação foi deitada fora, e nenhum nó a jusante a
    // traz de volta.
    let outro = pontos_nus(np).with(
        "size",
        Column::Vec2((0..np).map(|i| [10.0 + i as f32 * 7.0; 2]).collect()),
    );
    let a = duplicate(&shape, &pontos, np, Pick::Off, 0, 0.0, Transfer::ShapeWins);
    let b = duplicate(&shape, &outro, np, Pick::Off, 0, 0.0, Transfer::ShapeWins);
    println!(
        "\n  com point_scale = 0, trocar os tamanhos do arranjo de [1,2,3,4] para [10,17,24,31]\n  \
         muda a saída? {}  <- a coluna do ponto é DEITADA FORA, e um `scale` a jusante\n     \
         não a pode recuperar: ela já não existe.\n",
        if assinatura(&a) == assinatura(&b) {
            "NÃO"
        } else {
            "sim"
        }
    );
}

/// ⭐⭐⭐ **O QUE O `TRANSFER` FAZ** — pergunta do Enio, 2026-09-06: *«e transfer faz o que?»*
///
/// Ele é a regra de **quem ganha quando os dois lados trazem a MESMA coluna**. Não é sobre cor:
/// a cor é só o caso comum. Vale para qualquer atributo que a forma e o ponto tenham com o mesmo
/// nome — ⛔ **menos** os que já têm lei própria: `P` e `rot` SOMAM sempre, `Index`/`Count` são
/// renumerados, e o `size` tem a porta dele (`point_scale`).
///
/// ⚠️ **E uma coluna que só o PONTO tem chega em TODO modo** — isso não é o `Transfer`, é a cura
/// de 2026-09-01: um modo que resolve conflito não decide sobre uma coluna que ninguém disputa.
#[test]
#[ignore = "sonda de medição — corra à mão"]
fn measure_what_transfer_decides() {
    let modos = [
        ("Shape Wins", Transfer::ShapeWins),
        ("Point Wins", Transfer::PointWins),
        ("Add", Transfer::Add),
        ("Multiply", Transfer::Multiply),
    ];
    let np = 3;
    let ler = |s: &Stream, nome: &str| match s.get(nome) {
        Some(Column::Scalar(v)) => format!("{v:?}"),
        _ => "(ausente)".to_string(),
    };

    // (a) A COLUNA DISPUTADA: a forma tem `heat = 10`, o arranjo tem `heat = [1, 2, 3]`.
    let forma = formas(1).with("heat", Column::Scalar(vec![10.0]));
    let pontos = pontos_nus(np).with("heat", Column::Scalar(vec![1.0, 2.0, 3.0]));
    println!("\n  (a) DISPUTADA — a forma tem heat=10, o arranjo tem heat=[1,2,3]\n");
    println!("  transfer    | o `heat` de cada cópia");
    println!("  ------------|-----------------------");
    for (nome, m) in modos {
        let out = duplicate(&forma, &pontos, np, Pick::Off, 0, 0.0, m);
        println!("  {nome:<11} | {}", ler(&out, "heat"));
    }

    // (b) SÓ NO PONTO: a forma não tem `heat` nenhum.
    println!("\n  (b) SÓ O ARRANJO A TEM — a forma não tem `heat`\n");
    println!("  transfer    | o `heat` de cada cópia");
    println!("  ------------|-----------------------");
    for (nome, m) in modos {
        let out = duplicate(&formas(1), &pontos, np, Pick::Off, 0, 0.0, m);
        println!("  {nome:<11} | {}", ler(&out, "heat"));
    }

    // (c) AS COLUNAS COM LEI PRÓPRIA: o `transfer` não lhes toca.
    let forma_p = Stream::new(1)
        .with("P", Column::Vec2(vec![[100.0, 0.0]]))
        .with("rot", Column::Scalar(vec![7.0]))
        .with("size", Column::Vec2(vec![[5.0, 5.0]]));
    let pontos_p = pontos_nus(np)
        .with("rot", Column::Scalar(vec![1.0, 2.0, 3.0]))
        .with("size", Column::Vec2(vec![[9.0, 9.0]; np]));
    println!("\n  (c) AS QUE TÊM LEI PRÓPRIA — `P` e `rot` SOMAM, `size` é do `point_scale`\n");
    println!("  transfer    | rot de cada cópia (forma=7, arranjo=[1,2,3])");
    println!("  ------------|---------------------------------------------");
    for (nome, m) in modos {
        let out = duplicate(&forma_p, &pontos_p, np, Pick::Off, 0, 0.0, m);
        println!("  {nome:<11} | {}", ler(&out, "rot"));
    }
    println!();
}
