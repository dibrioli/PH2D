//! **O OBJETO ASSADO** — os canais que uma malha doou a um sprite, e a luz que os lê.
//!
//! ⚠️ **Este módulo NÃO está atrás da feature `sculpt3d`, e é essa ausência que É a wave.** O
//! `docs/3D/02.2` chama de *rota A* o caminho em que o G-buffer é gerado uma vez, vira canal do
//! sprite, e **a malha some do build** — e a frase que a torna uma promessa em vez de prosa é *"o
//! runtime lê os canais sem o módulo 3D"*. Enquanto a acendida morasse dentro da feature, reabrir um
//! projeto num binário sem escultura devolveria um objeto que ninguém consegue iluminar. Aqui mora
//! tudo o que um objeto assado precisa **depois** de a escultura ter ido embora; o gesto de assar —
//! que precisa da malha — fica na `ph2d_app_sculpt3d::bake`, atrás da feature, onde deve estar.
//! ⚠️ **Ela não é alcançável daqui, e é esse o desenho**: esta folha é o que sobra quando o
//! módulo 3D vai embora, então uma seta dela para a família seria a dependência que ela recusa.
//!
//! ## O que fica guardado, e por que cada um
//!
//! | guardado | por quê |
//! |---|---|
//! | `base` — os pixels ANTES da luz | re-acender a partir do que já está aceso **compõe**, e a arte escurece a cada toque de lâmpada |
//! | `form` — `[nx, ny, nz, peso]` por texel | é o G-buffer; ele **não depende do rig**, então mover a lâmpada NÃO re-rasteriza a malha |
//! | `rig` — o rig que acendeu | reabrir sem ele acenderia o objeto com a luz DEFAULT, e a arte mudaria em silêncio |
//! | `texture_id` — o slot do sprite | re-acender copia para o MESMO slot: nenhuma textura nova por passo de lâmpada |
//!
//! ⚠️ **É por isso que o objeto é RELUMINÁVEL e não apenas "assado bonito".** Um bake que só
//! escrevesse pixels acesos entregaria uma sprite que o artista não pode mais iluminar — e iluminar
//! é a palavra inteira do objetivo 2.
//!
//! ## ⛔ A LEI da luz já não é UMA — são DUAS, e nenhuma delas é escrita aqui
//!
//! ⚠️ **Esta secção dizia *«a lei da luz é UMA»* e a premissa morreu** quando o objectivo 2 chegou
//! (`docs/Render3d/15`): hoje o artista escolhe entre a lei da **TINTA** (o `ImpastoLightPass`, o
//! mesmo passe que acende a tinta do Painter, e o valor de fábrica) e a lei da **FORMA** (o OpenPBR,
//! que dá a um plástico vermelho um destaque BRANCO em vez da cor da peça). A porta que escolhe é a
//! [`acende_com`], e a [`crate::lei_da_luz`] é quem a decide.
//!
//! ⭐ **O que NÃO mudou é a razão de a frase existir:** nenhum kernel de iluminação é escrito neste
//! módulo. A lei da tinta vive na `ph2d-render` e a da forma na `ph2d-form-pbr` — uma redacção local
//! seria a **segunda resposta** a *como uma normal vira luz*, que é a falha de duas-portas que este
//! módulo já recusou no rig (`ph2d-light`, W3).
//!
//! ## ⚠️ ABERTO, e NOMEADO por ser compartilhado
//!
//! O mapa é chaveado por **bits de entidade**, e o undo global **RESPAWNA** as entidades (bits
//! novos, mesmo `Name`). Depois de um Ctrl+Z que refaça o mundo, a entrada fica órfã: o sprite
//! continua apontando para o slot aceso — então **a tela não muda** —, mas mover a lâmpada deixa de
//! o alcançar e um save seguinte não o encontra.
//!
//! ⚠️ **Não foi consertado aqui de propósito, e a razão é que o defeito não é desta wave:** o
//! `PaintedDoc` do Painter tem exatamente a mesma forma (o `doc_cache` dele também é keyed por bits,
//! e o `restore_painted_docs` só é chamado pelo LOAD, nunca pelo undo — conferido por grep). Curar
//! um dos dois e não o outro seria **duas respostas** para *como um documento reencontra o seu
//! objeto depois de um respawn*, que é precisamente a falha que a identidade estável existe para
//! remover. A cura é a mesma para os dois — re-key pelos componentes estáveis depois de todo
//! restore — e é wave própria, do dono dos dois lados.

