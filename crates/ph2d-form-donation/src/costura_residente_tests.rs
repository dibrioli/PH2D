//! ⭐⭐⭐ **A COSTURA RESIDENTE — a mesma lei, com a forma já na placa.**
//!
//! A [`super::passe_da_forma::PasseDaForma::acende_residente`] é a porta da **rota B** (o catavento,
//! `docs/Render3d/17_a_rota_b_o_catavento.md`): ela recebe os dois planos da forma como **vistas de
//! textura**, em vez de fatias da CPU que carrega a cada acendida.
//!
//! ⛔⛔ **Sem estes gates ela é uma promessa.** Uma porta nova que devolve pixels plausíveis é
//! exactamente a forma de defeito que esta casa mais paga, e aqui há **duas** afirmações a fazer,
//! não uma:
//!
//! 1. **É a MESMA LEI** — com as mesmas texturas (`Rgba32Float`/`R32Float`, as que a
//!    [`super::passe_da_forma::PasseDaForma::acende`] cria), as duas rotas saem **BYTE-IDÊNTICAS**.
//!    *Uma barra de tolerância aqui esconderia uma segunda redacção da lei.*
//! 2. **E ela aceita o formato que a RASTERIZAÇÃO produz** (`Rgba16Float`/`R16Float`), que é a razão
//!    de ela existir — com a divergência **MEDIDA** e não estipulada.
//!
//! ⚠️ **A metade `2` é a que decide se a rota B é afordável sem um passe de conversão**, e ela não é
//! óbvia: o layout declara `Float { filterable: false }` e o shader só faz `textureLoad`, logo um
//! `f16` liga-se ali — *mas isso é um facto sobre o `wgpu` e sobre este shader, e um facto assim
//! afirma-se, não se supõe.*

use super::*;
use ph2d_render::TextureAtlas;

fn placa() -> Option<GpuContext> {
    use std::sync::OnceLock;
    static PARTILHADA: OnceLock<Option<GpuContext>> = OnceLock::new();
    PARTILHADA
        .get_or_init(|| GpuContext::new(GpuContext::default_instance(), None).ok())
        .clone()
}

const LADO: u32 = 128;

/// Uma bola com normais que variam em todo o disco — *um plano chato não discrimina formato
/// nenhum, porque uma normal constante quantiza-se sem erro.*
fn bola() -> BakedForm {
    let n = (LADO * LADO) as usize;
    let (mut base, mut form, mut occ) = (vec![0u8; n * 4], vec![0f32; n * 4], vec![1f32; n]);
    let meio = f64::from(LADO) * 0.5;
    let r = meio * 0.85;
    for y in 0..LADO {
        for x in 0..LADO {
            let i = (y * LADO + x) as usize;
            let (dx, dy) = (f64::from(x) - meio, f64::from(y) - meio);
            let d2 = dx * dx + dy * dy;
            if d2 <= r * r {
                form[i * 4] = (dx / r) as f32;
                form[i * 4 + 1] = (dy / r) as f32;
                form[i * 4 + 2] = (1.0 - d2 / (r * r)).sqrt() as f32;
                form[i * 4 + 3] = 1.0;
                base[i * 4..i * 4 + 4].copy_from_slice(&[200, 120, 90, 255]);
                // ⚠️ **A oclusão VARIA**, senão o canal dela é uma constante e a metade que o
                // formato podia estragar nele fica fora da medição.
                occ[i] = 0.3 + 0.7 * (d2 / (r * r)) as f32;
            } else {
                form[i * 4 + 2] = 1.0;
            }
        }
    }
    BakedForm {
        size: (LADO, LADO),
        base,
        form,
        form_occ: occ,
        texture_id: 0,
        rig: LightRig::default(),
        lit_with: None,
        lei: crate::lei_da_luz::Lei::Forma,
        materia_da_forma: false,
        recorte: None,
    }
}

