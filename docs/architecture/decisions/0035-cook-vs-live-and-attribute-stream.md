# ADR-0035 — Cook vs live por-subgrafo; stream de atributos ≠ ECS; cloner

**Status:** Accepted (ratificado pelo Enio 2026-05-21; implementação pendente)
**Data:** 2026-05-21
**Decisor(es):** Enio + Claude (arquiteto).
**Depende de:** ADR-0030, ADR-0034, ADR-0021.

## 1. Contexto

O protótipo MiniCavalry avalia o grafo de motion por-frame porque é preview. Em produção, avaliar grafo por-frame para motion estático desperdiça frame budget (HR-4). Houdini/Blender mostram que parte do grafo deve ser **cozida** em asset; parte precisa rodar **viva**.

## 2. Decisão

**Decisão por-subgrafo, critério = a membrana:**

> Subgrafo que **não lê atributo de sim/runtime** → estático → **cozido em asset nativo** (atlas / cena Vello / clip / material WGSL). Subgrafo que **lê do `PresentWorld` a cada frame** → **avaliado vivo** (ADR-0034), sob budget HR-4 + pools HR-3.

O cooker (em `ph2d-asset` / `tools/asset-cooker`) particiona o grafo analisando "lê sim/runtime?". 80% deve cozinhar; live só o dirigido por gameplay. Vale para todos os domínios de apresentação (motion/shader/sound). O formato do bundle cozido é por classe de nó (a especificar com o protótipo).

**Modelo de dados do stream:**
- **Atributos de stream ≠ componentes ECS:** efêmeros, sem identidade, por-instância; vivem no avaliador, leem do `PresentWorld`, **nunca** armazenam no ECS.
- **Cloner** (1 nó → N instâncias) **não tem análogo ECS** — é multiplicador de stream → baixa para GPU instancing (M5 escala 100k @ 60Hz). Não forçar dentro do bevy_ecs.

## 3. Consequências

**Aceitas:**
- Motion estático não paga avaliação por-frame; só o dinâmico roda vivo.
- Budget-aware: subgrafo que estoura o budget vira candidato a cook obrigatório.

**Riscos:**
- HR-3 no avaliador vivo → pools de instância pré-alocados + `bumpalo` reset por frame; sem `Vec::push` realocante.
- Over-engineering do avaliador vivo (runtime completo quando 80% deveria cozinhar) → cook-first.

## 4. Alternativas consideradas

- **Tudo vivo (avaliar por-frame, modelo do preview):** rejeitado — desperdiça frame budget pro que é constante.
- **Tudo cozido:** rejeitado — motion dirigido por gameplay precisa ler estado em runtime.
- **Stream como componentes ECS / cloner como spawn de entidades:** rejeitado — atributos efêmeros sem identidade não são componentes; cloner é multiplicador de stream, não criador de entidades.
