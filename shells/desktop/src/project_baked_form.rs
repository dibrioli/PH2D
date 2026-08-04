//! **OS CANAIS ASSADOS dentro do arquivo de projeto** (irmão de [`crate::project`] e gêmeo do
//! [`crate::project_painter`]).
//!
//! O que estava quebrado: assar a forma de uma malha num sprite (`docs/3D/02.2`) o torna
//! `SpriteSource::Individual { texture_id }`, e esse `texture_id` é um id de runtime da GPU — noutra
//! sessão ele aponta para um slot vazio. Assar, `Ctrl+S`, reabrir: o objeto voltava **em branco**, e
//! com ele ia embora a única coisa que a rota A promete — *o objeto continua reluminável depois de a
//! malha sair do build*.
//!
//! A correção tem as mesmas três peças do Painter, e a do meio é de novo a que faltava:
//!
//! 1. **Uma identidade estável** — [`ph2d_ecs::BakedForm`], carimbada no sprite pelo gesto de assar.
//!    Os bits de entidade morrem no restore (ele despawna tudo e recria); a componente viaja no
//!    `WorldSnapshot`, logo sobrevive ao arquivo *e* ao undo.
//! 2. **OS CANAIS no arquivo, não o resultado**: o `base` (os pixels antes de qualquer luz), a
//!    `form` (o G-buffer que a malha doou) e o **rig** com que aquilo foi aceso. Salvar só os pixels
//!    acesos devolveria uma *fotografia* — bonita, e impossível de re-iluminar. É a mesma diferença
//!    entre reabrir um trabalho e reabrir a impressão dele.
//! 3. **A textura re-materializada no load**: o slot novo é criado vazio e o objeto é aceso pela
//!    **mesma porta** que o gesto usa ([`crate::baked_form::light`]) no primeiro frame, pelo passe de
//!    re-acendida que já roda. Sem isto o sprite carregado referenciaria a textura morta do save.
//!
//! ## Duas decisões que este arquivo NÃO tomou sozinho
//!
//! ⚠️ **Os canais NÃO viajam no blob da escultura** (`ProjectFile.sculpt`), embora ele já exista e já
//! guarde as malhas. O parser daquele blob é `#[cfg(feature = "sculpt3d")]`; um documento guardado
//! ali seria legível **só com o módulo 3D no build**, que é o oposto exato do que a rota A promete.
//! Ele é campo de sprite, ao lado do `painted`.
//!
//! ⚠️ **A forma viaja como IMAGEM RGBA8**, não como `f32`. É medido: 4× menos disco (16 → 4 MiB por
//! sprite a 1024²) por **≤ 3 de 255** no pixel aceso — ver [`crate::baked_form::form_to_rgba8`]. É
//! também o que a indústria inteira shipa, e nenhum deles guarda a malha.

use ph2d_ecs::{BakedForm as BakedFormId, Entity};
use ph2d_light::LightRig;
use ph2d_render::{Sprite, SpriteSource};
use std::collections::BTreeMap;

use crate::baked_form::{BakedForm, form_from_rgba8, form_to_rgba8};

/// Os canais de UM objeto assado, como o arquivo os guarda.
#[derive(serde::Serialize, serde::Deserialize)]
pub(crate) struct BakedFormDocument {
    /// A identidade estável ([`ph2d_ecs::BakedForm`]) do sprite que carrega estes canais.
    pub(crate) id: u32,
    pub(crate) width: u32,
    pub(crate) height: u32,
    /// Os pixels ANTES de qualquer luz, RGBA8 com alfa **direto**.
    pub(crate) base: Vec<u8>,
    /// O G-buffer, RGBA8: normal em `n × 0,5 + 0,5`, peso no alfa.
    pub(crate) form: Vec<u8>,
    /// **O rig com que o artista assou.**
    ///
    /// ⚠️ Sem ele o load acenderia com o rig DEFAULT, e a arte mudaria de luz ao ser reaberta — em
    /// silêncio, e sem nada na tela dizendo por quê. Ele é o único campo aqui que não é pixel, e é o
    /// que separa *reabrir o trabalho* de *reabrir uma aproximação dele*.
    pub(crate) rig: LightRig,
}

