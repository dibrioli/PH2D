//! ⭐⭐⭐ **AS PROPRIEDADES DO FILTRO DE TECIDO, medidas.**
//!
//! ⛔⛔ **Estes gates nasceram de uma pergunta do dono** (2026-09-08): *«cloth
//! filter tem as propriedades do tecido? Já foram implementadas para cloth
//! filter?»* — e a resposta medida era **em parte, e por empréstimo**.
//!
//! ⚠️⚠️ **A bancada do oráculo não podia responder**, e a razão é estrutural:
//! ela monta o `Pincel` da lei **directamente** do cabeçalho de cada fixture e
//! **nunca passa pelo mapeamento do produto**. Ela julga a LEI; ninguém julgava
//! a tradução *«os botões do painel → os números do solver»*. É esse buraco que
//! este ficheiro fecha.

use crate::{ClothFilterKind, ClothFilterProps, ClothFilterStep, SculptStroke};
use ph2d_mesh::shapes::uv_sphere;

fn passo(s: f32) -> ClothFilterStep {
    ClothFilterStep {
        s,
        gravity_axis: [0.0, -1.0, 0.0],
        frame: [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]],
        axes: [true; 3],
        eye: [0.0, 0.0, 1.0],
    }
}

/// Corre um gesto de filtro com estas propriedades e devolve as posições.
fn corre(props: ClothFilterProps, kind: ClothFilterKind, passos: usize) -> Vec<[f32; 3]> {
    let mut mesh = uv_sphere(16, 24, 1.0);
    let mut st = SculptStroke::default();
    st.cloth_filter_begin(&mesh, props, kind, [0.0; 3]);
    for _ in 0..passos {
        st.cloth_filter_step(&mut mesh, kind, &passo(1.0));
    }
    mesh.positions().to_vec()
}

/// A maior distância entre duas corridas.
fn pior(a: &[[f32; 3]], b: &[[f32; 3]]) -> f32 {
    a.iter()
        .zip(b)
        .map(|(p, q)| {
            ((p[0] - q[0]).powi(2) + (p[1] - q[1]).powi(2) + (p[2] - q[2]).powi(2)).sqrt()
        })
        .fold(0.0f32, f32::max)
}

/// ⭐⭐⭐ **AS QUATRO PROPRIEDADES CHEGAM AO SOLVER — e MOVEM a peça.**
///
/// ⚠️ **A régua é a PEÇA, e não o campo do `Pincel`.** Comparar o número que se
/// escreveu com o número que o `pincel_do_filtro` copiou seria a mesma linha
/// escrita duas vezes, verde por construção mesmo com o solver a ignorá-la. O
/// que se mede é a saída: mudar cada uma **sozinha** tem de mudar a malha.
///
/// ⚠️ **E cada uma é medida no gesto em que ela MORDE.** Uma massa diferente sob
/// uma gravidade que translada rigidamente muda a distância percorrida; as
/// varreduras só têm o que fazer quando há restrição violada, e num plano em
/// queda livre **não há nenhuma** — foi a primeira fixture desta wave, e ela
/// media `0,00000` de esticão de 1 a 64 varreduras. *Uma fixtura em que o knob
/// não morde não testa o knob.*
#[test]
fn as_quatro_propriedades_do_filtro_chegam_ao_solver() {
    let base = ClothFilterProps::default();
    let ref_ = corre(base, ClothFilterKind::Gravity, 6);
    for (nome, mudada, kind) in [
        (
            "massa",
            ClothFilterProps {
                mass: 2.0,
                ..base
            },
            ClothFilterKind::Gravity,
        ),
        (
            "amortecimento",
            ClothFilterProps {
                damping: 0.5,
                ..base
            },
            ClothFilterKind::Gravity,
        ),
        (
            "plasticidade",
            ClothFilterProps {
                plasticity: 1.0,
                ..base
            },
            ClothFilterKind::Pinch,
        ),
        (
            "varreduras",
            ClothFilterProps {
                sweeps: 32,
                ..base
            },
            ClothFilterKind::Pinch,
        ),
    ] {
        let a = corre(base, kind, 6);
        let b = corre(mudada, kind, 6);
        let d = pior(&a, &b);
        println!("{nome:>14} ({kind:?}): a peca move-se {d:.6}");
        assert!(
            d > 1e-4,
            "mudar `{nome}` sozinha nao mudou a peca ({d:.6}) -- ou ela nao chega ao solver, ou \
             a fixtura escolheu um gesto em que ela nao morde"
        );
    }
    // ⚠️ **O CONTROLO**: as omissões dão a mesma peça duas vezes. Sem ele, um
    // solver que devolvesse ruído passaria em todos os braços acima.
    let outra = corre(ClothFilterProps::default(), ClothFilterKind::Gravity, 6);
    assert_eq!(
        pior(&ref_, &outra),
        0.0,
        "duas corridas com as MESMAS propriedades deram pecas diferentes -- a lei nao e' \
         determinista, e nada acima afirma o que diz afirmar"
    );
}

