# ADR-0201 — A montagem de juntas é um objeto, e o gabarito é dado puro

- **Status:** Accepted — produto do dono (12/11, versão B da crítica «Joint Editor»); desenho técnico delegado (padrão-ouro).
- **Data:** 2026-11-12
- **Linha:** `line/JointEditor`
- **Estende:** [ADR-0110](0110-vector-nodes-are-ecs-entities-one-hierarchy.md) (uma Hierarquia só) e [ADR-0131](0131-physics-global-runtime-truth-rapier-ecs-bridge.md) (física runtime-truth); não revoga nenhum.
- **Amparo:** [plano](../../JointEditor/01_plano.md) · [números medidos](../../JointEditor/02_gabaritos_medidos.md) · memória `project_joint_editor_version_b_2026_11_12.md`

> ⚠️ Número por recontar na integração (§5.0): escrito como `0201` com o `main` em `0200`.

## Contexto

Montar um carro hoje é 13 gestos (crítica 12/11): duas juntas Wheel feitas à mão, a segunda por copiar e colar, o motor
desligado na roda da frente. Nada no projeto sabe que as duas juntas são «o carro», nem que roda é a da frente. O dono escolheu
a versão B: uma aba com gabaritos (Wheel · Car, Pin · Door, Rope · Bridge…) onde se arrasta cada objeto para o papel dele, sem
tirar nada do Inspector, e sem objeto que surja do nada.

## Decisão

1. **O gabarito é dado puro** (`crates/ph2d-joint-templates`): papéis (nome, forma-guia, posição no desenho, fila de N peças,
   «o cenário pode fazer este papel») e regras de junta (tipo, par de papéis, `Pair`/`Chain`/`ToFirst`/`ToLast`, âncora em
   frações da caixa da peça ou a emenda, eixo, grupo «iguais entre si») mais as diferenças sobre `PhysicsJoint::of_kind`.
   18 gabaritos em 8 tipos; Custom é a junta feita à mão. O `TemplateId` é append-only e pinado por gate.
2. **A montagem é um OBJETO**: a entidade `JointAssembly { template, parts: [(papel, StableId)], linked }`, nomeada («Car 1»),
   com `Transform` e `RootOrder`, na Hierarquia como qualquer objeto. Apagá-la, nomeá-la, desfazê-la e salvá-la é a máquina
   que já existe.
3. **A junta aponta para a montagem, nunca o contrário**: `PhysicsJoint.assembly` (o `StableId` da montagem, `0` = avulsa) e
   `PhysicsJoint.role` (o índice da regra), no FIM do struct. Uma fonte só: a lista das juntas de «Car 1» é uma consulta.
4. **Uma porta de montagem** (`ph2d_app_physics::joint_assembly`): `place_part` põe UM objeto num papel e `sync_joints`
   reconcilia as juntas com o gabarito — cria as que passaram a ter as duas peças, apaga as que perderam o par (a ponta da
   ponte segue a ÚLTIMA tábua) e não toca nas que existem (os números afinados ficam). A junta nasce por `create_joint_at` /
   `create_world_pin_at`, nunca por um terceiro caminho. Um papel `may_be_world` sem peça vira junta presa ao cenário.
5. **Formato**: `JointAssembly` no registo (`+1`) e os dois campos ⇒ `PROJECT_SCHEMA` 238, sem leitura do formato anterior
   (§0.11). `parts[].body` e `PhysicsJoint.assembly` são `RefKind::Object` com remapeador (`remap_assembly_refs`,
   `remap_joint_refs`): copiar a montagem com as peças leva tudo junto.

## Alternativas recusadas

| Alternativa | Por que não |
|---|---|
| A montagem guarda a lista das juntas | Duas fontes para o mesmo fato: apagar uma junta pela Hierarquia deixaria a lista a mentir. |
| Sem entidade: só `assembly_name` nas juntas | «Car 1» não seria selecionável nem nomeável, e uma montagem sem juntas ainda (o carro com só o chassi solto) não existiria. |
| Gabaritos como cenas `.ph2dproj` de exemplo | Seriam objetos mágicos (o dono proibiu) e não leriam as peças que o artista já desenhou. |
| Gabarito dentro de `ph2d-app-physics` | A aba (W3) e a fábrica de peças (W4) leem os mesmos dados; uma folha pura não puxa a família inteira. |
| Âncora em metros fixos | Quebraria com qualquer peça de outro tamanho; frações da caixa seguem a peça do artista. |

## Consequências

- O Inspector continua com os gestos de hoje; uma junta de montagem é uma junta como as outras, com um papel a mais (W2).
- `with_properties_of` não cola `assembly`/`role` (identidade, como `body_a/b`); o merge a três do preview leva os dois.
- Uma montagem cujo gabarito some numa versão futura carrega e não faz nada (`by_id` → `None`), as juntas continuam.
