//! A régua do [`crate::tape_schedule`] — ver o módulo.

use ph2d_field::{FieldDoc, FillRule, NodeId, Primitive, Profile, Xform};

/// Um polígono regular de `n` lados — o mesmo contorno da sonda do dispositivo
/// (`mede_o_preco_de_uma_aresta_de_perfil`), para que as duas réguas falem da mesma peça.
///
/// ⚠️ **O raio ONDULA de propósito:** um círculo perfeito deixa a `fidget` desduplicar arestas
/// iguais, e a sonda mediria um polígono que ninguém desenha.
fn anel(n: u32) -> Profile {
    let pts = (0..n)
        .map(|i| {
            #[allow(clippy::cast_precision_loss)]
            let a = std::f32::consts::TAU * i as f32 / n as f32;
            let r = 0.35 + 0.06 * (a * 3.0).cos();
            [r * a.cos(), r * a.sin()]
        })
        .collect();
    Profile::new(vec![pts], FillRule::NonZero, 1.0e-4).expect("o anel")
}

fn extrudado(n: u32) -> FieldDoc {
    FieldDoc::new(
        vec![crate::leaf(
            Primitive::Extrude {
                profile: anel(n),
                half_height: 0.25,
                round: 0.0,
                chamfer: 0.0,
            },
            Xform::IDENTITY,
        )],
        NodeId(0),
    )
    .expect("a peça extrudada")
}

/// ⭐⭐⭐ **A TABELA que abriu a wave: quanto do `vivos` era a ORDEM, e não o grafo.**
///
/// | arestas | instruções | vivos CRU | vivos ESCALONADO | razão |
/// |---:|---:|---:|---:|---:|
/// | `32` | `1 040` | `68` | `28` | `2,4×` |
/// | `64` | `2 060` | `130` | `29` | `4,5×` |
/// | `128` | `4 080` | `251` | `33` | `7,6×` |
/// | `256` | `8 121` | `492` | **`48`** | **`10,2×`** |
///
/// ⚠️ **É uma CONTAGEM e não um relógio** — vale com a máquina cheia, que é metade da razão de ela
/// existir: a travessia que ela mede não depende de carga nenhuma.
///
/// ⚠️ **A barra é no MAIOR contorno, e não «em todos»** — a `32` arestas a ordem crua já tem pouco
/// para desperdiçar (`68` vivos), e exigir `8×` ali seria uma barra que a aritmética não pode dar.
/// *O que esta wave compra cresce com o tamanho da peça, porque é isso que o defeito fazia.*
#[test]
fn a_ordem_da_fita_e_o_que_punha_os_vivos_a_crescer_com_as_arestas() {
    let reg = crate::hybrid::Registry::new();
    println!("\n  arestas · instruções · vivos CRU · vivos ESCALONADO · razão");
    let (mut cru32, mut cru256) = (0usize, 0usize);
    let mut razao_maior = 0.0f32;
    for n in [32u32, 64, 128, 256] {
        let doc = extrudado(n);
        let cru = crate::device::DeviceField::new_com(&doc, &reg, false).expect("a peça");
        let esc = crate::device::DeviceField::new_com(&doc, &reg, true).expect("a peça");
        let (a, b) = (
            cru.tape_shape().expect("a fita"),
            esc.tape_shape().expect("a fita"),
        );
        assert_eq!(a.ops, b.ops, "o escalonamento é uma PERMUTAÇÃO da fita");
        #[allow(clippy::cast_precision_loss)]
        let razao = a.vivos as f32 / b.vivos as f32;
        println!(
            "  {n:>5} · {:>7} · {:>6} · {:>6} · {razao:.1}×",
            a.ops, a.vivos, b.vivos
        );
        if n == 256 {
            println!("    pico escalonado, por espécie: {:?}", esc.peak_kinds());
        }
        if n == 32 {
            cru32 = a.vivos;
        }
        if n == 256 {
            cru256 = a.vivos;
            razao_maior = razao;
        }
    }
    // ⚠️ **O CONTROLO vem primeiro:** sem ele, uma fixtura que deixasse de exercitar o defeito
    // (um contorno que a `fidget` desduplicasse, por exemplo) faria a razão colapsar e o gate
    // reprovaria pelo motivo errado — *uma razão que cai pode ser a cura a falhar ou o sujeito a
    // desaparecer, e os dois lêem-se igual.*
    #[allow(clippy::cast_precision_loss)]
    let inclinacao_crua = cru256 as f32 / cru32 as f32;
    assert!(
        inclinacao_crua > 5.0,
        "a ordem CRUA tinha de seguir as arestas para haver o que cortar: {cru32} a 32 e {cru256} a 256"
    );
    assert!(
        razao_maior > 8.0,
        "o escalonador tem de cortar o scratch por thread em mais de 8× no maior contorno (medido: {razao_maior:.1}×)"
    );
}

