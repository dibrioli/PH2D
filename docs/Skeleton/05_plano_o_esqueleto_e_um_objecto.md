# 05 — PLANO: o ESQUELETO é um OBJECTO (ordem do dono, 2026-10-05)

> *«temos um bug que me impede de testar. Os ossos só aparecem no modo Edit de Vector. E no modo edit
> de Vector não consigo mover os ossos. Melhor transformar os ossos em objeto separado de Vector,
> mesmo porque ele é usado em img e será usado no flip»* — o dono, 05/10.
>
> Leitor: a próxima janela da `line/Vector`. Decisões de produto: as do dono (acima) e as que seguem o
> Blender (padrão-ouro: a Armature), escritas aqui para ele as poder recusar no smoke.

## 0. O defeito, e porque a cura é um objecto

- Os gestos do osso (criar, agarrar, rodar, pintar o peso) são um **modo da ferramenta Vector**
  (`DrawMode::Bone`, `ph2d-tool-vector`) e correm dentro do despacho de cliques do VETOR
  (`shells/desktop/src/input_dispatch/despacho_clique_vetor_premido.rs:329…`, `bone_gesture::press`). O
  desenho deles também: `ph2d-app-vec/src/overlay.rs` — `bones: vector_active`.
- Desde a onda dos modos (spec/06 F3, `fd89818a8`…, e a 2.ª volta da `line/UIUX`, «cada forma é um
  objecto», `8d344c9de`) a ferramenta Vector só está na mão DENTRO do Edit de uma forma ⇒ os ossos só se
  vêem ali, e ali o clique é da edição da forma (gizmo/laço/nós). **O osso ficou preso a uma ferramenta
  que deixou de existir fora do Edit.**
- Os ossos servem a imagem (malha presa), o vetor, e vão servir o Flip ⇒ não são parte de NENHUM dos
  três: são um objecto com os seus modos (Blender: *Armature* ▸ Object · Edit · Pose; o spec/06 já lista
  «armadura | Object · Edit · Pose» na tabela do oráculo).

## 1. O produto (o que o dono vê)

| | |
|---|---|
| **Add ▸ Skeleton** (grupo 2D) | cria um esqueleto com UM osso, e entra no Edit dele (como o Add do vetor) |
| **Hierarquia** | o esqueleto é uma linha com os ossos por baixo (a corrente pai→filho) |
| **Object** | o gizmo move/roda/escala o esqueleto INTEIRO (o que estiver preso vai junto) |
| **Edit** | criar ossos (o *Create* de hoje: arrastar a partir de uma ponta encadeia), mover pontas, apagar — a pose de repouso |
| **Pose** | rodar/mover ossos (o *Transform* de hoje, IK, limites, Smart Bones) e corrigir o peso (o *Weight* de hoje, como ferramenta do Pose) |
| **Tab / Ctrl+Tab / seletor *Mode*** | os do spec/06, sem nada novo |
| **Ver os ossos** | o esqueleto DESENHA-SE sempre que visível (o olho da Hierarquia esconde): em Object finos, em Edit/Pose com os pegadores |
| **Prender** (*Bind to Skeleton*) | em Object, a forma/imagem e o esqueleto seleccionados (o botão do painel Bones, ou `Ctrl+P` como no Blender) |
| **Projectos antigos** | ao abrir, cada raiz de ossos sem esqueleto ganha um esqueleto por cima, com o nome da raiz (nada a refazer à mão) |

⛔ O *Weight Paint* do Blender é um modo da MALHA; aqui ele fica como ferramenta do **Pose** do
esqueleto — a correcção de peso deste repo é uma mancha no espaço (F26), não uma tabela por vértice, e
pô-la num modo do vetor/imagem/Flip seria tocar em três famílias que são de outras linhas.

## 2. A obra, por camada (fan-out da DIRETRIZ §2/§3.A)

1. **ECS** (`ph2d-skeleton-ecs`): marcador `Skeleton` (componente vazio, serializável); os ossos-raiz
   passam a ser filhos (`ChildOf`) da entidade do esqueleto. ⚠️ Registo de componentes = número que SOMA
   entre linhas: CONTAR (`CLAUDE.md` §5.0).
