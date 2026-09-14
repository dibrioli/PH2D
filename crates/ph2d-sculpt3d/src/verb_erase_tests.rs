//! **O APAGADOR DE DESLOCAMENTO** — a lei dele, medida contra a espec
//! `SPEC_unblocked_brushes.md` §§1 e 4.
//!
//! ⭐ **Nenhum destes gates precisa de GPU nem de uma pilha de multiresolução:**
//! a lei é *«caminha uma fracção até a referência»*, e a referência entra pelo
//! [`SculptStroke::reference`]. Fotografá-la é trabalho da shell; o que **ela
//! diz** é medido aqui, sobre uma referência construída para discriminar.
//!
//! Filho de `tests`, logo `use super::*` alcança as fixtures partilhadas.

use super::*;

/// O dab de referência destes gates: no pólo, com a curva **Constant**.
///
/// ⚠️ **A curva é `Constant` porque a espec mede assim** (`*_constante_1passo`):
/// com ela o peso vale `1` em toda a pegada, e a fracção percorrida fica a
/// depender **só** da força — que é a grandeza que a §1.1 pina.
fn pincel_apagador(strength: f32) -> Brush {
    Brush {
        verb: Verb::EraseMultires,
        radius: 0.6,
        strength,
        falloff: Falloff::Constant,
        ..Brush::default()
    }
}

/// `(antes, depois, referência)` — as três nuvens que os gates comparam.
type TresNuvens = (Vec<[f32; 3]>, Vec<[f32; 3]>, Vec<[f32; 3]>);

/// Corre UM dab com a referência dada e devolve `(antes, depois, referência)`.
fn um_dab_com(referencia: impl Fn([f32; 3]) -> [f32; 3], strength: f32) -> TresNuvens {
    let mut mesh = sphere();
    let antes = snapshot(&mesh);
    let r: Vec<[f32; 3]> = antes.iter().map(|&p| referencia(p)).collect();
    let mut s = SculptStroke::default();
    s.begin(&mesh);
    s.reference = r.clone();
    let b = pincel_apagador(strength);
    s.dab(
        &mut mesh,
        &b,
        &dab_at([0.0, 1.0, 0.0], b.radius),
        Symmetry::default(),
    );
    (antes, snapshot(&mesh), r)
}

/// Os vértices que de facto se moveram, com a fracção percorrida até à
/// referência.
fn fraccoes(antes: &[[f32; 3]], depois: &[[f32; 3]], r: &[[f32; 3]]) -> Vec<f32> {
    let mut out = Vec::new();
    for i in 0..antes.len() {
        let alvo = [
            r[i][0] - antes[i][0],
            r[i][1] - antes[i][1],
            r[i][2] - antes[i][2],
        ];
        let l = (alvo[0] * alvo[0] + alvo[1] * alvo[1] + alvo[2] * alvo[2]).sqrt();
        // ⚠️ **Vértices cujo deslocamento já era ~zero são SALTADOS**, e a espec
        // §4.1 di-lo: ali a razão é `0/0` numericamente, e é de lá que vem o
        // `−0,000002` do piso que ela reporta.
        if l < 1e-4 {
            continue;
        }
        let andou = [
            depois[i][0] - antes[i][0],
            depois[i][1] - antes[i][1],
            depois[i][2] - antes[i][2],
        ];
        let proj = (andou[0] * alvo[0] + andou[1] * alvo[1] + andou[2] * alvo[2]) / (l * l);
        if proj.abs() > 1e-6 {
            out.push(proj);
        }
    }
    out
}

/// ⭐⭐⭐ **A FORÇA ENTRA AO QUADRADO, e a fracção percorrida é EXACTA**
/// (espec §1.1 e §4.1).
///
/// **Medido no oráculo**, com a curva *Constant* e UM dab:
///
/// | força UI | fracção percorrida até à referência |
/// |---|---|
/// | `1,0` | **`1,000000`** — o vértice pousa **NA** referência |
/// | `0,5` | **`0,250000`** — e não `0,500`, que é o que `força¹` daria |
///
/// ⚠️ **É a metade de baixo do slider que ganha resolução**, e isso é
/// deliberado nos autores do alvo (§9.1) — não um acidente numérico.
/// *Implementar `força¹` faz o pincel parecer o dobro de forte a meio curso.*
///
/// ⭐ **Na nossa cadeia isso sai por CONSTRUÇÃO e não por um `powi(2)`:** o alvo
/// leva UM peso e o aplicador multiplica pelo `accum`, que leva o outro — a
/// mesma composição do [`Verb::Draw`] e do [`Verb::Thumb`], já medida contra os
/// oráculos a um ULP.
#[test]
fn o_apagador_percorre_a_fraccao_que_a_forca_ao_quadrado_pede() {
    for (forca, esperado) in [(1.0f32, 1.0f32), (0.5, 0.25)] {
        let (antes, depois, r) = um_dab_com(|p| [p[0] * 0.8, p[1] * 0.8, p[2] * 0.8], forca);
        let f = fraccoes(&antes, &depois, &r);
        assert!(
            f.len() > 20,
            "força {forca}: só {} vértices se moveram — a fixtura não contém o \
             fenómeno",
            f.len()
        );
        let pior = f.iter().fold(0.0f32, |m, &x| m.max((x - esperado).abs()));
        assert!(
            pior < 1e-4,
            "força {forca}: a fracção percorrida desvia {pior:e} de {esperado} — \
             com `força¹` ela leria {}",
            forca
        );
    }
}

