# HANDOFF (continuação) — `line/Vector` F48: o fecho da imagem presa (2026-10-02)

> Continuação, NÃO integração: o último handoff de integração segue sendo o de
> [2026-10-01](HANDOFF_INTEGRACAO_line_Vector_A_SILHUETA_DA_PELE_2026-10-01.md) (o §6 dele é o que esta nota actualiza).

## 0. Onde está

- Worktree `/home/enio/Documentos/Projetos/PH2D/Worktrees/line-Vector` · ramo `line/Vector` · HEAD `7cf837a9a`.
- Base: `main` `1ad60a1ce`, após `git reset --keep main` (a linha já estava integrada; só a F48 está à frente).
- Antes de ler: `cd` + `pwd` + `git branch --show-current` (tem de dar `line/Vector`).

## 1. O que a F48 fez (mecanismo e medições na [fila §F48](../01_a_fila.md))

1. Bug: a IMAGEM presa mostrava um risquinho de fundo onde dois membros se encontram (pixel `(870,658)` = fundo puro a `(36°, −144°)` `TRACO=0.3`, cena `=4`).
2. Causa: um VÃO (~`20 px²`) entre a borda de um membro e a do membro dobrado de volta; a malha conforma (`0` nós pendurados em `9 091` triângulos). Não é a placa (`PH2D_SKIN_GPU=0` igual).
3. Cura: `ph2d_vec_boolean::fecho_da_borda` (união sem solda → bola por fora → ilhas) devolve o acréscimo; `skin_image_fecho::malha_desenhada` tria-o com a UV da beira e junta-o à malha.
4. Quadro com vão posa na CPU; sem vão a placa posa ao bit. `PH2D_SKIN_CONTACTO=0` desliga; `PH2D_BONE_LOG=1` imprime a espessura dos vãos.
5. `ESPESSURA_MINIMA_EM_TEXELS = 1/32` (medido: ruído `0,0008`–`0,0031` texel; vãos reais `0,12`–`0,88`).
6. Gates novos nos três crates; mutação `10/10`. Duas armadilhas MEDIDAS ficam recusadas na fila (enchimento contra borda crua; contra união soldada).

## 2. Foundational / partilhado tocado

- `ph2d-vec-boolean` `overlap.rs`: `fn pub fecho_da_borda` NOVA + re-export em `lib.rs`; `fecha_as_ilhas` EXTRAÍDA de `silhueta_da_pele` SEM mudar a silhueta (byte-idêntica, gates da F41–F47 intactos).
- `ph2d-skeleton-live` `skin_image.rs` só PERDEU linhas (`692 → 690`); a lógica nova vive em `skin_image_fecho.rs`.
- Zero `PROJECT_SCHEMA`, zero contador, zero ADR, zero contrato congelado.

## 3. Estado dos ABERTOS do handoff de 01/10 §6

1. **A IMAGEM presa (risquinho)** — ✅ FECHADO pela F48.
2. **«O caminho de GPU da imagem não conhece as manchas de peso nem a lei nova da junta (dívida com gate)»** — ⚠️ NOTA DESACTUALIZADA, já fechada na F9 W2 (2026-09-20): `sprite_mesh_para_a_placa` sobe `weights_corrected` com as correcções, a mistura em círculo (a do produto) e a tabela de juntas; a `Desdobrado` (desligada, `PH2D_SKIN_ANGULO=1`) delega na CPU. O gate de dívida `a_pele_da_placa_nao_conhece_as_correccoes_e_isso_esta_nomeado` já não existe; foi substituído por `a_placa_e_a_cpu_posam_cada_vertice_no_mesmo_sitio` e `as_correccoes_a_mao_chegam_a_tabela_da_placa` (`skin_image_gpu_tests.rs`). Medido ponta-a-ponta: fotos da cena `=4` a `(36°, −144°)` com placa e com `PH2D_SKIN_GPU=0` ⇒ `0` pixels diferentes na região da imagem (diferença máxima `0`). Ressalva: a cena não tem manchas; essa metade fica com o gate de tabela.
3. **Forma com EFEITOS não passa pelo desenho fiel** — ABERTO; é o próximo da linha, numa janela NOVA.
4. **Vinco ~`90°`** e 5. **bake no desenho / Strength do envelope** — decisões do DONO, paradas, inalteradas.

## 4. O que falta ao fechar a linha

- Gate batched `1×` sobre o diff acumulado: `scripts/nextest-impacted.sh` + clippy `--all-targets` + auditoria (nenhum correu em bloco; só os gates dirigidos e a mutação `10/10`).
- Binário de smoke (`ph2d-host-desktop`, `--profile smoke`) construído para o dono.
- Ao fechar: `bash scripts/agent-loop-profile.sh` e `rm -rf target/*/incremental`; escrever o handoff de INTEGRAÇÃO (a F48 entra nele) e trocar o link da linha do `CLAUDE.md` §5.1.

## 5. Smoke da F48 (para o Enio)

1. `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-Vector && env PH2D_VEC_BONE_SMOKE=4 PH2D_VEC_BONE_DOBRA=36 PH2D_VEC_BONE_DOBRA2=-144 PH2D_VEC_BONE_TRACO=0.3 cargo run -p ph2d-host-desktop --profile smoke`
2. Olhe a imagem de BAIXO, no canto de dentro, onde os dois membros se encontram.
3. Tem de estar tudo laranja, sem linha de fundo.
4. Errado = um fio cinzento fino dentro do laranja. Para ver o antes, repita o comando com `PH2D_SKIN_CONTACTO=0` na frente de `PH2D_VEC_BONE_SMOKE=4`: aí o fio deve aparecer.
