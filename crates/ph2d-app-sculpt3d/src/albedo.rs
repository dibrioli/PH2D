//! ⭐⭐⭐⭐ **A MATÉRIA** — de que cor é o barro, e porque é que a resposta tem de ser **UMA**.
//!
//! Irmão da [`super::bake`] pelo corte de sempre (responsabilidade): lá mora *o GESTO de assar*,
//! aqui mora *de que é feita a superfície que ele vai acender* — e a razão de este módulo existir
//! é que **a mesma pergunta tem dois leitores**.
//!
//! ## O report que o obrigou a existir
//!
//! Três vezes seguidas, cada uma depois de uma correcção enviada, o dono devolveu a mesma coisa:
//! *«o bake não é idêntico ao que se vê em 3d»* · *«a malha 3d parece ter mais luz indireta que a
//! imagem do Bake. Mas precisa ser idêntica»* · *«nada ainda»*. As duas primeiras causas eram de
//! LUZ (o visor abria num matcap; o bake corria a lei da tinta) e foram fechadas. A terceira **não
//! é luz nenhuma**: o visor pintava um barro cravado no shader e o bake pintava **os pixels da
//! sprite**.
//!
//! **Medido em 2026-09-21**, nos enquadramentos do produto e com a mesma forma e a mesma luz:
//!
//! | o que difere | desvio médio (códigos de 8 bits) |
//! |---|---|
//! | lei + enquadramento + oclusão de tela | `0,52` |
//! | **só o albedo** | **`31,68`** |
//!
//! ⇒ `61×`. *Uma diferença de MATÉRIA não se corrige com luz*, e nenhuma das duas waves anteriores
//! podia tê-la fechado.
//!
//! ## A lei, e porque ela não pode ser escrita duas vezes
//!
//! ⛔⛔ **Uma sprite JÁ ASSADA não devolve a matéria dela pela porta de leitura.** Depois do
//! primeiro bake os pixels da sprite são `base × luz`; lê-los como fonte faria a segunda acendida
//! acender o que já está aceso, e o objecto escureceria a cada gesto. É por isso que o
//! [`ph2d_form_donation::baked_form::BakedForm`] guarda o `base` — e é por isso que **quem quiser
//! saber de que é feita aquela superfície tem de perguntar primeiro à tabela dos assados, e só
//! depois ao device**.
//!
//! ⚠️ Essa é uma lei de DUAS metades numa ordem que importa, e ela tinha **um** leitor
//! (`bake::bake_one`). O visor é o segundo — e uma segunda cópia da ordem divergiria exactamente
//! no caso que ela existe para impedir. ⇒ [`materia_para`], com os dois a lê-la.

use std::collections::BTreeMap;

use ph2d_ecs::SimWorld;
use ph2d_form_donation::baked_form::BakedForm;
use ph2d_gpu::GpuContext;
use ph2d_render::SpriteRenderer;

use super::Sculpt3dScene;

/// **DE QUE É FEITA A SUPERFÍCIE DESTA SPRITE** — os pixels que o bake vai acender, em alfa
/// **DIRECTO** e no tamanho dela.
///
/// ⚠️ **A ordem é a lei** (ver o cabeçalho do módulo): a tabela dos assados primeiro, o device
/// depois. Invertê-la devolve `base × luz` a quem pediu `base`.
///
/// ⚠️ **Alfa DIREITO**, e não é cosmético: a acendida multiplica a cor pela luz, e num buffer
/// pré-multiplicado a multiplicação aconteceria sobre `cor × alfa` — o resultado escureceria pela
/// borda do recorte, que é a assinatura clássica de tratar pré-multiplicado como directo.
///
/// ⚠️ **O leitor da fonte chega por ASSINATURA e é PREGUIÇOSO**: uma sprite já assada nunca chega a
/// perguntar, e a leitura custa uma volta ao device. A porta que a faz
/// (`hero_intents::texture_edit`) é da shell.
///
/// ⭐⭐ **E ele não recebe ARGUMENTO nenhum, de propósito:** com o mundo e o renderizador na
/// assinatura esta função só era testável com um device, e o que ela tem de afirmar — *uma sprite
/// já assada NÃO volta a ler a tela* — não tem um pixel dentro. Com o fecho a capturar o que
/// precisa, os dois chamadores ficam iguais e o gate corre sempre
/// (`re_assar_nao_le_a_tela_de_volta`).
pub(crate) fn materia_para(
    forms: &BTreeMap<u64, BakedForm>,
    bits: u64,
    ler_fonte: &mut dyn FnMut() -> Option<ph2d_render::SpriteImage>,
) -> Result<(Vec<u8>, (u32, u32)), String> {
    if let Some(b) = forms.get(&bits) {
        return Ok((b.base.clone(), b.size));
    }
    let src = ler_fonte()
        .ok_or_else(|| "nao consegui ler os pixels do sprite selecionado".to_string())?;
    let straight = src.into_straight();
    let size = (straight.width, straight.height);
    if size.0 == 0 || size.1 == 0 {
        return Err("o sprite selecionado nao tem pixels".into());
    }
    Ok((straight.pixels, size))
}

