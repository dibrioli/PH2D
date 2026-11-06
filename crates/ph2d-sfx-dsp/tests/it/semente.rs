//! **A semente não muda quando o vizinho é editado.** O vizinho tem ganho zero: qualquer diferença no resultado só pode
//! vir da semente do alvo. O ADR-0198 recusou o FunDSP por esta pergunta.

use ph2d_sfx_dsp::{pcm_hash, render};
use ph2d_sfx_recipe::{
    Body, ENGINE_V1, Envelope, Layer, LayerId, ModeLaw, Noise, NoiseFilter, Recipe, Source, Takes,
    Tone, Wave,
};

const ALVO: LayerId = LayerId(7);

fn alvos() -> [Source; 2] {
    [
        Source::Noise(Noise {
            filter: NoiseFilter::Band,
            hz: 2_400.0,
            hz_end: 2_400.0,
            q: 1.2,
        }),
        // O corpo também é semeado: o contato leva ruído.
        Source::Body(Body {
            law: ModeLaw::Bar,
            base_hz: 300.0,
            modes: 8,
            t60: 0.3,
            damping: 0.1,
            rolloff: 1.0,
            contact_ms: 1.0,
            noise: 0.8,
        }),
    ]
}

fn camada(id: u64, gain: f32, source: Source) -> Layer {
    Layer {
        id: LayerId(id),
        gain,
        start: 0.0,
        source,
        envelope: Envelope::default(),
    }
}

fn vizinho(hz: f32) -> Source {
    Source::Tone(Tone {
        wave: Wave::Saw,
        hz,
        hz_end: hz,
        glide: 0.0,
    })
}

fn hash(layers: Vec<Layer>) -> u64 {
    let r = Recipe {
        layers,
        ..Recipe::new(9, 0.25)
    };
    pcm_hash(
        &render(&r, r.seed, ENGINE_V1, &Takes::default())
            .unwrap()
            .samples,
    )
}

#[test]
fn a_semente_nao_muda_quando_o_vizinho_e_editado() {
    for fonte in alvos() {
        let alvo = || camada(ALVO.0, 0.5, fonte.clone());
        let v = |id, hz| camada(id, 0.0, vizinho(hz));
        let base = hash(vec![v(1, 440.0), alvo()]);
        assert_eq!(
            base,
            hash(vec![v(1, 880.0), alvo()]),
            "parâmetro do vizinho"
        );
        assert_eq!(
            base,
            hash(vec![v(1, 440.0), v(2, 90.0), alvo()]),
            "vizinho novo antes do alvo"
        );
        assert_eq!(
            base,
            hash(vec![alvo(), v(1, 440.0)]),
            "alvo movido na pilha"
        );
        assert_eq!(base, hash(vec![alvo()]), "vizinho apagado");
    }
}

/// O controlo positivo: o detector enxerga uma troca de semente quando ela de fato acontece.
#[test]
fn o_id_e_a_semente_da_receita_decidem_a_camada() {
    for fonte in alvos() {
        let com = |id: u64, semente: u64| {
            let r = Recipe {
                layers: vec![camada(id, 0.5, fonte.clone())],
                ..Recipe::new(semente, 0.25)
            };
            pcm_hash(
                &render(&r, semente, ENGINE_V1, &Takes::default())
                    .unwrap()
                    .samples,
            )
        };
        assert_ne!(com(7, 9), com(8, 9), "id diferente, mesma semente");
        assert_ne!(com(7, 9), com(7, 10), "semente da receita diferente");
    }
}

/// Uma camada nova nunca herda o id de uma apagada.
#[test]
fn id_apagado_nao_volta() {
    let mut r = Recipe::new(1, 0.1);
    let a = r.add(1.0, vizinho(100.0), Envelope::default());
    r.layers.clear();
    let b = r.add(1.0, vizinho(100.0), Envelope::default());
    assert_ne!(a, b);
}
