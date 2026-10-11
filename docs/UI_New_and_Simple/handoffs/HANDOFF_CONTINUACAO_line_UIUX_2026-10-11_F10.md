# HANDOFF DE CONTINUAÇÃO — `line/UIUX`, 11/10 — depois da F10

## As regras (de `docs/IntegracaoMultiAgente/PROMPT_LINHAS_RAPIDAS_2026-10-11.md` §2)

- **R1 — Janela curta.** Uma onda = uma janela. Passou de ~150 chamadas de ferramenta ou fechou a
  onda: commit, handoff de continuação ≤ 8 KB, PARE.
- **R2 — Edite pela ferramenta `Edit`.** Arquivo novo = `Write`; existente = `Edit`. Script só para a
  MESMA troca em N arquivos, com `assert` de contagem.
- **R3 — Edição e teste nunca no mesmo comando.** Edite; `bash scripts/cargo-check-narrow.sh <crate>`;
  só então teste. (⚠️ o `check` estreito não compila os `#[cfg(test)]` — um erro de tipo num teste só
  aparece no `nextest`.)
- **R4 — Teste dirigido por NOME no laço.** A suíte inteira e o `nextest-impacted.sh` correm 1× no
  fecho, pelo `verificador`.
- **R5 — Nada de `until … sleep`.** Comando > 2 min: `run_in_background` e espere pelo aviso.
- **R6 — Chamadas independentes na MESMA mensagem.**
- **R7 — Doc > 40 KB nunca se lê inteiro:** `grep -n` + `Read` com `offset`/`limit`.
- **R8 — Escreva menos.** Comentário: a lei em 1–3 linhas; o porquê no doc.
- **R9 — Nenhum teste da suíte normal passa de 10 s em `dev`.** Cura: `opt-level` da crate quente;
  fixture menor que mantém a lei (prove com mutação); `#[ignore]` só para o que não afirma nada.

## Onde a linha está

- **Branch** `line/UIUX`, worktree `Worktrees/line-UIUX`, HEAD `d0caf99a1`, merge-base `8f20c60de`
  (o `main` não andou). **F9 + F10 fechadas, NÃO integradas** — integrar só por ordem do Enio.
- O handoff de INTEGRAÇÃO (o que o integrador lê): [F10](HANDOFF_INTEGRACAO_line_UIUX_2026-10-11_F10_PARAMETROS.md)
  (que cita o da F9). Lei, oráculos, recusas: `vetor_nova_ui/15_mais_parametros_das_formas.md`
  (§1b = a estrela do Affinity pelas fotos).
- Binário de smoke compilado (`--profile smoke`); smokes `PH2D_OBJECT_MODE_SMOKE=15` e `=16`.
- Portão da 2.ª redacção: `nextest-impacted` 19 564 / 19 564 · clippy do `ship.sh` ✓ · fmt ✓;
  mutações da estrela 10/10 (2 curadas com teste).

## O que a F10 entregou (uma linha cada)

- Parâmetros novos, apensos e neutros em zero (tipo do canto, envergadura, liso, torção, espiral
  divergente/raio interno, dente, trapézio solto, meio do losango).
- O cartão lateral (os campos sem alça, junto da forma, os mesmos ids da fila ②).
- As alças no modo FORMA (nunca no Select), com prioridade sobre desenhar; a caixa só com alças.
- A estrela do Affinity (2.ª redacção, depois do «ficou horrível» do dono): os pontos junto da ponta
  de cima, o raio interior só radial, o ⇕ das pontas na borda da caixa, a estrela regular (0,382).

## O que está ABERTO (por ordem)

1. **Esperar o smoke do dono** da estrela nova (cena 16 e desenhar à mão). Se ele reprovar de novo:
   ler as fotos guardadas no doc 15 §1b antes de mexer; a lei do produto dele manda.
2. **Perguntas ao dono** (no último relatório, sem resposta): a posição do cartão; os botões
   *Inverter*/*Fechar* da pizza; o tipo de canto POR canto (pede subir o tecto de 8 campos, que é do
   documento — foundational, migrar saves e `PROJECT_SCHEMA`).
3. Sem harness de janela: a ordem do press no modo Forma e o Move/Up do arrasto da caixa estão
   provados por gates de texto + as leis, não por um rato real.
4. Herdado da F9: o botão direito a meio do arrasto não cancela; um dedo real ainda não agarrou uma
   alça.
5. As outras formas com campos ainda seguem o desenho da F9 (só a estrela e o polígono foram
   refeitos pelas fotos): se o dono pedir, o mesmo método — fotos do Affinity/Illustrator → doc 15.

## Para a próxima janela

`cd Worktrees/line-UIUX && pwd && git branch --show-current` antes de ler (MODELO_TROCA_DE_AGENTE).
Leia este handoff e a linha UI/UX do `CLAUDE.md` §5.1 — não o histórico.