/// Sobe os dois planos para texturas do formato pedido, com `TEXTURE_BINDING`.
///
/// ⚠️ **As fatias de `f16` são convertidas AQUI e não pelo backend** — é isso que torna a metade
/// `2` uma medição do FORMATO e não de um caminho de conversão escondido.
fn texturas_da_forma(
    gpu: &GpuContext,
    bake: &BakedForm,
    meia: bool,
) -> (wgpu::Texture, wgpu::Texture) {
    let (w, h) = bake.size;
    let mk = |nome: &str, formato| {
        gpu.device.create_texture(&wgpu::TextureDescriptor {
            label: Some(nome),
            size: wgpu::Extent3d {
                width: w,
                height: h,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: formato,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        })
    };
    let (ff, fo) = if meia {
        (
            wgpu::TextureFormat::Rgba16Float,
            wgpu::TextureFormat::R16Float,
        )
    } else {
        (
            wgpu::TextureFormat::Rgba32Float,
            wgpu::TextureFormat::R32Float,
        )
    };
    let (ft, ot) = (mk("costura form", ff), mk("costura occ", fo));
    let (bytes_f, bytes_o): (Vec<u8>, Vec<u8>) = if meia {
        (
            bake.form
                .iter()
                .flat_map(|v| half::f16::from_f32(*v).to_le_bytes())
                .collect(),
            bake.form_occ
                .iter()
                .flat_map(|v| half::f16::from_f32(*v).to_le_bytes())
                .collect(),
        )
    } else {
        (
            bytemuck::cast_slice(&bake.form).to_vec(),
            bytemuck::cast_slice(&bake.form_occ).to_vec(),
        )
    };
    let (lf, lo) = if meia {
        (w * 8, w * 2)
    } else {
        (w * 16, w * 4)
    };
    for (tex, dados, linha) in [(&ft, &bytes_f, lf), (&ot, &bytes_o, lo)] {
        gpu.queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: tex,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            dados,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(linha),
                rows_per_image: Some(h),
            },
            wgpu::Extent3d {
                width: w,
                height: h,
                depth_or_array_layers: 1,
            },
        );
    }
    (ft, ot)
}

/// Os pixels que a rota CARREGADA (a de sempre) produz, pela porta do produto.
fn pela_rota_carregada(gpu: &GpuContext, bake: &BakedForm) -> Vec<u8> {
    let atlas = TextureAtlas::new(gpu, LADO.max(256));
    let mut renderer = SpriteRenderer::new(gpu.clone(), wgpu::TextureFormat::Rgba8Unorm, atlas, 64);
    let mut b = BakedForm {
        texture_id: renderer
            .acquire_individual(LADO, LADO, &bake.base)
            .expect("slot"),
        base: bake.base.clone(),
        form: bake.form.clone(),
        form_occ: bake.form_occ.clone(),
        rig: bake.rig,
        ..*bake
    };
    b.lit_with = None;
    acende_com(
        crate::lei_da_luz::Lei::Forma,
        gpu,
        &mut renderer,
        &mut passes_novos(),
        &b.rig,
        &b,
    )
    .expect("a rota carregada acende");
    let (_, _, px) = renderer
        .readback_individual(b.texture_id)
        .expect("le de volta");
    px
}

fn passes_novos() -> PassesDaLuz {
    PassesDaLuz::default()
}

/// Os pixels que a rota RESIDENTE produz, com a forma em texturas do formato pedido.
fn pela_rota_residente(gpu: &GpuContext, bake: &BakedForm, meia: bool) -> Vec<u8> {
    let (ft, ot) = texturas_da_forma(gpu, bake, meia);
    let v = |t: &wgpu::Texture| t.create_view(&wgpu::TextureViewDescriptor::default());
    let (fv, ov) = (v(&ft), v(&ot));
    let atlas = TextureAtlas::new(gpu, LADO.max(256));
    let mut renderer = SpriteRenderer::new(gpu.clone(), wgpu::TextureFormat::Rgba8Unorm, atlas, 64);
    let slot = renderer
        .acquire_individual(LADO, LADO, &bake.base)
        .expect("slot");

    let lampadas = lampadas_do_rig(&bake.rig).expect("lampadas");
    let mut passe = passe_da_forma::PasseDaForma::new(gpu);
    let out = passe
        .acende_residente(
            gpu,
            passe_da_forma::LuzDaCena {
                material: &crate::lei_da_luz::material_da_forma(),
                lampadas: &lampadas,
                ceu: ceu_do_rig(&lampadas),
                olhar: crate::lei_da_luz::OLHAR_DA_FORMA,
                // CONTROLO: a lei de sempre. Ver `BakedForm::materia_da_forma`.
                materia_da_forma: false,
            },
            bake.size,
            &bake.base,
            (&fv, &ov),
        )
        .expect("a rota residente acende");
    renderer
        .copy_texture_into_individual(slot, out, LADO, LADO)
        .expect("copia para o slot");
    let (_, _, px) = renderer.readback_individual(slot).expect("le de volta");
    px
}

fn pior(a: &[u8], b: &[u8]) -> (u8, usize) {
    let mut pior = 0u8;
    let mut fora = 0usize;
    for (x, y) in a.iter().zip(b) {
        let d = x.abs_diff(*y);
        if d > 0 {
            fora += 1;
        }
        pior = pior.max(d);
    }
    (pior, fora)
}