impl crate::App {
    /// Os objetos assados para embutir no arquivo. Vazio quando nada foi assado.
    ///
    /// ⚠️ **A identidade é lida do MUNDO, nunca inventada aqui.** Quem a carimba é o gesto de assar
    /// (`sculpt3d::bake`), no mesmo commit em que cria os canais; um sprite assado sem componente é
    /// um bug daquele lado, e gravá-lo sob um id novo aqui esconderia o bug atrás de um documento
    /// que o load não saberia reatar.
    pub(crate) fn collect_baked_forms(&self) -> Vec<BakedFormDocument> {
        let Some(gfx) = self.gfx.as_ref() else {
            return Vec::new();
        };
        if gfx.baked_forms.is_empty() {
            return Vec::new();
        }
        let world = gfx.sim.world();
        let mut out = Vec::new();
        for (bits, bake) in &gfx.baked_forms {
            let entity = Entity::from_bits(*bits);
            let Ok(e) = world.get_entity(entity) else {
                continue; // o sprite já não existe; os canais morrem com ele
            };
            let Some(id) = e.get::<BakedFormId>() else {
                continue;
            };
            out.push(BakedFormDocument {
                id: id.0,
                width: bake.size.0,
                height: bake.size.1,
                base: bake.base.clone(),
                form: form_to_rgba8(&bake.form),
                rig: bake.rig,
            });
        }
        out
    }

    /// Devolve cada documento ao seu sprite e re-materializa a textura que ele mostra.
    ///
    /// Roda DEPOIS de `apply_project` (o mundo já foi restaurado, com bits novos): varre os sprites
    /// procurando o [`ph2d_ecs::BakedForm`] e reata cada um aos canais de mesmo id. É o mesmo padrão
    /// do `restore_painted_docs` — o snapshot referencia simbolicamente, um bridge materializa.
    ///
    /// ⚠️ **Ele NÃO acende.** O slot nasce vazio, o objeto entra no mapa com `lit_with: None`, e
    /// quem o acende é a [`crate::baked_form::relight_stale`] no primeiro frame — a **mesma** porta
    /// da re-acendida por lâmpada. Uma acendida escrita aqui seria a segunda resposta a *como um
    /// objeto assado vira pixels*, e a arte SALTARIA ao reabrir o arquivo (o defeito que o ADR-0128
    /// pagou cinco vezes). Aqui ele teria a forma mais cruel: o objeto fica certo enquanto o app
    /// está aberto.
    pub(crate) fn restore_baked_forms(&mut self, docs: Vec<BakedFormDocument>) {
        if docs.is_empty() {
            return;
        }
        let Some(gfx) = self.gfx.as_mut() else {
            return;
        };
        // Ids futuros nesta sessão não podem colidir com os do projeto (o contrato do
        // `next_import_cell`), senão assar algo novo sobrescreveria o documento de um objeto que
        // acabou de ser carregado.
        if let Some(max) = docs.iter().map(|d| d.id).max() {
            gfx.next_baked_form = gfx.next_baked_form.max(max.saturating_add(1));
        }
        // 1. Que sprite (bits NOVOS) tem que canais (id estável)?
        let mut targets: Vec<(u64, u32)> = Vec::new();
        {
            let world = gfx.sim.world_mut();
            let mut q = world.query::<(Entity, &BakedFormId)>();
            for (e, d) in q.iter(world) {
                targets.push((e.to_bits(), d.0));
            }
        }
        if targets.is_empty() {
            return;
        }
        let mut by_id: BTreeMap<u32, BakedFormDocument> =
            docs.into_iter().map(|d| (d.id, d)).collect();

        for (bits, id) in targets {
            let Some(doc) = by_id.remove(&id) else {
                continue; // sprite carimbado sem canais no arquivo — nada a devolver
            };
            let size = (doc.width, doc.height);
            if size.0 == 0 || size.1 == 0 {
                continue;
            }
            let texture_id = gfx.renderer.acquire_individual_empty(size.0, size.1);
            let entity = Entity::from_bits(bits);
            if let Some(mut sprite) = gfx.sim.world_mut().get_mut::<Sprite>(entity) {
                reattach_texture(&mut sprite, texture_id);
            }
            gfx.baked_forms.insert(
                bits,
                BakedForm {
                    size,
                    base: doc.base,
                    form: form_from_rgba8(&doc.form),
                    texture_id,
                    rig: doc.rig,
                    // Nunca aceso NESTA sessão: é isto que faz o passe de re-acendida trabalhar no
                    // primeiro frame, pela porta única.
                    lit_with: None,
                },
            );
        }
    }
}

