//! Os gates do corredor. ⚠️ Cada um com o controlo ao lado.

use super::super::{Ceu, Lampada, OpenPbr, Surface, acende_texel};
use super::{Look, Planos, acende_imagem_com};

fn superficie() -> Surface {
    OpenPbr::default().prepare()
}

fn lampada() -> Lampada {
    Lampada {
        para_a_luz: [0.3, 0.4, 0.866_025_4],
        radiancia: [1.0, 0.9, 0.8],
    }
}

/// Um sprite plausível: metade com forma, metade fora da silhueta, alfas variados.
struct Peca {
    size: (u32, u32),
    base: Vec<u8>,
    form: Vec<f32>,
    occ: Vec<f32>,
}

impl Peca {
    fn nova(w: u32, h: u32) -> Self {
        let n = (w * h) as usize;
        let mut base = vec![0u8; n * 4];
        let mut form = vec![0f32; n * 4];
        let mut occ = vec![0f32; n];
        for i in 0..n {
            let a = (i % 37) as f32 / 37.0 - 0.5;
            let b = (i % 41) as f32 / 41.0 - 0.5;
            base[i * 4] = (i % 251) as u8;
            base[i * 4 + 1] = (i % 253) as u8;
            base[i * 4 + 2] = (i % 249) as u8;
            base[i * 4 + 3] = (i % 256) as u8; // ⚠️ o alfa VARIA — ver o gate que o segue
            let dentro = i % 3 != 0;
            form[i * 4] = if dentro { a } else { 0.0 };
            form[i * 4 + 1] = if dentro { b } else { 0.0 };
            form[i * 4 + 2] = 1.0;
            form[i * 4 + 3] = if dentro { 1.0 } else { 0.0 };
            occ[i] = if dentro { 0.7 } else { 1.0 };
        }
        Self {
            size: (w, h),
            base,
            form,
            occ,
        }
    }

    fn planos(&self) -> Planos<'_> {
        Planos {
            size: self.size,
            base: &self.base,
            form: &self.form,
            form_occ: &self.occ,
        }
    }
}

/// ⭐⭐⭐ **A REPARTIÇÃO NÃO MUDA UM BIT.**
///
/// ⚠️ É verdade **por construção** — cada texel é independente e o albedo lê-se da própria fatia —,
/// e é exactamente por isso que tem de haver um gate: uma propriedade que ninguém afirma é uma
/// propriedade que a primeira optimização (uma média de vizinhos, uma normalização por faixa) parte
/// **em silêncio**, e o sintoma seriam costuras horizontais que mudam de sítio a cada corrida.
#[test]
fn a_reparticao_nao_muda_um_bit() {
    let s = superficie();
    let p = Peca::nova(29, 17); // ⚠️ primos: um `w` que divide o número de faixas esconderia o resto
    let um = acende_imagem_com(
        &s,
        &p.planos(),
        &[lampada()],
        Ceu::chapado([0.1; 3]),
        Look::default(),
        1,
    )
    .unwrap();
    for faixas in [2usize, 3, 7, 64, 5000] {
        let n = acende_imagem_com(
            &s,
            &p.planos(),
            &[lampada()],
            Ceu::chapado([0.1; 3]),
            Look::default(),
            faixas,
        )
        .unwrap();
        assert_eq!(n, um, "com {faixas} faixas a imagem mudou");
    }

    // ⭐ **O CONTROLO**: a lei TEM de ter mexido nos pixels — senão o gate acima compara duas
    // cópias do `base` e fica verde sobre um corredor que não acende nada.
    assert_ne!(um, p.base, "controlo: a luz tem de mudar o sprite");
}

