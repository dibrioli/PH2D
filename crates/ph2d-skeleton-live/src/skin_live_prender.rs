//! ⭐⭐⭐ **PRENDER** — o [`bind`], a porta parametrizada dele ([`bind_com`]) e o que ele guarda
//! (tendões e eixos), num irmão do [`super`] pelo tecto de LOC (2026-10-03: o Bind passou a cozer
//! os efeitos e o pai passou das `700` linhas). Corte por RESPONSABILIDADE: o pai deforma por
//! quadro; este prende uma vez.

use super::*;

/// **Prende as formas ao esqueleto.** Devolve quantas prendeu.
///
/// A fonte é a geometria que a forma tem **agora** — o que faz um segundo Bind ser um *re-bind na
/// pose actual*, que é o gesto que todo o pacote de rig oferece. E como a pose de repouso é a
/// identidade por construção (§2.5 do doc 47), **prender não move um pixel**.
///
/// ⛔⛔⛔ **E ele NÃO acrescenta um ponto à forma do artista** — ordem do dono, 2026-09-20:
/// *«retire a criação automática de ponto no bind»*. ⚠️ **A capacidade não foi afinada, foi
/// RETIRADA do gesto:** a subdivisão graduada pelas juntas continua medida e alcançável por
/// [`bind_com`], e o que a produção deixou de fazer está afirmado — com o CONTROLO ao lado — em
/// `skin_live::tests::binding_a_shape_moves_nothing_and_adds_no_point` e em
/// `subdivisao::subdivisao_tests::a_lei_retirada_poe_os_pontos_a_vista_e_o_produto_deixa_os_oito`.
///
/// ⚠️ **O que isso custa está medido e não é pequeno** — a tabela vive no cabeçalho de
/// [`crate::subdivisao`]. *Ela fica ali de propósito: o passo seguinte tem de a bater, e sem o
/// número ao lado ninguém saberia por quanto.*
///
/// ⭐⭐⭐ **E OS EFEITOS são COZIDOS no desenho ao prender** (ordem do dono, 2026-10-03: *«ao aplicar
/// os bones, os efeitos são cozidos antes. E uma vez com bones, o vetor não pode receber efeitos»*)
/// — o *Expand Appearance* ([`VecScene::bake_cooked`]) na cena e na fonte guardada, e a pilha sai
/// vazia. É por isso que a cena entra para ESCRITA.
pub fn bind(
    sim: &mut SimWorld,
    scene: &mut VecScene,
    map: &VecEntityMap,
    paths: &[VecPathId],
    seed: Option<Entity>,
) -> usize {
    bind_com(sim, scene, map, paths, seed, false)
}

