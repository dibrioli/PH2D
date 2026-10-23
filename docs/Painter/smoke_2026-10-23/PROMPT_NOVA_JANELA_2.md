# Prompt da janela seguinte — line/PainterWatercolor, a BIBLIOTECA de pincéis (23/10)

> Smoke do dono 23/10: **OK** para o Smudge na poça seca (camada transparente), a falha do Rewet com o
> Continuous e a Aquarela de fábrica com os números dele + o preset «Watercolor Basic» (`7b669cecb9`).
> A linha NÃO está integrada.

```
Linha line/PainterWatercolor (Modo L), continuação de 23/10.
FASE 0: cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-PainterWatercolor && pwd && git branch --show-current
Leia: CLAUDE.md (§0, §2), project-memory/MEMORY.md, o handoff
docs/Painter/handoffs/HANDOFF_INTEGRACAO_line_PainterWatercolor_2026-10-22_PINCEIS.md (§8 o navegador por
categoria) e `git log --oneline f788cdb5c7..HEAD` (o que esta linha fez desde a integração).
Delegue leitura e verificação a agentes (explorador, verificador).

⛔ REGRA DO DONO (23/10): a suíte COMPLETA só antes da INTEGRAÇÃO. Na linha, mesmo no fecho, só testes
por NOME (`bash scripts/cargo-test-narrow.sh <crate> <nome>`) e provas de mutação — nada de
nextest-impacted, `PH2D_SUITE=fecho` nem `motivo:`. Clippy -D warnings só das crates tocadas, 1× no fim.

MISSÃO — a BIBLIOTECA de pincéis do Painter (o navegador/dropdown de presets). As quatro tarefas são da
MESMA família (a lista e o ficheiro do preset): um plano, um lote de portões (CLAUDE.md §0.10).
Onde ler primeiro: crates/ph2d-brush-presets (biblioteca.rs, estado.rs, modelo.rs, png.rs — `Origem`,
`Biblioteca::carregar`, os chunks `iTXt ph2d-brush`/`phTx`), crates/ph2d-app-painter/src/
painter_bridge_presets*.rs e painter_bridge_categorias.rs, crates/ph2d-panel-painter-layers/src/
paint_wetpaint_presets.rs (`opcoes_do_preset`), assets/brushes/ (os 46 de fábrica) e a pasta do ARTISTA
(onde «Save» grava — confirme no código).

1. APAGAR pincéis que não são de fábrica (`Origem` do artista; os de fábrica nunca). Na UI do navegador
   (item → apagar), com confirmação; texto por ph2d-i18n (HR-15). Decida e documente: desfazer ou
   confirmação basta. O pincel apagado em uso continua na mão até o artista trocar.
2. A ORDEM MUDA SOZINHA (report do dono): reproduza e MEÇA antes de curar — de onde sai a ordem hoje
   (`read_dir` sem ordenar? data de modificação? «recentes»? HashMap?) e em que gesto ela troca (gravar
   um preset, reabrir o app, mudar de meio…). Cure na RAIZ: a mesma biblioteca dá sempre a mesma ordem.
3. ORDEM MANUAL: o artista arrasta um pincel para outra posição no dropdown (dentro da categoria) e a
   ordem PERSISTE entre sessões; opção «Reset positions» devolve a ordem por omissão. Onde a ordem vive
   (ficheiro na pasta do artista, preferências) é decisão técnica sua — padrão-ouro, e diga porquê.
   Pincel novo/apagado/de fábrica novo convive com uma ordem gravada (sem buracos nem duplicados).
4. EXPORTAR um pincel com TUDO: todos os campos do meio (`campos_do_meio`) e as IMAGENS de todos os
   slots — Shape (e Shape 2/dual), Grain, Paper, Flow, mangueira…, incluindo texturas que o artista criou
   ou trouxe de uma camada («Use as …»). Primeiro faça o CENSO: que slots existem, quais o ficheiro de
   hoje já leva, quais ficam para trás (meça com um pincel que tenha uma imagem em cada slot). Se não
   houver IMPORTAR, exportar não serve — garanta o caminho de volta. Portão: exportar → importar numa
   tela nova dá o MESMO pincel (campos ao bit, imagens ao byte) e a MESMA pintura.

Portões red-first + prova de mutação para cada lei (ordem estável, ordem manual sobrevive a reabrir,
reset, apagar não toca a fábrica, ida-e-volta do export). Foto antes/depois da UI pelo teste, olhada.
Fecho: handoff da linha (técnico) + resposta ao Enio curta, sem jargão, smoke em passos numerados com o cd.
```