use ph2d_gpu::GpuContext;
use ph2d_light::{LightRig, MAX_LIGHTS};
use ph2d_painter_brush::material::SpecLut;
use ph2d_render::{ImpastoLightPass, SpriteRenderer};

/// **O ENQUADRAMENTO** — ver o irmão.
#[path = "baked_form_recorte.rs"]
mod recorte;
pub use recorte::Recorte;

/// **O que um SPRITE empresta ao passe da tinta** — os planos que ele não tem, fabricados.
///
/// ⚠️ Módulo próprio, e o corte é o que o quadro do topo já desenha: *o passe fala o vocabulário da
/// TINTA*. De um lado o que um objeto assado É (os canais, o carimbo, a acendida); do outro o que o
/// passe EXIGE (relevo, cobertura, material, lâmpadas, a entrada).
pub mod planes;

/// **A LEI DA FORMA NO DISPOSITIVO** — o gémeo rápido da [`acende_pela_forma`].
///
/// ⚠️ Módulo próprio pela mesma razão do irmão acima, e o corte é outro: lá mora *o que o passe da
/// TINTA exige*; aqui, *como a lei da FORMA corre na placa*. Ver o cabeçalho dele para porque a
/// morada é esta e não a `ph2d-render` que o plano escreveu.
pub mod passe_da_forma;

use planes::{BakePlanes, build_input, neutral_planes, resolved_lamps, upload_rgba};

/// Quantos `u32` o carimbo do rig ocupa: a contagem, mais nove floats por lâmpada
/// (`dir` + `half` + `tint`).
const STAMP_LEN: usize = 1 + MAX_LIGHTS * 9;

/// **O rig com que estes pixels foram acesos.**
///
/// ⚠️ **Por BITS, nunca por valor** — a mesma lei do `FormStamp` da doação, e pela mesma razão: um
/// rig degenerado (`NaN` num ângulo) nunca compararia igual a si mesmo, e o sprite seria re-aceso
/// **todo frame, para sempre**, sem nada na tela dizendo por quê.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct RigStamp([u32; STAMP_LEN]);

/// O carimbo do rig, como **função pura** — separado do resto para o gate poder exercitá-lo sem um
/// `wgpu::Device` (o precedente é o `stamp_of` da doação).
pub fn rig_stamp(rig: &LightRig) -> RigStamp {
    let mut out = [0u32; STAMP_LEN];
    if let Some(resolved) = ph2d_light::resolve(rig) {
        let lamps = resolved.lamps();
        out[0] = u32::try_from(lamps.len()).unwrap_or(0);
        for (i, l) in lamps.iter().enumerate() {
            let slot = 1 + i * 9;
            for (j, v) in l
                .dir
                .iter()
                .chain(l.half.iter())
                .chain(l.tint.iter())
                .enumerate()
            {
                out[slot + j] = v.to_bits();
            }
        }
    }
    RigStamp(out)
}

