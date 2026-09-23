//! ⭐⭐⭐ **A TABELA DE MATERIAIS DA PEÇA** — o que cada folha é à luz, e como um pixel a encontra.
//!
//! # As duas metades, e porque têm ritmos diferentes
//!
//! | metade | do que depende | quando se refaz |
//! |---|---|---|
//! | [`Table::owners`] | a **GEOMETRIA** (uma fita compilada por folha) | quando o documento muda |
//! | [`Table::surfaces`] | os **NÚMEROS** do material | quando um deles muda |
//!
//! ⚠️ **Separá-las não é arrumação, é preço:** compilar a fita de uma folha é um **JIT**, e arrastar
//! um slider de cor não muda geometria nenhuma. Uma tabela só, refeita sempre que qualquer das duas
//! mudasse, pagaria um JIT por folha a cada quadro de um arrasto de cor.
//!
//! ⚠️ **A ORDEM é a mesma nas duas**, e é a de [`ph2d_field_ecs::walk`] filtrada a folhas — a mesma
//! que o [`crate::pick`] usa. ⛔ Duas ordens pintariam cada peça com a cor da vizinha, sem erro
//! nenhum: *um índice válido nunca parece errado.*

use ph2d_field::{FieldDoc, Node, NodeId, NodeKind, NodeShape};
use ph2d_field_ecs::{FieldMaterial, FieldNode};

/// **O que o sombreamento precisa de saber sobre os materiais de uma peça.**
pub struct Table {
    /// De quem é cada ponto. `None` numa peça de uma folha só — ver [`Table::surfaces_for`].
    pub owners: Option<ph2d_field_eval::owners::Owners>,
    /// Um material por folha, na ordem das folhas. **Nunca vazio.**
    pub surfaces: Vec<ph2d_material::Surface>,
    /// Os números de que as [`Table::surfaces`] foram feitas — a chave que diz se elas envelheceram.
    pub authored: Vec<FieldMaterial>,
}

/// ⭐ **O OpenPBR de um material autorado** — a tradução, num sítio só.
///
/// ⚠️ **Os campos que não se autoram ficam no padrão da nodedef**: um material com **dez** das
/// **quinze** entradas não é um OpenPBR diferente, é o mesmo com dez entradas escolhidas. *Inventar
/// valores para as outras cinco seria escrever um material que ninguém pediu.*
///
/// ⚠️⚠️ **As contagens desta página CONTAM-SE daqui, nunca de cabeça** — as cinco que faltam são
/// `base_weight`, `base_diffuse_roughness`, `specular_weight`, `specular_color` e `specular_ior`, e
/// esta linha já esteve errada duas vezes no mesmo dia (*«sete»*, *«doze»*) por ser somada de
/// memória enquanto a lista crescia.
///
/// ⭐⭐ **A EMISSÃO entrou em 2026-09-14** (`docs/Render3d/05` §20), e a razão é o inverso da regra
/// acima: ela **já era paga** — a [`ph2d_material::Surface::emission`] corria por amostra e somava
/// `[0,0,0]`, `4,6 %` do relógio de sombreamento — e nenhum controlo lhe chegava. *Uma capacidade
/// viva sem botão nenhum é o defeito de que o §5.1 do `CLAUDE.md` fala, não uma poupança.*
/// ⭐ **O interruptor de DIAGNÓSTICO da matiz que segue a profundidade** — `PH2D_SSS_DEPTH_HUE`.
///
/// ⚠️⚠️ **Isto NÃO é o botão do produto, e a diferença é deliberada.** O botão do produto é um
/// campo da [`ph2d_field_ecs::FieldMaterial`] com uma fileira no painel, e custa um degrau de
/// `PROJECT_SCHEMA` — ele nasce no dia em que o dono decidir que a lei ship. Até lá o dono precisa
/// de **VER** a diferença para poder decidir, e *uma lei que ele não consegue olhar não é uma lei
/// que ele possa aprovar* (CLAUDE.md §0.8: o smoke é onde ele aprende).
///
/// ⛔ **Ele é lido UMA vez e vale para a sessão inteira** — não é estado de cena, não entra no
/// ficheiro, não entra no desfazer. E **ausente ⇒ `0,0`**, logo o caminho de omissão é
/// byte-idêntico e as paridades ficam de pé.
///
/// ```text
/// cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-3DModeling
/// env PH2D_FIELD_SMOKE=33 PH2D_FIELD_GPU=0 PH2D_SSS_DEPTH_HUE=1 \
///   cargo run -p ph2d-host-desktop --profile smoke
/// ```
fn depth_hue_de_diagnostico() -> f32 {
    use std::sync::OnceLock;
    static V: OnceLock<f32> = OnceLock::new();
    *V.get_or_init(|| depth_hue_de(std::env::var("PH2D_SSS_DEPTH_HUE").ok().as_deref()))
}

