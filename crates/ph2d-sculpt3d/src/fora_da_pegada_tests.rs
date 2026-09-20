//! ⭐⭐⭐⭐ **A PORTA DO DOMÍNIO DAS CURVAS DE PESO** — medida nas DUAS famílias,
//! e o censo que impede a terceira de nascer sem ela.
//!
//! O report que a trouxe e o mecanismo estão no doc de [`super::fora_da_pegada`].
//! Aqui ficam as três perguntas que um gate pode responder: *a porta separa o
//! que diz que separa?*, *as duas curvas obedecem-lhe?* e *alguém escreveu uma
//! terceira que não lhe pergunta?*

use crate::{Brush, Falloff, fora_da_pegada};

/// O `t` que o produto mediu quando as manchas apareceram — e os vizinhos dele.
/// ⚠️ **O primeiro valor é de uma MEDIÇÃO**, não um número escolhido: é o
/// menor `t` fora da pegada que a sonda do report observou.
const FORA: [f32; 6] = [1.0005066, 1.0041301, 1.0107825, 1.5, 4.0, f32::INFINITY];

/// A dureza de FÁBRICA do canal de cor — a que torna o expoente uma raiz
/// quadrada, e por isso a que produz o `NaN`. ⚠️ Ela é lida do `Brush`, nunca
/// escrita aqui: *um gate que crava o valor de fábrica deixa de o medir no dia
/// em que ele mudar.*
fn dureza_de_fabrica() -> f32 {
    Brush::default().paint_hardness
}

/// ⭐ **A PORTA SEPARA O QUE DIZ QUE SEPARA.**
#[test]
fn a_porta_separa_dentro_de_fora() {
    for t in [0.0, 0.25, 0.5, 0.999_99] {
        assert!(!fora_da_pegada(t), "{t} está DENTRO da pegada");
    }
    for t in FORA {
        assert!(fora_da_pegada(t), "{t} está FORA da pegada");
    }
    assert!(fora_da_pegada(1.0), "a borda exacta não recebe peso");
    assert!(fora_da_pegada(f32::NAN), "um NaN nunca é uma distância");
    assert!(
        fora_da_pegada(f32::NEG_INFINITY),
        "e o infinito negativo também não"
    );
}

/// ⭐⭐⭐ **NENHUMA CURVA DE PESO DEVOLVE `NaN`, EM `t` NENHUM** — a régua do
/// report, na lei.
///
/// ⚠️ **O expoente da curva de canal é `2·(1 − hardness)`, e o `NaN` só nasce
/// quando ele é FRACCIONÁRIO:** com `hardness ∈ {0; 0,5; 1}` ele vale `2`, `1`
/// e `0`, todos inteiros, e `(−0,2)²` é um número perfeitamente bom. *Uma
/// varredura que só experimentasse durezas redondas ficaria verde sobre o
/// defeito*, e é por isso que a de fábrica (`0,75` ⇒ expoente `0,5`) está na
/// lista com o nome do que ela é.
#[test]
fn nenhuma_curva_de_peso_devolve_nan() {
    let durezas = [0.0, 0.1, 0.25, dureza_de_fabrica(), 0.5, 0.9, 1.0];
    let mut b = Brush::default();
    for h in durezas {
        b.paint_hardness = h;
        b.mask_hardness = h;
        for t in FORA {
            for (nome, w) in [
                ("paint_weight", b.paint_weight(t)),
                ("mask_weight", b.mask_weight(t)),
                ("channel_weight", b.channel_weight(t, h)),
            ] {
                assert!(
                    w == 0.0,
                    "{nome}(t = {t}, hardness = {h}) = {w:?} — fora da pegada o \
                     peso é ZERO. Um NaN aqui contamina a interpolação da FACE \
                     inteira no device, que é a mancha preta do report de 20/09."
                );
            }
        }
        // ⭐ E a família da GEOMETRIA, que já obedecia — o CONTROLO que prova
        // que a varredura percorre um caminho vivo.
        for f in Falloff::ALL {
            for t in FORA {
                let w = f.weight(t);
                assert!(w == 0.0, "{f:?}.weight({t}) = {w:?}");
            }
        }
    }
}