/// **Um sprite que a forma acende.** Ver o quadro do topo para o que cada campo compra.
pub struct BakedForm {
    pub size: (u32, u32),
    /// Os pixels do sprite **antes** de qualquer luz — a fonte de toda re-acendida.
    pub base: Vec<u8>,
    /// O G-buffer da malha: `[nx, ny, nz, peso]` por texel. Não depende do rig.
    pub form: Vec<f32>,
    /// **A OCLUSÃO DE FORMA** — cavidade × os dois AOs, um escalar por texel. Tampouco depende do rig.
    ///
    /// ⚠️ **Ela viaja em vez de ser assada no [`Self::base`], e o motivo está no `bake_one`:** um
    /// re-bake REUSA o `base` (ler a tela de volta acenderia o que já está aceso), então
    /// pré-multiplicar a oclusão ali a comporia a cada gesto e o objeto escureceria sozinho — o
    /// defeito exato que o `base` existe para impedir, um nível acima.
    pub form_occ: Vec<f32>,
    /// O slot individual que o sprite passou a apontar. Re-acender COPIA para ele.
    pub texture_id: u32,
    /// **O rig AUTORADO destes pixels** — o que o artista tinha na mão quando assou.
    ///
    /// ⚠️ Ele viaja no documento, e sem isso reabrir o projeto acenderia o objeto com o rig DEFAULT:
    /// a arte mudaria de luz ao ser aberta, em silêncio, e ninguém saberia dizer por quê.
    pub rig: LightRig,
    /// O rig com que os pixels visíveis foram acesos — `None` até a primeira acendida.
    pub lit_with: Option<RigStamp>,
    /// ⭐⭐⭐⭐ **A LEI QUE ACENDE ESTES PIXELS** — e ela viaja no documento, como o `rig`.
    ///
    /// ⚠️ **O argumento é o do vizinho, letra por letra:** reabrir sem ele acenderia o objecto com a
    /// lei de fábrica **de quem o abre**, e a arte mudaria em silêncio. É por isso que este campo
    /// mora aqui e não numa variável de ambiente — ver o cabeçalho da
    /// [`crate::lei_da_luz`], que carregou o plano desta wave escrito desde que existe.
    ///
    /// ⛔ **Quem o lê é a [`light`], através da [`crate::lei_da_luz::efectiva`]** — nunca
    /// directamente, senão o bissector deixaria de alcançar metade dos caminhos.
    pub lei: crate::lei_da_luz::Lei,
    /// ⭐⭐⭐⭐ **O ENQUADRAMENTO com que a forma foi rasterizada** — ver [`Recorte`].
    ///
    /// ⚠️ **`None` = a vista inteira**, que é o que todo documento anterior a 2026-09-21 quer
    /// dizer e o que uma cena sem canvas publicado produz. Ele **não** é um valor de fábrica
    /// disfarçado: a ausência é a resposta (*«ninguém enquadrou isto»*), e é ela que faz um
    /// projecto velho reabrir exactamente como era.
    ///
    /// ⛔⛔ **E é por ele que o CATAVENTO não salta:** a rota B re-rasteriza por quadro, e sem
    /// esta memória ela usaria a vista inteira enquanto o assado por baixo dela tinha um recorte —
    /// a peça saltaria para encher o sprite no primeiro quadro em que o relógio andasse.
    pub recorte: Option<Recorte>,
}

/// ⭐⭐⭐ **AS DUAS LEIS, CADA UMA COM O SEU PASSE — numa ranhura só.**
///
/// ⚠️ **Ela existe para o consumidor não ter de saber que são duas.** A shell guarda UM campo e
/// passa-o a UMA porta; qual dos dois pipelines é construído decide-o a [`acende_com`], e ele nasce
/// **preguiçoso** — quem nunca liga a lei nova nunca compila o shader dela.
///
/// ⛔ A alternativa era um segundo `Option<…>` ao lado do primeiro, na `App`, no `FrameGfx` e nas
/// duas fases do quadro: *quatro sítios que teriam de concordar, e uma terceira lei a custar quatro
/// edições outra vez.* Aqui ela custa **um campo**, e o consumidor não se mexe.
/// ⭐⭐⭐ **A rota B — a mesma luz com a forma já na placa.** Ver [`forma_viva`].
///
/// ⚠️ Ela é **filha deste módulo** e não irmã, e a razão é o campo `forma` do [`PassesDaLuz`]: um
/// módulo irmão não o alcança, e torná-lo público abriria o passe a quem não passa por uma porta.
pub mod forma_viva;

