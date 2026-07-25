# HANDOFF DE REABERTURA — `line/motion-value` (recado do integrador, 2026-07-25)

> **Para o agente que reabrir esta linha.** Não sou o autor dela — sou o integrador da jornada de
> 2026-07-25, que a encontrou parada e a manteve viva de propósito. Este documento é o que eu
> **medi** sobre ela hoje, não o que ela mesma diz. O handoff do autor continua valendo para o
> *conteúdo* — ⚠️ mas ele **não existe no `main`**, e sim dentro da linha; leia-o de lá:
> `git show wip/motion-value-2026-07-15:docs/HANDOFF_line_motion_value_continuacao_2026-07-14.md`
> (ou abra o arquivo depois do `cd` na worktree). Este cobre a **distância até o `main` de hoje**.

## §0 — Antes de ler qualquer arquivo

```
cd Worktrees/line-motion-value && pwd && git branch --show-current
```

A janela abre na raiz (= `main`) e **o mesmo path relativo existe nas duas árvores**: editar a
errada compila e commita sem erro nenhum
([`MODELO_TROCA_DE_AGENTE_NA_LINHA`](IntegracaoMultiAgente/MODELO_TROCA_DE_AGENTE_NA_LINHA.md)).

Âncoras: branch `line/motion-value` · tag **`wip/motion-value-2026-07-15`** (criada hoje; se a
branch sumir, é por aqui que os 17 commits voltam) · base comum `4d203d485` (**2026-07-14**).
O `target/` foi limpo no fim-de-dia de 25/07 ⇒ **primeiro build FRIO**, servido pelo
`~/.cache/sccache` (~46 GB, quente — nunca apague).

## §1 — O que existe SÓ aqui (17 commits, 54 arquivos, 14–15/07)

| | |
|---|---|
| `ph2d-node-fx-glow` | **FX de PASSE**: RT HDR próprio, aditivo, byte-idêntico no neutro. O Kawase de passe único saiu **quadrado**; virou **mip bloom** (COD/Jimenez). Depois ganhou **cor** — saturation + tint com swatch OKLCH |
| `ph2d-node-motion-delay` | o último nó da fila do M-value |
| `ph2d-node-motion-path` | *"o canal que faltava: como qualquer coisa que o APP possui entra no grafo"* (doc 65) |
| `ph2d-nodegraph::external` | **foundational NOVO** — o canal acima, + `cook_external_tests.rs` |
| **W4.T4** | a **timeline DOCA no Motion** — *"não faltava sistema: faltava geometria"* (doc 64) |
| docs 63 · 64 · **66** · 67 | notas de decisão. O **66** é o mais importante: declara **FALSA** a premissa do plano para os FX de passe |

⚠️ **A §5 do `CLAUDE.md` no `main` SUBESTIMA isto:** ela lista o W4.T4 como *"DESBLOQUEADO"* (a
fazer). Ele está **construído aqui**. Quem for atualizar a §5 depois da recuperação, corrija a frase
em vez de acrescentar ao lado dela.

## §2 — A distância até hoje, MEDIDA (não estimada)

A linha está **~1158 commits atrás**. Mas *quantidade não é dificuldade* — o que importa é a
**sobreposição**. Medida:

**O foundational que esta linha toca quase não se moveu.** Todo o trabalho de GPU das jornadas de
21 e 23/07 entrou como **módulos IRMÃOS novos** — `gpu.rs` (+553) · `reduce_meta.rs` (+276) ·
`column.rs` (+216) · `stream_op_meta.rs` (+79) · `algorithm_meta.rs` (+56) — que é exatamente o
isolamento append-only que a §0.2 manda projetar. No que esta linha usa:

| arquivo | o `main` mudou | a linha muda |
|---|---|---|
| `ph2d-nodegraph/src/cook.rs` | **+13 / −0** | **+79 / −77** |
| `ph2d-nodegraph/src/cook_fingerprint.rs` | **nada** | (reescrito) |
| `ph2d-nodegraph/src/lib.rs` | +9 / −1 | +1 |

> ⚠️ **Isto corrige uma frase que eu mesmo escrevi no `SESSION_ACTIVE.md` e apaguei:** eu disse que
> *"o `cook.rs` foi reconstruído pelas linhas de GPU"*. **É falso** — ele ganhou 13 linhas. Quem
> reconstruiu foi o *vizinho*, não ele. Não repita meu erro: **meça o diff antes de decidir a rota.**

