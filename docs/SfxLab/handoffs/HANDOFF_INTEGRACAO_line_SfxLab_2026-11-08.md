# HANDOFF de integração — line/SfxLab, 08/11: o roteiro do teste do dono

**Linha:** `line/SfxLab` em `Worktrees/line-SfxLab`, reaberta sobre o `main` em 07/11 (`652b506b58`, depois da rodada de 4 linhas de 07/11 que levou a W3–W4). **Só documentação.** Zero código, zero teste, zero contrato.

O estado do módulo, os números medidos e o ABERTO de código seguem no [handoff de 06/11](HANDOFF_INTEGRACAO_line_SfxLab_2026-11-06.md) §8–§10. Este só acrescenta o que mudou desde então.

## 1. O que entra

| Ficheiro | O quê |
|---|---|
| `docs/SfxLab/06_roteiro_de_teste_do_dono.md` (novo) | O roteiro que substitui o teste às cegas com profissional: 12 encomendas no app (5 min cada, registo de tempo até o 1.º Keep, Heard, Lock/Explore), a espada até ao Play (cena `PH2D_SFX_SMOKE=3`), os mesmos 12 no jsfxr (2 min cada), a audição às cegas no dia seguinte com 2 casos repetidos e a marca «reconheci». A régua do [W4 plano §7](../05_W4_plano.md) fica, com a leitura assimétrica: quem julga fez os sons. |
| `docs/SfxLab/05_W4_plano.md` §7 | Nota «Revisto em 07/11»: não há profissional; o dono testa pelo roteiro. |
| `CLAUDE.md` §5.1, entrada SFX Lab | O link «Último» passa para este handoff. Mesmo número de bytes que antes (`W3–W4, 06/11` e `roteiro, 08/11` têm ambos 14 bytes; o nome do ficheiro tem o mesmo comprimento). |

**Fora desta árvore:** `project-memory/project_sfx_w4_owner_delegated_decisions_2026_11_06.md`, no checkout primário e por commitar, ganhou o item 4 revisto (dono 07/11: sem profissional; smoke da W4 sobre o main aprovado, cursor incluído). Está misturado com memórias de outras linhas na mesma árvore suja; o integrador comita-o com elas.

## 2. Smoke do dono sobre o main (07/11)

Binário `--profile smoke` montado do `652b506b58`. Cenas `PH2D_SFX_SMOKE=5` e `6`: o arrasto de **Attack (ms)** solto sobre **Kept only** devolve o cursor e não liga o filtro (fecha o item 10.a do handoff de 06/11); Roll/Mutate no mesmo volume, Keep, Kept only, as oito variações e o `Enter`. Veredito: «SMoke OK».

## 3. Portões

Árvore só de docs. Corrido por nome em 08/11: `architecture_claude_md_cabe_no_orcamento` (`ph2d-editor-core`), **3 passaram**, 0 ignorados. Suíte completa e impactados ficam para a integração (regra do dono, 23/10). Contratos: nenhum ficheiro `.rs` tocado (`git diff --stat main -- '*.rs'` vazio).

## 4. ABERTO, em ordem

1. **O roteiro, Partes 1–3:** com o dono, depois da integração.
2. **Construir a Parte 4**, que só se faz depois de o dono entregar o projeto `teste_som` e a pasta `teste_som_jsfxr`: extrair os 12 sons do projeto (ainda sem ferramenta: renderizar as receitas do blob `PH2DSFXL` headless pelo `ph2d-sfx-lab`), buscar 12 gravações com licença conferida por artefacto, nivelar tudo a −23 pelo `maior_momento`, gerar a página offline em `docs/SfxLab/teste_cegas/` com a chave escondida, 14 rodadas e a contagem com e sem «reconheci».
3. **Por conferir na tela:** o primeiro **New Sound** de um projeto novo chama-se **Sound 1** (o roteiro depende disso; lido no código, `SfxLabDocs::nome_livre`, sem ver na janela). Os nomes dos botões do jsfxr (`Pickup/Coin`, `Jump`, `Blip/Select`, `Hit/Hurt`, `Randomize`, `Export .WAV`) estão escritos de memória.
4. O resto do [handoff de 06/11](HANDOFF_INTEGRACAO_line_SfxLab_2026-11-06.md) §10, menos o item a (fechado no §2 acima) e o item b (substituído pelo roteiro).