/// ⭐⭐⭐ **A metade que torna a outra aceitável: o `vivos` deixa de CRESCER com as arestas.**
///
/// ⚠️ **A régua é a INCLINAÇÃO e não o valor** — um corte de `10×` que continuasse linear só
/// adiaria o degrau da ocupação para um contorno dez vezes maior. O que a placa precisa é de o
/// `vivos` ser uma **propriedade da peça** e não do número de arestas dela.
#[test]
fn com_o_escalonador_os_vivos_deixam_de_seguir_o_numero_de_arestas() {
    let reg = crate::hybrid::Registry::new();
    let vivos = |n: u32| {
        crate::device::DeviceField::new(&extrudado(n), &reg)
            .and_then(|f| f.tape_shape())
            .expect("a fita")
            .vivos
    };
    let (v32, v256) = (vivos(32), vivos(256));
    // `8×` as arestas: a ordem da travessia multiplica o `vivos` por ~7,2 (o controlo do gate
    // acima). Aqui ele tem de ficar abaixo do DOBRO.
    assert!(
        v256 < v32 * 2,
        "o `vivos` ainda segue as arestas: {v32} a 32 e {v256} a 256"
    );
}

/// ⭐⭐⭐ **E a permutação não toca em NENHUM valor** — a defesa que torna a troca barata de aceitar.
///
/// ⚠️ **A fixture é a peça com ESCULTURA de fora**: o `Field::at` é o avaliador de CPU, e ele é quem
/// sabe responder um número. A afirmação é sobre bits, não sobre uma folga.
#[test]
fn a_fita_escalonada_responde_os_mesmos_bits() {
    for n in [7u32, 32, 64] {
        let doc = extrudado(n);
        let cru = crate::Field::new_com(&doc, false);
        let esc = crate::Field::new_com(&doc, true);
        let mut vistos = 0u32;
        for i in 0..11 {
            for j in 0..11 {
                for k in 0..5 {
                    let p = [
                        -0.6 + 0.12 * f64::from(i),
                        -0.6 + 0.12 * f64::from(j),
                        -0.4 + 0.2 * f64::from(k),
                    ];
                    let (a, b) = (cru.at(p[0], p[1], p[2]), esc.at(p[0], p[1], p[2]));
                    assert_eq!(
                        a.to_bits(),
                        b.to_bits(),
                        "a ordem mudou o valor em {p:?} com {n} arestas: {a} contra {b}"
                    );
                    vistos += 1;
                }
            }
        }
        assert_eq!(vistos, 605, "o piso de população da varredura");
    }
}

/// ⛔⛔⛔ **A MESMA PEÇA TEM DE DAR O MESMO TEXTO** — senão a cache de pipelines morre.
///
/// A chave do cache de shaders é o **texto** do WGSL, e é isso que faz um arrasto de slider
/// reescrever um buffer sem recompilar nada (`crate::wgsl`, nota do módulo). ⛔ Um escalonador cuja
/// ordem dependesse da iteração de um `HashMap`, do endereço de um ponteiro ou da ordem de inserção
/// num monte devolveria **um texto diferente por quadro** — e o quadro passaria a pagar `6` a
/// `49 ms` de compilação, em silêncio, com a imagem perfeita.
///
/// ⚠️ **Nenhuma régua de imagem veria isso**: os valores são os mesmos. O que o apanha é comparar o
/// TEXTO de duas construções independentes.
#[test]
fn a_mesma_peca_da_sempre_o_mesmo_texto_de_shader() {
    let reg = crate::hybrid::Registry::new();
    for n in [7u32, 64, 192] {
        let texto = |_: u32| {
            crate::device::DeviceField::new(&extrudado(n), &reg)
                .and_then(|c| c.tape_wgsl())
                .expect("a fita")
                .source
        };
        let (a, b) = (texto(0), texto(1));
        assert_eq!(a.len(), b.len(), "com {n} arestas o texto mudou de tamanho");
        assert!(
            a == b,
            "com {n} arestas duas construções deram textos diferentes"
        );
        // ⭐ **O CONTROLO:** sem ele, um `tape_wgsl` que devolvesse sempre a mesma string vazia
        // passaria. *Um zero de «igual» e um de «nenhum dos dois olhou» são o mesmo byte.*
        assert!(
            a.lines().count() > n as usize,
            "com {n} arestas a fita tem {} linhas — a sonda não mediu a peça",
            a.lines().count()
        );
    }
}

