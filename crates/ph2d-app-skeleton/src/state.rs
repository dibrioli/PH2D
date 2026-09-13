//! ⭐ **O estado da família `skeleton` que a `App` guarda — o gesto, o realce e a cache da 2.ª
//! mídia, com um dono só** (`line/render-loop`, A9 da auditoria de arquitectura, 2026-09-12).
//!
//! # Porque um tipo, e porque NESTA crate
//!
//! Eram OITO campos soltos na `App`: dois com o prefixo `vec_` (`vec_bone_drag`, `vec_bone_pose` —
//! o nome de antes de o Esqueleto ser módulo próprio, ADR-0169) e seis sem prefixo nenhum
//! (`smart_pick`, `osso_revelado`, `bone_arm_pending`, `bone_hover`, `bone_preview`,
//! `skin_image_cache`), que só o ASSUNTO juntava. ⚠️ **Um censo por prefixo `vec_*` via dois dos
//! oito** — é a HOWTO §2.15 (*o prefixo não é a família*) do lado da `App`.
//!
//! Os tipos deles já moravam em crates do esqueleto — o `BoneBirth` aqui, o `BoneHover`/`BonePart`
//! na `ph2d-skeleton-render`, a cache na `ph2d-skeleton-live` (que ganhou o nome `SkinImageCache`
//! para esta mudança não precisar de uma aresta nova para a `ph2d-asset`) —, logo o grupo desce
//! inteiro. É o molde da `FlipState` e do `VecState`.
//!
//! ⚠️ **Não é o documento.** Nenhum destes campos entra no `ProjectState` nem no save: são o que o
//! ponteiro está a fazer, o que está aceso e uma cache de pixels. Mover não toca `PROJECT_SCHEMA`
//! nem os registos do `ph2d-ecs`.
//!
//! # ⚠️ `Default` é derivado, e isso foi VERIFICADO campo a campo
//!
//! Os oito inicializadores do `main.rs` eram `None` e `BTreeMap::new()` — exactamente o `Default`.
//! O nome de cada campo perdeu o prefixo `vec_` onde o tinha; a prosa de cada um viajou com ele, e a
//! do `bone_preview`, que estava colada por cima da da cache (o rustdoc dava as duas à cache),
//! voltou ao campo dela.