/// ⭐⭐⭐ **A OMISSÃO DO FILTRO É A DO ALVO, E NÃO A DO PINCEL.**
///
/// ⛔⛔ **É o defeito exacto que a pergunta do dono expôs.** O filtro lia
/// `brush.cloth_damping`, cuja omissão é `0,01` **e cuja faixa começa em
/// `0,01`** — enquanto a espec §7 diz que *o filtro nasce **sem** perda de
/// velocidade nenhuma* (`0`, faixa `0..1`). ⇒ **o filtro não conseguia o próprio
/// valor de omissão**, e nenhuma régua deste repo o via.
///
/// ⚠️ **A régua é a PEÇA outra vez**: com o amortecimento do pincel a peça pára
/// mais cedo, e é essa diferença que se mede.
#[test]
fn a_omissao_do_filtro_e_a_do_alvo_e_nao_a_do_pincel() {
    let filtro = ClothFilterProps::default();
    assert_eq!(
        filtro.damping, 0.0,
        "a omissao do amortecimento do FILTRO e' `0` (espec §7) -- o `0,01` e' a do PINCEL"
    );
    assert_eq!(filtro.plasticity, 0.0, "o alvo cria a simulacao do filtro sem memoria de forma");
    assert_eq!(
        filtro.sweeps,
        ph2d_cloth::verlet::VARREDURAS,
        "a omissao das varreduras e' a do alvo -- e' ela que mantem os 17 tracos da bancada onde \
         estao"
    );
    // ⚠️ **E a diferença é OBSERVÁVEL**: se o `0` e o `0,01` dessem a mesma peça,
    // o gate acima seria uma afirmação sobre um número que não faz nada.
    let como_o_pincel = ClothFilterProps {
        damping: 0.01,
        ..filtro
    };
    let d = pior(
        &corre(filtro, ClothFilterKind::Gravity, 8),
        &corre(como_o_pincel, ClothFilterKind::Gravity, 8),
    );
    println!("o `0` do filtro contra o `0,01` do pincel: {d:.6}");
    assert!(
        d > 1e-5,
        "o amortecimento do filtro e o do pincel dao a MESMA peca ({d:.6}) -- entao a omissao \
         errada nao era observavel, e este gate nao afirma nada"
    );
}

/// ⭐⭐ **A PORTA PRENDE, e prende SÓ ELA.**
///
/// ⚠️ **Um `clamp` escrito nos dois sítios esconde de qual dos dois o número
/// saiu** — a mesma lei que o `t_at`/`with_t` do layout já escreve. Aqui ele
/// vive na [`ClothFilterProps::clamped`], e o `pincel_do_filtro` chama-a.
#[test]
fn a_porta_prende_os_quatro_numeros() {
    let louco = ClothFilterProps {
        mass: 99.0,
        damping: -3.0,
        plasticity: 7.0,
        sweeps: 0,
        collisions: true,
    }
    .clamped();
    assert_eq!(louco.mass, ClothFilterProps::MASS.1);
    assert_eq!(louco.damping, ClothFilterProps::DAMPING.0);
    assert_eq!(louco.plasticity, ClothFilterProps::PLASTICITY.1);
    assert_eq!(
        louco.sweeps,
        ClothFilterProps::SWEEPS.0,
        "zero varreduras seria um passo sem relaxacao nenhuma -- o piso e' `1`"
    );
    // E um valor legal atravessa intocado.
    let bom = ClothFilterProps {
        mass: 0.5,
        damping: 0.25,
        plasticity: 0.75,
        sweeps: 9,
        collisions: true,
    };
    assert_eq!(bom.clamped(), bom, "a porta mexeu num valor que ja' estava na faixa");
}