#[derive(Default)]
pub struct PassesDaLuz {
    /// O passe da TINTA — a lei de sempre, e o valor de fábrica.
    tinta: Option<ImpastoLightPass>,
    /// O passe da FORMA — o OpenPBR no dispositivo. Ver [`passe_da_forma`].
    forma: Option<passe_da_forma::PasseDaForma>,
}

/// **ACENDE um objeto assado** e copia o resultado para o slot do sprite.
///
/// ⚠️ **Sem leitura de volta.** A saída do passe vai direto para a textura individual
/// (`copy_texture_into_individual`), que é o mesmo caminho que o preview do Painter usa. Um
/// round-trip pela CPU custaria o dobro da tela por passo de lâmpada para produzir bytes que ninguém
/// do lado da CPU lê.
///
/// ⚠️ **A porta é UMA**, e é ela que o load chama. Uma segunda acendida escrita do lado da
/// persistência faria a arte **SALTAR** ao reabrir o arquivo — o defeito que o ADR-0128 pagou cinco
/// vezes, e que aqui teria a forma mais cruel: o objeto está certo enquanto o app está aberto.
pub fn light(
    gpu: &GpuContext,
    renderer: &mut SpriteRenderer,
    passes: &mut PassesDaLuz,
    rig: &LightRig,
    bake: &BakedForm,
) -> Result<(), String> {
    acende_com(
        crate::lei_da_luz::efectiva(bake.lei),
        gpu,
        renderer,
        passes,
        rig,
        bake,
    )
}

/// **O mesmo, com a LEI dita** em vez de lida do ambiente.
///
/// ⚠️ Ela existe para o gate, e a razão é uma lei desta casa: *um gate que lê o ambiente mede a
/// máquina*. Com a lei como PARÂMETRO, «as duas acendem» e «a de sempre não mudou um bit» são
/// propriedades que se afirmam.
///
/// # Errors
/// Ver [`light`].
pub fn acende_com(
    lei: crate::lei_da_luz::Lei,
    gpu: &GpuContext,
    renderer: &mut SpriteRenderer,
    passes: &mut PassesDaLuz,
    rig: &LightRig,
    bake: &BakedForm,
) -> Result<(), String> {
    match lei {
        crate::lei_da_luz::Lei::Tinta => acende_pela_tinta(gpu, renderer, passes, rig, bake),
        crate::lei_da_luz::Lei::Forma => acende_pela_forma(gpu, renderer, passes, rig, bake),
    }
}

/// ⭐⭐⭐ **O OpenPBR sobre os canais do objecto assado — NO DISPOSITIVO.**
///
/// ⛔⛔ **Ele corria na CPU e a medição obrigou a trocar** (`docs/Render3d/15` §7): em paralelo, a
/// `1024²`, o corredor de referência custa `11,1 ms` com **uma** lâmpada e `34,1 ms` com **quatro**,
/// contra um orçamento de quadro de `16,7 ms` ⇒ *ele atravessa o orçamento à SEGUNDA lâmpada, e o
/// rig permite quatro*. Como a [`relight_stale`] corre **por quadro** enquanto o artista arrasta a
/// lâmpada, o passe não é aceleração: é a condição de a promessa daquela função ser verdade.
///
/// ⚠️ **A régua continua a existir e chama-se [`pixels_pela_forma_na_cpu`]** — a mesma lei, na
/// mesma crate, com o mesmo material e as mesmas lâmpadas.
///
/// ⚠️ **A cauda é a MESMA do passe da tinta** (`copy_texture_into_individual`): uma segunda maneira
/// de pôr pixels no slot do sprite divergiria no dia em que a primeira mudasse. ⭐ E aqui ela é
/// ainda mais barata — a saída do passe **já é** uma textura, logo não há upload nenhum entre
/// acender e copiar (o caminho da CPU pagava um `upload_rgba_copiavel` da tela inteira).
fn acende_pela_forma(
    gpu: &GpuContext,
    renderer: &mut SpriteRenderer,
    passes: &mut PassesDaLuz,
    rig: &LightRig,
    bake: &BakedForm,
) -> Result<(), String> {
    let (w, h) = bake.size;
    let lampadas = lampadas_do_rig(rig)?;
    let planos = planos_de(bake);
    let passe = passes
        .forma
        .get_or_insert_with(|| passe_da_forma::PasseDaForma::new(gpu));
    let out = passe.acende(
        gpu,
        passe_da_forma::LuzDaCena {
            material: &crate::lei_da_luz::material_da_forma(),
            lampadas: &lampadas,
            ceu: ceu_do_rig(&lampadas),
            olhar: crate::lei_da_luz::OLHAR_DA_FORMA,
        },
        &planos,
    )?;
    renderer
        .copy_texture_into_individual(bake.texture_id, out, w, h)
        .map_err(|e| format!("nao consegui copiar para o slot do sprite: {e}"))
}