/// ⭐⭐⭐ **A MESMA LEI: com as MESMAS texturas, as duas rotas saem BYTE-IDÊNTICAS.**
///
/// ⛔ **Sem folga nenhuma, e isso é a afirmação:** a rota residente não é uma aproximação da
/// carregada — é a mesma lei com outra fonte para dois bindings. *Uma barra aqui deixaria passar
/// uma segunda redacção do despacho, que é exactamente o que o [`super::passe_da_forma::PasseDaForma::despacha`]
/// existe para impedir.*
#[test]
#[ignore = "precisa de adapter"]
fn a_rota_residente_e_a_mesma_lei_ao_bit() {
    let Some(gpu) = placa() else {
        eprintln!("sem placa — a sonda desiste (skip gracioso NÃO é verde)");
        return;
    };
    let bake = bola();
    let carregada = pela_rota_carregada(&gpu, &bake);
    let residente = pela_rota_residente(&gpu, &bake, false);

    // ⭐⭐⭐ **O CONTROLO, e ele mede a VARIAÇÃO DENTRO DA SILHUETA.** As duas metades desta frase
    // foram escritas por **mutações que SOBREVIVERAM**, e cada uma curou um vácuo diferente:
    //
    // 1. A 1.ª redacção contava pixels «acesos» (`p[0] > 8`): pôr a fixtura toda PRETA no `base`
    //    **não a apaga**, porque a luz acrescenta ambiente e especular e um `base_color` preto
    //    ainda tem destaque. ⇒ a grandeza passou a ser a **excursão** do canal, não o brilho.
    // 2. A 2.ª media a excursão da IMAGEM INTEIRA: zerar a **cobertura** (`form.w`) faz o shader
    //    devolver o albedo CRU (ver o fim da [`ph2d_form_pbr`]`::wgsl::forma_acende_texel`), ou
    //    seja um disco **chapado** sobre fundo transparente — e a excursão continuava a ler o
    //    degrau entre o disco e o fundo. *Uma régua que soma o fundo mede o RECORTE do objecto e
    //    não a forma dele*, e o recorte não é o que esta porta pode estragar.
    //
    // ⛔ *O vácuo que este gate corre é o de as duas rotas concordarem por não haver FORMA nenhuma*
    // — e uma imagem sem forma é **CHATA**, não escura nem vazia.
    //
    // ⭐⭐ **A barra sai de um VALE MEDIDO, e o vale inclui o lado bom** (2026-09-21, esta placa):
    //
    // | fixtura | pixels na silhueta | excursão |
    // |---|---|---|
    // | a boa | `9 289` | **`188`** |
    // | com o `base` todo PRETO | `9 289` | **`246`** |
    // | com a cobertura a `0` | `9 289` | **`0`** |
    // | sem relevo nenhum (`dx = dy = 0`) | `16 384` | **`0`** |
    //
    // ⛔⛔ **A 2.ª linha é porque a fixtura preta NÃO é um vácuo e não se cura:** um `base_color`
    // preto tira a COR e não a FORMA — o destaque especular não sai do albedo —, e a imagem fica
    // **mais** contrastada, não menos. *Uma mutação que aumenta a grandeza sob teste não alcança a
    // propriedade, e curá-la seria curar outra coisa.* ⇒ ela fica NOMEADA aqui, nunca silenciada.
    let dentro: Vec<&[u8; 4]> = carregada
        .as_chunks::<4>()
        .0
        .iter()
        .filter(|p| p[3] > 0)
        .collect();
    assert!(
        dentro.len() > 1_000,
        "a fixtura não tem SILHUETA ({} pixels opacos): sem população, a excursão abaixo é o \
         valor de uma dobra vazia e este gate afirma o nada",
        dentro.len()
    );
    let (lo, hi) = dentro
        .iter()
        .fold((255u8, 0u8), |(lo, hi), p| (lo.min(p[0]), hi.max(p[0])));
    // ⚠️ **O número SAI IMPRESSO mesmo quando o gate passa** — a barra dele foi calibrada num vale
    // MEDIDO entre a fixtura boa e os dois vácuos, e *uma barra cujo valor corrente ninguém vê é a
    // que envelhece em silêncio quando a fixtura muda*.
    eprintln!(
        "controlo do vácuo: {} pixels na silhueta, excursão {} no vermelho",
        dentro.len(),
        hi - lo
    );
    assert!(
        hi - lo > 40,
        "a fixtura não tem FORMA (excursão {} no vermelho, DENTRO da silhueta): sem relevo as duas \
         rotas concordam por vácuo e este gate afirma o nada",
        hi - lo
    );

    let (pior_byte, fora) = pior(&carregada, &residente);
    assert_eq!(
        (pior_byte, fora),
        (0, 0),
        "as duas rotas divergem: pior byte {pior_byte}, {fora} componentes fora"
    );
}