/// A leitura, **separada da variável de ambiente** para poder ser afirmada.
///
/// ⛔ Sem esta separação o valor viveria dentro de um `OnceLock` que a primeira chamada do processo
/// congela — e *um gate que não consegue escolher a entrada não afirma nada sobre a saída*.
///
/// ⚠️ **Lixo lê-se como `0`, e não como «ligado»:** o valor de omissão de um diagnóstico tem de ser
/// o caminho byte-idêntico, senão um erro de escrita muda o produto em silêncio.
fn depth_hue_de(v: Option<&str>) -> f32 {
    v.and_then(|s| s.trim().parse::<f32>().ok())
        .unwrap_or(0.0)
        .clamp(0.0, 1.0)
}

#[cfg(test)]
mod diagnostico_da_matiz {
    /// ⭐ A leitura: ausente e lixo caem no caminho byte-idêntico; o resto é limitado à faixa da lei.
    #[test]
    fn a_leitura_do_interruptor_cai_no_neutro_quando_nao_e_um_numero() {
        assert!(
            (super::depth_hue_de(None) - 0.0).abs() < f32::EPSILON,
            "ausente"
        );
        assert!(
            (super::depth_hue_de(Some("lixo")) - 0.0).abs() < f32::EPSILON,
            "lixo"
        );
        assert!(
            (super::depth_hue_de(Some("")) - 0.0).abs() < f32::EPSILON,
            "vazio"
        );
        assert!(
            (super::depth_hue_de(Some(" 1 ")) - 1.0).abs() < f32::EPSILON,
            "com espaços"
        );
        assert!(
            (super::depth_hue_de(Some("0.5")) - 0.5).abs() < f32::EPSILON,
            "fracção"
        );
        assert!(
            (super::depth_hue_de(Some("5")) - 1.0).abs() < f32::EPSILON,
            "acima da faixa"
        );
        assert!(
            (super::depth_hue_de(Some("-3")) - 0.0).abs() < f32::EPSILON,
            "abaixo da faixa"
        );
    }

    /// ⭐⭐⭐ **A metade que este repo cobra sempre: o valor CHEGA ao consumidor.**
    ///
    /// ⛔ A régua acima prova que a leitura está certa e é **cega** a se alguém a liga ao material —
    /// um interruptor lido e deitado fora lê-se exactamente como um interruptor ligado. Esta lê o
    /// próprio ficheiro: se a linha da fiação sair, ela reprova com o endereço.
    ///
    /// ⛔⛔⛔ **A 1.ª redacção SOBREVIVEU à mutação, e a causa vale para todo gate por
    /// `include_str!` deste repo:** ela escrevia a agulha à letra, e o ficheiro que ela lê é **o
    /// próprio** ⇒ *a asserção encontrava-se a si mesma* e ficava verde com a fiação cortada. A
    /// agulha passa a ser **montada em pedaços** (`concat!` corre antes, mas o ficheiro só contém os
    /// pedaços) — e o controlo disto é a mutação, que agora sangra.
    #[test]
    fn o_interruptor_chega_ao_material() {
        let fonte = include_str!("materials.rs");
        let agulha = concat!("subsurface_depth_hue: depth_hue", "_de_diagnostico()");
        assert!(
            fonte.contains(agulha),
            "a fiação do interruptor para o material saiu do `surface_of`"
        );
        let nome = concat!("PH2D_SSS_", "DEPTH_HUE");
        assert!(
            fonte.contains(nome),
            "o nome da variável de diagnóstico saiu — o smoke do dono deixa de ter interruptor"
        );
    }
}

