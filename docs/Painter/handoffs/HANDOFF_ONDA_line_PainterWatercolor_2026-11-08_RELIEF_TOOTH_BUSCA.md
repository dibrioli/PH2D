# Handoff de ONDA — Paper Relief×Tooth + busca (08/11) — a linha CONTINUA

Linha `line/PainterWatercolor`. Nada integrado nem enviado. Commits da onda (sobre `95ac930fbf`): `01fb209238` (feat: Relief×Tooth + busca em faixa fixa) · `9f6647c6bd` (chore: miniaturas aprovadas, teste corrigido, patch WIP). Binário de smoke `target/smoke/ph2d-host-desktop` = `01fb209238` (depois só miniaturas/teste).
Fotos: [`material_2026-11-08_paper_busca/`](material_2026-11-08_paper_busca/) (`paper_relief_tooth.png`, `busca_antes_depois.png`, `busca_aberta/gou/rolada(+_crop).png`, `paper_dica_tooth.png`, `paper_dica_relief.png`, `miniaturas_smear_blur.png`). Bugs: [`BUGS_painter.md`](../BUGS_painter.md).

## Decisões do dono nesta onda

- «Multiplicador novo no Paper»: perguntado se seria controlo novo, respondeu **«Usar o Tooth»** — efeito = Relief × Tooth/`PAPER_TOOTH_MAX` (memória `project_tooth_is_the_paper_effect_multiplier_2026_11_08.md`). Nenhum controlo novo.
- **«não existem desenhos antigos. App em fase inicial»** — compatibilidade de documentos NÃO é critério para leis/defaults do Painter.

## O que entrou (`01fb209238`)

- `dab_spec.rs::mordida_do_papel` (Relief × Tooth / `PAPER_TOOTH_MAX`) nos dois caminhos do Digital (papel-como-grain e o slot `dente`).
- `wetpaint/grao_no_filme.rs::relief_na_tinta`: do piso `max(0.6)` para rampa 0.6→1 (extremos iguais). «Same as Paper» usa `relief_na_tinta` em todos os meios (Relief linear no Digital foi RECUSADO pelo censo `a_linha_esmaecida_e_a_que_nao_age_no_digital`: o Paper Kind ficaria inerte sem esmaecer).
- Dicas Relief/Tooth (`panel.painter_layers.paper.relief_tip`/`tooth_tip`, nos 2 painéis via `cards_papel::poe_dicas`).
- Medido (alfa no miolo do traço, Relief 0.2/0.5/0.8/1, Tooth 1): sem Grain 0.954/0.879/0.804/0.753; com Grain 0.601/0.553/0.506/0.474; lei antiga 0.753 em todo Relief (mutação vermelha).
- **Busca em faixa fixa** (`paint_wetpaint_presets_busca.rs`): faixa no topo do popover cobrindo as `LINHAS_DO_CAMPO`=2 cabeças vazias; campo do cartão (`FIELD_H`, centrado, `LIST_PAD_X`); `TextInput::anel_fino` (anel de foco fino opt-in, `editor-core`); separador só quando rolado; a faixa inteira é o alvo do clique; tinta do cartão do chip (`card_ink::troca_popover`).
- Testes: `o_efeito_do_papel_cresce_com_o_relief_e_o_tooth`, `o_campo_fica_no_topo_quando_a_lista_rola_e_a_faixa_tapa_as_linhas` (ambos provados vermelhos por mutação); `same_as_paper_acompanha_cada_controlo_do_cartao_paper` atualizado (Relief 0.2→1 a cada passo).

## `9f6647c6bd`

- 4 miniaturas regravadas (`PH2D_PINCEIS_APROVA=1`): Finger Tip, Gouache Blender, Smear Basic, Blur Basic.
- `cor_do_papel_tests::no_digital_a_tinta_entra_no_dente` tinha parte velha («com Grain o Tooth não mexe»), vermelha desde `ac3afbdad5` (contradiz o Paper que morde por cima do Grain) → agora `assert_ne!`.
- `material_2026-11-08_paper_busca/WIP_relevo_do_papel_no_digital.patch` (NÃO aplicado; ver defeito novo).

## Veredito do dono (smoke 08/11)

- Relief×Tooth e busca: «demais: smoke OK».
- «Você aprova as miniaturas novas de Finger Tip, Gouache Blender? SIM... muito bom» · «Smear Basic e Blur Basic não está aparecendo com miniatura. Mas pode colocar no mesmo nível» · «Smear Basic e Blur Basic ficam com essas faixas? SIM».
- **ABERTO:** «não está aparecendo com miniatura» — os PNGs antigos NÃO eram vazios (só o traço vermelho), logo não sabemos onde ele viu faltar: perguntar no próximo smoke (na lista de Presets do app? no relatório?).
- Menor: a caixa da dica do Tooth termina na borda direita da tela e cobre por um instante rótulos debaixo dela.

## DEFEITO NOVO (dono, verbatim) — o relevo do papel no Digital