/// ⛔⛔ **A CERCA DO TAMANHO RECUSA EM VOZ ALTA** — e ela existe por uma MUTAÇÃO SOBREVIVENTE.
///
/// A `acende` confere o pedido pela [`ph2d_form_pbr::imagem::Planos::confere`], cujo doc já escreve
/// a lei: *«um segundo predicado de "este pedido está bem formado" continuaria a passar depois de o
/// primeiro ficar torto»*. ⚠️ **A rota residente não tem `Planos`** — ela não recebe fatias da forma
/// —, logo a única metade que lhe resta conferir é o `base`, e essa cerca **é** um segundo
/// predicado. *O que a impede de ser a dívida que aquele doc condena é ter um gate.*
///
/// ⭐ **E ela não é higiene:** sem ela o `write_texture` do `wgpu` faz **PANIC** com fatia curta — e
/// esta porta corre **por quadro**, onde um `Err` devolvido é uma cena que continua e um panic é o
/// app a fechar.
#[test]
#[ignore = "precisa de adapter"]
fn a_cerca_do_tamanho_do_base_recusa_em_voz_alta() {
    let Some(gpu) = placa() else {
        eprintln!("sem placa — a sonda desiste (skip gracioso NÃO é verde)");
        return;
    };
    let bake = bola();
    let (ft, ot) = texturas_da_forma(&gpu, &bake, false);
    let v = |t: &wgpu::Texture| t.create_view(&wgpu::TextureViewDescriptor::default());
    let (fv, ov) = (v(&ft), v(&ot));
    let lampadas = lampadas_do_rig(&bake.rig).expect("lampadas");
    let mut passe = passe_da_forma::PasseDaForma::new(&gpu);
    let mut acende = |base: &[u8]| {
        passe
            .acende_residente(
                &gpu,
                passe_da_forma::LuzDaCena {
                    material: &crate::lei_da_luz::material_da_forma(),
                    lampadas: &lampadas,
                    ceu: ceu_do_rig(&lampadas),
                    olhar: crate::lei_da_luz::OLHAR_DA_FORMA,
                    // CONTROLO: a lei de sempre. Ver `BakedForm::materia_da_forma`.
                    materia_da_forma: false,
                },
                bake.size,
                base,
                (&fv, &ov),
            )
            .map(|_| ())
    };
    // ⚠️ **O CONTROLO POSITIVO vem primeiro**: sem ele, uma porta que recusasse SEMPRE passaria
    // este gate — e a mensagem dele leria exactamente igual.
    acende(&bake.base).expect("o base do tamanho certo é aceite");

    let curto = &bake.base[..bake.base.len() - 4];
    let erro = acende(curto).expect_err("um base curto tem de ser RECUSADO");
    assert!(
        erro.contains("base") && erro.contains(&format!("{}", bake.base.len())),
        "a recusa não diz o que está errado nem por quanto: {erro}"
    );
}

/// ⭐⭐ **E ELA ACEITA O FORMATO QUE A RASTERIZAÇÃO PRODUZ** — `Rgba16Float`/`R16Float`.
///
/// ⚠️ **É esta metade que torna a rota B afordável sem um passe de conversão**, e a divergência é
/// **MEDIDA** e não estipulada: o `f16` tem 10 bits de mantissa contra 23, logo as normais
/// quantizam — a pergunta é se isso chega a um byte de cor.
///
/// A barra sai da medição com a folga que a própria quantização impõe, e o **CONTROLO** é o gate
/// irmão: com `f32` a mesma montagem lê `0`, logo o que este número mede é o FORMATO e nada mais.
#[test]
#[ignore = "precisa de adapter"]
fn a_rota_residente_aceita_a_meia_precisao_da_rasterizacao() {
    let Some(gpu) = placa() else {
        eprintln!("sem placa — a sonda desiste (skip gracioso NÃO é verde)");
        return;
    };
    let bake = bola();
    let carregada = pela_rota_carregada(&gpu, &bake);
    let meia = pela_rota_residente(&gpu, &bake, true);
    let (pior_byte, fora) = pior(&carregada, &meia);
    eprintln!(
        "f16 contra f32: pior byte {pior_byte}, {fora} de {} componentes fora",
        carregada.len()
    );
    assert!(
        pior_byte <= 2,
        "a meia precisão custa {pior_byte} bytes no pior pixel — acima do que a quantização de uma \
         normal explica, logo há outra coisa a acontecer"
    );
}