#[must_use]
pub fn surface_of(m: FieldMaterial) -> ph2d_material::Surface {
    ph2d_material::OpenPbr {
        base_weight: m.base_weight,
        base_color: m.base_color,
        base_diffuse_roughness: m.base_diffuse_roughness,
        base_metalness: m.metalness,
        specular_weight: m.specular_weight,
        specular_color: m.specular_color,
        specular_roughness: m.roughness,
        specular_ior: m.specular_ior,
        coat_weight: m.coat,
        coat_color: m.coat_color,
        coat_roughness: m.coat_roughness,
        coat_ior: m.coat_ior,
        coat_darkening: m.coat_darkening,
        emission_luminance: m.emission,
        emission_color: m.emission_color,
        subsurface_weight: m.subsurface_weight,
        subsurface_color: m.subsurface_color,
        subsurface_radius: m.subsurface_radius,
        subsurface_radius_scale: m.subsurface_radius_scale,
        subsurface_scatter_anisotropy: m.subsurface_scatter_anisotropy,
        // ⛔⛔ **A MATIZ QUE SEGUE A PROFUNDIDADE ainda NÃO TEM DONO NA CENA, e o zero é literal.**
        //
        // A lei existe e está calibrada ([`ph2d_material::OpenPbr::subsurface_depth_hue`]), e o que
        // falta é a DECISÃO: ligá-la muda toda peça translúcida de toda cena e move a paridade
        // contra o renderizador de referência (uma divergência declarada, `docs/Render3d/10` §18).
        // ⇒ enquanto essa decisão não for tomada, o caminho da cena é **byte-idêntico**.
        //
        // ⚠️ E quando for tomada, o campo entra na [`ph2d_field_ecs::FieldMaterial`] — que É
        // serializada — logo custa um degrau de `PROJECT_SCHEMA`, ao contrário deste, que não custa
        // nenhum. *É por isso que a lei nasce onde nasceu.*
        subsurface_depth_hue: depth_hue_de_diagnostico(),
        // ⚠️ O booleano viaja como número porque a tabela do painel é de `f32` — ver
        // [`ph2d_field_ecs::FieldMaterial::thin_walled`].
        geometry_thin_walled: m.thin_walled > 0.5,
    }
    .prepare()
}

/// ⭐⭐⭐ **A TRAVESSIA sRGB → LINEAR DE UMA COR AUTORADA, e o seu par** — num sítio só.
///
/// # ⚠️ Por que é uma porta, e não duas linhas onde cada uma é precisa
///
/// O documento guarda as cores em **linear** (é isso que o OpenPBR integra) e o selector de cor da
/// casa fala **sRGB8** (é isso que um humano escolhe). A conversão é precisa em **dois** sítios
/// distantes — a construção da linha do painel ([`crate::scene_panel::param_rows`]) e o dreno do
/// pedido ([`crate::scene_intents`]) —, e escrita duas vezes seriam duas curvas: o dia em que uma
/// ganhasse um `clamp` ou um arredondamento diferente, o ida-e-volta deixaria de fechar e a cor
/// **derivaria a cada abertura do selector**, um passo de undo de cada vez.
///
/// ⚠️ **E o ida-e-volta TEM de fechar ao bit**, porque o painel pergunta *«mudou?»* comparando
/// bytes: uma cor que não voltasse ao mesmo byte pediria uma edição **por quadro** enquanto o
/// selector estivesse aberto. É isso que o gate
/// [`the_round_trip_through_the_document_is_exact`](crate::materials::colour_row_tests::the_round_trip_through_the_document_is_exact)
/// mede, sobre os 256 bytes.
///
/// ⛔ **A curva não é local:** ela é a do [`ph2d_color::srgb`], que é a mesma que o resto do app usa.
///
/// ⚠️⚠️ **O nome deixou de dizer «base» em 14/09**, e isso é a lei e não arrumação: com o brilho
/// próprio (§20) há **duas** cores autoradas a atravessar aqui, e uma porta chamada `base_color_*`
/// convida a segunda a escrever a conversão outra vez ao lado. *Uma lei escrita em dois sítios ainda
/// não é uma lei — só uma PORTA é.*
#[must_use]
pub fn colour_srgb8(linear: [f32; 3]) -> [u8; 3] {
    linear.map(ph2d_color::srgb::linear_to_srgb_byte)
}

/// O outro sentido de [`colour_srgb8`] — o que o artista apontou, no espaço do documento.
#[must_use]
pub fn colour_from_srgb8(srgb: [u8; 3]) -> [f32; 3] {
    srgb.map(ph2d_color::srgb::srgb_to_linear_byte)
}