/// ⭐⭐ **O MOVIMENTO É COLINEAR COM `R − p`, SEMPRE** (espec §4.2).
///
/// ⛔⛔ **É isto que prova que a lei NÃO é *«mover ao longo da normal até à
/// referência»***, e a fixtura é construída para discriminar: a referência é a
/// malha deslocada por um vector **CONSTANTE**, que não é radial em vértice
/// nenhum. Se o movimento seguisse a normal, a componente perpendicular a
/// `R − p` seria da ordem do próprio deslocamento; medida no oráculo ela é
/// **`3,48e-08`** — ruído de `f32`.
///
/// ⚠️ **Uma referência RADIAL não serviria**, e foi a primeira que escrevi: ali
/// `R − p` e a normal apontam para o mesmo lado, e as duas leis dariam o mesmo
/// resultado. *Uma fixtura em que duas hipóteses coincidem não escolhe entre
/// elas.*
#[test]
fn o_apagador_move_em_linha_recta_ate_a_referencia() {
    let desvio = [0.11f32, 0.05, -0.07];
    let (antes, depois, r) = um_dab_com(
        |p| [p[0] + desvio[0], p[1] + desvio[1], p[2] + desvio[2]],
        1.0,
    );
    let mut pior_perp = 0.0f32;
    let mut movidos = 0;
    for i in 0..antes.len() {
        let andou = [
            depois[i][0] - antes[i][0],
            depois[i][1] - antes[i][1],
            depois[i][2] - antes[i][2],
        ];
        let l2 = andou[0] * andou[0] + andou[1] * andou[1] + andou[2] * andou[2];
        if l2 < 1e-12 {
            continue;
        }
        movidos += 1;
        let alvo = [
            r[i][0] - antes[i][0],
            r[i][1] - antes[i][1],
            r[i][2] - antes[i][2],
        ];
        let ll = (alvo[0] * alvo[0] + alvo[1] * alvo[1] + alvo[2] * alvo[2]).sqrt();
        let u = [alvo[0] / ll, alvo[1] / ll, alvo[2] / ll];
        let proj = andou[0] * u[0] + andou[1] * u[1] + andou[2] * u[2];
        let perp = [
            andou[0] - proj * u[0],
            andou[1] - proj * u[1],
            andou[2] - proj * u[2],
        ];
        pior_perp =
            pior_perp.max((perp[0] * perp[0] + perp[1] * perp[1] + perp[2] * perp[2]).sqrt());
    }
    assert!(
        movidos > 20,
        "só {movidos} vértices se moveram — a fixtura não contém o fenómeno"
    );
    assert!(
        pior_perp < 1e-6,
        "a componente perpendicular a `R − p` mede {pior_perp:e} — a lei ganhou \
         uma direcção privilegiada, e a espec §4.2 mede `3,48e-08` no alvo"
    );
}

/// ⛔ **INVERTER NÃO FAZ NADA, e a saída é BYTE-IDÊNTICA** (espec §§1.2 e 4.3).
///
/// ⚠️ *Um pincel que ignora o `Ctrl` não é um pincel a que falta uma feature* —
/// apagar deslocamento tem um só sentido (o deslocamento zero), e *«apagar ao
/// contrário»* não nomeia nada.
///
/// ⭐ A barra é a **igualdade ao bit** e não um epsilon: o sinal não entra no
/// factor deste pincel, logo não há nada que possa diferir por arredondamento.
#[test]
fn inverter_o_apagador_e_byte_identico() {
    let corre = |invert: bool| {
        let mut mesh = sphere();
        let antes = snapshot(&mesh);
        let r: Vec<[f32; 3]> = antes
            .iter()
            .map(|p| [p[0] * 0.8, p[1] * 0.8, p[2] * 0.8])
            .collect();
        let mut s = SculptStroke::default();
        s.begin(&mesh);
        s.reference = r;
        let b = Brush {
            invert,
            ..pincel_apagador(1.0)
        };
        s.dab(
            &mut mesh,
            &b,
            &dab_at([0.0, 1.0, 0.0], b.radius),
            Symmetry::default(),
        );
        snapshot(&mesh)
    };
    let (normal, invertido) = (corre(false), corre(true));
    // ⚠️ **Anti-vácuo:** sem isto, um pincel que não movesse nada passaria.
    let base = sphere();
    let mexeu = normal
        .iter()
        .zip(base.positions())
        .filter(|(a, b)| a != b)
        .count();
    assert!(
        mexeu > 20,
        "só {mexeu} vértices se moveram — a fixtura não contém o fenómeno"
    );
    assert_eq!(
        normal, invertido,
        "o `Ctrl` mudou a saída do apagador — o sinal não entra no factor dele"
    );
}

