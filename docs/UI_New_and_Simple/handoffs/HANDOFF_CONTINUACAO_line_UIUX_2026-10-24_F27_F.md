# HANDOFF DE CONTINUAÇÃO — `line/UIUX`, 24/10 — F27 Bloco F: a palavra de uma escolha, um losango por canal, uma propriedade um canal

- Branch `line/UIUX`, worktree `Worktrees/line-UIUX`. Ponta antes da onda `f8189dcadb`, rebaseada em `main` `b3d38f53c1` →
  `1942b3648f` (conflito único em `docs/Components/ferramentas/fotografa_cena.sh`, resolvido juntando as duas guardas: recusa
  de `:0` e a sessão do dono). **Sem integrar.** Anterior: [F27 E](HANDOFF_CONTINUACAO_line_UIUX_2026-10-24_F27_E.md);
  integração a fechar: [F18–F26](HANDOFF_INTEGRACAO_line_UIUX_2026-10-22_F18_A_F26.md).
- Desenho, censo e recusas: [`../F27_F_plano.md`](../F27_F_plano.md). **Não está copiado aqui.**
- Estado: construído e verificado por nome; **smoke do dono (§4) por fazer**. Próxima LLM: janela nova.

## 1 — Commits (`git log --oneline 1942b3648f..HEAD`)

| commit | o quê |
|---|---|
| `2aa7ed832e` | a coluna do valor escreve a PALAVRA de uma escolha; um canal, um losango no Script |
| `e5ed506aa3` | uma propriedade, um canal — o catálogo cede às famílias à mão (tabela `DONOS`) |
| `526e1526bd` | clippy: o intervalo do índice e o braço das âncoras sem sobreposição |
| (este) | este handoff |

## 2 — O que mudou

### 2.1 (a) Palavra na coluna do valor

- `CampoInfo.especie: Especie` (substitui os bools; `discreto`/`texto` viram métodos).
- Porta única `ph2d_ecs::canal_palavra::palavra` → `Palavra::Chave` (traduz-se ao pintar) | `Palavra::DoArtista` (não se traduz).
- `TrackValues` guarda `Palavra` (`publish_com_mundo`); o painel traduz (`tracks_value::valor_da_row`, `dita`); o gráfico
  lê o mesmo (`PropKind::nomeia`).
- Catálogo: `options!["x" => X @ "chave do cartão"]` (`Opcao { nome, valor, rotulo }`); recurso `NomesDosScripts`.
- Censo: 110 discretos das famílias à mão (24 ligado, 16 número, 59 opções, 11 outros) + 37 campos `options!` (133 opções).
- Recusada a tabela de apelidos em `ph2d-i18n` (o censo de língua leu `tween.ao_acabar.*` como frase).

### 2.2 (b) Um canal, um losango no Script

O seletor leva o losango; o campo de texto do caminho é `Rotulo::vista`. Gate
`o_ficheiro_do_script_tem_um_losango_so_no_seletor`; no `architecture_every_editable_field_is_a_channel`
`INSP_SCRIPT_SOURCE` é `Canal`.

### 2.3 (c) Corte de script recomeça o script

Lei do Godot (padrão-ouro delegado pelo dono, 24/10): ao trocar de script o objecto começa do zero; os valores do Inspector
ficam. Gates que já existiam em `ph2d-script`: `trocar_de_ficheiro_renasce`, `trocar_de_script_a_meio_do_quadro_renasce`.
**Zero código em `ph2d-script`** (sem colisão com `line/CodeEditor`).

### 2.4 (d) Uma propriedade, um canal (achado pela foto da cena 33)

A mesma propriedade era gravada por DOIS canais (família à mão + catálogo), introduzido pelo `78fd037958` desta linha (não
está no `main`; nenhum documento gravado a migrar). Cura: tabela `DONOS`
(`crates/ph2d-ecs/src/canal_componente_donos.rs`), `Dono::Canal(FieldKey) | Dono::Prop(u64)`: **60 campos / 81 faixas** —
OBJETO 9/27, FÍSICA 25/28, JUNTA 26/26, FLIP/VETOR 0. Gates: `todo_dono_aponta_um_campo_real_e_um_canal_compativel`,
`uma_faixa_com_dono_nao_e_canal_do_catalogo`, `escrever_pelo_objeto_muda_o_que_o_catalogo_le`,
`o_catalogo_nao_publica_o_que_o_objeto_ja_e`, `escrever_pela_fisica_muda_o_que_o_catalogo_le`,
`escrever_pela_junta_muda_o_que_o_catalogo_le`, `os_quatro_numeros_da_junta_sao_os_prop_kind`.

### 2.5 A cena `PH2D_AUTOKEY_SMOKE=33`

`crates/ph2d-app-components/src/autokey_smoke_componentes.rs` (o app-vec reserva "32"|"33"). Log medido:
«7 faixas (tem de ser 7)» e «palavras q18: … OK». Fotos lidas: em inglês, 7 linhas («Red · Tint Fill On», «Sorting Layer
Front», «Score · Source Tag», «Source Name Enemy», «Blue · Blend Mode Multi…», «Spinner · Script bob.l…»), secção Script com
um losango só no seletor; com `PH2D_LANG=teste` Tag/On/Multiply saem deformados e bob.luau, Enemy, Front, speed ficam iguais
(nomes do artista).

## 3 — Superfície de colisão (um merge pode partir)

- **`ph2d-component-desc`:** `options!` mudou de forma (`@ "chave"`; `Opcao { nome, valor, rotulo }`) e `CampoInfo.discreto`/
  `.texto` viraram métodos. **A `line/CodeEditor` lê as opções do catálogo** (`property.rs` usa `Opcao.nome/valor`): quem
  integra reconcilia.