/// ⭐⭐ **A MESMA LEI, na CPU** — o caminho de REFERÊNCIA, que devolve os pixels em vez de os pintar.
///
/// ⚠️ **Ela não é o produto e não é um caminho alternativo:** é a régua. A paridade do passe mede-se
/// contra ESTES bytes (`ph2d_form_pbr::imagem` é a única redacção da lei em Rust), e uma segunda
/// montagem escrita dentro de um gate continuaria a passar depois de a do produto ficar torta.
///
/// ⚠️ **Ela partilha com o produto as TRÊS decisões que não são a lei** — as lâmpadas
/// ([`lampadas_do_rig`]), os planos ([`planos_de`]) e o material
/// ([`crate::lei_da_luz::material_da_forma`]) —, e é isso que torna a comparação honesta: o que fica
/// a variar entre os dois lados é **só onde a aritmética corre**.
///
/// # Errors
/// Se o rig estiver todo apagado, ou se algum plano não medir o que o `size` pede.
pub fn pixels_pela_forma_na_cpu(bake: &BakedForm, rig: &LightRig) -> Result<Vec<u8>, String> {
    ph2d_form_pbr::imagem::acende_imagem(
        &crate::lei_da_luz::material_da_forma(),
        &planos_de(bake),
        &lampadas_do_rig(rig)?,
        ceu_do_rig(&lampadas_do_rig(rig)?),
        crate::lei_da_luz::OLHAR_DA_FORMA,
    )
}

