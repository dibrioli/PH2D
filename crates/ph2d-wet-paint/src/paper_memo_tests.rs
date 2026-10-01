//! Gates da memória dos papéis. ⚠️ A memória é GLOBAL e os testes correm em threads lado a lado:
//! cada gate usa knobs PRÓPRIOS (um contraste que nenhum outro usa), para que a contagem de gerações
//! seja dele. O contador é por thread.

use super::*;

fn gerados() -> usize {
    crate::paper::GERADOS.with(std::cell::Cell::get)
}

fn knobs(contrast: f64) -> PaperKnobs {
    PaperKnobs {
        contrast,
        fibres: 0.75,
        grooves: 1.25,
    }
}

fn bits(t: &[f32]) -> Vec<u32> {
    t.iter().map(|v| v.to_bits()).collect()
}

/// **O que a memória devolve é o que o gerador daria, ao bit** — no primeiro pedido (a geração) e no
/// segundo (o acerto), contra um tile gerado de fresco pela porta crua.
#[test]
fn a_memoria_devolve_o_tile_do_gerador_ao_bit() {
    let k = knobs(0.617_283_1);
    let fresco = generate_paper_tile(PaperPreset::Rough, 2, k);
    let primeiro = paper_tile(PaperPreset::Rough, 2, k);
    let segundo = paper_tile(PaperPreset::Rough, 2, k);
    assert_eq!(bits(&primeiro), bits(&fresco), "a geração");
    assert_eq!(bits(&segundo), bits(&fresco), "o acerto");
    // Controlo: outra folha dá outro tile — senão a igualdade acima não distingue nada.
    assert_ne!(bits(&paper_tile(PaperPreset::Rough, 3, k)), bits(&fresco));
}

/// **O segundo pedido não corre o gerador**, e o primeiro corre-o exactamente uma vez (o controlo:
/// sem ele um gerador nunca chamado passaria a metade do acerto).
#[test]
fn o_segundo_pedido_nao_gera() {
    let k = knobs(0.729_104_7);
    let antes = gerados();
    let _ = paper_tile(PaperPreset::Hot, 1, k);
    assert_eq!(gerados() - antes, 1, "o primeiro pedido tem de gerar");
    let _ = paper_tile(PaperPreset::Hot, 1, k);
    assert_eq!(gerados() - antes, 1, "o segundo pedido não pode gerar");
}

/// **A chave são os BITS:** `-0.0` e `0.0` são iguais no `==` e geram cada um o seu tile. Partilhá-los
/// só seria seguro se o gerador não lesse o sinal de um zero, e isso não está medido.
#[test]
fn a_chave_sao_os_bits_e_nao_o_igual() {
    let (mut a, mut b) = (knobs(0.0), knobs(0.0));
    a.grooves = 0.0;
    b.grooves = -0.0;
    a.fibres = 0.318_805_2;
    b.fibres = 0.318_805_2;
    let antes = gerados();
    let _ = paper_tile(PaperPreset::Cold, 3, a);
    let _ = paper_tile(PaperPreset::Cold, 3, b);
    assert_eq!(gerados() - antes, 2);
}

/// **O tecto é o que diz:** depois de [`MEMO_CAP`] chaves novas, a mais velha foi esquecida e volta a
/// gerar. (Outros testes a inserir ao lado só a esquecem mais cedo — o gate não depende deles.)
#[test]
fn a_memoria_esquece_a_mais_velha_depois_do_tecto() {
    let velha = knobs(0.901_337_9);
    let _ = paper_tile(PaperPreset::Cold, 1, velha);
    for i in 0..MEMO_CAP {
        let _ = paper_tile(PaperPreset::Cold, 1, knobs(0.4 + i as f64 * 1e-6));
    }
    let antes = gerados();
    let _ = paper_tile(PaperPreset::Cold, 1, velha);
    assert_eq!(
        gerados() - antes,
        1,
        "a mais velha devia ter sido esquecida"
    );
}

/// **Quem é USADO fica — o esquecimento é pelo uso e não pela chegada.** O papel de fábrica é pedido
/// a cada nascimento do motor; numa fila por chegada ele sairia ao fim de [`MEMO_CAP`] experiências
/// com os knobs, por mais que fosse usado.
#[test]
fn quem_e_usado_nao_e_esquecido() {
    let usada = knobs(0.812_604_4);
    let _ = paper_tile(PaperPreset::Hot, 2, usada);
    for i in 0..MEMO_CAP - 1 {
        let _ = paper_tile(PaperPreset::Hot, 2, knobs(0.3 + i as f64 * 1e-6));
    }
    let _ = paper_tile(PaperPreset::Hot, 2, usada); // o uso põe-na à frente
    let _ = paper_tile(PaperPreset::Hot, 2, knobs(0.35));
    let antes = gerados();
    let _ = paper_tile(PaperPreset::Hot, 2, usada);
    assert_eq!(
        gerados() - antes,
        0,
        "a usada foi esquecida como se fosse a mais velha"
    );
}

/// **A fiação: o motor coze o papel PELA memória.** Um segundo motor nascido na mesma thread não
/// corre o gerador — com o `rebake_paper` a chamar a porta crua ele correria uma vez por motor.
#[test]
fn um_motor_que_renasce_nao_refaz_o_papel() {
    let _primeiro = crate::painter::Engine::new(32, 32);
    let antes = gerados();
    let _segundo = crate::painter::Engine::new(32, 32);
    assert_eq!(gerados() - antes, 0, "o motor refez o papel");
}

/// Sonda de relógio: o gerador contra um acerto, a frio e a quente.
#[test]
#[ignore = "sonda de relógio: corre à mão, em --release e com a máquina calma"]
fn diag_o_preco_do_papel() {
    let carga = std::fs::read_to_string("/proc/loadavg").unwrap_or_default();
    let k = knobs(0.555_121_3);
    let mut gerar = f64::MAX;
    for _ in 0..5 {
        let t = std::time::Instant::now();
        let _ = generate_paper_tile(PaperPreset::Cold, 0, k);
        gerar = gerar.min(t.elapsed().as_secs_f64() * 1e3);
    }
    let _ = paper_tile(PaperPreset::Cold, 0, k);
    let mut acerto = f64::MAX;
    for _ in 0..5 {
        let t = std::time::Instant::now();
        let _ = paper_tile(PaperPreset::Cold, 0, k);
        acerto = acerto.min(t.elapsed().as_secs_f64() * 1e3);
    }
    println!(
        "\n  O PAPEL  load {}\n  gerar: {gerar:.3} ms · acerto na memória: {acerto:.3} ms",
        carga.trim()
    );
}