/// **O `bind` com a subdivisão como PARÂMETRO** — ver [`crate::subdivisao`].
///
/// ⚠️⚠️ **A lei viaja num argumento e nunca numa variável de ambiente**, e esta casa pagou por
/// isso: uma porta global lida dentro da lei é um CANAL entre testes (o `cargo test` corre-os em
/// threads do MESMO processo), e a suíte reprova junta e passa sozinha. É a mesma forma do
/// [`recook_com`].
///
/// ⛔⛔ **`subdividir = true` é a lei RETIRADA do produto em 2026-09-20** (ordem do dono), e ela
/// fica aqui por duas razões que não são a mesma: ela é o **contrafactual** dos gates que medem o
/// que a subdivisão comprava, e é o **sujeito** de toda forma GRAVADA entre 19 e 20 de Setembro —
/// *um ficheiro daqueles traz a forma já subdividida, e ela tem de continuar a ser deformada.*
///
/// ⚠️ **Nenhum caminho de produto desta crate passa `true`**, e isso é gateado por censo
/// (`subdivisao::subdivisao_tests::nenhum_caminho_de_produto_pede_a_subdivisao`, que conta
/// parênteses e não linhas) — *uma capacidade retirada do gesto e viva na porta é exactamente o
/// que volta sozinho.*
pub fn bind_com(
    sim: &mut SimWorld,
    scene: &mut VecScene,
    map: &VecEntityMap,
    paths: &[VecPathId],
    seed: Option<Entity>,
    subdividir: bool,
) -> usize {
    let ossos = skeleton_of(sim, seed);
    if ossos.is_empty() {
        return 0;
    }
    // ⚠️ **Um osso criado NESTE quadro ainda não tem `StableId`** — a varredura corre uma vez por
    // quadro, e o gesto de prender pode vir antes dela. Semear aqui é o que o
    // `inspector_joint_create` já faz pela mesma razão, e sem isto o tendão nomearia `NONE`.
    ph2d_ecs::assign_missing_stable_ids(sim.world_mut());
    let mut feitos = 0;
    for &id in paths {
        let Some(&bits) = map.get(&id) else { continue };
        let shape = Entity::from_bits(bits);
        if sim.world().get_entity(shape).is_err() {
            continue;
        }
        let Some(src) = scene.paths().iter().find(|p| p.id == id) else {
            continue;
        };
        let Some(shape_inv) = world_of(sim, shape).inverse() else {
            continue;
        };
        let pares = tendons_and_axes(sim, &ossos, shape_inv);
        // ⭐⭐⭐ **OS PESOS DO PADRÃO-OURO TAMBÉM PARA O VECTOR** (2026-09-15, 2.ª metade). Os eixos
        // já vêm no espaço da forma — que é o espaço em que os vértices dela vivem.
        //
        // ⛔ **Vazio é uma resposta:** um caminho ABERTO não tem interior, logo não tem domínio para
        // a energia, e ele fica na lei derivada. *Inventar um domínio para uma linha seria inventar
        // uma arte que o artista não desenhou.*
        let eixos: Vec<ph2d_skin_weights::Handle> = pares
            .iter()
            .map(|o| ph2d_skin_weights::Handle { a: o.a, b: o.b })
            .collect();
        // ⭐⭐⭐ **A SUBDIVISÃO NASCE AQUI** (ordem do dono, 2026-09-19: *«sem saber onde os pontos
        // estão não fica legal — melhor criar a subdivisão visível logo na associação com os
        // ossos»*). É a mesma lei que a 2.ª mídia já tinha: uma IMAGEM presa ganha no bind a malha
        // graduada pelas articulações, e uma FORMA ganha agora os nós que o peso precisa.
        //
        // ⚠️ **Antes dos pesos, e é isso que a torna barata:** o solver do padrão-ouro corre UMA vez,
        // sobre a forma já subdividida. *Subdividir depois obrigaria a interpolar a tabela, que é a
        // lei do ponto novo — boa para um ponto, uma aproximação para trinta.*
        // ⭐⭐⭐ Os efeitos ACTIVOS cozidos; os desligados saem com a pilha (ver o [`bind`]). As voltas
        // apertadas do efeito viram NÓS (F50-h, exacto em repouso), e a cena recebe a MESMA geometria.
        let coze = src.effects.iter().any(ph2d_vec_scene::effect::FxEntry::is_active);
        let mut src = if coze {
            crate::skin_desenho_voltas::parte_nas_voltas(src.cooked().into_owned())
        } else {
            src.clone()
        };
        src.effects.clear();
        let cozido = coze.then(|| src.clone());
        if let Some(alvo) = subdividir
            .then(|| crate::subdivisao::alvo_dos_eixos(&eixos))
            .flatten()
        {
            crate::subdivisao::subdivide(&mut src, alvo, crate::subdivisao::VERTICES_MAX);
        }
        // ⭐⭐⭐ **O CAMPO resolve-se UMA vez e serve os DOIS consumidores** (2026-09-20): a tabela
        // por ponto de controlo (o que a lei dos NÓS lê) é amostrada dele, e ele próprio é guardado
        // para a lei da CURVA poder perguntar por qualquer ponto do interior.
        // ⚠️ O caro são os `31,9 ms` do padrão-ouro; amostrar é o barato. *Chamar as duas portas
        // antigas pagaria o solver duas vezes.*
        let campo = ph2d_vec_skin::pesos::campo_do_caminho(&src, &eixos);
        let pesos = campo
            .as_ref()
            .map(|c| ph2d_vec_skin::pesos::pesos_dos_pontos(&src, c))
            .unwrap_or_default();
        let guardado = crate::skinned_mesh::SkinnedPath {
            path: src,
            pesos,
            campo,
            efeitos_cozidos: coze,
        };
        let Some(bytes) = crate::skinned_mesh::grava(&guardado) else {
            continue;
        };
        let tendoes = pares.into_iter().map(|o| o.tendon).collect();
        sim.world_mut()
            .entity_mut(shape)
            .insert(SkinBind::new(bytes, tendoes));
        if let Some(p) = scene.path_mut(id) {
            if let Some(g) = cozido {
                p.replace_geometry(g);
            }
            p.effects.clear();
        }
        feitos += 1;
    }
    feitos
}