**O contrato congelado NÃO mudou desde 14/07** — `NodeOp=2` / `OpResolver=1` / `NodeManifest=8`,
idêntico. É precisamente para isto que a §6 existe: uma drop-crate escrita contra ele **ainda
compila** 1158 commits depois. E as 3 crates-nó **não existem no `main`** ⇒ entram como diretórios
novos, **conflito impossível**.

## §3 — ⚠️ A ARMADILHA: existem DOIS "motion path", e eles não têm relação

Esta é a coisa mais fácil de errar aqui, e ela nasceu **hoje**:

| | |
|---|---|
| `shells/desktop/src/motion_path_smoke.rs` **no `main`** | o **MOTION PATH da TIMELINE** (ADR-0141, integrado em 25/07): a posição de um objeto vira um CAMINHO, a track escalar é a distância percorrida |
| `shells/desktop/src/motion_path_smoke.rs` **nesta linha** | o nó **`motion.path`** dos Motion Nodes: o canal do app para o grafo |

**Mesmo nome de arquivo, features diferentes, donos diferentes.** No rebase os dois colidem como
"mesmo símbolo" (DIRETRIZ §1.5.5) — e a resolução **não é** fundir os dois: é **renomear o desta
linha** (ex.: `motion_node_path_smoke.rs`) e escolher outro nome para o smoke. Fundi-los produziria
uma cena que não demonstra nenhum dos dois.

## §4 — A rota recomendada (e por que não é "rebase e pronto")

1. **As 3 crates-nó vêm como drop-crates.** Sem conflito possível (§2). É a parte barata.
2. **O `external` é a parte cara** — não porque o vizinho mudou, mas porque ele é **um canal
   novo no substrato**, e desde julho o repo firmou o idioma para isso: `KernelResolver` já tem
   **seis** canais de *side-metadata*, cada um nascendo com default vazio (`grid` · `state_select` ·
   `stream_op` · `algorithm` · `reduces` · `luts`). ⚠️ **Se o `external` não seguir esse molde,
   ele vai parecer contrato** — e contrato exige ADR (§6). Confira se o desenho de 14/07 ainda é o
   certo depois de os seis irmãos existirem.
3. **Releia o doc 66 ANTES de replayar o glow.** Ele já declarou uma premissa de FX de passe
   **falsa uma vez**. E desde então o `main` ganhou um **precedente que não existia em julho**: o
   `ph2d-render::ImpastoLightPass` (Painter, 18/07) — um passe **pós-composite**, irmão do
   `PreviewPremul`, com o argumento escrito de por que um efeito não-local **não** é um `LayerOp`.
   Se o `fx.glow` for redesenhado hoje, essa é a referência viva a comparar.
4. **Só então decida rebase × cherry-pick.** Com a medição da §2 na mão, um `git rebase main` é
   plausivelmente mais barato do que parece — a sobreposição real é pequena, exceto pelo `cook.rs`
   e pela colisão da §3. **Meça você mesmo** (o `main` terá andado de novo):
   ```
   comm -12 <(git diff --name-only $(git merge-base main HEAD) HEAD | sort) \
            <(git diff --name-only $(git merge-base main HEAD) main | sort)
   ```

## §5 — O que NÃO fazer

- **Não landar os docs 63–67 sozinhos** para "salvar as decisões". Eles descrevem nós que o `main`
  não tem; docs que descrevem código inexistente são a mesma doença que a §5 do `CLAUDE.md` levou
  um ano para drenar. Os docs vão **junto** com o código.
- **Não confiar nesta página sobre números.** Toda tabela aqui é um retrato de 2026-07-25. A fonte
  é o `git diff` / `rev-list` no dia em que você ler.
- **Não integrar nem pushar sozinho** (§0.7). Fecha a linha, escreve o handoff, e PARA.

## §6 — O contexto de quem parou isto aqui

A linha não foi abandonada por estar errada: a jornada de 14–15/07 acabou e as seis linhas
seguintes (Painter · Vector · motion-nodes · anim · physics · FLIP) consumiram as janelas
seguintes. Ela é **a única worktree do repo com trabalho que nunca chegou ao `main`** — a irmã dela,
`line/cook-parallel`, foi descartada em 25/07 porque **estava** subsumida (o `main` a ultrapassou);
esta **não está**. Um FX de glow completo, dois nós e um canal foundational estão parados aqui.
