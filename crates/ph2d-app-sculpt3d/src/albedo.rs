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
    let src = ler_fonte().ok_or_else(|| {
        ph2d_i18n::tr("app.sculpt3d.albedo.nao_consegui_ler_os_pixels").to_string()
    })?;
    let straight = src.into_straight();
    let size = (straight.width, straight.height);
    if size.0 == 0 || size.1 == 0 {
        return Err(ph2d_i18n::tr("app.sculpt3d.albedo.sem_pixels").to_string());
    }
    Ok((straight.pixels, size))
}

/// ⭐⭐⭐⭐ **UM SPRITE VAZIO VESTE A SILHUETA DA FORMA** — a lei que o report de 21/09 pediu, e
/// que substitui a RECUSA que a 1.ª leitura dele produziu.
///
/// ## O que o dono disse, e o que eu li ao contrário
///
/// Ele escreveu *«em sprite transparente o bake fica invisível»* e eu respondi com uma **recusa**.
/// A foto seguinte — *«o objeto continua sem assar»*, com o aviso na tela — é o mesmo pedido pela
/// segunda vez: ⛔ *ele não queria ser impedido; ele queria que funcionasse.*
///
/// ## Porque é que ele ficava invisível, e porque é que a cura é a FORMA
///
/// O passe escreve `vec4(cor_acesa, px.a)` — *o alfa atravessa intacto, ele é a silhueta do
/// sprite*. Sobre uma tela vazia isso é `a = 0` em toda parte, e **nenhuma lâmpada a traz de
/// volta**: o objecto fica assado, virável, aceso — e invisível.
///
/// ⭐ A silhueta que falta **já está rasterizada ao lado**: o G-buffer é `[nx, ny, nz, COBERTURA]`
/// por texel ([`ph2d_mesh_render::FormPlanes::normal`]), e a cobertura é exactamente *«onde a peça
/// está»*. ⇒ onde o sprite não tem nada e a peça tem, a matéria passa a ser a **NEUTRA**.
///
/// ⚠️ **E o branco não é escolha minha:** é o que as cenas de bake desta casa já põem na mesa
/// (`donation::canvas_wanted` pede `bg: 2`), com a razão escrita no gate do gesto — *«a luz da
/// forma MULTIPLICA, então sobre branco o que se vê é ela e mais nada»*. Branco é o **neutro
/// multiplicativo**, e usá-lo aqui é aplicar a escolha que o produto já fez.
///
/// ## ⛔⛔ A CERCA é «o sprite inteiro está vazio», e sem ela isto seria uma regressão grave
///
/// Um personagem **recortado** sobre transparente é o caso normal deste app. Se a lei valesse
/// texel a texel, a peça pintaria branco em toda a zona recortada — *o recorte deixaria de ser
/// recorte*. ⇒ ela só arma quando **nenhum** texel tem alfa, que é a condição que o report
/// descreve, e para toda sprite com arte a saída é **byte-idêntica** (o `false` sai antes de
/// escrever um byte).
///
/// ## ⚠️ O alfa é a COBERTURA e não `255`
///
/// A cobertura da borda é fraccionária (é ela que dá o anti-serrilhado da rasterização). Escrever
/// `255` onde `cobertura > 0` devolveria a peça com a borda **serrilhada**, e o canal já tem a
/// resposta suave — custa o mesmo.
///
/// Devolve **quantos texels** vestiu, `0` quando não armou: é o que o toast diz ao artista, e é o
/// que um gate mede sem ter de olhar para pixels.
pub(crate) fn veste_a_forma(base: &mut [u8], forma: Cobertura<'_>) -> usize {
    if base.as_chunks::<4>().0.iter().any(|p| p[3] > 0) {
        return 0;
    }
    let mut vestidos = 0usize;
    for (i, px) in base.as_chunks_mut::<4>().0.iter_mut().enumerate() {
        let cobertura = forma.em(i);
        if cobertura > 0.0 {
            // LITERAL-COLOR-OK: o NEUTRO multiplicativo da luz da forma, com o alfa da cobertura
            *px = [255, 255, 255, (cobertura * 255.0).round() as u8];
            vestidos += 1;
        }
    }
    vestidos
}