/// ⭐⭐⭐ **O FILTRO NÃO RECEBE UM PINCEL — e é a forma mais forte da lei.**
///
/// ⛔ *Não é possível ler por engano um campo que não chega.* Enquanto o
/// `cloth_filter_begin` recebia um `&Brush`, nada impedia o próximo campo de
/// tecido que alguém acrescentasse ao pincel de ser lido aqui — e foi
/// exactamente assim que os três actuais lá foram parar.
///
/// ⚠️ **Censo de FONTE porque é uma afirmação sobre a ASSINATURA**, e uma
/// assinatura não tem comportamento a medir.
#[test]
fn o_filtro_nao_recebe_um_pincel() {
    let fonte = include_str!("stroke_cloth_filter.rs");
    let i = fonte
        .find("pub fn cloth_filter_begin(")
        .expect("controlo positivo: o `cloth_filter_begin` mudou de nome ou de ficheiro");
    let assinatura = &fonte[i..i + fonte[i..].find(") {").unwrap_or(400)];
    assert!(
        !assinatura.contains("Brush"),
        "o `cloth_filter_begin` voltou a receber um `Brush` -- as propriedades do FILTRO sao dele \
         ({assinatura})"
    );
    assert!(
        assinatura.contains("ClothFilterProps"),
        "o `cloth_filter_begin` deixou de receber as propriedades do filtro"
    );
}

/// ⭐⭐⭐ **AS COLISÕES DO FILTRO EXISTEM — e o pano PARA no obstáculo.**
///
/// ⛔⛔ **A espec §7 diz que o filtro as tem** (*«idem §5.6, opção nasce
/// desligada»*) e nós passávamos-lhe uma lista **vazia**. A construção dos
/// colisores era ~40 linhas inline no traço; hoje é uma porta que os dois
/// partilham, e é por isso que o pano bate com a mesma lei nos dois gestos.
///
/// ⚠️ **A régua é a PEÇA contra o OBSTÁCULO**, e não a bandeira: com um plano no
/// caminho da queda, os vértices que o atravessariam têm de ficar do lado de cá.
/// *Medir que o `bool` chegou não afirma que ele muda alguma coisa.*
///
/// ⚠️ **E o CONTROLO é a mesma corrida sem colisor nenhum na lista** — sem ele
/// o gate não distingue *«colidiu»* de *«a gravidade não chegava lá»*.
#[test]
fn as_colisoes_do_filtro_param_o_pano_no_obstaculo() {
    use ph2d_mesh::{Pose, shapes};

    /// O `y` mais baixo que a peça alcança.
    fn fundo(p: &[[f32; 3]]) -> f32 {
        p.iter().map(|q| q[1]).fold(f32::INFINITY, f32::min)
    }
    /// Corre a gravidade com (ou sem) um plano no caminho.
    fn cai(com_colisor: bool) -> f32 {
        let mut mesh = uv_sphere(16, 24, 1.0);
        let mut st = SculptStroke::default();
        // ⚠️ **Uma placa LARGA e bem abaixo da esfera**: o raio da colisão é o
        // segmento que o vértice percorre no passo, então o obstáculo tem de
        // estar no caminho de facto.
        if com_colisor {
            let placa = shapes::uv_sphere(12, 18, 3.0);
            st.cloth_colliders
                .push((placa, Pose::at([0.0, -4.0, 0.0])));
        }
        let props = ClothFilterProps {
            collisions: com_colisor,
            ..ClothFilterProps::default()
        };
        st.cloth_filter_begin(&mesh, props, ClothFilterKind::Gravity, [0.0; 3]);
        for _ in 0..40 {
            st.cloth_filter_step(&mut mesh, ClothFilterKind::Gravity, &passo(4.0));
        }
        fundo(mesh.positions())
    }
    let livre = cai(false);
    let travado = cai(true);
    println!("queda livre: {livre:.4} | com obstaculo: {travado:.4}");
    assert!(
        livre < -3.0,
        "o CONTROLO nao caiu o bastante para chegar ao obstaculo ({livre:.4}) -- a fixtura nao \
         produz o fenomeno, e o gate abaixo nao afirmaria nada"
    );
    assert!(
        travado > livre + 0.5,
        "com as colisoes LIGADAS o pano desceu a {travado:.4}, praticamente tanto quanto sem elas \
         ({livre:.4}) -- ou os colisores nao chegam a` lei, ou a bandeira nao e' lida"
    );
}
