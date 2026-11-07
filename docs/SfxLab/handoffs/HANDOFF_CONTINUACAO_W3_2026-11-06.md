# HANDOFF de continuação — line/SfxLab, onda W3 «Lab» (2026-11-06)

Leitor: a próxima LLM da linha (ou o integrador). Worktree `/home/enio/Documentos/Projetos/PH2D/Worktrees/line-SfxLab`, branch `line/SfxLab`, base `bbfefaf393` (main alinhado, com W0–W2 dentro). **13 commits na linha, último `59203bae8d`. Nada enviado, nada integrado.** Estado: W3 feita e **aprovada pelo dono em 06/11** (dois smokes). Suíte completa e impactados só na integração. Mecanismo, desenho e números medidos: [04_W3_plano.md](../04_W3_plano.md) (§2 e «Decisões de construção» para o desenho, §6 para os números, §7 para o que o dono mandou depois do smoke) — não copiados aqui. Antecedente: [W2](HANDOFF_CONTINUACAO_W2_2026-11-05.md).

Commits: `git log bbfefaf393..HEAD --oneline` (13; do plano `fa5a4a3533` ao `59203bae8d` «Use in Game abre a janela com busca e conta-gotas; o S toca a faixa em solo»).

## 1. Superfície de colisão (saída de `bash scripts/collision-surface.sh`, resumida; rode de novo no main do dia)

```
merge-base bbfefaf393 · 13 commit(s) · 101 arquivo(s)
PROJECT_SCHEMA 217 (base 217) · tripla (217, 13, 29) (base igual) · VEC 29 · FLIP 13 · DOC_VERSION 25
ph2d-ecs 118 (base 118) · ph2d-render 119 (119) · ph2d-script 119 (119)
node.rs intocado · tool.rs intocado
ADR: nenhum criado (último 0198, próximo livre 0199)
Cargo.lock: +ph2d-app-sfxlab, +ph2d-host-sfxlab-scenes, +ph2d-object-picker, +ph2d-panel-sfxlab, +ph2d-sfx-lab
Marcadores de conflito: nenhum · Tetos de LOC: nenhum arquivo passa
```

## 2. Delta para o integrador contar contra o main do dia

- **PROJECT_SCHEMA +0**, registo de componentes **+0 / +0 / +0**, ADR **+0**: W3 não mexe no formato gravado. Recontar mesmo assim (`python3 scripts/schema-recount.py`); o zero é desta linha, não do main.
- **Cargo.lock +5 crates internas** (acima); nenhum pacote externo novo apontado pelo script.
- **Crate nova `ph2d-object-picker`** (janela de escolha de objeto: busca, lista, conta-gotas). **Crate de composição nova `shells/sfxlab-scenes`** (pacote `ph2d-host-sfxlab-scenes`): recebeu os gates de tela que estavam em `ph2d-panel-sfxlab/tests/it`.
- **`ph2d_panel_hierarchy::{arma_escolha, toma_escolha}`** (em `crates/ph2d-panel-hierarchy/src/state.rs`) usam `HierarchyEntity::entity`; **`PointerLook::Pick`** é variante nova (a linha da shell que a mapeia para a cruz só compilou).
- Cena `PH2D_SFX_SMOKE`: agora `1..4` (3 = a aba aberta em «Metal Hit» com objeto sem som; 4 = a 3 com a janela do Use in Game aberta).

## 3. O que um merge pode partir

- ⚠️ **Teto de `ph2d-app-sfxlab`: 2690 de 2692 linhas** (`TETOS_DAS_CRATES_APP`). Código da W4 vai para crate nova ou folha; nunca subir o número.
- ⚠️ **Shell quase no teto** (`the_shell_only_shrinks`; na W2 a folga era 22): o integrador reconta contra o main do dia. A linha só moveu/compôs na shell; confira o número ao integrar.
- **Tocadas fora da família** (colisão provável com outras linhas): `ph2d-panel-hierarchy` (modo de escolha), `ph2d-panel-inspector`, `ph2d-app-audio`, `ph2d-i18n`, `ph2d-ecs`, `ph2d-editor-core` (`PointerLook`), `shells/desktop`, `ph2d-app-registry-init`, `ph2d-panel-registry-init`. Chaves i18n novas: o censo `toda_chave_do_load_tem_texto` e os de texto de painel devem seguir verdes.
- **`ph2d-object-picker` depende de `Field` de `ph2d-kanban-views`** (com `Target::Scratch`). Se a linha do Kanban renomear/mover o `Field`, aqui quebra. A cura certa (mover o `Field` genérico para `ph2d-editor-core`) toca o Kanban e ficou ABERTA.
- Panel → family em `[dev-dependencies]` reprova `no_dependency_climbs_a_layer`; por isso os gates de tela vivem em `shells/sfxlab-scenes`. Outra linha que repita o padrão tropeça no mesmo gate.
- `ph2d-sfx-lab`: tabela de campos fora de `no_magic_numeric` — conferir que não é evasão (ver §6).

## 4. O que W3 entregou (resumo; mecanismo no plano)