/// O estado da família `skeleton` que a `App` guarda. Ver o cabeçalho do módulo.
#[derive(Default)]
pub struct SkeletonState {
    /// ⭐⭐⭐ **O OSSO cujo *Pick Object* está armado** — os bits dele, à espera de que a selecção
    /// mude para outra coisa.
    ///
    /// ⚠️ **DUAS maneiras de achar o objecto, UMA porta para o que se faz com ele**
    /// (`skeleton_smart::set_target`): no **canvas** o pick é MODAL e resolve por hit-test — ele tem
    /// de consumir o press, senão a ferramenta Bone cria um osso por baixo do gesto (report do dono,
    /// 2026-09-08) —, e na **Hierarquia** resolve porque a selecção mudou.
    ///
    /// ⛔ A 1.ª redacção desta nota dizia *«resolve pela SELECÇÃO e não por um hit-test próprio»*, e
    /// o commit seguinte construiu o hit-test sem voltar aqui. *Como se ACHA é genuinamente
    /// diferente entre as duas superfícies; o que se FAZ com o achado é que tem de ser uma coisa
    /// só.*
    ///
    /// ⚠️ **O osso é CAPTURADO no arm**, pela razão do `PathPick` da família `vec`: o clique
    /// seguinte MUDA a selecção, então ler o sujeito nesse instante leria o alvo.
    pub smart_pick: Option<u64>,
    /// ⭐⭐⭐ **O OSSO que o painel já revelou** — a memória que faz *revelar-ao-focar* disparar uma
    /// vez por osso NOVO, e não a cada quadro.
    ///
    /// ⛔⛔ **Report do dono (2026-09-08): *«selecionar o bone nem sempre abre a secção de skeleton
    /// no painel»*.** Medido: o cabeçalho da secção cai em `y = 1316 px` com só um osso escolhido
    /// (e em `1978` com uma forma de traço na selecção) sobre uma faixa visível de `900` — *ela
    /// nunca cabe na tela por si*, e o *«nem sempre»* é o painel já estar rolado até lá.
    ///
    /// ⚠️ **É uma ARESTA, não um estado:** rolar a cada quadro em que um osso está em foco tiraria
    /// do artista o sítio onde ele estava a ler, e ele não conseguiria rolar para lado nenhum
    /// enquanto o osso estivesse escolhido.
    ///
    /// ⚠️ **`None` quando nada é osso**, o que faz re-escolher o MESMO osso depois de o largar
    /// revelar outra vez — que é o gesto do report.
    pub osso_revelado: Option<u64>,
    /// ⭐⭐⭐ **O VERBO do osso a armar no próximo passe da ferramenta.**
    ///
    /// ⛔⛔ Ordem do dono (2026-09-09): *«ao seleccionar o osso … o botão Transform é
    /// seleccionado»*. Quem descobre isso é a aresta do foco, que corre com o `gfx` já emprestado a
    /// `sim`/`hero` — e a ferramenta vive em `gfx.tools`. ⇒ o pedido atravessa num campo e é
    /// consumido **antes** de a ferramenta republicar o espelho do modo, senão ela reverteria a
    /// escrita da aresta no mesmo quadro.
    pub bone_arm_pending: Option<ph2d_tool_vector::BoneAction>,
    /// ⭐⭐ **A METADE DE OSSO sob o ponteiro** neste quadro (Enio, 2026-09-06: *«precisamos de um
    /// efeito hover na bolinha e no corpo do osso»*), e `None` quando ele não aponta osso nenhum.
    ///
    /// ⚠️ **É a metade, não o osso** — a bolinha DESLOCA e o corpo GIRA, e as duas alças estão uma
    /// dentro da outra. Sem dizer qual está acesa, a única forma de descobrir o verbo é executá-lo.
    ///
    /// ⚠️ Resolvido UMA vez por quadro pelas **mesmas** funções que o clique usa
    /// ([`crate::bone_pick::hover`]) — a lei do `App::hovered_object`, e aqui mais apertada.
    pub bone_hover: Option<ph2d_skeleton_render::BoneHover>,
    /// ⭐⭐ **O OSSO QUE ESTÁ A NASCER** neste quadro — `(origem, ponta, chega-a-ser-osso)`, e `None`
    /// fora de um arrasto de criação (Enio, 2026-09-07).
    ///
    /// ⚠️ Resolvido ao lado do [`Self::bone_hover`] porque é a MESMA pergunta — *o que o ponteiro
    /// significa agora* — e porque o sítio do desenho já não tem `&self` livre: o `gfx` está
    /// emprestado mutável, e ler a câmara ali seria o segundo empréstimo.
    pub bone_preview: Option<([f64; 2], [f64; 2], bool)>,
    /// ⭐ **Os pixels de cada imagem PRESA ao esqueleto, uma vez por conteúdo.**
    ///
    /// ⚠️ **Sem ela o desenho copiava a imagem inteira 60 vezes por segundo:** o `AssetDb` entrega
    /// um `Cow` e o Vello consome um `Arc`, então a conversão por quadro seria uma cópia por
    /// quadro. A chave é o `AssetId` (o hash do CONTEÚDO), logo duas sprites com a mesma arte
    /// partilham a entrada — e um `Ctrl+Z` que troque os pixels traz um id novo, não uma entrada
    /// obsoleta.
    pub skin_image_cache: ph2d_skeleton_live::skin_image::SkinImageCache,
    /// ⭐⭐⭐ **O OSSO em desenho** (estudo 42 item 5) — o que o press decidiu (origem em MUNDO e
    /// PAI), e `None` fora do gesto. O `release` faz o osso dali até onde a mão soltou.
    ///
    /// ⚠️ **O pai viaja aqui desde 2026-09-09, e é o que faz a ordem do dono ser uma lei só**: ele
    /// é decidido no press, onde o dedo apontou a ponta, e lido no release. Re-derivá-lo no `Up` (a
    /// selecção, que era a lei antiga) seria a segunda resposta à mesma pergunta.
    ///
    /// ⚠️ **Estado de GESTO, não de documento**: ele não entra no snapshot, não tem undo e morre
    /// no Up — como o `envelope_drag` e o `marquee` do `VecState`.
    pub bone_drag: Option<crate::bone_gesture::BoneBirth>,
    /// ⭐ **O osso a ser POSADO** e por onde ele foi agarrado (`true` = pela junta ⇒ desloca;
    /// `false` = pelo corpo ⇒ gira). `None` fora do gesto.
    ///
    /// ⚠️ Como o irmão acima, é estado de GESTO: morre no Up, não entra no snapshot.
    pub bone_pose: Option<(u64, ph2d_skeleton_render::BonePart)>,
}
