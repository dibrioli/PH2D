//! **O CORPUS DO ORÁCULO vira gates** (§0.9: *cada corrida vira um gate*).
//!
//! A fixtura é a saída do Godot 4.7.2 (MIT) corrido sem interface — `godot_projectile_probe.gd`,
//! versionada em `docs/Components/ferramentas/` —, com cabeçalho e proveniência. Aqui ela é lida e
//! comparada com o que a **nossa** lei manda fazer.
//!
//! # ⚠️ O que se compara, e em que unidade
//!
//! O corpus está em **pixels** do Godot e a lei está em **metros**. ⇒ as grandezas comparadas são
//! **adimensionais**: o ÂNGULO do espelho e a **fracção do orçamento**. ⛔ Comparar comprimentos
//! seria comparar duas escalas e chamar-lhe paridade.
//!
//! ⚠️ **E o corpus mede `bounciness = 1`**, porque o `Vector2.bounce` do Godot não tem coeficiente.
//! A perda por salto é desenho NOSSO e tem gates próprios no `bounce_tests.rs` — ⛔ nunca
//! apresentada como paridade com ninguém.

use ph2d_projectile::{ProjectileLaw, SweepStep, bounce, mirror, normalize};

/// O texto do corpus, embutido — ⚠️ `include_str!`, que **falha a compilar** se o ficheiro mudar de
/// sítio (a espécie de gate partido que avisa alto, e não a que fica verde a medir nada).
const CORPUS: &str = include_str!("../fixtures/godot_bounce.txt");

fn linhas(marca: &str) -> Vec<String> {
    CORPUS
        .lines()
        .filter(|l| l.starts_with(marca))
        .map(str::to_string)
        .collect()
}

/// Lê `chave=valor` de uma linha do corpus.
fn campo(linha: &str, chave: &str) -> f32 {
    let i = linha
        .find(chave)
        .unwrap_or_else(|| panic!("falta `{chave}` em: {linha}"));
    let resto = &linha[i + chave.len()..];
    let fim = resto
        .find(|c: char| !(c.is_ascii_digit() || c == '.' || c == '-' || c == '+'))
        .unwrap_or(resto.len());
    resto[..fim]
        .parse()
        .unwrap_or_else(|e| panic!("`{chave}` ilegivel em `{linha}`: {e}"))
}

/// ⭐⭐⭐ **CLÁUSULA 1 — o orçamento parte-se e SOMA, sem nada evaporar.**
#[test]
fn o_orcamento_parte_se_e_soma() {
    let l = linhas("RESTO ");
    // ⛔ **Piso de população**: uma fixtura que deixasse de ser lida faria este gate medir NADA.
    assert_eq!(l.len(), 1, "o corpus do resto mudou de tamanho");
    let andou = campo(&l[0], "andou=");
    let resto = campo(&l[0], "resto=");
    let soma = campo(&l[0], "soma=");
    assert!(
        (andou + resto - soma).abs() < 1.0e-5,
        "o oraculo perdeu orcamento: {andou} + {resto} != {soma}"
    );
    // E a NOSSA lei devolve exactamente o resto que ele mediu, na mesma escala.
    let passo = SweepStep {
        dir: [1.0, 0.0],
        budget: soma,
        steps_left: 4,
    };
    let nosso = ph2d_sweep::remaining(passo, andou).expect("sobra orcamento");
    assert!(
        (nosso - resto).abs() < 1.0e-5,
        "a nossa lei dá {nosso} onde o oraculo dá {resto}"
    );
}

/// ⭐⭐⭐ **CLÁUSULA 2 — a direcção nova é o ESPELHO, nos seis ângulos.**
#[test]
fn a_direccao_nova_e_o_espelho_em_todos_os_angulos_do_corpus() {
    let l = linhas("BOUNCE ");
    assert_eq!(l.len(), 6, "o corpus do espelho mudou de tamanho");
    let mut piores = Vec::new();
    for linha in &l {
        let erro = campo(linha, "erro ");
        // ⚠️ **O oráculo tem de concordar consigo mesmo primeiro**: se ele lesse um erro grande, o
        // que estaria partido era a sonda, e comparar-nos com ela mediria outro programa.
        assert!(
            erro < 1.0e-3,
            "o proprio oraculo discorda do espelho publicado: {linha}"
        );
        // A NOSSA lei, no mesmo ângulo: o corpus dá a incidência na coluna `BOUNCE <g>`.
        let graus: f32 = linha["BOUNCE ".len()..]
            .split('|')
            .next()
            .expect("coluna")
            .trim()
            .parse()
            .expect("graus");
        let r = graus.to_radians();
        let dir = normalize([r.sin(), r.cos()]).expect("unitaria");
        let n = [-1.0_f32, 0.0];
        let nosso = normalize(mirror(dir, n)).expect("espelho");
        // Contra uma parede vertical o espelho inverte `x` e preserva `y`.
        if (nosso[0] + dir[0]).abs() > 1.0e-4 || (nosso[1] - dir[1]).abs() > 1.0e-4 {
            piores.push(format!("{graus}°: {nosso:?}"));
        }
        // E o RESTO atravessa o salto inteiro: |saída| = resto.
        let resto = campo(linha, "resto=");
        let saida = campo(linha, "|saida|=");
        assert!(
            (resto - saida).abs() < 1.0e-3,
            "o oraculo encolheu o resto no salto: {linha}"
        );
    }
    assert!(
        piores.is_empty(),
        "a nossa lei discorda do espelho em {} de {}:\n  {}",
        piores.len(),
        l.len(),
        piores.join("\n  ")
    );
}

