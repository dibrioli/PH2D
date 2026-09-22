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
            materia_da_forma: false,
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
                materia_da_forma: false,
            },
        ),
        (
            "form",
            Planos {
                size: p.size,
                base: &p.base,
                form: &p.form[..4],
                form_occ: &p.occ,
                materia_da_forma: false,
            },
        ),
        (
            "form_occ",
            Planos {
                size: p.size,
                base: &p.base,
                form: &p.form,
                form_occ: &p.occ[..1],
                materia_da_forma: false,
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
            materia_da_forma: false,
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

/// ⭐⭐⭐⭐ **COM A MATÉRIA DA FORMA O ALFA É A COBERTURA DESTE QUADRO** — e sem ela é a do `base`,
/// que é o defeito do report.
///
/// ⛔⛔ **O report do dono (2026-09-21, foto com uma seta):** *«logo que roda o objeto o fundo
/// aparece … parece que vc criou uma máscara»*. Não era uma máscara: era a silhueta que o PRIMEIRO
/// bake gravou no [`Planos::base`], com a forma a ser re-rasterizada por quadro debaixo dela.
///
/// ⚠️ **A fixtura encena exactamente isso** — um `base` cuja silhueta é a metade ESQUERDA (o gesto
/// de ontem) e uma forma cuja cobertura é a metade DIREITA (a peça depois de virar). São dois
/// conjuntos **disjuntos**, logo as duas leis não podem concordar por acidente.
#[test]
fn com_a_materia_da_forma_o_alfa_e_a_cobertura_deste_quadro() {
    let s = superficie();
    let (w, h) = (16u32, 16u32);
    let n = (w * h) as usize;
    let (mut base, mut form) = (vec![0u8; n * 4], vec![0f32; n * 4]);
    let occ = vec![1f32; n];
    for i in 0..n {
        let esquerda = (i as u32 % w) < w / 2;
        // O gesto de ONTEM: a peça estava à esquerda, e o `veste_a_forma` gravou isto.
        base[i * 4..i * 4 + 3].copy_from_slice(&[255, 255, 255]);
        base[i * 4 + 3] = u8::from(esquerda) * 255;
        // O quadro de HOJE: ela virou e está à direita.
        form[i * 4 + 2] = 1.0;
        form[i * 4 + 3] = f32::from(!esquerda);
    }
    let planos = |m| Planos {
        size: (w, h),
        base: &base,
        form: &form,
        form_occ: &occ,
        materia_da_forma: m,
    };
    let acende = |m| {
        acende_imagem_com(
            &s,
            &planos(m),
            &[lampada()],
            Ceu::chapado([0.2; 3]),
            Look::default(),
            4,
        )
        .unwrap()
    };
    let (congelada, viva) = (acende(false), acende(true));

    let mut fugiu = 0usize; // opaco onde a peça JÁ NÃO está — o que o dono vê como «fundo»
    let mut acompanha = 0usize;
    for i in 0..n {
        let esquerda = (i as u32 % w) < w / 2;
        if esquerda && congelada[i * 4 + 3] > 0 {
            fugiu += 1;
        }
        assert_eq!(
            viva[i * 4 + 3],
            u8::from(!esquerda) * 255,
            "o texel {i} tem de mostrar a cobertura DESTE quadro"
        );
        if !esquerda && viva[i * 4 + 3] > 0 {
            acompanha += 1;
        }
    }
    // ⭐ **O CONTROLO é a metade que reproduz o report:** sem ele este gate ficaria verde sobre uma
    // fixtura onde as duas silhuetas por acaso coincidem, e não afirmaria nada.
    assert_eq!(
        fugiu,
        n / 2,
        "controlo: sem a lei, o alfa tem de ficar preso à silhueta de ontem"
    );
    assert_eq!(acompanha, n / 2, "controlo: a peça de hoje tem de aparecer");
}

/// ⭐⭐⭐ **NA POSE DO BAKE, um texel CHEIO não muda um bit — e um texel de BORDA muda, de
/// propósito.**
///
/// ⛔⛔ **A redacção anterior deste gate prometia as duas metades** (*«na pose do bake a saída não
/// muda um bit»*) e a promessa era **falsa na borda** — ela só era verdade porque a fixtura de
/// então tinha o `base` já vestido e ninguém olhava para o passo entre os dois lados. O report
/// seguinte do dono (*«o objeto fica com uma outline branca pixelada indesejada»*, com foto) é
/// exactamente essa metade, e a cura dela faz este gate reprovar. ⇒ *a premissa morre à vista no
/// diff*, e o irmão [`a_materia_da_forma_nao_deixa_um_degrau_de_albedo_na_borda`] é quem mede o
/// que ela escondia.
///
/// ⚠️ Num texel **CHEIO** a mistura da cobertura é a identidade (`c = 1`), logo entrar com ela
/// cheia não muda nada — e é ali que o *«a cura não move o que o dono já aprovou»* continua
/// verdadeiro, ao bit.
#[test]
fn na_pose_do_bake_um_texel_cheio_nao_muda_um_bit_e_um_de_borda_muda() {
    let s = superficie();
    let (w, h) = (16u32, 16u32);
    let n = (w * h) as usize;
    let (mut base, mut form) = (vec![0u8; n * 4], vec![0f32; n * 4]);
    let occ = vec![1f32; n];
    let mut cobertos = 0usize;
    for i in 0..n {
        // ⚠️ **As TRÊS espécies de texel, e a mistura é medida:** fora (`0`), CHEIO (`1`) e de
        // BORDA (fraccionário). A 1.ª redacção só tinha `(i % 17) / 16` e dava `10` cheios em
        // `256` — *um piso de população a reprovar sobre uma fixtura que não continha a metade
        // que o gate promete*.
        let c = match i % 3 {
            0 => 0.0,
            1 => 1.0,
            _ => (i % 17) as f32 / 17.0,
        };
        form[i * 4] = (i % 37) as f32 / 37.0 - 0.5;
        form[i * 4 + 1] = (i % 41) as f32 / 41.0 - 0.5;
        form[i * 4 + 2] = 1.0;
        form[i * 4 + 3] = c;
        if c > 0.0 {
            cobertos += 1;
            // Letra por letra o que o `ph2d_app_sculpt3d::albedo::veste_a_forma` escreve.
            base[i * 4..i * 4 + 4].copy_from_slice(&[255, 255, 255, (c * 255.0).round() as u8]);
        }
    }
    let planos = |m| Planos {
        size: (w, h),
        base: &base,
        form: &form,
        form_occ: &occ,
        materia_da_forma: m,
    };
    let acende = |m| {
        acende_imagem_com(
            &s,
            &planos(m),
            &[lampada()],
            Ceu::chapado([0.2; 3]),
            Look::default(),
            4,
        )
        .unwrap()
    };
    let (antes, depois) = (acende(false), acende(true));
    let (mut cheios, mut bordas) = (0usize, 0usize);
    for i in 0..n {
        let c = form[i * 4 + 3];
        if c >= 1.0 {
            cheios += 1;
            assert_eq!(
                depois[i * 4..i * 4 + 4],
                antes[i * 4..i * 4 + 4],
                "o texel CHEIO {i} mudou na pose do bake"
            );
        } else if c > 0.0 {
            bordas += 1;
            // ⭐ E a mudança tem SENTIDO: a lei nova devolve a luz e não uma mistura com o branco
            // do vestido, logo o texel só pode ficar MAIS ESCURO ou igual.
            assert!(
                depois[i * 4] <= antes[i * 4],
                "o texel de BORDA {i} ficou mais claro ({} contra {}) — a mistura com o albedo \
                 branco é exactamente a orla que esta cura tira",
                depois[i * 4],
                antes[i * 4]
            );
        } else {
            assert_eq!(depois[i * 4 + 3], 0, "fora da cobertura o alfa é zero");
            assert_eq!(antes[i * 4 + 3], 0, "controlo: e já era zero antes");
        }
    }
    // ⛔ **Os DOIS pisos de população**, porque cada metade sozinha fica trivialmente verdadeira
    // sobre um conjunto vazio — e a fixtura tem de conter as duas espécies de texel.
    assert!(
        cheios >= n / 10,
        "controlo: a fixtura tem de ter texels CHEIOS ({cheios})"
    );
    assert!(
        bordas >= n / 10,
        "controlo: a fixtura tem de ter texels de BORDA ({bordas})"
    );
    assert!(
        cobertos >= n / 3,
        "controlo: a fixtura tem de ter texels cobertos ({cobertos})"
    );
    // ⭐ **E o CONTROLO de que a lei não é inerte na fixtura**: sem ele bastaria um corredor que
    // devolvesse o `base` para este gate passar.
    assert_ne!(antes, base, "controlo: a luz tem de mexer nos pixels");
}

/// ⭐⭐⭐ **COM A MATÉRIA DA FORMA O ALBEDO É O NEUTRO** — a outra metade da lei.
///
/// ⚠️ **Ela precisa de um gate PRÓPRIO porque os dois gates irmãos são cegos a ela:** as fixturas
/// deles têm o `base` **branco** (é o que o vestir escreve), e ali `para_luz(255)` **é** `1.0` —
/// logo a metade do albedo lê-se igual com a lei e sem ela. *Uma fixtura no ponto neutro de uma
/// metade não testa essa metade.*
///
/// ⛔ Este `base` é COLORIDO de propósito: ele encena um sprite que chegou vazio e a que alguém
/// escreveu bytes por outro caminho — e afirma que a lei não os lê.
#[test]
fn com_a_materia_da_forma_o_albedo_e_o_neutro() {
    let s = superficie();
    let (w, h) = (12u32, 12u32);
    let n = (w * h) as usize;
    let (mut base, mut form) = (vec![0u8; n * 4], vec![0f32; n * 4]);
    let occ = vec![1f32; n];
    for i in 0..n {
        base[i * 4..i * 4 + 4].copy_from_slice(&[40, 200, 90, 255]);
        form[i * 4 + 2] = 1.0;
        form[i * 4 + 3] = 1.0;
    }
    // A MESMA peça, com o `base` já branco: é essa a saída que a lei tem de reproduzir.
    let mut branco = base.clone();
    for p in branco.as_chunks_mut::<4>().0 {
        *p = [255, 255, 255, 255];
    }
    let acende = |b: &[u8], m: bool| {
        acende_imagem_com(
            &s,
            &Planos {
                size: (w, h),
                base: b,
                form: &form,
                form_occ: &occ,
                materia_da_forma: m,
            },
            &[lampada()],
            Ceu::chapado([0.2; 3]),
            Look::default(),
            2,
        )
        .unwrap()
    };
    assert_eq!(
        acende(&base, true),
        acende(&branco, false),
        "com a matéria da forma o albedo tem de ser o NEUTRO, e não os bytes do `base`"
    );
    // ⭐ **O CONTROLO**: sem a lei, os bytes do `base` mandam — senão o gate acima compara duas
    // corridas de uma lei que já ignorava o albedo.
    assert_ne!(
        acende(&base, false),
        acende(&branco, false),
        "controlo: sem a lei, um `base` verde e um branco TÊM de dar imagens diferentes"
    );
}

/// ⭐⭐⭐⭐ **NÃO HÁ DEGRAU DE ALBEDO NA BORDA** — a orla branca do report de 2026-09-21.
///
/// ⛔⛔ **O report, com foto:** *«funcionou mas o objeto fica com uma outline branca pixelada
/// indesejada»*, sobre a silhueta recortada que a wave anterior destravou.
///
/// **O mecanismo:** a mistura do [`super::super::acende_texel`] é uma **COMPOSIÇÃO sobre o
/// albedo** — num texel de borda meio coberto, metade do pixel é a arte por baixo e metade é a
/// peça. Ela está certa quando existe arte por baixo. Com a matéria a ser a forma **não existe**:
/// o albedo é o neutro (branco) e o que está por baixo é *nada*, dito pelo alfa. ⇒ fora da
/// silhueta a lei devolvia **branco puro** ao lado de um miolo **ACESO** (medido: `255` contra
/// `183`, **`72` códigos**), e toda amostragem bilinear através da borda arrasta esse degrau para
/// dentro dela. É a orla, e ela é **pixelada** porque segue a silhueta rasterizada.
///
/// ⚠️⚠️ **E isto era uma premissa MINHA, escrita como decisão:** o `§11.7` do `docs/Render3d/17`
/// dizia que o branco fora da silhueta era *«deliberado e melhor, porque branco ao pé de branco não
/// deixa orla escura»*. Ele comparava o exterior com o **ALBEDO** (branco) em vez de com a
/// **SAÍDA** (o cinzento aceso). *Uma troca declarada sem a medição ao lado é um palpite com cara
/// de decisão.*
///
/// ⭐ A cura é a cobertura entrar **cheia** na lei, porque ela já viaja no alfa.
#[test]
fn a_materia_da_forma_nao_deixa_um_degrau_de_albedo_na_borda() {
    let s = superficie();
    let (w, h) = (48u32, 48u32);
    let n = (w * h) as usize;
    let (mut base, mut form) = (vec![0u8; n * 4], vec![0f32; n * 4]);
    let occ = vec![1f32; n];
    // Um disco: metade esquerda dentro, metade direita fora. O que se mede é o DEGRAU entre os
    // dois lados da fronteira, que é o que a filtragem mistura.
    let (cx, cy, r) = (23.5f32, 23.5, 16.0);
    for i in 0..n {
        let (x, y) = ((i as u32 % w) as f32, (i as u32 / w) as f32);
        let d = ((x - cx).powi(2) + (y - cy).powi(2)).sqrt();
        let dentro = d <= r;
        form[i * 4] = (x - cx) / r * 0.4;
        form[i * 4 + 1] = (y - cy) / r * 0.4;
        form[i * 4 + 2] = 1.0;
        form[i * 4 + 3] = f32::from(dentro);
        // O `base` VESTIDO, letra por letra o que o `veste_a_forma` escreve.
        if dentro {
            base[i * 4..i * 4 + 4].copy_from_slice(&[255, 255, 255, 255]);
        }
    }
    let acende = |b: &[u8], m: bool| {
        acende_imagem_com(
            &s,
            &Planos {
                size: (w, h),
                base: b,
                form: &form,
                form_occ: &occ,
                materia_da_forma: m,
            },
            &[lampada()],
            Ceu::chapado([0.2; 3]),
            Look::default(),
            4,
        )
        .unwrap()
    };
    // O degrau: o miolo contra o texel logo a seguir à fronteira, na mesma linha.
    let linha = (h / 2) as usize;
    let degrau = |px: &[u8]| {
        let miolo = i32::from(px[(linha * w as usize + (cx as usize)) * 4]);
        let fora = i32::from(px[(linha * w as usize + (cx as usize + r as usize + 3)) * 4]);
        (fora - miolo).abs()
    };

    let vestido = degrau(&acende(&base, true));
    assert!(
        vestido <= 4,
        "a matéria da forma deixa um degrau de {vestido} códigos na borda — é a orla branca que o \
         dono fotografou, e ela nasce de a cobertura ser aplicada DUAS vezes (na cor e no alfa)"
    );

    // ⭐⭐ **O CONTROLO, e ele é uma lei CERTA e não um defeito:** com arte por baixo — um cartão
    // branco OPACO — o texel fora da silhueta É o cartão, logo o degrau existe e tem de existir.
    // *Sem esta metade a régua acima ficaria verde sobre uma lei que devolvesse sempre a mesma
    // cor, e não saberíamos que ela é capaz de LER um degrau.*
    let mut cartao = base.clone();
    for p in cartao.as_chunks_mut::<4>().0 {
        *p = [255, 255, 255, 255];
    }
    let com_arte = degrau(&acende(&cartao, false));
    assert!(
        com_arte >= 40,
        "controlo: sobre um cartão branco o degrau é a própria arte e tem de ser legível \
         ({com_arte} códigos) — a régua tem de saber ler um degrau"
    );
}