/// ⭐⭐⭐ **O CÉU DESTA LEI SAI DO RIG, e a razão de ele não ser uma constante é a MEDIÇÃO.**
///
/// # ⛔⛔ A redacção anterior era `[0,0,0]` e a razão escrita ao lado estava ERRADA
///
/// Ela dizia: *«o rig desta casa é `KEY + 3 × FILL`, ou seja as lâmpadas de preenchimento são o
/// ambiente dele»*. **Medido:** as três de preenchimento nascem `on: false`
/// ([`ph2d_light::Light::FILL`]), logo na configuração de fábrica há **UMA** lâmpada acesa e mais
/// nada — e com o ambiente a zero **`25,03 %` da peça saía PRETA ao bit**, com a sombra a ler
/// luminância média `0,000`. A lei da casa nomeia este defeito **antes de ele acontecer**, no doc do
/// [`ph2d_light::AMBIENT`]: *«os dois consumidores (tinta e forma) têm de dobrar a razão do MESMO
/// jeito, senão a mesma lâmpada deixaria a escultura mais escura na sombra que a pintura ao lado
/// dela»*.
///
/// # ⛔⛔⛔ E a 2.ª redacção era um ERRO DE CATEGORIA — `AMBIENT` não é uma radiância
///
/// Ela punha `env_ambient` como irradiância absoluta. Mas o [`ph2d_light::AMBIENT`] declara-se, à
/// letra, como *«o que uma face totalmente virada PARA LONGE da luz ainda devolve»* — ou seja uma
/// **fracção da resposta PLANA**, não uma quantidade de luz. Usada como absoluta, ela saturava a
/// peça: com a exposição calibrada sem céu, a média do miolo cinzento saltou de `186` para **`255`**
/// (a tabela inteira está no `diag_a_escada_do_olhar_com_ceu`).
///
/// # ⭐ A tradução certa é DERIVADA, e não tem uma constante escolhida
///
/// A resposta plana que o rig entrega a uma difusa é `Σ max(l·z, 0) · tint` (a irradiância das
/// lâmpadas), e a normalizada é essa sobre `π` — a mesma unidade que o [`ph2d_form_pbr::Ceu`] pede.
/// O piso do modelo RELATIVO entra como termo ADITIVO por `f = A/(1 − A)`, que é a **conversão em
/// forma fechada** entre *«a sombra é `A` do plano»* (uma interpolação) e *«o ambiente SOMA-SE ao
/// directo»* (a nossa lei): com ela, `sombra/plano` volta a valer exactamente `A`, e há gate.
///
/// ⭐⭐ **E isto responde à objecção que a 1.ª redacção levantava:** *«o artista veria a peça a não
/// escurecer por mais que apagasse lâmpadas»*. Com o céu derivado do rig, apagar as lâmpadas apaga
/// o céu — o estúdio é o rig, e não um mundo por trás dele. É a mesma escolha que o barro vivo e a
/// tinta já fazem, e é por isso que ela não é *«menos física»*: aqui o ambiente **é** um instrumento
/// de estúdio.
///
/// ⚠️ A CHROMA multiplica a do rig pela do céu, como no barro vivo: o [`ph2d_light::ENV_BASE`] tem
/// luminância exactamente `1`, logo ele **redistribui** e não expõe — a média sobre todas as normais
/// continua a ser o piso.
#[must_use]
pub fn ceu_do_rig(lampadas: &[ph2d_form_pbr::Lampada]) -> ph2d_form_pbr::Ceu {
    // ⭐⭐ **A conversão vive na [`ph2d_light::env_ramp`] desde 2026-09-21, e não aqui.** Ela estava
    // escrita neste corpo — o `f = A/(1 − A)`, o `/π` e as duas constantes do estúdio — e ganhou um
    // SEGUNDO consumidor no dia em que o barro vivo passou a acender pela mesma lei. *O que dois
    // consumidores têm de responder igual mora onde os dois alcançam*, que é a frase que a
    // `ph2d-light` já escreve sobre o realce do barro.
    //
    // ⚠️ **O `plano` continua a ser calculado aqui e isso é declarado:** a porta irmã
    // ([`ph2d_light::flat_response`]) lê um [`ph2d_light::ResolvedRig`] e esta função recebe
    // [`ph2d_form_pbr::Lampada`], que é o vocabulário da ÓPTICA — as sondas desta crate constroem
    // lâmpadas à mão e nunca têm um rig resolvido. ⭐ As duas contas são a mesma e há gate a
    // afirmá-lo (`as_duas_portas_do_plano_concordam`).
    let mut plano = [0.0f32; 3];
    for l in lampadas {
        // ⚠️ A normal PLANA é a [`ph2d_form_pbr::VISTA`], logo `n·l` é a componente `z` da lâmpada.
        let ndl = l.para_a_luz[2].max(0.0);
        for (p, r) in plano.iter_mut().zip(l.radiancia) {
            *p += ndl * r;
        }
    }
    let (base, inclinacao) = ph2d_light::env_ramp(plano);
    ph2d_form_pbr::Ceu { base, inclinacao }
}

/// As lâmpadas do rig, no vocabulário que a óptica pede.
///
/// ⚠️ `dir` é **da superfície PARA a luz** e `tint` é a cor já PESADA pela intensidade — é o
/// vocabulário do rig traduzido, e é por isso que a conversão mora aqui e não na folha da lei.
///
/// # Errors
/// Rig todo apagado: **não há acendida a fazer**, e deixar os pixels como estão é a resposta honesta
/// — o sprite fica com a última luz que teve.
pub fn lampadas_do_rig(rig: &LightRig) -> Result<Vec<ph2d_form_pbr::Lampada>, String> {
    let Some(resolved) = ph2d_light::resolve(rig) else {
        return Err("todas as lampadas estao apagadas".into());
    };
    Ok(resolved
        .lamps()
        .iter()
        .map(|l| ph2d_form_pbr::Lampada {
            para_a_luz: l.dir,
            radiancia: l.tint,
        })
        .collect())
}