- **`ph2d-ecs`:** `canal_palavra.rs` e `canal_componente_donos.rs` novos; `chaves_de`/`todas`/`resolver`/`info` do catálogo
  cedem faixas com dono.
- **Painéis/timeline:** `ph2d-timeline` `track_values`; `ph2d-panel-timeline` `tracks_value`, gráfico; `ph2d-panel-inspector`
  secção Script.
- **Shell:** o recurso `NomesDosScripts` é publicado na mesma linha de `canais_script::publicar_declaracoes`.
- **Formato:** nenhum (`TimelineDoc` v23; `PROJECT_SCHEMA` igual).
- **Delta de contadores:** cenas de smoke +1 (`=33`; `CLAUDE.md` §5 já diz `1..33`); ADR 0; `TimelineDoc` 0;
  opções de canal do catálogo 133 → 106. **Número da cena provisório — o integrador reconta.**

## 4 — Smoke do dono (em linguagem simples)

1. `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-UIUX && PH2D_AUTOKEY_SMOKE=33 ./target/smoke/ph2d-host-desktop`
2. Abre com 4 quadrados (vermelho Red sobre o azul Blue, amarelo Score, verde Spinner) e a linha do tempo parada no quadro 18
   com 7 linhas.
3. Na coluna à direita do nome de cada linha leem-se PALAVRAS: Red · Tint Fill «On», Red · Sorting Layer «Front»,
   Score · Source «Tag», Score · Source Name «Enemy», Blue · Blend Mode «Multiply», Spinner · Script «bob.luau».
   Spinner · speed «3.50» (esse é número).
4. Clique no Spinner: na secção Script do Inspector o losango só aparece ao lado da lista «bob.luau», não ao lado do campo «Script».
5. Arraste o cursor de tempo para antes de 0,5 s: as palavras mudam (Off, Default, Counter, Mix, spin.luau).
6. Carregue em Play: o verde gira com spin.luau antes do quadro 12 e balança com bob.luau entre 12 e 24, recomeçando o
   movimento do início na troca.

Deu errado se: aparece um número (3.00, 5.00, 2.00) numa linha de escolha; há duas linhas «Red · Tint Fill»; há dois
losangos na secção Script.

Extra opcional: o mesmo com `PH2D_LANG=teste` antes do `./target…`: as palavras saem deformadas entre colchetes; os nomes
(bob.luau, Enemy, Front) não.

## 5 — Verificação

- Testes por nome verdes: `ph2d-ecs` `canal_` (30) · `ph2d-app-components` `canais_` (21) e marcas (9) · `ph2d-app-physics`
  `canais_` (21) · `ph2d-app-vec` `canais_` (25) · `ph2d-timeline` `track_values` (5) · `ph2d-panel-timeline`
  `a_coluna_do_valor`/`tracks_value` · `ph2d-editor-core` `architecture_every_editable_field_is_a_channel` (6) ·
  `ph2d-panel-inspector` `a_seccao_script` (16) · `ph2d-i18n` gates de língua · `ph2d-component-desc` (20) · `ph2d-script` `props::` (23).
- `cargo fmt` limpo; clippy `-D warnings` verde nas 11 crates tocadas após `526e1526bd`.
- **Suíte impactada** (`PH2D_SUITE=onda`, 1 428 s): 19 950 corridos · 19 948 verdes · 2 vermelhos de CARGA, nenhum desta onda:
  `ph2d-panel-painter-layers` `seam_estado_escondido::cada_interruptor_que_neutraliza_pinta_como_o_pincel_limpo` (TIMEOUT 360 s, o
  conhecido) e `ph2d-host-code-scenes` `a_cena_3_conduzida_passo_a_passo` (a lista do `luau-lsp` não chegou no passo 3; `loadavg`
  15 min = 126) — sozinho por nome: verde (`loadavg` 27).
- **Mutação:** 8/8 da palavra/losango sangram (M3 precisou do teste novo `um_codigo_sem_variante_fica_numero`) + 7/7 dos DONOS
  (tabela no plano, §(d)).

## 6 — ABERTO, por ordem

1. Smoke do dono `=33` (§4).
2. **FORA** (igual à E): itens de LISTA (Timers, Tweens, StateMachine, CounterWatch, SignalActions, VecAnchors, ShakeEmitter,
   resistências), `ph2d-panel-painter-layers`, gravidade do mundo, física a meio, IkBendSide, Rope Length, áudio do mixer.
3. Achados do censo da palavra, **não curados** — para decisão técnica futura:
   - `MISTURA`/`PILHA_MISTURA` guardam 22 códigos e o menu do painel lista 19 (a linha do menu não casa com o código a partir do 4).
   - `ENVELOPE_WARP`: o painel pinta os nomes dos presets em inglês cru (HR-15), apesar de existirem chaves `panel.vector.engine.efeito.*`.
   - `TINTA_TRACO`: tipo com buracos 1–3; `N_PARAGENS`/`N_PONTOS` do traço declarados discretos sem leitor.
   - `PILHA_DILATA_JUNCAO` e `TEXTO_ALINHAMENTO` aceitam 0..255 em vez de 0..2.
4. Próxima cena: `PH2D_AUTOKEY_SMOKE=34`.
5. **NÃO integrar nem fazer push sem ordem do Enio.** Na integração recontar: número do ADR, número da cena, `TimelineDoc`;
   assinalar a colisão do `options!` com a `line/CodeEditor` (§3).