/// ⚠️ **O QUE O ESCALONADOR CUSTA A QUEM O PAGA** — e quem o paga é um arrasto de slider.
///
/// A montagem desta fita é um **JIT** e corre **a cada mudança de geometria** (o contador é o
/// [`crate::POINT_TAPES`]). Um custo que nenhuma sonda conta é um custo que nenhuma mutação mata —
/// e esta wave acrescentou um passe `O(fita × prontos)` ao caminho que um arrasto percorre 60 vezes
/// por segundo.
///
/// ⚠️ **É uma SONDA e não um gate** (`#[ignore]`): ela lê um relógio, logo é candidata natural à
/// família das flakes de carga. O que ela imprime tem de ser lido com o `load` ao lado.
#[test]
#[ignore = "sonda de relógio — leia o load ao lado"]
fn mede_o_que_o_escalonador_custa_a_montar() {
    println!(
        "\n  arestas · montagem CRUA · montagem ESCALONADA · acréscimo · load {}",
        std::fs::read_to_string("/proc/loadavg")
            .unwrap_or_default()
            .trim()
    );
    let reg = crate::hybrid::Registry::new();
    for n in [32u32, 64, 128, 256] {
        let doc = extrudado(n);
        let mede = |escalonar: bool| {
            // ⚠️ O MÍNIMO de cinco, e não a média: sob carga a média mede quem mais usa a máquina.
            let mut v: Vec<f32> = Vec::new();
            for _ in 0..5 {
                let t0 = std::time::Instant::now();
                let c = crate::device::DeviceField::new_com(&doc, &reg, escalonar).expect("a peça");
                std::hint::black_box(c.tape_shape());
                #[allow(clippy::cast_possible_truncation)]
                v.push(t0.elapsed().as_secs_f32() * 1e3);
            }
            v.sort_by(f32::total_cmp);
            v[0]
        };
        let (cru, esc) = (mede(false), mede(true));
        println!(
            "  {n:>7} · {cru:>13.3} ms · {esc:>19.3} ms · {:>+8.1} %",
            (esc / cru - 1.0) * 100.0
        );
    }
}

/// ⛔⛔ **AS DUAS METADES DA MESMA LEI TÊM DE CONCORDAR** — a régua e o emissor.
///
/// [`crate::point_tape::Instr::ocupa_registo`] é lida por **dois** consumidores: o
/// `TapeShape::vivos` (que diz quantos registos a fita gasta) e o [`crate::wgsl`] (que decide o que
/// ganha um `let` e o que se escreve onde é usado). ⛔ **Se elas divergirem, a régua conta um
/// recurso que o shader não gasta — ou deixa de contar um que ele gasta — e nenhum gate de imagem
/// o veria**, porque os valores continuam os mesmos.
///
/// ⇒ o gate conta as duas coisas e exige que sejam o MESMO número.
#[test]
fn o_que_a_regua_conta_como_registo_e_exactamente_o_que_o_emissor_guarda() {
    let reg = crate::hybrid::Registry::new();
    let mut vistos = 0u32;
    for n in [7u32, 32, 128] {
        let doc = extrudado(n);
        let campo = crate::device::DeviceField::new(&doc, &reg).expect("a peça");
        let fita = campo.tape_wgsl().expect("a fita");
        let lets = fita
            .source
            .lines()
            .filter(|l| l.trim_start().starts_with("let v"))
            .count();
        let guarda = campo.probe_ocupam_registo();
        assert_eq!(
            lets, guarda,
            "com {n} arestas o emissor guarda {lets} valores e a régua conta {guarda}"
        );
        // ⭐ **E o CONTROLO:** sem ele, um emissor que não escrevesse `let` nenhum e uma régua que
        // contasse zero passariam este gate de braço dado. *Um zero de «igual» e um de «nenhum dos
        // dois olhou» são o mesmo byte.*
        assert!(
            lets > n as usize,
            "com {n} arestas a fita só guarda {lets} valores — a sonda não mediu a peça"
        );
        vistos += 1;
    }
    assert_eq!(vistos, 3, "o piso de população da varredura");
}