2. **Tipo** (`ph2d-component-desc::ObjectKind::Skeleton` + `ph2d-app-components::component_attach::kind_of`
   + o gate `every_marker_derives_its_kind`).
3. **Modo** (`ph2d-editor-core::object_mode::ObjectMode::Pose`, ANEXADO ao fim de `ALL`, chave i18n
   `object_mode.pose`, id `OBJECT_MODE_POSE`) — fundação da `line/UIUX`: só acrescentar.
4. **Família** (`ph2d-app-skeleton::skeleton_mode::Family`, `impl ModeFamily`) — modos `[Edit, Pose]`;
   `enter` arma os gestos do osso SEM a ferramenta Vector; composta em
   `shells/desktop/src/render_loop/fase_object_mode.rs` (mais uma na lista) e com `ENTRIES = &[SKELETON]`
   em `fase_object_add.rs`.
5. **Despacho** — os gestos do osso saem do despacho do VETOR para um despacho do esqueleto armado pelo
   modo (a shell é composição: o corpo vai para `ph2d-app-skeleton`, HOWTO de partir a shell §2). Os 16
   sítios de `DrawMode::Bone` (`git grep -ln 'DrawMode::Bone'`) migram para «o modo do esqueleto activo»;
   o `DrawMode::Bone` da ferramenta Vector SAI (um caminho só — o gate
   `the_bone_pickers_are_modal` vira o do modo).
6. **Desenho** — o overlay dos ossos sai do `vec_overlay_plan` (`bones: vector_active`) e passa a ser por
   esqueleto visível (fase `fase_vector_bone_overlay.rs` → do esqueleto).
7. **Painel Bones** — as três ferramentas viram as do modo: Edit = *Create*; Pose = *Transform*, *Weight*;
   Object = *Bind to Skeleton*.
8. **Migração** — degrau de `PROJECT_SCHEMA` (escada + tripla, três sítios; `python3
   scripts/schema-recount.py` na integração): raiz de ossos sem esqueleto ⇒ esqueleto novo por cima.
9. **Cenas** — as `PH2D_VEC_BONE_SMOKE=1…6` criam esqueletos; uma cena nova `=7` ensina os três modos.

## 3. Gates (cada um com o seu controlo)

- Em **Pose**, arrastar a ponta de um osso RODA-O (e a forma presa segue) — costura de clique pela porta
  real do despacho, sem a ferramenta Vector na mão. Controlo: em Object o mesmo arrasto move o
  esqueleto inteiro.
- Em **Edit**, arrastar de uma ponta CRIA um osso filho; em Pose o mesmo gesto não cria.
- Os ossos DESENHAM-SE com uma forma vetorial em Object (o defeito do dono). Controlo: com o olho do
  esqueleto fechado, não.
- Abrir um projecto antigo (fixture com ossos soltos) dá um esqueleto por raiz, ao bit na pose.
- O cadeado do modo (spec/06): em Pose do esqueleto, clicar numa forma recusa com a frase do modo.
- Censos: i18n, `NodeId`, o registo de componentes contado.

## 4. Antes de começar (FASE 0 da janela nova)

- ⚠️ **A linha está EMPILHADA sobre a `line/UIUX`** (rebase de 05/10 sobre `71056d29f`, «cada forma é um
  objecto», aprovada pelo dono e com handoff de integração): a `line/UIUX` tem de entrar no `main` ANTES
  desta. Se ela integrar entretanto: `git rebase main` (os commits dela saem por igualdade).
- A `line/UIUX` é a dona do sistema de modos: os toques em `object_mode.rs`, `mode_drive.rs` e no menu Add
  são ANEXOS (variante no fim, entrada nova na lista) — nunca reescrever o que lá está.
- Medir antes de construir (`CLAUDE.md` §5.0): ler `spec/06` §3 (o cadeado, `publish_parts`) e a família
  do Flip (`ph2d-app-flip/src/flip_mode.rs`) — a mais parecida (um modo de criação que arma um gesto
  próprio).
