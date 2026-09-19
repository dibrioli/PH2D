//! **A família do ESQUELETO** — os 7 componentes de `ph2d-skeleton-ecs`.
//!
//! ⚠️ **Família própria, e não uma prateleira do vetor.** Ela nasceu em 2026-09-06, quando os
//! ossos saíram de dentro do módulo vectorial: nas quatro referências do mercado um esqueleto só
//! serve várias mídias — Blender deforma malha, curva, texto, treliça **e desenho 2D** com o
//! mesmo *Armature*; Moho prende os mesmos ossos a camada vectorial, **de imagem** e 3D; Rive
//! deforma formas **e** imagens; Spine deforma malhas de imagem. O que muda por mídia é só *o que
//! é um ponto ali*.
//!
//! ⚠️ **`applies_to` é `ANY`, e é uma decisão com mecanismo, não uma folga.** O `SkinBind` guarda
//! a fonte autorada em **bytes opacos**, então ele não sabe — nem precisa de saber — se do outro
//! lado está um `VecPath` ou uma malha raster. Estreitar isto para `VECTOR` escreveria na tabela
//! uma limitação que o código não tem, e seria preciso desfazê-la no primeiro cliente novo.
//! ⛔ O que hoje só existe para o vetor é o **GESTO** (*Bind*, no modo Bone) e o **cliente** que
//! sabe percorrer um caminho (`ph2d-vec-skin`) — não o modelo.
//!
//! ⚠️ **O nome do TIPO e o rótulo diferem no `Skin`, de propósito:** o tipo é `SkinBind` (diz o
//! que se GUARDA — a fonte mais as matrizes de repouso) e o rótulo é "Skin" (diz o que a coisa
//! É). Este catálogo é exactamente o sítio onde os dois nomes se encontram.

use crate::{ComponentCategory as C, ComponentDesc as D, ObjectKinds as O};

/// Ordenado por `canonical_name` (gate `the_catalog_is_sorted_and_unique`).
pub const DESCS: &[D] = &[
    // ⭐ O OSSO — `authored`: ele TEM `Default` (comprimento 1, força 1 é um osso legítimo), logo
    // a paleta do `+` consegue construí-lo no ponto neutro.
    D::authored(
        "ph2d::skeleton::Bone",
        "component.bone.name",
        C::Skeleton,
        O::ANY,
        &[],
    ),
    // ⭐ O LIMITE — `authored` porque a faixa de nascimento é a volta inteira, que não apara nada:
    // pendurá-lo pela paleta é um no-op exacto, e o artista aperta-o depois. ⚠️ É a diferença
    // com a âncora, que sem alvo seria inerte e sem caminho para o artista a completar.
    D::authored(
        "ph2d::skeleton::BoneLimit",
        "component.bone_limit.name",
        C::Skeleton,
        O::ANY,
        &[],
    ),
    // ⭐ O REPOUSO — `intrinsic`, e a ausência do default é a LEI e não uma folga: o valor neutro de
    // uma pose é a identidade, que é o defeito medido que este componente existe para curar (ver o
    // cabeçalho de `ph2d_skeleton_ecs::bone_rest`). Ele chega com o osso, ou com *Set Rest Pose*.
    D::intrinsic("ph2d::skeleton::BoneRest", "Rest Pose", C::Skeleton, &[]),
    // ⭐ A ÂNCORA — `intrinsic` pela mesma razão da pele: ela chega com o gesto (*Add IK*), que cria
    // o ALVO no mesmo passo. Pendurá-la por paleta daria uma restrição sem alvo — inerte, e sem
    // caminho pelo qual o artista a completasse.
    //
    // ⚠️ O rótulo é o nome que as quatro referências usam (*IK*), e não o do tipo.
    D::intrinsic(
        "ph2d::skeleton::IkGoal",
        "component.ik_goal.name",
        C::Skeleton,
        &[],
    ),
    // ⭐ A MARCA do alvo — `intrinsic` pela mesma razão: ela chega com o gesto, e pendurá-la à mão
    // num objecto qualquer só o esconderia do anel de objecto vazio sem lhe dar alça nenhuma.
    D::intrinsic(
        "ph2d::skeleton::IkTarget",
        "component.ik_target.name",
        C::Skeleton,
        &[],
    ),
    // ⭐ A PELE — `intrinsic`: ela chega com o GESTO (*Bind*) e **não tem `Default`**, porque uma
    // pele sem a fonte autorada dentro não é uma pele, é uma forma prestes a sumir.
    D::intrinsic(
        "ph2d::skeleton::Skin",
        "component.skin.name",
        C::Skeleton,
        &[],
    ),
    // ⭐ O OSSO INTELIGENTE — `authored`, e a razão MUDOU em 2026-09-08. Ele era `intrinsic` porque
    // o gesto lhe dava a acção e a paleta não tinha caminho para o artista o completar; hoje o gesto
    // **não cria nada** (ordem do dono) e as duas linhas da secção Skeleton — *Pick Object* e
    // *Action* — completam-no. ⭐ Nascer vazio é um no-op exacto (o `drive` salta um clip vazio),
    // que é a mesma razão do `BoneLimit`.
    D::authored(
        "ph2d::skeleton::SmartBone",
        "component.smart_bone.name",
        C::Skeleton,
        O::ANY,
        &[],
    ),
];