/// **O QUE O QUADRO TEM DE FAZER COM A MATÉRIA** — a saída de [`decide`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Decisao {
    /// **NADA**, e é o caso de todo quadro menos aquele em que o artista escolheu outra coisa.
    Nada,
    /// O visor volta ao barro cravado no shader.
    Esquece,
    /// Ler a matéria desta sprite e subi-la ao device.
    Le(u64),
}

/// ⭐⭐⭐ **A DECISÃO, PURA** — o que muda a resposta, e o que NÃO a muda.
///
/// ⛔⛔ **Ela é uma função e não três linhas dentro da [`sincroniza`] por uma razão medida nesta
/// casa:** *quando um gate precisa de um device para medir uma decisão que não tem pixel nenhum, a
/// lei está no sítio errado*. O que esta wave tem de afirmar — **que a leitura acontece uma vez e
/// não por quadro** — não tem um pixel dentro, e escrita lá dentro ela nasceria `#[ignore]` e o CI
/// nunca a correria.
///
/// ## A chave, e o recurso que a escolheu
///
/// ⚠️ **A memória é `(sprite, já-assada?)` e o recurso é o RELÓGIO DO DEVICE:** ler os pixels de
/// uma sprite `Individual` é um `readback` — uma volta completa à placa mais uma cópia de
/// `w × h × 4` bytes. Numa sprite de `1024²` isso é `4 MiB` por leitura; a **60 Hz** seria um estol
/// por quadro para responder a uma pergunta cuja resposta só muda quando o artista **escolhe outro
/// objecto**. ⇒ lê-se na mudança, que é um gesto.
///
/// ⚠️ **A segunda metade da chave não é decoração:** assar troca a matéria (a sprite passa a ser
/// `base × luz` e a tabela passa a ter o `base`), logo sem ela o quadro a seguir ao primeiro bake
/// continuaria a mostrar a leitura antiga.
///
/// ⚠️ **Só o modo que a LÊ a pede:** o albedo entra pelo braço [`ph2d_mesh_render::Lighting::Pbr`]
/// do shader e por mais nenhum — nos outros a matéria nem é consultada, e subir uma textura para
/// um modo que não a lê seria pagar o `readback` por nada. ⭐ O par está **gateado dos dois lados**
/// (`o_albedo_so_e_lido_no_ramo_da_lei_que_assa`, na crate do shader), senão o dia em que outro
/// ramo passar a ler a matéria encontra a textura vazia — que é branco, e branco não se lê como
/// falta.
pub(crate) fn decide(
    memo: &mut Option<(u64, bool)>,
    selected: Option<u64>,
    lighting: ph2d_mesh_render::Lighting,
    ja_assada: bool,
) -> Decisao {
    let quer = match selected {
        Some(bits) if lighting == ph2d_mesh_render::Lighting::Pbr => Some((bits, ja_assada)),
        _ => None,
    };
    if quer == *memo {
        return Decisao::Nada;
    }
    *memo = quer;
    match quer {
        Some((bits, _)) => Decisao::Le(bits),
        None => Decisao::Esquece,
    }
}

/// ⭐⭐⭐⭐ **O VISOR PASSA A PINTAR A MATÉRIA QUE O BAKE VAI ACENDER.**
///
/// Corre uma vez por quadro e **não toca no device em nenhum deles**, menos naquele em que a
/// [`decide`] muda de resposta — a mesma forma da família `ensure_*` do renderizador.
///
/// ⛔ **DIVERGÊNCIA DECLARADA:** pintar a sprite em 2D com o visor aberto **não** re-lê a matéria —
/// não há, hoje, contador de revisão numa textura individual, e inventar um aqui seria a segunda
/// resposta a *«esta imagem mudou?»*. A cura do artista é escolher outro objecto e voltar; a cura
/// deste código é aquele contador, na crate que é dona da textura.
pub fn sincroniza(
    scene: &mut Sculpt3dScene,
    forms: &BTreeMap<u64, BakedForm>,
    gpu: &GpuContext,
    selected: Option<u64>,
    sim: &mut SimWorld,
    renderer: &mut SpriteRenderer,
    ler_fonte: &mut dyn FnMut(
        &mut SimWorld,
        &mut SpriteRenderer,
    ) -> Option<ph2d_render::SpriteImage>,
) {
    let assada = selected.is_some_and(|bits| forms.contains_key(&bits));
    let bits = match decide(&mut scene.albedo_de, selected, scene.lighting, assada) {
        Decisao::Nada => return,
        Decisao::Esquece => {
            scene.renderer.clear_albedo_source();
            return;
        }
        Decisao::Le(bits) => bits,
    };
    // ⚠️ **Uma leitura que falha é MEMOIZADA na mesma** (a [`decide`] já escreveu a chave): o que a
    // faz falhar é a sprite não ter fonte legível, que é um facto estável dela — e re-tentar por
    // quadro poria o `readback` no laço que esta porta existe para evitar.
    match materia_para(forms, bits, &mut || ler_fonte(sim, renderer)) {
        Ok((px, size)) => scene
            .renderer
            .set_albedo_source(&gpu.device, &gpu.queue, &px, size),
        Err(_) => scene.renderer.clear_albedo_source(),
    }
}

#[cfg(test)]
#[path = "albedo_tests.rs"]
mod albedo_tests;
