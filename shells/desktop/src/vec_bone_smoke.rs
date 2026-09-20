//! A PONTE da cena `PH2D_VEC_BONE_SMOKE` — o prólogo, e só ele.
//!
//! ⭐ **Os dois tempos e o roteador vivem em [`ph2d_app_vec::smoke_bone`]** (W2 Fase B, 2.ª volta).
//! O que fica aqui é a máquina de dois tempos, porque ela pergunta ao `gfx` e ao mapa de entidades
//! da shell — *o que sai são os CORPOS; o que decide a ordem do quadro fica* (HOWTO §4).

// ⭐ Os quatro nomes que ATRAVESSAM: o `bone_smart_probe` (um diagnóstico da shell) lê-os.
// ⚠️ São exactamente os que atravessam, e nada mais — abrir a mais paga `dead_code` do outro lado
// (HOWTO §2.5).
#[allow(
    unused_imports,
    reason = "quatro destes oito só atravessam em `cfg(test)`; o build do binário vê-os por usar"
)]
pub(crate) use ph2d_skeleton_demo::{
    ARM_A, ARM_B, ARM_BONES, ARM_ELBOW_BEND, DEMO_ACTION, TENTACLE_LIMIT_HALF, cadeia,
    seed_demo_action,
};

impl crate::App {
    /// No prólogo do frame. No-op sem a env.
    pub(crate) fn vec_bone_smoke(&mut self) {
        if !ph2d_app_vec::smoke_bone::armed() || self.gfx.is_none() {
            return;
        }
        // O `ppm` do projecto, UMA vez: o 1.º tempo mede a imagem com ele e o 2.º prende-a com ele
        // — dois valores dariam à régua da imagem duas âncoras.
        let ppm = self
            .gfx
            .as_ref()
            .and_then(|gfx| gfx.hero_screen.as_ref())
            .map_or(ph2d_editor_core::DEFAULT_PIXELS_PER_METER, |h| {
                h.project.pixels_per_meter.max(crate::EPS_PIXELS_PER_METER)
            });
        match self.vec.bone_smoke_step {
            0 => {
                let gfx = self.gfx.as_mut().expect("gfx");
                let _ = gfx
                    .tools
                    .set_active(&ph2d_editor_core::ToolId::new("vector"));
                ph2d_app_vec::smoke_bone::build(
                    &mut gfx.vec_scene,
                    &mut gfx.sim,
                    &mut gfx.renderer,
                    &mut gfx.asset_db,
                    ppm,
                    &mut self.vec,
                );
            }
            // ⚠️⚠️ **NÃO se conta QUADROS aqui, pergunta-se o FATO.** Prender exige a ENTIDADE de
            // cada forma, e quem a cria (`vec_entities::sync`) corre no MEIO do quadro — que um
            // quadro inicial pode nunca alcançar (superfície ainda por configurar). Um contador
            // acertaria na máquina que testou e prenderia **zero** noutra, em silêncio, e o
            // sintoma seria exactamente *"nenhuma forma pode ser deformada"*.
            1 => {
                let prontas =
                    self.vec.bone_smoke_pend.as_ref().is_some_and(|p| {
                        p.iter().all(|(id, _)| self.vec.entities.contains_key(id))
                    });
                if prontas {
                    let gfx = self.gfx.as_mut().expect("gfx");
                    ph2d_app_vec::smoke_bone::bind(
                        &mut gfx.vec_scene,
                        &mut gfx.sim,
                        &mut self.timeline.doc,
                        &gfx.asset_db,
                        ppm,
                        &mut self.vec,
                    );
                    // ⭐⭐⭐ **O PRÓLOGO DA CENA DO ENVELOPE, e ele corre AQUI e não no 1.º tempo.**
                    //
                    // ⚠️ **Enquadrar antes de prender enquadraria a arte de REPOUSO**, que é recta
                    // — e a cena abre DOBRADA. *O que o `Frame All` tem de medir é o que o dono vai
                    // ver.*
                    //
                    // ⚠️ **A DECISÃO é da cena** (`smoke_bone_envelope::prologo`, gateada sem janela
                    // nenhuma); o que mora aqui é o EFEITO, porque fechar um painel e empurrar uma
                    // acção no barramento são duas coisas da `App`. *O molde é o da física: o que
                    // sai são os CORPOS; o que decide a ordem do quadro fica.*
                    {
                        let pro = ph2d_app_vec::smoke_bone_envelope::prologo_do_nivel(
                            ph2d_app_vec::smoke_bone::nivel(),
                        );
                        if let Some(hero) = gfx.hero_screen.as_mut() {
                            if pro.timeline_fechada {
                                <_ as ph2d_editor_core::panel::PanelHostInternal>::set_panel_visible(
                                    hero,
                                    <ph2d_panel_timeline::TimelinePanel as ph2d_editor_core::panel::Panel>::ID,
                                    false,
                                );
                            }
                            // ⭐ O painel de ossos, ANTES do enquadramento: ele é uma coluna
                            // lateral, e abri-lo depois mudaria a área que o `Frame All` mediu.
                            if pro.painel_do_osso {
                                <_ as ph2d_editor_core::panel::PanelHostInternal>::set_panel_visible(
                                    hero,
                                    <ph2d_panel_skeleton::SkeletonPanel as ph2d_editor_core::panel::Panel>::ID,
                                    true,
                                );
                                // ⛔⛔ **ABRIR NÃO É PÔR À FRENTE, e a FOTO é que o disse.** O painel
                                // de ossos partilha a coluna com o do vector, e a aba escolhida de
                                // um encaixe é **o ocupante mais ao topo da ordem z** — abri-lo
                                // deixava-o ATRÁS, e o passo (3) do roteiro mandava ler uma fileira
                                // numa aba que o artista não vê.
                                //
                                // ⚠️ **O `bump` corre DEPOIS do `reconcile_z` deste quadro** (ele
                                // corre no início), logo ele fica mesmo no topo — e no quadro
                                // seguinte a poda mantém-no, porque ele já está na lista. *É a
                                // armadilha que a foto do emissor de partículas pagou.*
                                hero.store.bump_panel_z(
                                    <ph2d_panel_skeleton::SkeletonPanel as ph2d_editor_core::panel::Panel>::NODE_ID,
                                );
                                // ⭐⭐⭐ **A SONDA DA FOTOGRAFIA** (`PH2D_VEC_WEIGHT_PROBE=1`) — ela
                                // arma o verbo `Weight`, escolhe o 1.º osso e pousa o cursor no meio
                                // da barra, que e' o estado que o artista alcanca com tres gestos.
                                //
                                // ⛔⛔ **Ela existe porque a lei da casa manda FOTOGRAFAR um smoke
                                // antes de ele ir ao dono, e os pontos do peso so' aparecem com o
                                // verbo ARMADO e o rato SOBRE a arte** — um arranque limpo nao os
                                // mostra, logo duas curas seguidas foram-lhe enviadas sem eu alguma
                                // vez ter visto o que ele veria. *Uma foto que nao contem o fenomeno
                                // nao prova nada sobre ele.*
                                //
                                // ⚠️ Ela NAO muda o caminho de omissao: sem a env nada disto corre, e
                                // o gate `a_sonda_do_peso_nao_toca_no_caminho_de_omissao` di-lo.
                                if std::env::var_os("PH2D_VEC_WEIGHT_PROBE").is_some() {
                                    // ⛔⛔ **Na FERRAMENTA, nunca no `draw_config`** — a 1.ª redacção
                                    // escreveu no segundo e a foto mostrou o painel do *Transform*: o
                                    // `draw_config` e' uma CO'PIA derivada, reescrita da ferramenta a
                                    // cada quadro pela `fase_tool_mirrors`. *Escrever num espelho lê-se
                                    // como escrever no objecto, até alguem fotografar.*
                                    ph2d_app_vec::vector_bridge::set_mode(
                                        &mut gfx.tools,
                                        ph2d_tool_vector::DrawMode::Bone,
                                    );
                                    ph2d_app_vec::vector_bridge::set_bone_action(
                                        &mut gfx.tools,
                                        ph2d_tool_vector::BoneAction::Weight,
                                    );
                                    // ⚠️ **Pelo NOME, e nunca «o primeiro que a iteracao der».**
                                    //
                                    // ⛔⛔ **E a razao escrita aqui estava REFUTADA, com o numero:**
                                    // ela dizia que *«o osso do MEIO de uma cadeia de tres nao possui
                                    // nada nesta arte»*, e medido pelo reticulo do produto
                                    // (`diag_qual_osso_mostra_mais`) os TRES chegam a `1,000` na
                                    // barra. O que os separa e' **onde** o quente cai:
                                    //
                                    // | osso | `x` do maximo | o canvas corta em `x ≈ −6,5` |
                                    // |---|---|---|
                                    // | `Bone 1` | `−8,21` | **atras do painel** |
                                    // | `Bone 2` | `−5,09` | a` vista |
                                    // | `Bone 3` | `−2,97` | a` vista |
                                    //
                                    // ⇒ a foto e' tirada com o do MEIO, que e' o unico cuja rampa
                                    // inteira cabe no enquadramento — *a mesma lei que a nota do
                                    // `=2` ja' escrevia para o braco pintado, aplicada ao contrario
                                    // por uma premissa que ninguem tinha medido*.
                                    let escolhido = gfx
                                        .sim
                                        .world()
                                        .iter_entities()
                                        .find(|er| {
                                            er.get::<ph2d_skeleton_ecs::Bone>().is_some()
                                                && er.get::<ph2d_ecs::Name>().is_some_and(|n| {
                                                    n.0 == if std::env::var("PH2D_VEC_WEIGHT_PROBE")
                                                        .as_deref()
                                                        == Ok("2")
                                                    {
                                                        "Bone 14"
                                                    } else {
                                                        "Bone 2"
                                                    }
                                                })
                                        })
                                        .map(|er| er.id().to_bits());
                                    if let Some(b) = escolhido {
                                        hero.gizmo.selection = Some(b);
                                    }
                                }
                            }
                            if pro.enquadrar {
                                hero.bus.push(
                                    ph2d_editor_core::action_bus::EditorAction::SetViewFocus {
                                        kind: ph2d_editor_core::ViewFocusKind::All,
                                    },
                                );
                            }
                        }
                    }
                }
            }
            _ => {}
        }
    }
}

/// ⭐⭐⭐ **ONDE A SONDA DA FOTOGRAFIA POUSA O CURSOR** — `None` sem a env, que é o caminho de todos.
///
/// O meio da barra laranja em mundo, que é onde o report do dono de 2026-09-19 estava: ali a arte
/// tem interior e nenhum vértice por perto, logo é o ponto que separa *«a porta pergunta pela
/// silhueta»* de *«a porta mede a distância a um canto»*.
///
/// ⚠️ **Lida UMA vez** (`OnceLock`): ela corre no caminho do cursor, que é por quadro.
pub(crate) fn sonda_do_peso() -> Option<[f64; 2]> {
    static P: std::sync::OnceLock<Option<[f64; 2]>> = std::sync::OnceLock::new();
    *P.get_or_init(|| {
        std::env::var("PH2D_VEC_WEIGHT_PROBE").ok().map(|v| {
            // `=2` aponta a` IMAGEM pintada (malha densa); o resto, ao meio da barra.
            if v == "2" { [5.0, 2.5] } else { [-5.0, 2.5] }
        })
    })
}