/// ⭐⭐⭐ **DE ONDE VEM A SILHUETA que uma matéria vazia veste** — as duas respostas desta casa.
///
/// ⛔⛔ **A segunda variante nasceu de um report do dono** (21/09, *«quando retiro a sprite branca
/// e coloco um transparente o objecto 3D fica PRETO»*): a [`sincroniza`] usa a MESMA
/// [`materia_para`] para pintar o barro com o albedo que o bake vai acender, e uma matéria
/// `[0,0,0,0]` subida ao device é **preto opaco** no visor.
///
/// ⚠️ **E a minha recusa de ontem escondia isto por ACIDENTE:** com o `Err` a matéria vazia nunca
/// chegava ao `set_albedo_source` e o barro ficava com o de sempre. *Retirar uma recusa devolve
/// todos os caminhos que ela calava, não só o que a motivou.*
///
/// ⚠️ A promessa deste módulo é *o visor pinta a matéria que o bake vai ACENDER* — logo, se o bake
/// vai vestir a peça de branco, o visor mostra branco. No visor **a peça é o barro inteiro**, e é
/// por isso que ali a cobertura é [`Self::Toda`].
#[derive(Clone, Copy)]
pub(crate) enum Cobertura<'a> {
    /// O canal `w` do G-buffer — `[nx, ny, nz, COBERTURA]` por texel. É a do BAKE.
    DaForma(&'a [f32]),
    /// A peça cobre tudo. É a do VISOR, onde o sujeito é o barro inteiro.
    Toda,
}

impl Cobertura<'_> {
    /// Quanto a peça cobre o texel `i` — `0.0` fora dela.
    fn em(self, i: usize) -> f32 {
        match self {
            // ⚠️ **Fora do plano é ZERO e não um `clamp`**: um `base` maior que o G-buffer é um
            // defeito de tamanho, e vesti-lo com a última cobertura esconderia-o.
            Self::DaForma(f) => f
                .as_chunks::<4>()
                .0
                .get(i)
                .map_or(0.0, |n| n[3].clamp(0.0, 1.0)),
            Self::Toda => 1.0,
        }
    }
}

