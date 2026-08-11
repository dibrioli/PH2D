//! **O QUE PASSA ENTRE DUAS AMOSTRAS** — a sonda da lacuna que a `W-ShapeCast`
//! fechou (plano 08 §4.3).
//!
//! Dois sensores do player liam o mundo com **três raios** e cada um nomeava a
//! mesma limitação por escrito:
//!
//! - `crouch::headroom_offsets` — *"um teto mais estreito que meia largura,
//!   entre duas amostras, não é visto … a cura para os três seria a mesma (um
//!   shape cast, que este wrapper ainda não tem)"*;
//! - `wall::wall_offsets` — *"uma fresta mais estreita que meia altura, entre
//!   duas amostras, segue invisível"*.
//!
//! ⚠️ **E o do agachar carregava uma AFIRMAÇÃO sobre a direção do erro** (o doc
//! do `probe_headroom`): *"o erro possível é ficar agachado onde caberia, nunca
//! levantar-se para dentro da pedra"*. Ela era verdade sobre a **caixa
//! envolvente contra a cápsula** — e esta sonda mediu a outra metade da mesma
//! pergunta, um obstáculo que cabe ENTRE os raios, e achou-a **falsa**.
//!
//! # O que esta sonda mediu, e quando
//!
//! | | antes (três raios) | depois (varredura) |
//! |---|---|---|
//! | pilar de 8 cm no vão | cabeça a **1,267** contra pedra em 1,25 | fica em 1,100 |
//! | pilar de 4 cm no vão | **foge de lado** 0,155 m e sobe a 1,600 | fica em 1,100 |
//! | pilar sobre um raio | fica em 1,100 | fica em 1,100 |
//!
//! Rodar: `cargo test -p ph2d-physics-ecs --release --test measure_the_gap_between_rays -- --ignored --nocapture`

#[path = "platform_pillar_rig.rs"]
mod pillar_fixture;

use pillar_fixture::{RADIUS, rig};

/// **O TETO QUE PASSA ENTRE OS RAIOS.**
///
/// A caixa do corpo mede `RADIUS` de meia-largura, então os três raios do
/// headroom nasciam em `−0,20 · 0,00 · +0,20`. Um pilar estreito posto em `+0,10`
/// caía exactamente no meio de duas amostras — e o corpo, ali, ainda mede
/// `0,3 + sqrt(0,2² − 0,1²) = 0,473` de meia-altura.
#[test]
#[ignore = "sonda: imprime a tabela, nao afirma nada"]
fn measure_the_ceiling_that_fits_between_the_rays() {
    // A face de baixo do pilar, logo acima da cabeça AGACHADA (0,6 + 0,5 = 1,1)
    // e bem abaixo da cabeça DE PÉ (1,1 + 0,5 = 1,6).
    const BOTTOM: f32 = 1.25;
    eprintln!("\n== O TETO ENTRE OS RAIOS (os raios nasciam em -0.20, 0.00, +0.20) ==");
    eprintln!("(pilar com a face de baixo em {BOTTOM:.2}; de pe' a cabeca chegaria a 1.60)\n");
    eprintln!("| pilar x | meia-larg | cobria um raio? | topo da cabeca | x final | veredito |");
    eprintln!("|---------|-----------|-----------------|----------------|---------|----------|");
    for (x, half) in [
        (0.0_f32, 0.05_f32), // sobre o raio do MEIO
        (0.20, 0.05),        // sobre o raio da BORDA
        (0.10, 0.04),        // ENTRE dois raios
        (0.10, 0.02),        // mais estreito ainda
        (-0.10, 0.04),       // o outro vao
        (0.0, 0.30),         // CONTROLE: um teto largo
    ] {
        let covers = [-RADIUS, 0.0, RADIUS].iter().any(|r| (r - x).abs() <= half);
        let (top, fx) = rig(Some((x, half, BOTTOM))).crouch_then_release(60, 120);
        let under = (fx - x).abs() < RADIUS + half;
        let verdict = if top <= BOTTOM {
            "ficou baixo"
        } else if under {
            "NA PEDRA"
        } else {
            "fugiu de lado"
        };
        eprintln!(
            "| {x:7.2} | {half:9.2} | {:15} | {top:14.3} | {fx:7.3} | {verdict:8} |",
            if covers { "sim" } else { "NAO" },
        );
    }
    eprintln!(
        "\nLEITURA: 'NA PEDRA' significa que a cabeca subiu ATRAVES do pilar e o solver\n\
         a segura la' dentro. 'fugiu de lado' e' o solver a expulsar o corpo de sob uma\n\
         pedra estreita demais para o segurar: sobe, mas nao onde o artista o deixou.\n\
         Com a varredura de corpo TODA a coluna diz 'ficou baixo'.\n"
    );
}

/// **A OUTRA METADE que a varredura devolve: a quina da CAIXA deixa de recusar.**
///
/// Um raio nasce no topo da caixa envolvente, e no canto dela a cápsula não
/// está. Esta varredura mede o vão entre *onde o raio ia* e *até onde o corpo
/// chega*, por altura de teto — a região em que o sensor antigo dizia "não cabe"
/// sobre espaço vazio.
#[test]
#[ignore = "sonda: imprime a tabela, nao afirma nada"]
fn measure_the_room_the_box_was_refusing() {
    eprintln!("\n== a QUINA DA CAIXA: um teto que so' a caixa toca ==");
    eprintln!("(a pedra comeca em x = +0.20 e vai para a direita)\n");
    eprintln!("| fundo da pedra | topo da cabeca | levantou? |");
    eprintln!("|----------------|----------------|-----------|");
    for bottom in [1.35_f32, 1.42, 1.45, 1.50, 1.58] {
        let (top, _) = rig(Some((10.0 + RADIUS, 10.0, bottom))).crouch_then_release(60, 120);
        eprintln!(
            "| {bottom:14.2} | {top:14.3} | {:9} |",
            if top > 1.2 { "SIM" } else { "nao" }
        );
    }
    eprintln!(
        "\nLEITURA: a capsula em x = +0.20 alcanca so' `centro + 0.30`, entao de pe'\n\
         (centro 1.10) ela chega a 1.40 -- e um teto entre 1.40 e 1.60 e' espaco que\n\
         o raio da BORDA via como pedra.\n"
    );
}

/// **A PAREDE COM FRESTA** — o outro sensor de três amostras, e o motivo de ele
/// **não** ter sido convertido junto.
///
/// Os raios do flanco nascem em `0 · −h · +h` (a cintura e as duas bordas da
/// caixa). Para os três passarem, a fresta tem de ser mais alta que o corpo — e
/// nesse caso não há parede à frente do corpo. O vão cego que sobra é o
/// **benigno**: um buraco reportado como parede.
#[test]
#[ignore = "sonda: imprime a tabela, nao afirma nada"]
fn measure_the_wall_with_a_slot() {
    eprintln!("\n== a fresta da parede: quantos dos 3 raios do flanco a atravessam ==");
    eprintln!("(raios em 0.00, -0.50, +0.50 relativos ao centro de um corpo de 1,0 m)\n");
    for slot in [0.10_f32, 0.30, 0.60, 1.10] {
        let hits = [0.0_f32, -0.5, 0.5]
            .iter()
            .filter(|o| o.abs() > slot * 0.5)
            .count();
        eprintln!("  fresta de {slot:4.2} m centrada na cintura -> {hits} de 3 raios veem parede");
    }
    eprintln!(
        "\nLEITURA: so' uma fresta MAIOR que o corpo apaga os tres, e ai' nao ha' parede\n\
         em frente ao corpo. O cego que sobra e' o benigno.\n"
    );
}
