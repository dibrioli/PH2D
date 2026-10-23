# PROMPT — fechar a `line/Macros-Automação` depois da W17 (2026-10-23)

> Rota «linha reaberta» (`MODELO_TROCA_DE_AGENTE_NA_LINHA.md`):
> `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-Macros-Automação && pwd && git branch --show-current`.
> Leia antes: [handoff W17](HANDOFF_INTEGRACAO_line_Macros_2026-10-23_W17.md) (§1–§6).

## Estado

- W17 feita e **aprovada no smoke pelo dono (23/10)**: o Fill sem Done (âncoras distintas, recusa
  `ColorDropAberto`, `tocadas` sem `break` calado), a cena `=6` com clique instantâneo no C&F e a
  conduzir o ponteiro, o losango largado sem o salto ao sítio antigo, a guarda do `fotografa_cena.sh`.
- Commits sobre `972f411a6f`: `d25c4262b1` · `9c1c53896f` · `7589c6b9d6` · `fb45c93aec` · `d0ac6a70d5` · o deste prompt.
- Corrido só por nome (ver §4 do handoff). **Nada de suíte nem clippy ainda.**

## O que fazer (`/pd-linha-fechar`, DIRETRIZ §1.5.9)

1. `git rebase main` (pare e reporte ao Enio se conflitar fora dos ficheiros da linha).
2. Portão batched 1× sobre o diff `972f411a6f..HEAD`, delegado ao `verificador`:
   `PH2D_SUITE=fecho bash scripts/nextest-impacted.sh` + clippy `--all-targets` (fecho) + censos
   (inclui `the_shell_only_shrinks` — a W17 pôs 3 linhas de composição em `shells/desktop/src/main.rs` —,
   o censo de chaves i18n por `panel.macros.error.fill_left_open`, e o tecto de 700 linhas por ficheiro
   em `sobrepor_tests.rs`/`macro_steps.rs`/`caminho.rs`). Mostre a lista inteira ANTES de consertar.
3. Conserte, comite por paths, e atualize o §4 do handoff W17 com o resultado do portão.
3b. Provas de mutação (agente `mutacao`, 4 controlos) dos gates W17 que só se viram verdes:
   `largado_o_losango_fica_onde_caiu_ate_a_edicao_chegar` (mutações no doc-comment),
   `a_cena_conduz_o_ponteiro_ate_ao_ultimo_gesto_e_devolve_o`,
   `duas_pinceladas_que_se_cruzam_ao_mesmo_tempo_nao_se_apagam` e o `ColorDropAberto` de
   `o_fill_sem_done_nao_leva_o_toque_seguinte_e_nada_some` (tirar a recusa em `etapas_desloca.rs`).

## Aberto que NÃO é deste fecho

- O retângulo branco + aspeto desbotado da 2.ª captura do dono (handoff §5): não reproduzido, causa
  desconhecida; o dono não o viu no smoke seguinte. Só se ele o trouxer de novo, com captura inteira.
4. `bash scripts/agent-loop-profile.sh` e `rm -rf target/*/incremental`.
5. PARE. Integrar e enviar só por ordem explícita do Enio.