/// ⚠️ **E DENTRO DA PEGADA NADA MUDOU** — a metade que impede a cura de virar
/// uma mudança de produto.
///
/// A lei é reconstruída aqui **à mão** (a expressão da referência, sem a
/// guarda), e é isso que a torna uma régua: comparar a função consigo própria
/// seria uma tautologia.
#[test]
fn dentro_da_pegada_a_curva_e_a_de_sempre() {
    let b = Brush::default();
    for h in [0.0, 0.25, dureza_de_fabrica(), 1.0] {
        for i in 0..1000 {
            let t = i as f32 / 1000.0;
            let softness = 2.0 * (1.0 - f64::from(h));
            let esperado = (1.0 - f64::from(t)).powf(softness) as f32;
            let lido = b.channel_weight(t, h);
            assert!(
                lido.to_bits() == esperado.to_bits(),
                "channel_weight({t}, {h}) = {lido:?}, e a lei da referência dá \
                 {esperado:?} — a guarda mudou o DENTRO da pegada"
            );
        }
    }
}

/// ⭐⭐⭐ **O CENSO: toda curva de peso desta crate pergunta à porta.**
///
/// ⛔⛔ Ele existe porque o defeito não foi uma linha errada — foi uma linha
/// AUSENTE numa das duas cópias de uma lei, com a outra a documentar o
/// mecanismo que a ausência provocaria. *Um ramo que se esqueça da porta fica
/// visível por AUSÊNCIA de chamada; duas cópias divergem outra vez em silêncio.*
///
/// ⚠️ **A população é NOMEADA e tem piso**, senão uma varredura que deixasse de
/// achar os ficheiros leria zero e ficaria verde a medir nada — a forma de
/// falha MUDA que esta casa já pagou num censo por prefixo de nome.
#[test]
fn toda_curva_de_peso_pergunta_a_porta() {
    let fontes = [
        ("falloff.rs", include_str!("falloff.rs")),
        ("brush_scale.rs", include_str!("brush_scale.rs")),
    ];
    let mut vistas = 0usize;
    for (nome, src) in fontes {
        // As assinaturas das curvas que devolvem um peso normalizado.
        for agulha in ["pub fn weight(self, t: f32)", "pub fn channel_weight("] {
            if let Some(i) = src.find(agulha) {
                vistas += 1;
                let corpo = &src[i..];
                let fim = corpo.find("\n    }").unwrap_or(corpo.len());
                // ⛔⛔ **OS COMENTÁRIOS SAEM ANTES DA VARREDURA, e a primeira
                // redacção não os tirava** — a prova de mutação apanhou-a: com a
                // chamada APAGADA o censo ficou VERDE, porque o comentário que
                // EXPLICA a chamada contém o nome dela. *Uma régua textual que
                // lê a prosa ao lado do código mede a prosa*, e é a família que
                // esta casa já pagou noutras varreduras.
                let codigo: String = corpo[..fim]
                    .lines()
                    .filter(|l| !l.trim_start().starts_with("//"))
                    .collect::<Vec<_>>()
                    .join("\n");
                assert!(
                    codigo.contains("fora_da_pegada("),
                    "{nome}: a curva `{agulha}` não pergunta à porta \
                     `fora_da_pegada`. Fora da pegada o peso é ZERO, e sem essa \
                     pergunta um `t > 1` produz NaN (report de 20/09)."
                );
            }
        }
    }
    assert_eq!(
        vistas, 2,
        "o censo encontrou {vistas} curvas de peso e a população são 2 — \
         uma varredura que deixa de as achar fica verde a medir nada"
    );
}