/// Os três canais do objecto assado, emprestados à lei.
fn planos_de(bake: &BakedForm) -> ph2d_form_pbr::imagem::Planos<'_> {
    ph2d_form_pbr::imagem::Planos {
        size: bake.size,
        base: &bake.base,
        form: &bake.form,
        form_occ: &bake.form_occ,
    }
}

/// O passe da TINTA — a lei de sempre, e o valor de fábrica. Ver [`light`].
fn acende_pela_tinta(
    gpu: &GpuContext,
    renderer: &mut SpriteRenderer,
    passes: &mut PassesDaLuz,
    rig: &LightRig,
    bake: &BakedForm,
) -> Result<(), String> {
    let (w, h) = bake.size;
    let Some(resolved) = ph2d_light::resolve(rig) else {
        // ⚠️ Rig todo apagado: **não há acendida a fazer**, e o passe recusaria um rig vazio
        // (`lamps` vazio é bug de chamador, pelo doc dele). Deixar os pixels como estão é a resposta
        // honesta — o sprite fica com a última luz que teve.
        return Err("todas as lampadas estao apagadas".into());
    };
    let lamps = resolved_lamps(&resolved);
    let (relief, cover, mat0, mat1) = neutral_planes(&bake.base);
    let planes = BakePlanes {
        relief,
        cover,
        mat0,
        mat1,
        lamps,
    };
    let src = upload_rgba(gpu, bake.size, &bake.base);
    let pass = passes
        .tinta
        .get_or_insert_with(|| ImpastoLightPass::new(gpu));
    let input = build_input(
        bake.size,
        &planes,
        &bake.form,
        &bake.form_occ,
        SpecLut::get(),
    );
    let out = pass
        .run(gpu, &src, &input)
        .map_err(|e| format!("o passe de luz recusou: {e:?}"))?;
    renderer
        .copy_texture_into_individual(bake.texture_id, out, w, h)
        .map_err(|e| format!("nao consegui copiar para o slot do sprite: {e}"))
}

/// **Estes pixels foram acesos pelo rig de agora?** A pergunta que decide a re-acendida.
pub fn needs_relight(stamped: Option<RigStamp>, now: RigStamp) -> bool {
    stamped != Some(now)
}

/// O carimbo depois de uma tentativa de acender.
///
/// ⚠️ **Um fracasso NÃO carimba**, e a consequência de errar isto é permanente: um rig todo apagado
/// marcaria os pixels como *"acesos por este rig"*, e quando o artista acendesse a lâmpada de volta
/// o objeto ficaria com a luz de antes — para sempre, sem nada dizendo por quê.
pub fn stamp_after(lit: bool, now: RigStamp, was: Option<RigStamp>) -> Option<RigStamp> {
    if lit { Some(now) } else { was }
}

/// **A RE-ACENDIDA** — passa nos objetos assados e re-acende os que a lâmpada envelheceu.
///
/// ⚠️ Ela **não re-rasteriza a malha**: a forma guardada não depende do rig, e é essa separação que
/// torna mover a lâmpada barato o bastante para ser um gesto contínuo. Roda por frame e quase sempre
/// não faz nada — com o rig parado custa um carimbo por objeto, sem tocar a GPU.
///
/// ⚠️ **O rig de cada objeto é o DELE.** Uma re-acendida que lesse um rig global re-acenderia com a
/// luz de outra coisa todo objeto que o artista carregou de um projeto salvo — e o `rig` do próprio
/// objeto é justamente o que o documento carrega para isso não acontecer.
pub fn relight_stale(
    forms: &mut std::collections::BTreeMap<u64, BakedForm>,
    gpu: &GpuContext,
    renderer: &mut SpriteRenderer,
    passes: &mut PassesDaLuz,
) {
    let stale: Vec<u64> = forms
        .iter()
        .filter(|(_, b)| needs_relight(b.lit_with, rig_stamp(&b.rig)))
        .map(|(k, _)| *k)
        .collect();
    for bits in stale {
        let Some(bake) = forms.get(&bits) else {
            continue;
        };
        let now = rig_stamp(&bake.rig);
        // O `light` empresta o mapa como imutável; a escrita do carimbo vem depois.
        let lit = light(gpu, renderer, passes, &bake.rig, bake).is_ok();
        if let Some(b) = forms.get_mut(&bits) {
            b.lit_with = stamp_after(lit, now, b.lit_with);
        }
    }
}

