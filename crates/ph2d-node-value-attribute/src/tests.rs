//! Guards for `value.attribute` (doc 50). `super` is the crate root.

use super::*;

fn stream() -> Stream {
    Stream::new(3)
        .with("age", Column::Scalar(vec![0.0, 1.5, 3.0]))
        .with(
            "vel",
            Column::Vec2(vec![[3.0, 4.0], [0.0, 0.0], [-1.0, 0.0]]),
        )
}

/// **Any column the stream carries becomes a value field.** This is the sentence the library
/// could not say before: *colour the sparks by how old they are*.
#[test]
fn a_named_column_becomes_a_value_field() {
    assert_eq!(field(&stream(), "age", 0), vec![0.0, 1.5, 3.0]);
}

/// `Length` mode reads a Vec2 column's magnitude — so `vel` reads as **speed**, which is what an
/// artist asking for speed means (and the 3-4-5 triangle says so).
#[test]
fn length_mode_turns_velocity_into_speed() {
    assert_eq!(field(&stream(), "vel", MODE_LENGTH), vec![5.0, 0.0, 1.0]);
}

/// **A column nobody wrote reads as ZERO, at full length** — not as an error, and above all not
/// as an EMPTY field.
///
/// An empty field would be broadcast downstream as a single global zero (that is what a length-1
/// value means in this library), which looks exactly like a working graph producing black. A
/// typo in an attribute name must not be indistinguishable from a correct graph.
#[test]
fn a_missing_column_reads_as_zeros_at_full_length() {
    assert_eq!(field(&stream(), "ag", 0), vec![0.0; 3], "a typo: zeros");
    assert_eq!(field(&stream(), "", 0), vec![0.0; 3], "…and so is nothing");
    // The shape is preserved: three elements in, three values out.
    assert_eq!(field(&stream(), "nope", 0).len(), stream().count());
}

/// Asking for a Vec2 column as a scalar (or the other way round) is a mistake, not a
/// reinterpretation: zeros, at full length. The stream's types are not guesses to be coerced.
#[test]
fn a_column_of_the_wrong_kind_is_not_coerced() {
    assert_eq!(
        field(&stream(), "vel", 0),
        vec![0.0; 3],
        "vel is not a scalar"
    );
    assert_eq!(field(&stream(), "age", MODE_LENGTH), vec![0.0; 3]);
}

/// **As COMPONENTES de um vetor tornam-se legiveis.** O dominio de valor lia qualquer coluna
/// pelo nome e so sabia devolver um escalar ou uma magnitude; esta escada devolve uma lane.
///
/// ⚠️ **Esta doc afirmava fechar o vao das cinco familias (doc 89 §10.0), e nao fecha** — duas
/// componentes SOLTAS nao sao uma direcao, e nada no dominio de valor as junta num angulo
/// (`value.math` e aritmetica, `value.unary` nao tem trigonometria inversa, o parser nao tem
/// `atan2`). Quem fecha aquela linha e o `MODE_ANGLE`, gateado abaixo.
///
/// ⚠️ The fixture's third element is `[-1, 0]`: its X is **−1** and its LENGTH is **+1**. A
/// component mode that quietly fell back to the magnitude would agree with this test on the
/// first two elements and disagree only there — which is why the fixture has a negative lane.
#[test]
fn a_vector_column_reads_lane_by_lane() {
    let x = MODE_COMPONENT_BASE;
    let y = MODE_COMPONENT_BASE + 1;
    assert_eq!(field(&stream(), "vel", x), vec![3.0, 0.0, -1.0], "X");
    assert_eq!(field(&stream(), "vel", y), vec![4.0, 0.0, 0.0], "Y");
    // The magnitude is still its own mode, and still says +1 where X says −1.
    assert_eq!(field(&stream(), "vel", MODE_LENGTH), vec![5.0, 0.0, 1.0]);
}