«não tenho certeza mas no digital parece que a sombra do relevo não aparece na tinta (foto1 à esquerda) se o grain não estiver checado com Same as Paper (foto1 à direita). Mas não deveria aparecer? o brush watercolor basic (modo watercolor) funciona muito bem nesse aspecto (foto2), assim como no WetPaint com brush wet Round. Ou seja: nesse aspecto digital não faz bom uso do relevo como os outros modos se não for configurado em grain. mas grain precisa estar livre para outros usos além desse. Então é importante paper funcionar bem no digital independente de Grain.» (memória `project_digital_paper_relief_must_show_on_paint_2026_11_08.md`)

**Mecanismo (medido):** `relief_fields.rs::height_at` soma o dente do papel pesado por `papel_a_vista` = Π(1 − α·opacidade) das camadas raster visíveis (cura §BUG-papel de 27/10, [`49_realshader/oraculo/README.md`](../49_realshader/oraculo/README.md) §BUG-papel). Tinta Digital opaca ⇒ v≈0 ⇒ a passada de luz vê dente plano. O doc 19 §1.2 já diz a lei certa: tinta grossa ENCHE o dente, fina o segue.

**WIP (não aplicado; `git apply` do patch):** `v = molhada · (1 − enche·(1 − outra))`, `enche = clamp(h / (CORPO_QUE_ENCHE·amplitude_em_cargas))`; `papel_a_vista` separado em (tinta da camada Wet, outra tinta) por uma marca por camada `tinta_molhada` posta no pen-down do Wet (`stroke_lifecycle.rs`); `Substrate::amplitude_em_cargas` novo; teste `no_digital_o_relevo_do_papel_ve_se_na_tinta_opaca` (Tooth 0 isola a luz; documento via `abre_a_sprite`): lei nova 7.29 × papel nu 14.32, lei antiga 0.00 (mutação vermelha). Gate do Óleo `o_dente_do_papel_nao_atravessa_a_tinta_grossa` verde COM a marca; SEM ela 0.573 (limite 0.10) — sonda varrendo `CORPO_QUE_ENCHE` 1…0.01 não mudou nada: a pilha do Wet Dabber nunca chega à altura do `ReliefFields` (o corpo vive na grelha Wet), então espessura sozinha não distingue Óleo Wet de Digital.

**POR QUE PAROU:** a marca dentro de `Layer` parte os formatos postcard posicionais: `macro_track::tests::o_documento_da_macro_tem_a_versao_do_seu_formato` ficou vermelho (`VERSAO_DO_DOCUMENTO`) e os leitores de migração congelados (`ProjectFileV95`, `ProjectFileV128`, `PaintedDocumentSemPapel`, `shells/desktop/src/project_migrate.rs`) embutem o `LayerStack` VIVO — qualquer campo em `Layer` corrompe toda leitura de schema antigo.

**PLANO (próxima janela):** a marca vira campo FINAL de `PaintedDocument` (ex.: `tintas_molhadas: BTreeSet<RtLayerId>`), com tipo legado congelado para o layout anterior (precedente: papel/vidros, degrau 183→184, `PaintedDocumentSemPapel`); `PROJECT_SCHEMA` 217→218 contado por `python3 scripts/schema-recount.py` (escada e tripla em ficheiros irmãos, três sítios); a migração lê o degrau anterior (vigiar a catraca `the_shell_only_shrinks`: o código de migração pode ter de morar fora da shell); macro `VERSAO_DO_DOCUMENTO` 1→2 lendo v1; a ferramenta guarda o conjunto (não o `Layer`). Conferir: undo/redo da marca; meio misto numa camada (traço Digital em camada Wet mantém o enterro antigo — documentar como conhecido); o produtor GPU herda de graça (`height_at` é a única porta); Watercolor fica como estava.

## FILA restante (da onda anterior, reordenar medindo)

1. Relevo do papel no Digital (acima).
2. (4) Pastel parecer pastel, não spray — oráculo do produto do dono ANTES; simular; o dono faz a triagem.
3. (5) Abertos herdados: Wet Wash Mix (grão não aparece + ~4× lento) · Splash (o dono olha) · filme do Óleo Real (SÓ por ordem) · `SPRAY_FIXO_MAX` não re-medido com Wet blend · painel antigo a 245 px ainda corta Strength/Size.
4. (6) Papel de imagem sem dente quando há Grain próprio; `uma_passagem.rs` lê o grão cru.
5. (7) Memória do 1.º traço (dono 07/11, «depois do CI»): conferir o CI — último `noite` no main 08/11 success, 07/11 failure.

## Avisos

- Fotos: Xvfb próprio, `import -window`, nunca `spectacle`; foto ANTES do smoke; cena `PH2D_COMPOSITE_SMOKE=1` é sobreposta pelas definições gravadas (usar `XDG_CONFIG_HOME`/`XDG_DATA_HOME` vazios).
- No fecho da LINHA (não agora): clippy/fmt das crates tocadas 1×, `rm -rf target/*/incremental`, handoff de integração.