- Motor pronto para a tela: cancelar render sem mudar um bit, **limitador da escuta** (não entra no arquivo), mudo, solo, som aprovado.
- Aba do som ao lado do Inspector: cartões na vaga do Inspector, régua, ondas, pilha de camadas, medidores, render em thread, desfazer (a receita, na aba do som).
- **Create sound**, **Use in Game** (janela modal sempre; 4 cliques até ouvir no jogo, eram 3 — aceito pelo dono), Preview sem Transform. Som criado soa em qualquer distância e leva o nome do objeto.
- **S com Play on change toca a faixa em solo; tirar o S toca o som inteiro.**
- Ctrl+Z: na aba do som desfaz a receita; a ligação com o objeto só na aba Scene.

## 5. Prova de fecho

- fmt: `cargo fmt --all` limpo. Clippy `-D warnings` limpo nas crates tocadas: dsp, recipe, lab, scene, app-sfxlab, panel-sfxlab, object-picker, panel-hierarchy, panel-inspector, app-audio, i18n, ecs, editor-core, sfxlab-scenes, desktop, app-registry-init, panel-registry-init.
- Testes por nome: dsp `escuta::` 10 · recipe `library::` 5 · sfx-lab 2 · sfxlab-scenes **44 passam + 3 ignorados** · app-sfxlab 17 · object-picker 10. Gates `the_shell_only_shrinks`, `no_dependency_climbs_a_layer`, `a_tool_panel_never_hides_the_inspector` verdes. **Suíte completa NÃO rodada (do integrador).**
- Gates-chave (ver plano §5 para a lista): `limitador_nunca_passa_do_teto_na_escuta_e_nao_entra_no_arquivo`, `play_on_change_toca_a_faixa_quando_o_solo_muda`.
- **Mutação desta rodada (sangraram, com gate nomeado):** S sem tocar · linha ignorando o conta-gotas · Hierarchy que nunca desarma · Espaço caindo na aba.
- **Lacunas honestas do vermelho-primeiro:** os gates dos passos 2–3 nasceram verdes e foram provados só por mutação; no passo 4, 15 de 21 vistos vermelhos antes; no passo 5, a maioria só por mutação.
- Números (CPU, sem GPU): quadro 0,449 ms (8 camadas), 0,871 ms (64). Por render a 2 s / 120 s: pico 0,009/0,6 ms · LUFS 0,40/23,9 ms · espectro 0,95/54 ms · limitador 0,954/52,6 ms · impressão digital 0,219/17,8 ms. Editar até tocável 2,53 ms a 2 s com camada em cache.

## 6. ABERTO, em ordem

1. **No fecho da LINHA, atualizar o §5 do `CLAUDE.md`** (NÃO feito agora, por ordem): a entrada do SFX Lab ainda diz «ainda sem tela» e `PH2D_SFX_SMOKE=1|2`; deve virar `1..4`, descrever a aba, e trocar o link para o handoff de integração novo (≤ 700 bytes).
2. **W4 Exploração** (plano `01_plano.md`, linha ~114): Roll, Mutate, Lock, oito filhos, mesmo volume, Keep/Next, Heard, Explore (`Tab`). Gates previstos: `fixado_nao_muda_em_mil_sorteios`, `filhos_tocam_no_mesmo_volume`, `heard_recupera_o_ouvido_ha_cem_sorteios`, `desfazer_cobre_o_sorteio`. **Precisa de plano primeiro (`/pd-feature`).** O plano exige **teste às cegas com profissional ao fim da W4**.
3. Mover o `Field` genérico para `ph2d-editor-core` (toca a linha do Kanban).
4. Verificar: mudar o layout com a aba do som ativa pode reabrir a disposição diferente. Fechar a aba no meio de um render junta na thread da UI (até ~140 ms, NÃO medido). Fechar a aba com a janela aberta e `PointerLook::Pick` ativo: só compilado (o dono viu a cruz no smoke).
5. Verificar: a tabela de campos de `ph2d-sfx-lab` fica fora de `no_magic_numeric`; conferir que não é evasão.
6. Intel/ARM do hash: prova no Mac em aberto (ver handoff da W2, §7.3).
7. Fonte de 10 m anexada à mão segue muda à distância (Create sound só aparece em objeto que já tem fonte, e isso não mudou). Fonte nova nasce com Panning 1, efeito NÃO medido.
8. Não fotografado: o estado armado do conta-gotas e a busca filtrando (a tela virtual não aceita clique); provado por evento real nos gates.
9. Do dono: biblioteca gravada (W4/W5) e o profissional do teste às cegas.

## 7. Smokes (comando inteiro)

Binário: `target/smoke/ph2d-host-desktop` (remonte com `--profile smoke` se a árvore andou).

1. Tela (a aba em «Metal Hit»): `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-SfxLab && env PH2D_SFX_SMOKE=3 ./target/smoke/ph2d-host-desktop`
2. Com a janela do Use in Game já aberta: `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-SfxLab && env PH2D_SFX_SMOKE=4 ./target/smoke/ph2d-host-desktop`

## Smoke do dono

**Aprovado em 06/11.** Primeiro smoke: a tela aprovada. Segundo smoke (mesmo dia, depois das duas ordens construídas): o S tocando a faixa em solo e a janela do Use in Game, aprovados; «seguir para a W4». Aceitos sem objeção: 4 cliques até ouvir no jogo (eram 3), cartões do Sound vivos com a janela aberta, cursor em cruz.
