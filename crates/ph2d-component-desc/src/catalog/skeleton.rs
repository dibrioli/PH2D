//! **A família do ESQUELETO** — os 6 componentes de `ph2d-skeleton-ecs`.
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
    D::authored("ph2d::skeleton::Bone", "Bone", C::Skeleton, O::ANY, &[]),
    // ⭐ O LIMITE — `authored` porque a faixa de nascimento é a volta inteira, que não apara nada:
    // pendurá-lo pela paleta é um no-op exacto, e o artista aperta-o depois. ⚠️ É a diferença
    // com a âncora, que sem alvo seria inerte e sem caminho para o artista a completar.
    D::authored(
        "ph2d::skeleton::BoneLimit",
        "Angle Limit",
        C::Skeleton,
        O::ANY,
        &[],
    ),
    // ⭐ A ÂNCORA — `intrinsic` pela mesma razão da pele: ela chega com o gesto (*Add IK*), que cria
    // o ALVO no mesmo passo. Pendurá-la por paleta daria uma restrição sem alvo — inerte, e sem
    // caminho pelo qual o artista a completasse.
    //
    // ⚠️ O rótulo é o nome que as quatro referências usam (*IK*), e não o do tipo.
    D::intrinsic("ph2d::skeleton::IkGoal", "IK Goal", C::Skeleton, &[]),
    // ⭐ A MARCA do alvo — `intrinsic` pela mesma razão: ela chega com o gesto, e pendurá-la à mão
    // num objecto qualquer só o esconderia do anel de objecto vazio sem lhe dar alça nenhuma.
    D::intrinsic("ph2d::skeleton::IkTarget", "IK Target", C::Skeleton, &[]),
    // ⭐ A PELE — `intrinsic`: ela chega com o GESTO (*Bind*) e **não tem `Default`**, porque uma
    // pele sem a fonte autorada dentro não é uma pele, é uma forma prestes a sumir.
    D::intrinsic("ph2d::skeleton::Skin", "Skin", C::Skeleton, &[]),
    // ⭐ O OSSO INTELIGENTE — `intrinsic` pela mesma razão da âncora e da pele: ele chega com o
    // GESTO (*Add Smart Bone*), que **cria** a acção com o nome do osso e abre a timeline nela.
    // Pendurá-lo por paleta daria um controlo sem acção — inerte, e sem caminho pelo qual o artista
    // o completasse. ⚠️ A nota antiga dizia *«lê qual acção está aberta»*, que era o desenho até
    // 2026-09-08 e o defeito inteiro de um report do dono (ver o doc do tipo).
    D::intrinsic("ph2d::skeleton::SmartBone", "Smart Bone", C::Skeleton, &[]),
];
