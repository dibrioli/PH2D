//! ⭐ **A política de DOBRA do Inspector** — que secções têm chevron e quais nascem dobradas. Saiu
//! do [`super`] (tecto de LOC, integração 02/11); o caminho público continua a ser
//! `pre_populate::marca_as_gavetas`, o que o arnês dos censos dos painéis chama.

use crate::ids;
use crate::interaction::WidgetStore;

/// ⭐⭐⭐ **Marca TODA gaveta que o despacho sabe dobrar** — a porta, com dois chamadores.
///
/// ⚠️ **Ela saiu do corpo do `populate_hierarchy_chrome` em 2026-09-21 porque o ARNÊS precisa
/// dela.** O censo dos painéis mede o que cada um pinta e tem de os medir **abertos** (senão ele
/// lê a política de dobra e chama-lhe a dívida do painel) — e para abrir precisa de saber *quais
/// são as gavetas*, que é este conjunto.
///
/// ⛔⛔ **A alternativa era o arnês chamar o [`populate_shared`], e ela PENDURA:** aquele caminho
/// toca no registo de painéis, e o censo já tem o *mutex* dele na mão. Medido — o binário ficou
/// `10 min` a `0 %` de CPU.
pub fn marca_as_gavetas(store: &mut WidgetStore) {
    // Every section header that the Inspector + Widget Gallery paint
    // is collapse-toggle eligible. UI canon post-2026-05-24: every
    // section is collapsible (vide
    // `docs/UI_Padrao/components/section_header.md`). Each id is
    // marked here once; the dispatch flips collapse on left-click.
    // ⚠️ **TODA seção viva, DERIVADA da tabela `ids::LIVE_SECTIONS`** — nunca enumerada aqui.
    //
    // Esta lista era escrita à mão e apodreceu exatamente como a irmã dos pontos de cor: as §11/§12
    // (física) foram acrescentadas quando alguém notou que «o chevron prometia uma dobra que não
    // podia acontecer», e as três vizinhas do sprite — **Ordering · Sampling · Material & Blend** —
    // ficaram de fora e passaram meses a pintar um chevron morto (auditoria de 2026-08-21).
    // *Uma cura escrita ao lado do buraco não o cobre.* Agora a seção nova nasce dobrável no dia em
    // que entra na tabela, e o gate `every_id_the_inspector_paints_can_actually_be_clicked` varre o
    // que o painel PINTA, por isso nem a tabela pode ficar para trás em silêncio.
    for id in ids::LIVE_SECTION_IDS {
        store.mark_collapsible_section(id);
    }
    for id in [
        // §8 Visibility Layer grid — collapsible sub-header (não é uma seção viva: vive DENTRO da
        // §8, por isso não entra na tabela).
        ids::INSP_VIS_LAYER_HEADER,
        // Widget Gallery showcase sections.
        ids::INSP_SECTION_INPUTS,
        ids::INSP_SECTION_SLIDER,
        ids::INSP_SECTION_SWITCHES,
        ids::INSP_SECTION_LISTS,
        ids::INSP_SECTION_VECTOR,
        ids::INSP_SECTION_STATUS,
        ids::INSP_SECTION_COLOR,
        ids::INSP_SECTION_ACTIONS,
        ids::INSP_SECTION_IDENTITY,
        ids::INSP_SECTION_CARD,
        ids::INSP_SECTION_W6,
    ] {
        store.mark_collapsible_section(id);
    }
    // §8 Visibility Layer grid defaults COLLAPSED — it's a tall, advanced
    // (camera cull-mask) control, rarely touched in the common flow.
    store.set_collapsed_if_unchosen(ids::INSP_VIS_LAYER_HEADER, true);

    // ⭐⭐⭐ **TODA SECÇÃO COM CHEVRON NASCE DOBRADA, MENOS A TRANSFORM** — o ecrã que o Inspector
    //    devolve.
    //
    // ⛔⛔ **MEDIDO em 2026-09-21, pela porta do produto:** com tudo aberto o Inspector desenha
    //    `2 175 px` para um **sprite simples**, `3 146` com um corpo e `5 736` para um herói de
    //    plataforma — **`2,5`, `3,6` e `6,5` ecrãs** contra uma dobra de `880`. *Até a imagem mais
    //    simples do app transbordava duas vezes e meia.*
    //
    // ⭐ **O mecanismo já estava completo** (`SectionFold`, o chevron, a animação, o clique): o que
    //    faltava era a POLÍTICA — nenhuma secção nascia dobrada.
    //
    // ⚠️⚠️ **A TRANSFORM não é uma preferência — é a única que CABE.** Medido, o orçamento da dobra
    //    dá para **uma** secção: com ela aberta o painel mede `849 px` (`1,0` ecrãs); com a
    //    `Render` no lugar dela `1 000`, com as duas `1 097`. ⇒ *abre-se a que ainda cabe, e a
    //    medição diz que há exactamente uma.* ⭐ E o número não depende do objecto: `849` nos três
    //    cenários, porque tudo o que varia entre eles está dobrado.
    //
    // ⛔⛔ **Ela mora AQUI e não no `Panel::populate` do Inspector, e o preço disse-o:** posta lá,
    //    ela reprova **342** gates da crate do painel — eles medem o que o painel PODE mostrar, e
    //    a dobra é uma decisão do ECRÃ DO EDITOR, que o painel não conhece (ele não sabe a altura
    //    da janela). *O painel declara o que tem; quem compõe o editor declara como ele abre.*
    //
    // ⚠️ `_if_unchosen`: a escolha do artista manda. ⭐ A ANIMATION abre sob a linha Image (plano GEF 15 D2).
    let aberta = |s| s == ids::INSP_LIVE_TRANSFORM_SECTION || s == ids::INSP_LIVE_ANIM_SECTION;
    for (seccao, _cor) in ids::LIVE_SECTIONS {
        store.set_collapsed_if_unchosen(seccao, !aberta(seccao));
    }
}