/// As folhas de uma peça, na ordem canónica: o par `(material autorado, documento de um nó posto no
/// mundo)`.
///
/// ⚠️ **A pose é a de MUNDO**, e não a local: quem avalia a folha avalia-a onde ela está. É a mesma
/// lei (e o mesmo erro evitado) do [`crate::pick::owners_under`].
fn leaves(
    world: &bevy_ecs::world::World,
    root: bevy_ecs::entity::Entity,
) -> (Vec<FieldMaterial>, Vec<FieldDoc>) {
    ph2d_field_ecs::walk(world, root)
        .into_iter()
        .filter_map(|(e, _)| {
            let FieldNode {
                shape: NodeShape::Leaf(prim),
            } = world.get::<FieldNode>(e)?
            else {
                return None;
            };
            let placed = FieldDoc::new(
                vec![Node {
                    xform: ph2d_field_ecs::world_xform(world, e),
                    kind: NodeKind::Leaf(prim.clone()),
                    mods: Vec::new(),
                    verb: None,
                }],
                NodeId(0),
            )
            .ok()?;
            // ⚠️ **A ausência do componente é o material de OMISSÃO** — ver [`FieldMaterial`].
            Some((
                world.get::<FieldMaterial>(e).copied().unwrap_or_default(),
                placed,
            ))
        })
        .unzip()
}

impl Table {
    /// ⭐⭐⭐ **Constrói a tabela de uma peça**, com a geometria compilada.
    ///
    /// ⚠️ **`half_extent` e `side_px` só entram para a MARGEM** ([`ph2d_field_render::hit_tolerance`]):
    /// o ponto que esta tabela vai receber foi produzido por uma marcha, e a tolerância dela é o que
    /// diz quão fora da superfície ele pode estar. *Uma margem inventada aqui seria a segunda
    /// resposta, e a que envelhece.*
    #[must_use]
    pub fn build(
        world: &bevy_ecs::world::World,
        root: bevy_ecs::entity::Entity,
        half_extent: f32,
        side_px: f32,
    ) -> Self {
        let (authored, placed) = leaves(world, root);
        // ⭐ **Uma folha só não precisa de dono**, e não perguntar é exactamente o custo zero — é
        // isto que faz o quadro de uma peça simples continuar a ser o de sempre.
        //
        // ⭐⭐⭐⭐ **E N folhas com o MESMO material também não precisam, e isso vale `300×`**
        // (`docs/Render3d/03` §W9, 2026-09-22). A lei do dono emite **uma fita inteira por folha**
        // ([`ph2d_field_eval::owners_wgsl`]: `dono_folha_0`, `dono_folha_1`, …), e essas fitas
        // entram no TEXTO do shader do pintor ⇒ *toda forma acrescentada é um texto novo e uma
        // compilação inteira do driver*. Medido: com lei do dono, acrescentar uma forma custa
        // `2 310 ms`; sem ela, **`7,85 ms`**.
        //
        // ⚠️⚠️ **A saída é byte-idêntica por CONSTRUÇÃO, e não por promessa:** com todos os
        // materiais iguais, o `dono_mix` devolve `(a, b, t)` cujos `ler_mat(a)` e `ler_mat(b)` dão
        // o MESMO `Mat`, logo a lei calcula `ca + (ca − ca) · t`, que é `ca` **exactamente** (o
        // termo é `0,0 · t`). Sem a lei ela calcula `ca` directamente. *As duas rotas são a mesma
        // conta.*
        //
        // ⛔ **E isto NÃO é uma optimização do caso raro: é o caso NORMAL de quem modela** — uma
        // peça a ser construída tem o material de omissão em toda folha, e só ganha materiais
        // distintos quando o artista os autora.
        let so_um_material = authored.windows(2).all(|w| w[0] == w[1]);
        let owners = (placed.len() > 1 && !so_um_material).then(|| {
            ph2d_field_eval::owners::Owners::new(
                &placed,
                &crate::smoke::sampled_registry(),
                ph2d_field_render::hit_tolerance(half_extent, side_px),
            )
        });
        let surfaces = if authored.is_empty() {
            // ⚠️ **Nunca vazia**: uma peça sem folha nenhuma (uma cena a ser apagada) ainda tem de
            // poder ser sombreada, e o `Surfaces::of` indexa `all[0]` como rede.
            vec![surface_of(FieldMaterial::default())]
        } else {
            authored.iter().copied().map(surface_of).collect()
        };
        Self {
            owners,
            surfaces,
            authored,
        }
    }

    /// ⭐⭐ **Re-traduz só os NÚMEROS**, sem tocar na geometria compilada — `true` se algo mudou.
    ///
    /// ⚠️ É esta metade que faz arrastar um slider de cor **não** custar um JIT por folha.
    pub fn refresh_authored(
        &mut self,
        world: &bevy_ecs::world::World,
        root: bevy_ecs::entity::Entity,
    ) -> bool {
        let (authored, _) = leaves(world, root);
        if authored == self.authored || authored.is_empty() {
            return false;
        }
        self.surfaces = authored.iter().copied().map(surface_of).collect();
        self.authored = authored;
        true
    }