/// **One rung, every width** — a colour is a `Vec4` and its lanes are R·G·B·A. Without this the
/// hue/saturation gap of family 9 stays inexpressible: nothing could read a colour back out.
#[test]
fn the_same_rung_reads_a_colour_and_a_vec3() {
    let s = Stream::new(2)
        .with(
            "tint",
            Column::Vec4(vec![[0.1, 0.2, 0.3, 0.4], [0.5, 0.6, 0.7, 0.8]]),
        )
        .with("nrm", Column::Vec3(vec![[1.0, 2.0, 3.0], [4.0, 5.0, 6.0]]));
    for (k, want) in [(0, [0.1, 0.5]), (1, [0.2, 0.6]), (3, [0.4, 0.8])] {
        let got = field(&s, "tint", MODE_COMPONENT_BASE + k);
        assert_eq!(got, want.to_vec(), "colour lane {k}");
    }
    assert_eq!(field(&s, "nrm", MODE_COMPONENT_BASE + 2), vec![3.0, 6.0]);
}

/// **A lane the column does not have is the ordinary miss, not an error** — the module's fence
/// stands: this rung adds a question the node can ANSWER; it does not change what happens when
/// it cannot. `Z` of a `Vec2` is zeros at full length, exactly like a mistyped name.
#[test]
fn a_lane_the_column_does_not_have_is_zeros_not_a_crash() {
    let z = MODE_COMPONENT_BASE + 2;
    assert_eq!(field(&stream(), "vel", z), vec![0.0; 3], "Z of a Vec2");
    assert_eq!(field(&stream(), "age", z), vec![0.0; 3], "Z of a scalar");
    // Lane 0 of a scalar IS the scalar: a scalar is a one-lane vector.
    let x = MODE_COMPONENT_BASE;
    assert_eq!(field(&stream(), "age", x), vec![0.0, 1.5, 3.0]);
}

/// **The rung is additive: the two modes that shipped are byte-identical.** A default that
/// changes what already-authored art does is not a new mode, it is a regression with a chip.
#[test]
fn the_modes_that_shipped_are_untouched() {
    assert_eq!(field(&stream(), "age", 0), vec![0.0, 1.5, 3.0]);
    assert_eq!(field(&stream(), "vel", MODE_LENGTH), vec![5.0, 0.0, 1.0]);
    assert_eq!(field(&stream(), "vel", 0), vec![0.0; 3], "Vec2 in Scalar");
    assert_eq!(field(&stream(), "age", MODE_LENGTH), vec![0.0; 3]);
}

/// **Every channel the picker offers must name a column something WRITES.**
///
/// A picker entry is a promise: the artist chooses a word and gets that quantity. The
/// entry named "Opacity" pointed at a column `"opacity"` — and nothing in the node library
/// writes one. `motion.drive`'s opacity channel writes **`tint` lane 3**
/// (`drive::channel::CH_OPACITY => "tint"`), and `lower_to_instances` reads the alpha from
/// exactly there (`RenderInstance.opacity` is hardcoded to `1.0` — there is no per-instance
/// opacity surface). So the entry resolved to the module's ORDINARY MISS: zeros at full
/// length, in silence, indistinguishable from a mistyped attribute.
///
/// This gate is written against the DATA, not against a list of channel names: it builds a
/// stream the way `motion.drive` leaves one and asserts the picker's own entry reads back
/// what was written. A gate that compared `column` to the literal `"tint"` would be a
/// mirror of the fix rather than a check on it.
#[test]
fn every_offered_channel_reads_a_column_the_library_actually_writes() {
    // A stream as `motion.drive`'s opacity channel leaves it: the alpha lives in `tint`.
    let drove = Stream::new(3).with(
        "tint",
        Column::Vec4(vec![
            [1.0, 1.0, 1.0, 0.25],
            [1.0, 1.0, 1.0, 0.50],
            [1.0, 1.0, 1.0, 0.75],
        ]),
    );
    let opacity = READ_CHANNELS
        .iter()
        .find(|c| c.label == "Opacity")
        .expect("the picker offers an Opacity channel");
    assert_eq!(
        field(&drove, opacity.column, opacity.mode),
        vec![0.25, 0.50, 0.75],
        "picking `Opacity` must read back the alpha `motion.drive` wrote"
    );
}