/// ⭐⭐ **CLÁUSULA 3 — DOIS ricochetes cabem num tique, logo o laço precisa de TECTO.**
#[test]
fn dois_ricochetes_cabem_num_tique_e_e_por_isso_que_ha_tecto() {
    let l = linhas("QUINA ");
    assert_eq!(l.len(), 1, "o corpus da quina mudou de tamanho");
    let saltos = campo(&l[0], "saltos_num_tique=");
    assert!(saltos >= 2.0, "o oraculo deixou de medir a quina: {}", l[0]);

    // A nossa lei, na mesma quina: duas paredes a 90°, o corpo a entrar na diagonal.
    let law = ProjectileLaw {
        bounciness: 1.0,
        max_bounces: 4,
        ..ProjectileLaw::default()
    };
    let mut passo = SweepStep {
        dir: normalize([1.0, -1.0]).expect("diagonal"),
        budget: 10.0,
        steps_left: 4,
    };
    let mut v = [passo.dir[0] * 12.0, passo.dir[1] * 12.0];
    let normais = [[-1.0_f32, 0.0], [0.0, 1.0]]; // a vertical, depois o chão
    let mut n_saltos = 0;
    for n in normais {
        let Some(b) = bounce::next_step(passo, v, 0.0, n, &law) else {
            break;
        };
        passo = b.step;
        v = b.velocity;
        n_saltos += 1;
    }
    assert_eq!(
        n_saltos, 2,
        "a nossa lei nao deu os dois saltos que o oraculo mediu"
    );
    // ⚠️ Depois dos dois, a direcção é a INVERSA da de entrada — que é o que uma quina faz.
    assert!(
        (passo.dir[0] + normalize([1.0, -1.0]).expect("d")[0]).abs() < 1.0e-4,
        "a quina nao devolveu o corpo por onde ele veio: {:?}",
        passo.dir
    );
    // E com o tecto a UM, o segundo salto não acontece — é isso que o tecto significa.
    let curto = SweepStep {
        steps_left: 1,
        ..SweepStep {
            dir: normalize([1.0, -1.0]).expect("d"),
            budget: 10.0,
            steps_left: 1,
        }
    };
    let b = bounce::next_step(curto, v, 0.0, normais[0], &law).expect("o primeiro cabe");
    assert!(
        bounce::next_step(b.step, b.velocity, 0.0, normais[1], &law).is_none(),
        "o tecto de UM deixou passar o segundo salto"
    );
}

/// ⚠️ **CLÁUSULA 4 — a normal é UNITÁRIA, em MUNDO, e aponta contra o movimento.**
#[test]
fn a_normal_do_oraculo_e_unitaria_e_opoe_se_ao_movimento() {
    let l = linhas("NORMAL ");
    assert_eq!(l.len(), 1, "o corpus da normal mudou de tamanho");
    let modulo = campo(&l[0], "|n|=");
    assert!((modulo - 1.0).abs() < 1.0e-4, "|n| = {modulo}");
    // O corpo andava para `+x`; a normal tem de ter `x` negativo.
    let x: f32 = l[0]["NORMAL (".len()..]
        .split(',')
        .next()
        .expect("x")
        .parse()
        .expect("x f32");
    assert!(x < 0.0, "a normal nao se opoe ao movimento: {}", l[0]);
    // ⚠️ E é ESSA a convenção que a nossa porta exige — com a normal ao contrário ela recusa o
    // salto em vez de mandar o corpo para dentro do sólido.
    let law = ProjectileLaw::default();
    let passo = SweepStep {
        dir: [1.0, 0.0],
        budget: 10.0,
        steps_left: 4,
    };
    assert!(bounce::next_step(passo, [12.0, 0.0], 0.0, [x, 0.0], &law).is_some());
    assert!(bounce::next_step(passo, [12.0, 0.0], 0.0, [-x, 0.0], &law).is_none());
}

/// ⚠️ **O CONTROLO do corpus**: ele tem cabeçalho com proveniência, e sem isso ninguém sabe de que
/// programa e de que versão ele saiu.
#[test]
fn o_corpus_traz_a_propria_proveniencia() {
    for agulha in [
        "Godot Engine 4.7.2",
        "godot_projectile_probe.gd",
        "--headless",
        "MIT",
    ] {
        assert!(
            CORPUS.contains(agulha),
            "o cabecalho do corpus perdeu `{agulha}` — uma fixtura sem proveniencia nao e' oraculo"
        );
    }
    // E os quatro blocos continuam lá (o piso de população, agora sobre o ficheiro inteiro).
    assert_eq!(
        CORPUS.lines().filter(|l| !l.starts_with('#')).count(),
        9,
        "o corpus mudou de tamanho — re-colha-o e re-leia as barras"
    );
}
