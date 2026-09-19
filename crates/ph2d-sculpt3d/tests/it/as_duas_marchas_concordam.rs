//! ⭐⭐⭐⭐ **O GATE QUE TORNA A DUPLICAÇÃO HONESTA** — as duas marchas de
//! Kimmel–Sethian deste repo têm de devolver o MESMO `f32`.
//!
//! # Porque há duas
//!
//! A [`ph2d_pose::pesos::atravessa`] nasceu em 2026-09-17, para a transição do
//! pincel de pose; a [`ph2d_mesh::atravessa`] nasceu em 2026-09-19, para a
//! máscara de alcance do carimbo. ⛔ **A segunda não chama a primeira, e isso é
//! uma decisão:** a `ph2d-pose` declara **zero dependências** no `Cargo.toml`
//! dela porque *não sabe o que é uma `Mesh`* — é isso que a mantém do lado de lá
//! da parede clean-room, e é o mesmo precedente que a `ph2d-boundary` já
//! escreveu para o `vetor.rs` dela (*«somar três `f32` é vocabulário; a guarda
//! em cima deles é que é a lei»*).
//!
//! # ⚠️ E porque a duplicação precisa de um gate para ser honesta
//!
//! A lei deste repo é que *uma lei escrita em dois sítios ainda não é uma lei —
//! só uma PORTA é*. Quando a porta é impossível (aqui, por arquitectura), o que
//! fica no lugar dela é **a concordância medida**: as duas são chamadas sobre o
//! mesmo corpus e a divergência passa a ser um portão vermelho em vez de uma
//! deriva muda.
//!
//! ⚠️⚠️ **A igualdade é AO BIT e não «dentro de uma folga».** As duas fazem a
//! mesma sequência de operações em `f32`; qualquer folga aqui esconderia
//! exactamente a classe de mudança que o gate existe para apanhar — alguém
//! reordenar um produto e as duas leis passarem a divergir no último dígito, que
//! é como uma marcha começa a dar respostas diferentes em malhas grandes.

/// Um caso degenerado: os três cantos e os dois tempos.
type Caso = ([f32; 3], [f32; 3], [f32; 3], f32, f32);

use ph2d_mesh::atravessa as da_malha;
use ph2d_pose::pesos::atravessa as da_pose;

/// Um gerador determinista — a fixtura é reproduzível e a crate não ganha
/// dependência por causa de um teste.
struct Xorshift(u64);
impl Xorshift {
    fn proximo(&mut self) -> f32 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        // `[-1, 1)`, que é onde os triângulos de uma peça vivem.
        ((self.0 >> 40) as f32 / 8_388_608.0) - 1.0
    }
}

/// ⭐⭐⭐ **As duas marchas concordam ao BIT sobre um corpus de triângulos.**
///
/// ⚠️ **O corpus tem de conter os RAMOS DE RECUSA**, não só o caso feliz: a
/// função devolve `None` em quatro sítios (tempo não finito · aresta de
/// comprimento zero · discriminante negativo · a direcção característica cai
/// fora do triângulo), e um corpus que só produzisse `Some` deixaria metade da
/// lei sem régua. O gate conta as duas populações e exige as duas.
#[test]
fn as_duas_marchas_concordam_ao_bit() {
    let mut r = Xorshift(0x9E37_79B9_7F4A_7C15);
    let (mut com_valor, mut recusadas) = (0usize, 0usize);
    for _ in 0..20_000 {
        let c = [r.proximo(), r.proximo(), r.proximo()];
        let p = [r.proximo(), r.proximo(), r.proximo()];
        let q = [r.proximo(), r.proximo(), r.proximo()];
        // ⚠️ Os tempos são da ordem das arestas — um corpus com tempos absurdos
        // cairia todo no ramo do discriminante e não mediria o resto.
        let tp = (r.proximo() + 1.0) * 0.8;
        let tq = (r.proximo() + 1.0) * 0.8;
        let a = da_malha(c, p, q, tp, tq);
        let b = da_pose(c, p, q, tp, tq);
        match (a, b) {
            (None, None) => recusadas += 1,
            (Some(x), Some(y)) => {
                com_valor += 1;
                assert_eq!(
                    x.to_bits(),
                    y.to_bits(),
                    "as duas marchas divergiram: malha {x:?} contra pose {y:?} \
                     em c={c:?} p={p:?} q={q:?} tp={tp} tq={tq}"
                );
            }
            _ => panic!(
                "as duas marchas discordaram no RAMO (uma recusou e a outra não): \
                 malha {a:?} contra pose {b:?} em c={c:?} p={p:?} q={q:?} tp={tp} tq={tq}"
            ),
        }
    }
    assert!(
        com_valor > 1_000 && recusadas > 1_000,
        "o corpus tem de conter os DOIS ramos ({com_valor} com valor, {recusadas} recusas) \
         -- senão metade da lei fica sem régua"
    );
}

/// ⭐⭐ **E os casos DEGENERADOS que um corpus aleatório nunca produz**, um a um.
///
/// ⚠️ Cada um deles é um `return None` escrito na lei, e um corpus de números ao
/// acaso tem medida nula sobre todos: um ponto repetido, um tempo infinito, dois
/// tempos iguais. *Um ramo que nenhuma fixtura alcança não está testado por ela.*
#[test]
fn as_duas_marchas_concordam_nos_degenerados() {
    let casos: [Caso; 7] = [
        // `p` em cima de `c` — aresta de comprimento zero.
        ([0.0, 0.0, 0.0], [0.0, 0.0, 0.0], [1.0, 0.0, 0.0], 0.5, 0.5),
        // `q` em cima de `c`.
        ([0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 0.0, 0.0], 0.5, 0.5),
        // tempo não finito.
        (
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            f32::INFINITY,
            0.5,
        ),
        (
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            0.5,
            f32::NAN,
        ),
        // os dois cantos no mesmo sítio — triângulo degenerado.
        ([0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [1.0, 0.0, 0.0], 0.3, 0.7),
        // tempos IGUAIS num canto recto — a frente chega pelos dois lados ao
        // mesmo tempo, que é o caso do quad de uma grelha.
        ([0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0], 1.0, 1.0),
        // ângulo OBTUSO em `c`, onde a forma dividida pelo cosseno inverteria.
        ([0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [-0.9, 0.4, 0.0], 0.4, 0.6),
    ];
    for (c, p, q, tp, tq) in casos {
        let a = da_malha(c, p, q, tp, tq);
        let b = da_pose(c, p, q, tp, tq);
        assert_eq!(
            a.map(f32::to_bits),
            b.map(f32::to_bits),
            "degenerado: malha {a:?} contra pose {b:?} em c={c:?} p={p:?} q={q:?}"
        );
    }
}