    /// A vista que o [`ph2d_field_render::shade_render`] consome.
    #[must_use]
    pub fn surfaces_for(&self) -> ph2d_field_render::Surfaces<'_> {
        ph2d_field_render::Surfaces {
            all: &self.surfaces,
            owners: self.owners.as_ref(),
        }
    }
}

#[cfg(test)]
#[path = "materials_tests.rs"]
mod tests;

/// ⭐⭐⭐ **A LINHA DA COR** — os três canais dobrados numa amostra (Enio, 2026-09-14). Irmão por
/// assunto: ele mede a ponte painel↔documento, não a tabela de materiais.
///
/// ⚠️ **`pub(crate)` desde 18/09, e a razão é o SEXTO consumidor:** o arnês dele (*uma peça de uma
/// folha* + *as linhas que o painel publica para ela*) tinha cinco leitores dentro deste módulo, e
/// o censo dos botões do material — que é de topo, porque mede o `ph2d-material` e não a tabela —
/// precisa do mesmo. ⛔ *Copiá-lo seria a segunda montagem de mundo a divergir da do produto*, que
/// é exactamente o defeito que os `rows_of` existem para não ter.
#[cfg(test)]
#[path = "colour_row_tests.rs"]
pub(crate) mod colour_row_tests;

/// ⭐⭐⭐ **A TABELA SEGUE A PEÇA** — chamada uma vez por quadro, depois do cozimento.
///
/// `doc_mudou` decide qual das duas metades se refaz (ver [`Table`]): a geometria compilada só com
/// documento novo, os números sempre que alguém lhes tocar.
///
/// ⚠️ **E as duas largam o pedido guardado de todos os viewports**, como o `Smoke::set_look`: um
/// material novo sobre o traçado velho é o congelador que o doc do `Viewport::requested` descreve.
pub(crate) fn sync(sim: &mut ph2d_ecs::SimWorld, doc_mudou: bool) {
    // ⚠️ `&mut` para uma LEITURA porque `World::query` o exige — a mesma nota do
    // [`crate::scene::world_has_a_part`]. O empréstimo mutável acaba aqui, de propósito.
    let root = {
        let world = sim.world_mut();
        let mut q = world.query::<(bevy_ecs::entity::Entity, &ph2d_field_ecs::FieldObject)>();
        q.iter(world).next().map(|(e, _)| e)
    };
    let Some(root) = root else {
        return;
    };
    let world = sim.world();
    crate::smoke::with_smoke(|s| {
        let (he, lado) = {
            let vp = s.vp();
            (
                vp.cam.half_extent,
                vp.area.map_or(480.0, |r| r.w.min(r.h).max(1.0)),
            )
        };
        let refez = match (&mut s.materials, doc_mudou) {
            (None, _) | (_, true) => {
                s.materials = Some(std::sync::Arc::new(Table::build(world, root, he, lado)));
                true
            }
            // ⚠️ **`get_mut` devolve `None` enquanto uma thread de traçado segura o `Arc`**, e isso
            // é a resposta certa: aquele traçado está a usar esta tabela **agora**. O quadro
            // seguinte apanha-a — e o artista vê a cor mudar um quadro depois, não nunca.
            (Some(t), false) => {
                std::sync::Arc::get_mut(t).is_some_and(|t| t.refresh_authored(world, root))
            }
        };
        if refez {
            s.forget_requests();
        }
    });
}

/// ⏱️⭐ **A EMISSÃO** — o que um brilho pinta e o que a chamada custa. Irmão por assunto: ele mede
/// uma capacidade do motor que o painel ainda não alcança.
#[cfg(test)]
#[path = "emission_tests.rs"]
mod emission_tests;

/// ⏱️⭐ **O VERNIZ** — quais dos cinco números dele movem o pixel. Irmão por assunto do
/// [`emission_tests`], e pela mesma razão: ele mede uma capacidade do motor antes de ela ter botão.
#[cfg(test)]
#[path = "coat_tests.rs"]
mod coat_tests;

/// ⏱️⭐ **AS CINCO QUE SOBRAM** do OpenPBR — a sonda que mede se elas ganham linha.
#[cfg(test)]
#[path = "base_specular_tests.rs"]
mod base_specular_tests;