/// The other entries are not collateral of the fix: each still reads its own column.
/// (Without this, pointing every channel at `tint` would satisfy the gate above.)
#[test]
fn the_other_channels_still_read_their_own_columns() {
    let s = Stream::new(3)
        .with("age", Column::Scalar(vec![0.1, 0.2, 0.3]))
        .with("tint", Column::Vec4(vec![[1.0, 1.0, 1.0, 0.9]; 3]));
    let age = READ_CHANNELS.iter().find(|c| c.label == "Age").unwrap();
    assert_eq!(field(&s, age.column, age.mode), vec![0.1, 0.2, 0.3]);
}

/// **O peso de um CAMPO é legível** — a entrada que faltava, medida contra a forma que
/// um `field.*` de fato deixa.
///
/// Escrito contra o DADO, como o irmão acima: a fixture é um stream com a coluna
/// `falloff` que as cinco `field.*` escrevem (`out.set("falloff", Column::Scalar(…))`),
/// e o gate lê de volta **pela própria entrada do picker**. Comparar `column` com o
/// literal `"falloff"` seria um espelho da correção em vez de uma checagem dela.
///
/// Sem esta linha de tabela a pergunta *"quanta influência este campo tem aqui?"* caía no
/// MISS ORDINÁRIO do módulo — zeros no comprimento cheio, em silêncio, indistinguível de
/// um atributo mal digitado.
#[test]
fn the_weight_a_field_leaves_is_readable_by_the_picker() {
    // Um stream como uma `field.*` o deixa: o peso por linha na coluna `falloff`.
    let shaped = Stream::new(3).with("falloff", Column::Scalar(vec![0.0, 0.5, 1.0]));
    let ch = READ_CHANNELS
        .iter()
        .find(|c| c.label == "Falloff")
        .expect("o picker oferece o canal Falloff");
    assert_eq!(
        field(&shaped, ch.column, ch.mode),
        vec![0.0, 0.5, 1.0],
        "escolher `Falloff` tem de devolver o peso que o campo escreveu"
    );
}

/// **A DIREÇÃO de uma coluna Vec2 — em GRAUS.** A linha da doc 89 §10.0 que CINCO famílias
/// (1·4·5·6·15) citaram como inexprimível: `Speed` sempre respondeu *quão rápido* e descartou
/// *para onde*, e nada no catálogo recuperava a segunda metade.
///
/// O oráculo são vetores cujo ângulo se sabe de cor, e o eixo −Y é o que separa `atan2(y, x)`
/// de `atan2(x, y)`: os dois concordam em (1,0) e discordam de 90° ali.
#[test]
fn the_direction_channel_reads_a_vec2_as_an_angle_in_degrees() {
    let s = Stream::new(5).with(
        "vel",
        Column::Vec2(vec![
            [1.0, 0.0],
            [0.0, 2.0],
            [-3.0, 0.0],
            [0.0, -1.0],
            [1.0, 1.0],
        ]),
    );
    let ch = READ_CHANNELS
        .iter()
        .find(|c| c.label == "Direction")
        .expect("o picker oferece um canal Direction");
    let got = field(&s, ch.column, ch.mode);
    for (i, (g, want)) in got.iter().zip([0.0, 90.0, 180.0, -90.0, 45.0]).enumerate() {
        assert!(
            (g - want).abs() < 1e-3,
            "elemento {i}: a direcao e {want} graus, o canal disse {g} — \
             57.3x disto seria a resposta em RADIANOS"
        );
    }
}