/// **O QUE O QUADRO TEM DE FAZER COM A MATÉRIA** — a saída de [`decide`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Decisao {
    /// **NADA**, e é o caso de todo quadro menos aquele em que o artista escolheu outra SPRITE.
    Nada,
    /// O visor volta ao barro cravado no shader. ⚠️ **Só ao SAIR da lei que lê a matéria** —
    /// largar a selecção **não** chega aqui desde 2026-09-21; ver o [`decide`].
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
/// # ⛔⛔⛔ A MATÉRIA É DA PEÇA E NÃO DA SELECÇÃO — e a redacção de ontem era o contrário
///
/// **Report do dono (2026-09-21):** *«Se seleciono a imagem, o objeto 3d fica com a aparência exata
/// do Bake. Mas se seleciono o objeto 3d, ele muda a aparência (fica mais brilhante).»*
///
/// A 1.ª redacção devolvia [`Decisao::Esquece`] assim que a selecção largasse a sprite — e **a
/// selecção larga a sprite exactamente quando o artista pega na PEÇA, que é o que ele tem de fazer
/// para esculpir**. ⇒ *a pré-visualização ficava errada precisamente durante a actividade para a
/// qual ela existe.*
///
/// ⭐⭐ **E o que se vê tem número** (sonda `diag_o_que_a_seleccao_faz_a_materia`, no ecrã, contra
/// a sprite assada como régua):
///
/// ```text
///   a IMAGEM escolhida (materia)      media  195,99   pior-vs-sprite    1
///   o OBJECTO 3D escolhido (CLAY)     media  169,66   pior-vs-sprite   42
/// ```
///
/// ⚠️ **E o barro é mais ESCURO, não mais claro** — *«brilhante»* ali quer dizer **lustroso**, e a
/// causa é a lei que a [`ph2d_form_pbr::acende_texel`] já declara: no OpenPBR **só o lóbulo difuso
/// escala com o `base_color`, o especular não escala nada**. Uma base mais escura deixa o MESMO
/// realçe a sobressair muito mais ⇒ plástico.
///
/// ⇒ largar a selecção devolve **[`Decisao::Nada`]**: a matéria fica onde estava, e só outra sprite
/// a troca. ⛔ **O `Esquece` sobra para uma coisa só** — sair da lei que lê a matéria.
///
/// ⚠️ **O que isto NÃO cura, e está declarado:** o BAKE continua a pedir uma sprite escolhida e
/// **recusa em voz alta** sem ela (*«selecione um SPRITE antes»*). Logo, com a peça na mão, o visor
/// mostra a matéria que o bake acenderia e o `Shift+B` recusa — *uma pré-visualização um gesto
/// adiantada, nunca uma errada*. Tornar o bake **peganhento** também é decisão do dono: ela troca
/// uma recusa por uma acção, e isso não se faz em silêncio.
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
    // ⛔ Fora da lei que lê a matéria, esquecer é honesto — e **uma vez só**, senão o visor
    // limparia a fonte em todo quadro.
    if lighting != ph2d_mesh_render::Lighting::Pbr {
        return if memo.take().is_some() {
            Decisao::Esquece
        } else {
            Decisao::Nada
        };
    }
    // ⭐ **Largar a selecção NÃO larga a matéria** — ver o cabeçalho: ela é da PEÇA, e a selecção
    // larga a sprite exactamente quando o artista pega na peça para esculpir.
    let Some(bits) = selected else {
        return Decisao::Nada;
    };
    let quer = Some((bits, ja_assada));
    if quer == *memo {
        return Decisao::Nada;
    }
    *memo = quer;
    Decisao::Le(bits)
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
    // ⛔⛔ **Uma leitura que FALHA deixa a matéria onde estava** — e isto é a outra metade do
    // report de 2026-09-21: escolher o objecto 3D é uma selecção que **não tem pixels**, logo ela
    // cai aqui. Limpar era devolver o `CLAY` pela porta das traseiras, com o mesmo sintoma.
    if let Ok((mut px, size)) = materia_para(forms, bits, &mut || ler_fonte(sim, renderer)) {
        // ⭐⭐⭐⭐ **UMA MATÉRIA VAZIA MOSTRA-SE BRANCA E NÃO PRETA** (report do dono, 21/09:
        // *«quando retiro a sprite branca e coloco um transparente o objecto 3D fica preto»*).
        //
        // ⚠️ `[0, 0, 0, 0]` subido ao device **é preto opaco no visor** — o barro passa a ser
        // pintado com a cor nula do albedo. ⭐ E a resposta certa não é «não pintar»: a promessa
        // deste módulo é *o visor pinta a matéria que o bake vai ACENDER*, e o bake veste uma
        // matéria vazia de branco (ver [`veste_a_forma`]) ⇒ mostrar branco é mostrar a verdade.
        //
        // ⚠️ **Aqui a cobertura é [`Cobertura::Toda`]** e não a do G-buffer: o sujeito é o barro
        // inteiro, e o visor não rasteriza forma nenhuma — ele corre por quadro.
        veste_a_forma(&mut px, Cobertura::Toda);
        scene
            .renderer
            .set_albedo_source(&gpu.device, &gpu.queue, &px, size);
    }
}

#[cfg(test)]
#[path = "albedo_tests.rs"]
mod albedo_tests;
