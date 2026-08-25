# NEXT — R-PÓS da obra das RESTRIÇÕES POR ELIMINAÇÃO (obra A: a costura)

> Entregue pela janela **I** em 2026-08-24. Cole o bloco abaixo numa janela E (ou nova).
> A identidade da janela I está no [`INBOX`](INBOX_quadwild.md) (linha do Passo 0), com o
> resultado do controlo da parede.

```
═══════════════════════════════════════════════════════════════════
CLEAN-ROOM · PAPEL R — REVISOR            (PH2D · SKILL_Cleanroom)
═══════════════════════════════════════════════════════════════════
Modo: PÓS · Módulo: 3D (quad remesh) · Alvo: quadwild
Ledger: docs/3D/cleanroom/LEDGER_quadwild.md

Você é o REVISOR: pode ver OS DOIS lados (o fonte do alvo e o nosso
código). Você NÃO escreve nem dita código de produto. Seus achados
voltam ao Implementador em termos FUNCIONAIS, nunca com trecho do
original, e nunca por mensagem direta — via emenda/handoff.
Modo PRÉ exige janela que NÃO seja a E (autofiltragem não se audita).

Leia: SKILL_Cleanroom §7 (e §4.2 no modo PRÉ).

Modo PRÉ (antes de o Implementador abrir):
1. Audite a espec contra §4.2: pseudo-código espelhado, wording de
   manual, nomes internos, tabela verbatim, organização
   transcrita. Achado → E reescreve; verde → ateste no cabeçalho.
2. Rode: bash scripts/cleanroom-sweep.sh <vassoura> <espec e anexos>
3. Confira o cabeçalho completo (§4) e registre o PRÉ no ledger.
4. HANDOFF DA CORRENTE (§10): preencha o BLOCO-I (espec + módulo;
   Modo L: prepare as DUAS mensagens — o bloco do MODELO_ABERTURA_
   LINHA preenchido e o BLOCO-I), rode o sweep SOBRE o handoff,
   salve em cleanroom/NEXT_I.md e IMPRIMA-O no fim da resposta:
   "Auditoria verde. Janela NOVA → cole o(s) bloco(s) abaixo."

Modo PÓS (após paridade verde):
1. Paridade: gates verdes, barra derivada, fase a fase onde há dumps.
2. Sweep total (§7.2): árvore rastreada + --git-history (mensagens e
   patches, incl. cleanroom/ e project-memory/) + linha do CLAUDE.md
   §5 + handoff. ZERO hits é a barra. Recomendado: sweep no
   transcript da janela I.
3. Revisão estrutural: convergência de EXPRESSÃO (decomposição
   arbitrária igual, ordem não-forçada, nomes traduzidos) —
   comportamento igual NÃO é achado, é o objetivo. Achado →
   re-derivação com restrição funcional explícita (§7.3.d).
4. Incidentes: cada um do INBOX transcrito e tratado (quarentena
   comparada; régua do "substancial" §6.2)?
5. Session-id de I fora de {janelas E, queimadas}?
6. Feche o ledger com o bloco de fechamento (§6). Reporte:
   "Ledger fechado. Módulo apto a integrar."
═══════════════════════════════════════════════════════════════════
```

## O que a janela I entregou (factos, para o passo 1 do modo PÓS)

- **Branch:** `line/seamelim`, base **`line/quadextract`** (⛔ não `main` — a ordem de
  integração não é livre).
- **Obra:** só a **A** (a costura). ⛔ A obra B (linhas de feição) não foi tocada.
- **Onde está o mecanismo, os números e as recusas medidas:**
  [`docs/3D/handoffs/HANDOFF_INTEGRACAO_line_seamelim_2026-08-24.md`](../handoffs/HANDOFF_INTEGRACAO_line_seamelim_2026-08-24.md)
  e a [auditoria de 2 lentes](../handoffs/AUDITORIA_line_seamelim_2026-08-24.md).
- **Gates:** 8 novos (2 deles `#[ignore]` por serem a cadeia inteira), o nº1 medido **no
  mapa por canto** com a barra **lida** das `fixtures/`. Gate batched: **4 262 impactados,
  4 262 verdes**. Clippy `--all-targets` nas 3 crates tocadas: **zero**.
- **Mutação:** duas, com os três controlos no arnês — a global (`1,86e10`) e a
  **cirúrgica** (só os fechos: eliminadas ficam em `2,38e-7`, fechos vão a `7,07`/`10,20`).

## ⚠️ Para o passo 4 (incidentes) — há UM, e é do Passo 0

O controlo positivo da parede **VAZOU**: o ficheiro-isca de `~/Referencias/` foi legível
tanto por shell como pela ferramenta de leitura. Causa medida: a janela corre em
`bypassPermissions` (ligado nos settings do utilizador, do projeto e do VSCode), modo em
que as regras `deny` não são aplicadas — e o `settings.local.json` do Passo 0 foi escrito
na raiz da **worktree** enquanto a sessão abriu na árvore **primária**.
⭐ **O Enio foi informado antes de qualquer linha de produto e decidiu prosseguir** («faça
tudo o que for possível para alcançar o estado da arte»). ⇒ a disciplina substituiu a
parede: **nenhuma leitura foi feita** em `~/Referencias/`, `ph2d-quadbench/oracle/` ou nos
`.jsonl` de `~/.claude/projects/`, e nenhum porte/fork do alvo foi aberto.

## ⚠️ Para o passo 3 (convergência de expressão) — o que a janela I de facto leu

Fontes usadas: a **espec**, os dois *papers* do mapa de leitura em `~/Literatura/papers/`
(MIQ 2009 §2, §5, §5.2, §5.4; QEx 2013 não foi preciso nesta obra), o **código do PH2D**, e
os **dumps** de `fixtures/` como dados. ⛔ Nenhum apêndice de listing foi aberto.
⚠️ **A decomposição é nossa e nasceu da medição**, não de uma forma vista: os quatro
módulos (`weld`, `weld_flat`, `weld_solve`, `weld_round`) seguem as fases funcionais, e
**três desenhos alternativos foram construídos e refutados por medição** antes do que
shipa (tabela no handoff §9).

## ⚠️ A pergunta que fica para o E emendar

A barra do gate nº1 passou a ser lida da referência — mas os mapas dela são **`f64`** e o
nosso `GridMap` é **`f32`**. A emenda cura o «literal contra medido» e não alcança o
«`f64` contra `f32`». A forma que o gate usa hoje, e o número, estão no handoff §10.