/// ⭐ **A codificação com que a forma viaja no arquivo** — ver o cabeçalho do irmão.
#[path = "baked_form_bytes.rs"]
mod bytes;
pub use bytes::{form_from_rgba8, form_to_rgba8, occlusion_from_r8, occlusion_to_r8};

#[cfg(test)]
#[path = "baked_form_lei_tests.rs"]
mod lei_tests;

#[cfg(test)]
#[path = "prova_da_placa.rs"]
mod prova_da_placa;

/// ⭐ **BLOCO D da §5.0 do catavento** — o preço de ACENDER por quadro; os blocos A–C vivem na
/// `ph2d-mesh-render` e mediam só o RASTERIZAR. Ver `docs/Render3d/17_a_rota_b_o_catavento.md`.
#[cfg(test)]
#[path = "mede_o_acender_por_quadro.rs"]
mod mede_o_acender_por_quadro;

/// ⭐⭐⭐ **Os gates da COSTURA RESIDENTE** — a porta da rota B, e as duas afirmações que a tornam
/// mais do que uma promessa. Ver `docs/Render3d/17_a_rota_b_o_catavento.md`.
#[cfg(test)]
#[path = "costura_residente_tests.rs"]
mod costura_residente_tests;

#[cfg(test)]
#[path = "baked_form_tests.rs"]
mod tests;

/// **O QUE UM OBJETO ASSADO CUSTA NO ARQUIVO** — a sonda que decidiu a representação.
///
/// ```text
/// cargo test -p ph2d-host-desktop --release --bins baked_form::probe -- --ignored --nocapture
/// ```
#[cfg(test)]
mod probe {
    use super::*;

    /// Imprime o tamanho de um documento nos tamanhos que o produto usa.
    ///
    /// ⚠️ Ela mede o **postcard do documento inteiro**, e não a aritmética dos planos: é o número
    /// que o artista vê no `[proj] salvo:`, e o que separa os dois é tudo o que a serialização
    /// acrescenta. Uma sonda que somasse `w × h × 4` estaria medindo a minha conta, não o arquivo.
    #[test]
    #[ignore = "sonda: mede, nao afirma"]
    fn measure_what_a_baked_object_costs_on_disk() {
        println!("lado | base f32 (MiB) | base RGBA8 | forma RGBA8 | doc postcard (MiB)");
        for side in [512u32, 1024, 2048] {
            let n = (side * side) as usize;
            let form: Vec<f32> = (0..n * 4).map(|i| ((i % 255) as f32) / 255.0).collect();
            let base = vec![200u8; n * 4];
            let mib = |b: usize| b as f64 / (1024.0 * 1024.0);
            // O que teria custado guardar a forma como `f32`, que é como ela vive na memória.
            let as_f32 = form.len() * 4;
            let doc_bytes = postcard::to_allocvec(&(
                7u32,
                side,
                side,
                &base,
                form_to_rgba8(&form),
                LightRig::default(),
            ))
            .expect("serializa");
            println!(
                "{side:>4} | {:>14.2} | {:>10.2} | {:>11.2} | {:>18.2}",
                mib(as_f32),
                mib(base.len()),
                mib(form.len()),
                mib(doc_bytes.len())
            );
        }
    }
}