/// ⛔⛔ **O `Accumulate` NÃO O ALCANÇA — e a saída é BYTE-IDÊNTICA** (espec §4.1:
/// *«sem direcção privilegiada, sem normal e sem acumulador»*).
///
/// ⚠️⚠️ **Este gate nasceu de uma mutação SOBREVIVENTE, e a causa é uma que
/// este repo já registou por escrito: *um corpus no ponto NEUTRO de um knob não
/// testa esse knob*.** Pôr o apagador de volta na família do
/// [`Verb::accumulates`] passava a suíte inteira — porque **todas** as fixturas
/// herdam `Brush::default()`, onde o `accumulate` nasce **desarmado**. O
/// interruptor nunca era ligado, logo a lista que decide quem o lê nunca era
/// consultada.
///
/// ⇒ a fixtura **arma-o** e compara. ⭐ E são **dois** dabs, porque acumular só
/// tem significado a partir do segundo: com um dab só, a lei aditiva e a normal
/// dão o mesmo resultado e o gate seria vácuo.
#[test]
fn o_acumular_nao_alcanca_o_apagador() {
    let corre = |accumulate: bool| {
        let mut mesh = sphere();
        let antes = snapshot(&mesh);
        let r: Vec<[f32; 3]> = antes
            .iter()
            .map(|p| [p[0] * 0.8, p[1] * 0.8, p[2] * 0.8])
            .collect();
        let mut s = SculptStroke::default();
        s.begin(&mesh);
        s.reference = r;
        let b = Brush {
            accumulate,
            ..pincel_apagador(0.5)
        };
        // ⚠️ **DOIS dabs**: acumular só tem significado a partir do segundo.
        for _ in 0..2 {
            s.dab(
                &mut mesh,
                &b,
                &dab_at([0.0, 1.0, 0.0], b.radius),
                Symmetry::default(),
            );
        }
        snapshot(&mesh)
    };
    let (desarmado, armado) = (corre(false), corre(true));
    // ⚠️ **Anti-vácuo:** sem isto, um pincel inerte passaria.
    let base = sphere();
    let mexeu = desarmado
        .iter()
        .zip(base.positions())
        .filter(|(a, b)| a != b)
        .count();
    assert!(
        mexeu > 20,
        "só {mexeu} vértices se moveram — a fixtura não contém o fenómeno"
    );
    assert_eq!(
        desarmado, armado,
        "o `Accumulate` mudou a saída do apagador — o alvo dele é ABSOLUTO (a \
         superfície de referência) e o tecto `min(f, 1)` diz que ele nunca \
         ultrapassa. *Acumular um alvo absoluto não nomeia nada.*"
    );
}

/// ⛔⛔ **SEM REFERÊNCIA ELE NÃO MOVE NADA — e essa é a rede, não a recusa.**
///
/// A recusa em voz alta é da shell (`open_reference_stroke`, espec §4.3). Esta
/// linha afirma o que acontece se alguém a **contornar**: a lei devolve a
/// posição viva. ⚠️ *Sem ela, um traço sem pilha inventaria uma superfície* —
/// e a tentação mais próxima (a previsão de um passo) é literalmente o outro
/// pincel que a §4.4 manda não construir com este nome.
///
/// ⭐ **O controlo positivo está dentro:** o MESMO dab, com referência, move.
#[test]
fn sem_referencia_o_apagador_nao_move_um_vertice() {
    let mut mesh = sphere();
    let antes = snapshot(&mesh);
    let mut s = SculptStroke::default();
    s.begin(&mesh);
    // ⚠️ **SEM `s.reference`** — é esta ausência que o gate mede.
    let b = pincel_apagador(1.0);
    s.dab(
        &mut mesh,
        &b,
        &dab_at([0.0, 1.0, 0.0], b.radius),
        Symmetry::default(),
    );
    assert_eq!(
        snapshot(&mesh),
        antes,
        "o apagador moveu barro sem referência — ele inventou uma superfície"
    );
    // ⭐ O controlo: o MESMO dab, com referência, move.
    let (a2, d2, _) = um_dab_com(|p| [p[0] * 0.8, p[1] * 0.8, p[2] * 0.8], 1.0);
    assert!(
        a2 != d2,
        "o controlo não moveu nada — o gate acima estaria a medir um dab inerte"
    );
}