/// ⭐⭐ **UM OSSO PRESO: o tendão que se guarda MAIS o eixo dele**, no espaço da coisa deformada.
///
/// ⚠️ **Os dois viajam juntos porque a coluna `j` da tabela de pesos é o tendão `j`** — separá-los
/// em duas listas é a forma como um osso sem `StableId` numa delas passa a descrever o osso
/// seguinte na outra, com a soma dos pesos a `1` e nenhum gate de geometria a acusar.
// ⚠️ Sem `Copy`: o [`Tendon`] carrega o `rest` e não o é.
#[derive(Clone, Debug, PartialEq)]
pub struct OssoPreso {
    /// O que a [`SkinBind`] guarda.
    pub tendon: Tendon,
    /// A raiz do eixo, no espaço da coisa deformada.
    pub a: [f64; 2],
    /// A ponta do eixo, no mesmo espaço.
    pub b: [f64; 2],
}

/// ⭐⭐⭐ **OS TENDÕES E O EIXO DE CADA UM, no espaço da coisa** — a porta que o padrão-ouro precisa.
///
/// ⚠️⚠️ **Ela existe para os dois NÃO poderem desalinhar-se.** Os pesos guardados são indexados
/// pela posição na lista de tendões, e o solver precisa do **eixo** de cada osso para saber que
/// pedaço da arte é de quem. Se as duas listas fossem produzidas por duas varreduras, bastaria um
/// osso sem `StableId` numa delas para a coluna `j` da tabela passar a descrever o osso `j+1` — e
/// a arte sairia deformada pelo osso errado, com a soma dos pesos a `1` e nenhum gate de geometria
/// a acusar. ⇒ **um percurso só, um `filter_map` só, dois valores por elemento.**
///
/// O eixo sai do próprio `rest` (`S⁻¹ ∘ B`), que é a única coisa que o tendão guarda — logo ele é,
/// por construção, o eixo que a pele vai usar no quadro.
///
/// ⚠️⚠️ **Ela é a ÚNICA porta do bind das DUAS mídias, e essa lei vem do `tendons_for` que ela
/// substituiu** (morto em 2026-09-15, quando os pesos passaram a precisar dos eixos): *a tentação
/// era copiar a lei para o bind novo — ela é curta e a cópia compilava. ⛔ Mas é exactamente a lei
/// cuja divergência ninguém veria: uma forma e uma imagem presas no mesmo gesto passariam a
/// responder a poses diferentes, e o sintoma seria «o braço desenhado não acompanha o braço
/// vectorial».*
///
/// ⚠️ **Um osso sem `StableId` é SALTADO**: `StableId::NONE` não nomeia ninguém, e guardá-lo daria
/// um tendão que resolve para nada — pior que um osso a menos, porque *parece* ligado.
#[must_use]
pub(crate) fn tendons_and_axes(
    sim: &SimWorld,
    ossos: &[Entity],
    shape_inv: Xform,
) -> Vec<OssoPreso> {
    ossos
        .iter()
        .filter_map(|&e| {
            let rest = world_of(sim, e).then(&shape_inv);
            let comprimento = sim.world().get::<Bone>(e)?.length;
            Some(OssoPreso {
                tendon: Tendon {
                    bone: ph2d_ecs::stable_id_of(sim.world(), e)?,
                    rest: rest.0,
                },
                a: rest.apply([0.0, 0.0]),
                b: rest.apply([comprimento, 0.0]),
            })
        })
        .collect()
}

/// ⭐⭐ **Os EIXOS dos ossos no REPOUSO do bind**, no espaço da forma — os que o [`bind_com`] deu ao
/// solver ([`tendons_and_axes`]), lidos do que a pele guardou: o repouso de cada tendão e o
/// comprimento do osso. Pela ordem dos tendões, que é a das colunas da tabela; vazio se um tendão
/// não resolve (e quem o lê cai no campo da fonte).
pub(crate) fn eixos_do_bind(
    sim: &SimWorld,
    skin: &SkinBind,
    index: &BoneIndex,
) -> Vec<ph2d_skin_weights::Handle> {
    let eixos: Option<Vec<_>> = skin
        .tendons
        .iter()
        .map(|t| {
            let comprimento = sim.world().get::<Bone>(*index.get(&t.bone)?)?.length;
            let rest = Xform(t.rest);
            Some(ph2d_skin_weights::Handle {
                a: rest.apply([0.0, 0.0]),
                b: rest.apply([comprimento, 0.0]),
            })
        })
        .collect();
    eixos.unwrap_or_default()
}