/// ⭐⭐ **Cada pixel é o que a lei por texel dá** — o corredor não é uma segunda lei.
#[test]
fn cada_pixel_e_o_que_a_lei_por_texel_da() {
    let s = superficie();
    let p = Peca::nova(8, 8);
    let out = acende_imagem_com(
        &s,
        &p.planos(),
        &[lampada()],
        Ceu::chapado([0.1; 3]),
        Look::default(),
        3,
    )
    .unwrap();
    for i in 0..64usize {
        let t = super::super::Texel {
            normal: [p.form[i * 4], p.form[i * 4 + 1], p.form[i * 4 + 2]],
            // ⚠️ Pela PORTA do produto ([`super::codigo`]) e nunca reconstruída aqui — ver o doc
            // dela: este arnês é o segundo consumidor que já divergiu uma vez.
            albedo: [
                super::codigo::para_luz(p.base[i * 4]),
                super::codigo::para_luz(p.base[i * 4 + 1]),
                super::codigo::para_luz(p.base[i * 4 + 2]),
            ],
            cobertura: p.form[i * 4 + 3],
            oclusao: p.occ[i],
        };
        let c = acende_texel(
            &s,
            &t,
            &[lampada()],
            Ceu::chapado([0.1; 3]),
            Look::default(),
        );
        for k in 0..3 {
            let quer = super::codigo::de_luz(c[k]);
            assert_eq!(out[i * 4 + k], quer, "texel {i}, canal {k}");
        }
    }
}

/// ⭐⭐⭐ **FORA DA SILHUETA o byte sai INTACTO, e o ALFA atravessa em todo lado.**
///
/// ⚠️ O alfa é a silhueta do sprite: uma lei de luz que lhe tocasse mudaria o RECORTE do objecto ao
/// mover a lâmpada, que é um defeito que nenhuma régua de cor apanharia.
#[test]
fn fora_da_silhueta_o_byte_sai_intacto_e_o_alfa_atravessa() {
    let s = superficie();
    let p = Peca::nova(16, 16);
    let out = acende_imagem_com(
        &s,
        &p.planos(),
        &[lampada()],
        Ceu::chapado([0.4; 3]),
        Look::default(),
        4,
    )
    .unwrap();

    let (mut fora, mut dentro) = (0usize, 0usize);
    for i in 0..256usize {
        assert_eq!(
            out[i * 4 + 3],
            p.base[i * 4 + 3],
            "o alfa do texel {i} mudou"
        );
        if p.form[i * 4 + 3] == 0.0 {
            fora += 1;
            assert_eq!(
                &out[i * 4..i * 4 + 3],
                &p.base[i * 4..i * 4 + 3],
                "o texel {i} está fora da silhueta e o byte mudou"
            );
        } else if out[i * 4..i * 4 + 3] != p.base[i * 4..i * 4 + 3] {
            dentro += 1;
        }
    }
    // **Os DOIS pisos de população.** Sem o primeiro a promessa do no-op seria sobre um conjunto
    // vazio; sem o segundo, sobre um corredor que não acende.
    assert!(
        fora >= 64,
        "controlo: a peça tem de ter texels fora da silhueta ({fora})"
    );
    assert!(
        dentro >= 64,
        "controlo: a luz tem de mexer nos de dentro ({dentro})"
    );
}

/// ⛔ **Um plano curto RECUSA, e a queixa NOMEIA qual.**
///
/// ⚠️ Sem isto o laço leria o que coubesse e devolveria uma imagem plausível — *um defeito de
/// TAMANHO lido como um defeito de LUZ*.
#[test]
fn um_plano_curto_recusa_e_diz_qual() {
    let s = superficie();
    let p = Peca::nova(8, 8);
    for (nome, planos) in [
        (
            "base",
            Planos {
                size: p.size,
                base: &p.base[..4],
                form: &p.form,
                form_occ: &p.occ,
            },
        ),
        (
            "form",
            Planos {
                size: p.size,
                base: &p.base,
                form: &p.form[..4],
                form_occ: &p.occ,
            },
        ),
        (
            "form_occ",
            Planos {
                size: p.size,
                base: &p.base,
                form: &p.form,
                form_occ: &p.occ[..1],
            },
        ),
    ] {
        let e = acende_imagem_com(
            &s,
            &planos,
            &[lampada()],
            Ceu::chapado([0.0; 3]),
            Look::default(),
            2,
        )
        .expect_err("um plano curto tem de recusar");
        assert!(e.contains(nome), "a queixa tem de nomear o `{nome}`: {e}");
    }

    // ⭐ **O CONTROLO**: com os três planos certos ela ACEITA — senão este gate ficaria verde sobre
    // uma porta que recusa tudo.
    assert!(
        acende_imagem_com(
            &s,
            &p.planos(),
            &[lampada()],
            Ceu::chapado([0.0; 3]),
            Look::default(),
            2
        )
        .is_ok()
    );
}

