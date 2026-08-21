//! WGSL gate for `src/shaders/sprite.wgsl` — parse + validate via naga
//! at `cargo test` time so a shader / vertex-layout drift is caught
//! before a GPU pipeline build (which only happens at runtime / smoke).
//!
//! Motivated by ADR-0070-amendment-4 (`rotation: f32` → `basis:
//! [f32; 4]`): the WGSL `InstanceInput.basis` and the Rust
//! `RenderInstance::VERTEX_ATTRIBUTES` `@location(6)` must agree, or the
//! shader silently reads the wrong bytes. Mirrors ph2d-painter-brush's
//! `stamp_wgsl_validates_via_naga`.

const SPRITE_WGSL: &str = include_str!("../src/shaders/sprite.wgsl");

#[test]
fn sprite_wgsl_parses_and_validates() {
    let module = naga::front::wgsl::parse_str(SPRITE_WGSL).unwrap_or_else(|e| {
        panic!(
            "sprite.wgsl failed naga parse:\n{}",
            e.emit_to_string(SPRITE_WGSL)
        )
    });
    let mut validator = naga::valid::Validator::new(
        naga::valid::ValidationFlags::all(),
        naga::valid::Capabilities::all(),
    );
    validator
        .validate(&module)
        .unwrap_or_else(|e| panic!("sprite.wgsl failed naga validation: {e:?}"));
}

#[test]
fn sprite_wgsl_instance_basis_is_vec4_at_location_6() {
    // The @location(6) instance input MUST be a 4-component f32 vector to
    // match `RenderInstance::basis: [f32; 4]` (vertex_attr `6 =>
    // Float32x4`). If a future edit shrinks it back to a scalar
    // `rotation`, the GPU would read 12 bytes of the next field as
    // garbage — pin it here.
    let module = naga::front::wgsl::parse_str(SPRITE_WGSL).expect("parse");
    // Find the global/struct member carrying @location(6) by scanning the
    // struct types for a member binding at location 6.
    let mut found_vec4_at_6 = false;
    for (_h, ty) in module.types.iter() {
        if let naga::TypeInner::Struct { members, .. } = &ty.inner {
            for m in members {
                if let Some(naga::Binding::Location { location: 6, .. }) = m.binding {
                    let inner = &module.types[m.ty].inner;
                    if let naga::TypeInner::Vector {
                        size: naga::VectorSize::Quad,
                        scalar,
                        ..
                    } = inner
                    {
                        assert_eq!(
                            scalar.kind,
                            naga::ScalarKind::Float,
                            "@location(6) must be a float vector (basis)"
                        );
                        found_vec4_at_6 = true;
                    }
                }
            }
        }
    }
    assert!(
        found_vec4_at_6,
        "sprite.wgsl @location(6) is not a vec4<f32> — the 2x2 `basis` instance \
         attribute (ADR-0070-amendment-4) is missing or the wrong shape"
    );
}

/// **A COR DA SPRITE PODE PASSAR DO BRANCO** — é disto que a emissão vive (plano
/// `docs/Sprite_projeto/18` W8).
///
/// # A pergunta que o Enio fez, e a razão de ela virar gate
///
/// Enio, 2026-08-21: *"Emissive funciona para 8 e 16 bits, confere?"* — **sim, e pelo mesmo
/// mecanismo nos dois casos**: o passe de emissão multiplica o `tint` da instância pela
/// intensidade, o fragmento multiplica o texel pelo tint, e o alvo é `Rgba16Float`, onde valores
/// acima de 1.0 **existem**. O que a textura guardou (8 ou 16 bits) não entra na conta — ela é
/// amostrada para `[0, 1]` nos dois casos, e quem passa do branco é o multiplicador.
///
/// ⚠️ **A diferença entre os dois é outra, e é a que vale a pena saber:** uma textura de 16 bits
/// pode guardar valores acima de 1.0 **por si** (um EXR importado), e então ela brilha **sem** o
/// componente. Uma de 8 bits satura em branco e precisa sempre do multiplicador.
///
/// # O que este gate impede
///
/// ⚠️ Um `clamp`/`saturate` no valor devolvido pelo fragmento mataria a emissão **inteira**, em
/// silêncio: a sprite continuaria a desenhar-se igual, o bright-pass deixaria de encontrar seja o
/// que for, e o halo desapareceria sem um erro em lado nenhum. É a única linha do shader de que a
/// feature depende, e não há nada no código dela a protegê-la.
#[test]
fn the_sprite_fragment_does_not_clamp_its_colour_output() {
    // As duas linhas de retorno do fragmento principal — o ramo premultiplicado e o reto.
    for needle in [
        "return vec4<f32>(rgb * extra_alpha, alpha);",
        "return vec4<f32>(rgb * alpha, alpha);",
    ] {
        assert!(
            SPRITE_WGSL.contains(needle),
            "o fragmento do sprite deixou de devolver `{needle}`.\n\n\
             Se o retorno passou a ser envolvido num `clamp`/`saturate`, a EMISSAO morre em \
             silencio: a sprite desenha-se igual, mas nenhuma cor passa de 1.0, o bright-pass do \
             bloom nao encontra nada, e o halo desaparece sem erro nenhum.\n\
             Ver `docs/Sprite_projeto/18` W8 e `shells/desktop/src/render_loop/sprite_emissive.rs`."
        );
    }
}
