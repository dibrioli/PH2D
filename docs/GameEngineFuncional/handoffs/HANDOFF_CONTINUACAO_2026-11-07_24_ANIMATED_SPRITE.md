# Handoff de continuação — GEF 24 Animated Sprite (07/11)

**Estado: `no smoke`.** E0–E6 feitos em `line/GameEngineFuncional`. Falta o smoke do dono. Ficha com as provas por nome: [02_animated_sprite.md](../componentes/02_animated_sprite.md). A linha não foi integrada; o link do CLAUDE.md §5 é do fecho dela, e o «`PH2D_GAME_SMOKE=1..4`» de lá passa a `1..5` no mesmo momento.

## Commits desta rodada

- `9407df6ec0` Add ▸ Animated Sprite abre a escolha sozinho.
- `c283350cfb` a aba do editor engolia o Ctrl+Z.
- `e372733cfb` E5, regra acesa.
- `336f4ddaa1` E6, cena 5 e provas reais.
- `fef48c7e9d` o guião da cena vira a folha `ph2d-sprite-anim-scene`; catracas de volta ao tecto.
- Este documento e a ficha.

## Provado

- **Na janela real, tela virtual, sem clique:** Play começa em Starts with depois de ▶ noutra, e o documento fica igual ao sair; Ctrl+Z de Split by rows (aba à frente), Add ▸ Animated Sprite (`PH2D_OBJECT_ADD_PICK`) abre Assets com o texto de escolha; sample hero, Cut again sem proposta e Keep the cut, um passo cada; a regra acesa troca com o Player a andar. Roteiros: `PH2D_GAME_SMOKE_FOTO=animado_prova_play|animado_prova_desfaz`, com `PH2D_GAME_SMOKE_DRIVE=editor` no Play; `[animado-prova]` e `[jogo]` no log.
- **Testes por nome:** os da ficha, `the_shell_only_shrinks` e os `architecture_*` tocados. Crate nova `ph2d-sprite-anim-scene`: 1. Crates inteiras por formato gravado (`SpriteAnimStart`, `SpriteCutNotice`, registo 119→120): `ph2d-ecs` 669 · `ph2d-component-desc` 24 · `ph2d-host-game-scenes` 168.
- **Mutações:** 16 mutações, por nome de teste; **16 sangraram, 0 sobreviveram**: início não usado · ▶ escreve o início · Split sem filtrar fileira vazia · Split sem transparência · aviso com corte novo · aviso nunca · acesa sempre a 1.ª · acesa sem limpar · acesa ignora o Play · acesa esquece o seguro · uma acesa por mundo · fila errada · LIVE sem acesa · aba engole o Ctrl+Z · aviso fora do registo · Add sem pedir. Não correu a crate inteira por mutante: não confere que só o gate certo sangra.
- **Censo:** **17 à vista e 48 no total, moldura 22** (`o_censo_dos_campos_do_animated_em_uso`, verde sobre o binário final).
- **Fotos:** `docs/Components/ferramentas/fotografa_cena.sh`, fora do repo, lista no relatório da rodada; cada uma foi aberta e conferida.

## Decisões técnicas, para o dono poder rever no smoke

- O aviso de imagem trocada grava-se no projeto: reabre enquanto ninguém respondeu.
- Etiqueta LIVE é o texto da fila, «Who Picks · LIVE», sem linha nova: nada salta no cartão quando a regra acende.
- Regra instantânea fica acesa enquanto segura a animação. Com o Play em pausa nada acende.
- «N components» do cabeçalho sobe 1 uma vez com `SpriteAnimStart`: não curado, é a conta honesta.
- `DocumentSurface::undo_is_the_projects` (default `false`): método novo do trait, não congelado.

## Não provado

- Clique real na tela virtual (XTest não chega): os gestos do roteiro entram pelo barramento e pelas portas dos verbos; o clique está nas costuras de `MockPanelHost` e em `shells/game-scenes`.
- Cut again COM proposta: só pela foto do desfazer.
- ◆ com o nome do sinal na foto: a foto do editor mostra o ◆; o nome aparece com o quadro seleccionado (`clicar_no_campo_renomeia_o_sinal`).

## Abertos

- Censo acima da meta do papel (17/48 contra 15/47): o dono julga.
- Nome do objeto cortado no piso do painel, «Start…»: chrome comum a todo objeto, fora deste item.
- A cena 5 não tem tutorial em PDF nem passa pelo jogo de plataforma completo; `=4` mostra o item em uso.

## Prompt da janela seguinte

```text
Continue GEF 24 Animated Sprite no PH2D. cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-GameEngineFuncional; pwd; git branch --show-current = line/GameEngineFuncional. Leia docs/GameEngineFuncional/handoffs/HANDOFF_CONTINUACAO_2026-11-07_24_ANIMATED_SPRITE.md e docs/GameEngineFuncional/componentes/02_animated_sprite.md. O item está no smoke do dono: PH2D_GAME_SMOKE=5 (peça a peça) e =4 (em uso). Registre o veredito dele na ficha; defeito achado é trabalho da onda; aprovado, o estado vira `fechado` e a fila 23_refundacao_plano.md anda para Add Object. Sem push, sem integrar, sem suíte completa. Comando pesado por bash scripts/ph2d-run.sh.
```