/// ⚠️ **Um sprite de área ZERO devolve o vazio sem estourar** — o `div_ceil` por zero texels e o
/// `spawn` de uma fatia vazia são dois sítios onde isto morreria.
#[test]
fn um_sprite_vazio_nao_estoura() {
    let s = superficie();
    let out = acende_imagem_com(
        &s,
        &Planos {
            size: (0, 0),
            base: &[],
            form: &[],
            form_occ: &[],
        },
        &[lampada()],
        Ceu::chapado([0.0; 3]),
        Look::default(),
        8,
    )
    .unwrap();
    assert!(out.is_empty());
}

/// ⭐⭐⭐ **FORA DA SILHUETA O BYTE SAI INTACTO PARA QUALQUER OLHAR** — e é por isto que a vista
/// entra ANTES da mistura da cobertura.
///
/// ⛔⛔ A 1.ª redacção aplicava a vista ao resultado JÁ misturado, e com `Look::default()` isso é
/// indistinguível — a identidade não mexe em nada. *Uma promessa que só vale no valor de fábrica
/// não é uma promessa*, e com uma exposição a sério a arte 2D do sprite que a forma **não tocou**
/// mudava de cor.
///
/// ⚠️ A régua varre olhares que MEXEM (as duas vistas, exposições para cima e para baixo) e o
/// **piso de população** exige que eles de facto mudem o miolo — senão o gate ficaria verde sobre
/// uma lista de identidades.
#[test]
fn fora_da_silhueta_o_byte_sai_intacto_para_qualquer_olhar() {
    use super::ViewTransform;

    let s = superficie();
    let p = Peca::nova(16, 16);
    let base = acende_imagem_com(
        &s,
        &p.planos(),
        &[lampada()],
        Ceu::chapado([0.0; 3]),
        Look::default(),
        3,
    )
    .unwrap();

    let mut mexeram = 0;
    for olhar in [
        Look {
            exposure_stops: 2.0,
            view: ViewTransform::Standard,
        },
        Look {
            exposure_stops: -2.0,
            view: ViewTransform::Standard,
        },
        Look {
            exposure_stops: 0.0,
            view: ViewTransform::Neutral,
        },
        Look {
            exposure_stops: 3.0,
            view: ViewTransform::Neutral,
        },
    ] {
        let out = acende_imagem_com(
            &s,
            &p.planos(),
            &[lampada()],
            Ceu::chapado([0.0; 3]),
            olhar,
            3,
        )
        .unwrap();
        let mut dentro_mudou = false;
        for i in 0..256usize {
            if p.form[i * 4 + 3] == 0.0 {
                assert_eq!(
                    &out[i * 4..i * 4 + 4],
                    &p.base[i * 4..i * 4 + 4],
                    "o texel {i} está FORA da silhueta e o olhar {olhar:?} mexeu-lhe"
                );
            } else if out[i * 4..i * 4 + 3] != base[i * 4..i * 4 + 3] {
                dentro_mudou = true;
            }
        }
        if dentro_mudou {
            mexeram += 1;
        }
    }
    // **O PISO**: sem isto, uma lista de olhares que não fizessem nada deixava o gate verde.
    assert!(
        mexeram >= 3,
        "controlo: os olhares TÊM de mexer no miolo ({mexeram} de 4)"
    );
}
