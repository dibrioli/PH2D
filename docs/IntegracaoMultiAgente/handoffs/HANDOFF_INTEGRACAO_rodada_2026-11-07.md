# HANDOFF — rodada de integração de 07/11 (4 linhas)

Leitor: a próxima LLM e o próximo integrador. Ordem do dono: integrar as 4 quando os handoffs ficassem prontos, corrigir todos os vermelhos, ship + CI, fim de dia e desligar o PC. Ele voltou durante a rodada: **não desligar sem ele confirmar**.

## Ordem, medida por `git merge-tree` par a par

PixelArt → PainterWatercolor → SfxLab → GameEngineFuncional, a partir de `bbfefaf393`. Conflitos reais: `CLAUDE.md` §5.1, `crates/ph2d-app-pixel/src/input_teclas.rs` (fica `Space+Tab/1/2/3` do Pixel; `Tab` sozinho passa a `F4` para a timeline, GEF) e `crates/ph2d-app-pixel/src/smoke.rs` (`max_level: 23`; o cartão do Pixel Sprite do GEF foi de 21 para **23**; 21/22 ficam do P10).

Handoffs das linhas: [Pixel P10](../../PixelArt/handoffs/HANDOFF_INTEGRACAO_line_PixelArt_2026-11-06_P10.md) · [Painter tinta viva](../../Painter/handoffs/HANDOFF_INTEGRACAO_line_PainterWatercolor_2026-11-07_TINTA_VIVA.md) · [SfxLab W3–W4](../../SfxLab/handoffs/HANDOFF_INTEGRACAO_line_SfxLab_2026-11-06.md) · [GEF Animated Sprite](../../GameEngineFuncional/handoffs/HANDOFF_INTEGRACAO_line_GameEngineFuncional_2026-11-07_ANIMATED_SPRITE.md).

## O que a árvore combinada pediu

- **Catracas na cadeia:** curadas por movimento: `a_tool_panel_never_hides_the_inspector.rs` para `ph2d-editor-core`; `tilemap_recusa.rs` para `shells/pixel-scenes`, e sai a dev-dep `ph2d-pixel-core` do registry-init; `recontar.py` e `collision-surface.sh` aprenderam a forma derivada do registo.
- **Suíte em `bd0c0c380a`:** 33 492 testes, 30 falhas e 1 timeout. Curados em um commit por linha (`fix(integ 07/11): …`); os 31 voltam verdes por nome.
- **B2:** impactado 20 317 verdes, censos 133, `architecture_*` 160, recontar. `a_luz_nasce_na_estufa` não repetiu.

### ⚠️ Portões mexidos, a rever

- `the_mode_keys_are_wired`: passa a distinguir o acorde `Space+Tab` do `Tab` sozinho (`e_acorde_com_espaco`), com controlo nos dois sentidos. Se o dono quiser o `Tab` livre também com `Space`, o braço sai.
- `the_tree_settles_before_the_capture` e `the_undo_preserves_the_vector_selection`: seguem o texto da GEF 23 (`671a55544f`, `cf07673a18`); comportamento conferido por leitura.
- `quantas_entradas_tem_cada_painel`: inspector 102 → 103, capacidade nova (`INSP_AUDIO_CREATE_SOUND`, SfxLab W3). O +1 foi atribuído por sonda e por diff, sem medir a árvore base.
- `OWN_RULER`: `ROW_H` do SFX fica 48 (faixa da mesa). `FORMAS_NAO_LITERAIS`: as tabelas do `ph2d-sfx-lab`, cobertas por `ids_da_tabela.rs`, novo.
- `o_painel_mostra_o_que_a_ferramenta_le`: as sondas seguem a tinta viva e a Borracha da tinta morta; duas mutações provaram que ainda acusam.
- `a_caixa_da_os_mesmos_planos_que_a_grade_inteira`: faixa lenta do nextest (23 s sozinho, timeout de 180 s sob carga 14–26).
- `medida_instantaneo_memoria`: `bytes_copiados` conta só o que a cópia duplica (fotos em tiles são `Arc`); razão 1,014, `FOLGA` intacta.
- ⛔ `TECTO_DA_MEMORIA_MIB` do Dabber: (386, 1892) → (515, 2175), medido + 5 %. A memória **cresceu de verdade**. Dono 07/11: a linha Painter reduz na próxima rodada, depois deste CI ([memória](../../../project-memory/project_first_stroke_undo_memory_cut_next_round_2026_11_07.md)).

## Para o dono ver no smoke (entrou sem ele ver)

1. **Nada abaixo foi aprovado por ele:** P10c, Animated Sprite e a tinta viva inteira. Os roteiros estão nos handoffs das linhas.
2. **Som:** rótulos `Roll (R)`, `Mutate (M)`, `Next (N)`, `Keep (K)`, `Lock (L)`, `Explore (Tab)`; itens do menu Add 4 px mais baixos (`MENU_ROW_H` 28 → 24); ícone do painel = raio (`Bolt`).
3. **Pixel:** ícone do painel de tiles = `PixelJumble`, por falta de uma grade livre no catálogo. **Game Engine:** o resumo «N frames» da lista de animações mede o texto; o sinal do marcador da cena passa a `footstep`.
4. **Abertos do Painter:** ver §6 do [handoff dele](../../Painter/handoffs/HANDOFF_INTEGRACAO_line_PainterWatercolor_2026-11-07_TINTA_VIVA.md): troca de modo do Wet Round, `PISO` 0,30 da Borracha, `foto_resto_da_borracha.png`.

## Aberto

- Painter: a causa do crescimento da memória é leitura de código (`wet_acorda` antes de `wet_guarda_o_passo`), sem medição contra a base.
- `ph2d-host-sfxlab-scenes`: o rename de `CMD_NEW` foi compilado; os testes de cena correm na varredura do impactado.
- Tags `lei-antiga`, `lei-nova-arquivada` e ramo `backup/lei-antiga-2026-11-05`: seguem só locais.

## Lições

- **5 h paradas:** `while pgrep -f …; do sleep 2; done; suite` acha o próprio shell. O guarda recusa agora (R10, R2 alargada) e a regra é começar direto e conferir que começou: [memória](../../../project-memory/feedback_a_wait_loop_on_pgrep_finds_itself.md).
- **A chave do B2 é a árvore inteira, com o que está por commitar e por rastrear:** salvar a memória durante o B2 deu `a árvore mudou — NADA registado`, 32 min depois. Handoff e memória entram ANTES do B2; quem mudar só docs depois repete o B2 com `PH2D_INTEG_IMPACTADO` a partir do commit já verde, como foi feito aqui a partir de `e8835ccadd`.