/// **A unidade é a do CONSUMIDOR, e é isto que o gate acima não consegue provar sozinho.**
///
/// A coluna `rot` — a que o `motion.drive(Rotation)` escreve — é em GRAUS, e o lowering só cruza
/// para radianos na borda do render (`ph2d-eval-motion::lower`, *"the app's authored-angle
/// unit"*). Então a pergunta que fecha a cadeia não é *"o número está certo?"*, é ***"levado pela
/// conversão do consumidor, ele aponta ao longo de `vel`?"***.
///
/// ⚠️ A conversão é RE-ESCRITA aqui de propósito: uma crate-nó não pode depender de outra
/// (ADR-0075), então o lowering está fora de alcance e o que este gate pina é a CONVENÇÃO dele.
/// Uma resposta em radianos passaria no gate acima com outros literais e **falharia aqui**, que
/// é exactamente onde o artista a veria — a peça girada para o lugar errado.
#[test]
fn the_angle_points_along_the_velocity_once_the_consumer_converts_it() {
    let vel = [[3.0_f32, 4.0], [-2.0, 0.0], [0.0, -5.0], [-1.0, -1.0]];
    let s = Stream::new(4).with("vel", Column::Vec2(vel.to_vec()));
    for (i, deg) in field(&s, "vel", MODE_ANGLE).iter().enumerate() {
        // A conversao do consumidor, verbatim: graus -> radianos -> base.
        let (sin_r, cos_r) = deg.to_radians().sin_cos();
        let len = (vel[i][0] * vel[i][0] + vel[i][1] * vel[i][1]).sqrt();
        let (ux, uy) = (vel[i][0] / len, vel[i][1] / len);
        assert!(
            (cos_r - ux).abs() < 1e-4 && (sin_r - uy).abs() < 1e-4,
            "elemento {i}: a base ({cos_r:.4}, {sin_r:.4}) tem de ser a velocidade \
             normalizada ({ux:.4}, {uy:.4})"
        );
    }
}

/// **Quem não se move não tem direção, e não pode girar.** `atan2(0, 0)` é `0`, e é a resposta
/// certa: um elemento parado mantém o ângulo em vez de saltar para um valor arbitrário.
#[test]
fn an_element_that_is_not_moving_keeps_its_angle() {
    let s = Stream::new(2).with("vel", Column::Vec2(vec![[0.0, 0.0], [0.0, 0.0]]));
    assert_eq!(field(&s, "vel", MODE_ANGLE), vec![0.0, 0.0]);
}

/// **A direção de uma coluna ESCALAR é o miss ORDINÁRIO do módulo** — zeros no comprimento
/// cheio, a mesma resposta que `Length` sobre um escalar já dava.
///
/// ⚠️ É o braço que a mutação encontra: sem a exclusão do `MODE_ANGLE` no arm escalar, pedir a
/// direção de `age` devolveria **o próprio `age` verbatim**, como se fosse um ângulo — a mentira
/// mais quieta que este nó sabe contar, porque um número plausível sai onde nada deveria sair.
#[test]
fn a_scalar_column_has_no_direction() {
    assert_eq!(field(&stream(), "age", MODE_ANGLE), vec![0.0; 3]);
    assert_eq!(
        field(&stream(), "age", MODE_LENGTH),
        vec![0.0; 3],
        "o irmao"
    );
}

/// **O degrau novo não move nenhum degrau velho** — o `mode` é um param que o grafo GUARDA, e
/// renumerar a escada re-apontaria em silêncio todo documento salvo. O `MODE_ANGLE` é negativo
/// precisamente para não precisar de espaço no meio dos que já shipam.
#[test]
fn the_new_rung_does_not_move_the_rungs_that_ship() {
    assert_eq!(MODE_LENGTH, 1);
    assert_eq!(MODE_COMPONENT_BASE, 2);
    // NEGATIVO: reducoes crescem para BAIXO, a escada de lanes e aberta para cima, e as duas
    // nunca colidem. O valor e pinado (e nao so o sinal) porque e um numero que os documentos
    // GUARDAM -- move-lo re-aponta em silencio o que ja foi salvo.
    assert_eq!(MODE_ANGLE, -1);
    // E a escada inteira continua a responder o que respondia.
    assert_eq!(field(&stream(), "age", 0), vec![0.0, 1.5, 3.0]);
    assert_eq!(field(&stream(), "vel", MODE_LENGTH), vec![5.0, 0.0, 1.0]);
    assert_eq!(
        field(&stream(), "vel", MODE_COMPONENT_BASE),
        vec![3.0, 0.0, -1.0]
    );
}