/// Reata o sprite à textura recém-materializada — **e não toca em mais nada**.
///
/// Em especial, não toca no `size`: ele é a pose do sprite em **unidades de mundo** (metros), e já
/// veio correta no snapshot. Escrever ali as dimensões da TEXTURA (que estão em pixels) fez um canvas
/// de 1024 px reabrir com 1024 **metros** de lado — a textura gigante do 1º smoke da persistência do
/// Painter, e o mesmo erro cabe aqui inteiro.
fn reattach_texture(sprite: &mut Sprite, texture_id: u32) {
    sprite.source = SpriteSource::Individual { texture_id };
    // ⚠️ **Alfa DIREITO**, como o bake escreve: o passe de luz multiplica a cor pela luz, e num
    // buffer pré-multiplicado isso aconteceria sobre `cor × alpha` — a borda do recorte escureceria,
    // a assinatura clássica de tratar premultiplicado como direto. É o oposto do documento pintado,
    // cujo composite sobe premultiplicado.
    sprite.premultiplied = false;
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **O DOCUMENTO ATRAVESSA O DISCO INTEIRO** — os canais e, sobretudo, o RIG.
    ///
    /// ⚠️ O rig é o único campo aqui que não é pixel, e é o mais fácil de perder em silêncio: um
    /// documento que gravasse só os dois planos continuaria carregando um objeto perfeito, com a luz
    /// errada. Este gate existe porque a consequência de errá-lo não parece um bug — parece uma
    /// escolha estética que ninguém fez.
    #[test]
    fn a_baked_document_survives_the_disk_with_the_rig_it_was_baked_with() {
        let mut authored = LightRig::default();
        authored.current_mut().angle_deg = 77;
        authored.current_mut().intensity = 0.42;
        authored.lights[1].on = true; // uma segunda lâmpada, para o array inteiro viajar

        let doc = BakedFormDocument {
            id: 3,
            width: 2,
            height: 1,
            base: vec![10, 20, 30, 255, 40, 50, 60, 128],
            form: crate::baked_form::form_to_rgba8(&[0.0, 0.0, 1.0, 1.0, 1.0, 0.0, 0.0, 0.5]),
            rig: authored,
        };
        let bytes = postcard::to_allocvec(&doc).expect("serializa");
        let back: BakedFormDocument = postcard::from_bytes(&bytes).expect("desserializa");

        assert_eq!(
            back.id, 3,
            "a identidade e' o que reata o documento ao sprite"
        );
        assert_eq!((back.width, back.height), (2, 1));
        assert_eq!(back.base, doc.base, "os pixels ANTES da luz");
        assert_eq!(back.form, doc.form, "o G-buffer");
        assert_eq!(
            back.rig, authored,
            "o RIG tem de voltar inteiro -- sem ele o objeto reabre com outra luz, em silencio"
        );
        assert_ne!(
            back.rig,
            LightRig::default(),
            "premissa: o rig autorado NAO e' o default, senao o gate nao distingue os dois"
        );
    }

    /// **REABRIR NÃO REDIMENSIONA O OBJETO** — e o alfa volta DIREITO, como o bake o escreve.
    ///
    /// ⚠️ A segunda metade não é cópia do gêmeo pintado: lá o composite sobe premultiplicado, aqui
    /// não pode. Herdar a linha do Painter deixaria a borda do recorte escurecendo a cada
    /// re-acendida, e o sintoma (um halo que piora quando a lâmpada anda) não aponta para o load.
    #[test]
    fn reattaching_channels_never_resizes_the_sprite() {
        // Um sprite de 1024 px a 100 px/m: 10,24 × 10,24 METROS na cena.
        let mut sprite = Sprite::atlas(0, [10.24, 10.24], [1.0, 1.0, 1.0, 1.0]);
        sprite.source = SpriteSource::Individual { texture_id: 1 }; // a textura MORTA do save
        sprite.premultiplied = true;
        let before = sprite.size;

        reattach_texture(&mut sprite, 42);

        assert!(
            matches!(sprite.source, SpriteSource::Individual { texture_id: 42 }),
            "o sprite passa a amostrar a textura NOVA"
        );
        assert!(
            !sprite.premultiplied,
            "o bake escreve alfa DIREITO -- premultiplicado escurece a borda do recorte a cada \
             re-acendida"
        );
        assert_eq!(
            sprite.size, before,
            "…e a pose em unidades de MUNDO não é tocada (a textura mede em pixels)"
        );
    }
}
