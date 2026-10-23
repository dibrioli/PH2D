# W18 — a macro é um OBJECTO, e vive numa biblioteca (PLANO, antes do código)

> Pedido: [PROMPT_W18](handoffs/PROMPT_W18_biblioteca_de_macros_2026-10-23.md) («criação, salvamento, exportação,
> importação de macros»). Base: `972f411a6f` + a W17 por integrar. Mapas lidos nesta sessão (4 exploradores,
> 23/10): o tocador do Painter, o do Motion, o painel de Assets, o documento do Motion.

## 0. As decisões do dono (23/10, verbatim)

| # | Pergunta | Resposta |
|---|---|---|
| 1 | Para quê | «As duas coisas» — receita pessoal **e** ficheiro para enviar a outra pessoa |
| 2 | O que leva | «a macro grava a criação da imagem ou outro objeto do projeto» — autossuficiente |
| 3 | Onde já há desenho | «Escolho na hora» |
| 4 | Onde | «lista no painel de macro com a possibilidade de salvar como asset no painel de asset» |
| 5 | Por cima de imagem de OUTRO tamanho | «tamanho original, mas não no centro. a macro deve ser objeto próprio com posição rot e scale próprios. deve ser criada como os outros objetos são criados no + da hierarchy e deve ter seu gizmo como um objeto vazio» |
| 6 | As pinceladas por cima | «Entram na imagem de baixo» (o objecto Macro diz ONDE, em que ângulo e tamanho) |
| 7 | O que a macro contém | «a macro não será só imagem. Tudo na game engine pode criar macros. Por exemplo: posso começar com uma pintura de um player e depois acrescentar um collider e depois simular a física. tudo pode ser macro e a macro pode conter qualquer coisa. Logo a macro é um objeto completo e independente» |
| 8 | O Motion também | «Sim, igual» |

## 1. Estado da arte (de conhecimento — nenhum destes está instalado como oráculo)

| App | O que faz | O que foi TENTADO e abandonado / a dor conhecida | Lição para nós |
|---|---|---|---|
| Photoshop *Actions* (`.atn`) | painel com *sets*; *Save/Load Actions* exporta um set; um passo «Make Document» cria o documento, sem ele a acção corre no activo | passos que nomeiam camadas/pincéis por NOME partem noutra máquina («The brush X is not currently available»); os sets vivem fora do PSD | a macro leva o que usa DENTRO; o que não leva recusa-se em voz alta |
| Illustrator *Actions* (`.aia`) | igual ao PS | ferramentas não gravadas são OMITIDAS em silêncio | cada porta DECLARA-SE (já é a lei do ADR-0186, `Recordability`) |
| Corel Painter *Scripts* | grava a sessão incluindo o *New* da tela; «Record Initial State»; toca noutra tela | o script sem estado inicial toca outra coisa noutro documento | o mais perto da decisão 2: a criação faz parte da gravação |
| Krita *Macro recorder* | — | **REMOVIDO no Krita 3.0** (2016): gravava acções por ferramenta, frágil, ninguém o mantinha | gravar COMANDOS RESOLVIDOS (o que o ADR-0186 já faz), com gate por porta |
| Blender *Asset Browser* | «Mark as Asset» por bloco de dados; bibliotecas = PASTAS no disco (preferências) + «Current File» | assets sem dependências embutidas («Pack») partem ao abrir noutra máquina | as DUAS casas: a biblioteca do utilizador (pasta) e os assets do PROJECTO |
| Godot *PackedScene* / Unity *Prefab* | um objecto completo e independente que se instancia com transformada própria | — | o objecto Macro é isto, mais o TEMPO: uma cena que se constrói a si mesma |
| Rive / After Effects | sem macros (AE tem *Animation Presets* `.ffx`: aplicam-se a uma camada existente) | o `.ffx` refere efeitos por nome | — |

**O que nos separa:** nenhum deles tem um objecto que **é** a gravação. Aqui o objecto Macro tem posição,
rotação e escala; aplicá-lo cria-o (o que ele construiu nasce dentro dele) ou encaixa-o por cima de um
objecto existente (as pinceladas entram na imagem de baixo, onde o gizmo dele está).

## 2. O desenho — uma porta por pergunta

### 2.1 «O que é um objecto Macro?» — o `ph2d_ecs::Macro` numa entidade com transformada
- **Nenhum marcador novo.** Um objecto Macro = uma entidade com `Transform` + `Name` + `Macro`. A lei de hoje
  já é «a macro grava o objecto que a tem» (o Painter grava a sprite dona); passa a ser «o objecto que a tem
  **e os filhos dele**». A sprite com `Macro` de antes da W18 é um objecto Macro de um só (compatível ao bit).
- **Add ▸ Macro** no «+» da Hierarchy: `ph2d_app_macros::object_add::ENTRIES` (molde de
  `motion_object_add.rs`): `spawn_root` + `Macro::default()`. Sem visual próprio ⇒ o gizmo é o do objecto vazio
  (a mesma regra que já pinta o vazio — conferir no código que o gizmo decide por «não tem visual», não por tipo).
- ⛔ Recusa: um tipo `MacroObjeto` à parte. Duas verdades («é macro?» pelo marcador e pelo componente)
  divergiriam no primeiro objecto que tivesse uma e não a outra.

### 2.2 «Quem grava?» — `Familias::quem_grava` ganha o objecto Macro
- Objecto Macro escolhido (ou qualquer descendente dele) + Gravar ⇒ arma **o objecto Macro**. Enquanto arma:
  o Painter grava a imagem que pinta SE ela é descendente dele (trilha com `target` = `StableId` dela); o
  Motion grava o objecto Motion em edição SE é descendente. Fora dele: recusa nomeada
  (`panel.macros.refusal.outside_the_macro`).
- Uma imagem ou um Motion criados ENQUANTO grava nascem filhos do objecto Macro (o pai da criação é ele) e a
  trilha deles abre no nascimento: o estado inicial é o objecto acabado de nascer (a tela de W×H vazia) —
  **a criação está na macro** (decisão 2). O instante de nascimento é `Track::inicio` (§2.4).
- Sem objecto Macro (a sprite pintada directamente, como até hoje): a sprite é o seu próprio objecto Macro.

### 2.3 «O que o objecto Macro pode conter?» — as famílias que gravam, e as que se RECUSAM em voz alta
Hoje gravam Painter e Motion. A decisão 7 (collider, física, «qualquer coisa») pede uma família **mundo**:
cada passo do undo dentro da sub-árvore (componente posto/tirado/mudado, objecto nascido/apagado, pai) como
diferença de linhas do `WorldSnapshot`, com os `StableId` locais. ⚠️ O nó dela é a REPRODUÇÃO: hoje um palco
mostra a gravação sem tocar no documento (`ph2d-preview-drive` cobre factos concretos, não «um objecto que
ainda não nasceu»). Isso é um plano próprio (medir o custo de um mundo-palco vs. escrita conduzida), e é
**a W19**. Na W18 a porta existe e declara-se: `Door { "mundo.*", Refused { why: "W19" } }`, e o painel diz
ao artista o que não ficou gravado (nunca omissão calada — a dor do Illustrator).

### 2.4 «Como a macro NASCE?» — a ficha de nascimento de cada trilha (núcleo, `ph2d-macro`)
- `Track::inicio: MacroTime` (etiqueta 6; ausente = 0): o instante em que o objecto da trilha nasce.
- `Track::objeto: Option<FichaDoObjeto>` (etiqueta 7): `nome`, a transformada LOCAL ao objecto Macro (pos,
  rot, escala) e o pai (o índice de outra trilha, ou o próprio objecto Macro). Ausente = o objecto Macro É o
  alvo (o caso de antes da W18).
- O TAMANHO e o conteúdo inicial não se repetem aqui: vivem no estado inicial da família (`Track::initial`:
  o `PaintedDocument` com `size` no Painter, o texto do grafo no Motion). ⛔ Recusa: um `w×h` no cabeçalho —
  duas verdades para o tamanho da tela.
- `exige "objeto"` (MINOR 3) só quando alguma trilha tem ficha: um leitor 1.2 recusa em voz alta em vez de
  tocar a pintura no objecto errado.

### 2.5 «Que bytes saem para o disco?» — o ficheiro `.ph2dmacro` (ADR novo, §3)
- `ficheiro = codec::write(MacroDoc)` — o MESMO formato que o componente guarda, mais duas etiquetas de topo
  só do ficheiro: `4 modo` (o índice de `ModoDaMacro`: um tutorial «fazer» enviado a um aluno abre «fazer») e
  `5 nome`. Uma porta: `ph2d_macro::ficheiro::{escreve, le}`; o componente nunca as escreve.
- **Autossuficiente, e PROVADO:** antes de exportar, `ficheiro::confere` toca cada trilha até ao fim num
  tocador limpo (Painter `MacroEmCena::novo`, Motion `TocadorDoMotion::novo`). Falhou ⇒ não exporta, diz
  porquê. (Os presets e texturas já vão embutidos nos passos e no inicial — medido no mapa do Painter: não
  há lookup por nome em disco.)
- **O Motion leva SÓ o grafo dele:** o `initial` de hoje é o documento do projecto INTEIRO (todos os
  objectos). Ao exportar, `inicial::so_do_dono(bytes, dono)` reescreve-o com `MotionDoc::import_owner` para
  um documento só com os nós do dono (e `Alvo::Existente` remapeado pelo mapa devolvido). ⛔ Sem isto, um
  ficheiro enviado a outra pessoa levaria o projecto todo dela.

### 2.6 «Onde moram as macros?» — duas casas, a do Blender
1. **Biblioteca do utilizador** (todas as janelas, todos os projectos): pasta
   `PH2D_MACROS_DIR` › `$XDG_DATA_HOME/ph2d/macros` › `~/.local/share/ph2d/macros`, um `<nome>.ph2dmacro`
   por macro — o molde de `ph2d-brush-presets/src/biblioteca.rs` (`pasta_do_utilizador`, `nome_livre`,
   `gravar`, `apagar`). O nome é o do ficheiro; renomear = renomear o ficheiro. Porta única:
   `ph2d_app_macros::biblioteca`.
2. **Assets do projecto** («guardar como asset»): o objecto Macro guarda-se como **receita** — uma entidade
   com `Name` + `Macro` + `MasterRoot` (a porta do «Make Prefab», ADR-0164). Viaja no `.ph2dproj` sem degrau
   de schema (os dois componentes já estão registados). O índice de assets mostra-a com o tipo **Macro**
   (`AssetKind::Macro`, derivado: um mestre COM `Macro`) — ⛔ recusa: um `AssetRef::Macro` com loja própria
   (segunda verdade sobre «que assets existem»; o índice é derivado do mundo, lei do `asset_index_build`).

### 2.7 «Aplicar» — uma porta, dois modos (`ph2d_app_macros::aplicar`)
- **Novo** (decisão 3a): nasce o objecto Macro no centro da vista (ou onde se largou o cartão) com o `Macro`
  da receita; cada trilha com ficha faz nascer o seu objecto filho pela porta da família —
  Painter: `ph2d_image_import::spawn_rgba` com a imagem do estado inicial; Motion: `object_add` + os nós do
  `initial` importados para o `StableId` novo (`import_owner`). Depois, **o fim da reprodução** instala-se
  como o documento do editor (a imagem acabada, o grafo acabado — como depois de Parar), e os `target` e
  `Alvo::Existente` da `Macro` instalada reescrevem-se para os ids novos (`remapeia`, uma por família).
- **Por cima** (decisão 3b + 5 + 6): o objecto Macro nasce sobre o objecto escolhido (Painter: uma imagem;
  Motion: um objecto Motion) e a trilha passa a ter esse objecto como alvo. Painter: a pintura entra na
  imagem de baixo **no tamanho original**, onde o gizmo do objecto Macro está (posição, rotação, escala):
  um `Encaixe` afim px-da-gravação → px-do-alvo, aplicado na porta única das posições (`passo::abs`) e na
  escala do pincel. Motion: os nós da macro juntam-se aos do objecto (`import_owner` para o dono existente).
  O estado inicial da trilha passa a ser o alvo como ele está ao aplicar (`inicial::captura` dele).
- **Compatível** = a família da trilha aceita o alvo (Painter: imagem com documento; Motion: objecto Motion).
  Uma macro com trilhas de duas famílias por cima de um objecto só: recusa nomeada (`…apply.over_mixed`).
- Mover o objecto Macro depois muda onde a reprodução seguinte pinta (decisão 6) — o encaixe é lido da
  transformada viva a cada reprodução, nunca assado.

## 3. Contratos congelados e schema — prova por grep

- Nada do §6: `grep -rn "ph2d_macro\|ph2d-macro" crates/ph2d-nodegraph crates/ph2d-tool-registry crates/ph2d-vector`
  ⇒ vazio (medir no fecho); os gates `architecture_*contract_surface` não leem estas crates.
- `PROJECT_SCHEMA`: **0 degraus**. O ficheiro do componente leva a versão PRÓPRIA (MINOR 3), como no `189`;
  a receita usa `MasterRoot` + `Macro` já registados. `AssetKind::Macro` é do índice (derivado, não gravado).
- ADR novo: **0190 — a macro é um objecto e leva tudo o que usa** (ficheiro `.ph2dmacro`, as duas casas, o
  encaixe, a família mundo adiada com porta declarada).

## 4. UI — as quatro condições, por superfície

| Superfície | Existe | Pintada e registada | O clique chega ao barramento | A sequência leva a algum lugar |
|---|---|---|---|---|
| Add ▸ Macro (Hierarchy «+») | `object_add::ENTRIES` da família | o menu das famílias já pinta `ENTRIES` | o verbo das famílias | Gravar fica activo com ele escolhido |
| Painel MACROS ▸ secção **Biblioteca** | ids novos `macros.lib.*` | lista + verbos na `BOTOES` (pintar/registar/despachar leem a MESMA tabela) | `MacroRequest::{Guarda, Aplica{modo}, RenomeiaNaBiblioteca, ApagaDaBiblioteca, Exporta, Importa, GuardaComoAsset}` | aplicar ⇒ objecto na Hierarchy e a reprodução no clipe |
| Aplicar: «Criar novo» / «Por cima de ‹nome›» | dois botões na linha escolhida | o «Por cima» só com alvo compatível; senão a frase diz porquê | `Aplica{modo}` | |
| Exportar…/Importar… | `ph2d_app_host::modal::{save_file, pick_file}` (a porta obrigatória) | — | — | o ficheiro aparece na lista / no disco |
| Painel de Assets: cartão Macro | `AssetKind::Macro` | chip de filtro + rótulo `asset.kind.macro` | duplo-clique ⇒ aplicar novo; arrasto ⇒ novo no vazio, por cima sobre imagem/Motion | o mesmo `aplicar` (uma porta) |

## 5. Gates — vermelhos primeiro, e a fixtura que contém o fenómeno

| Gate | Fixtura | Vermelho antes porque |
|---|---|---|
| `o_ficheiro_vai_e_volta_ao_bit` (núcleo) | MacroDoc com ficha, inicio, edições, modo, nome | `ficheiro` não existe |
| `uma_gravacao_sem_ficha_continua_a_ser_1_2_ao_bit` | a fixtura da W16 | protege os ficheiros gravados |
| `um_leitor_1_2_recusa_a_ficha_em_voz_alta` | ficheiro com `exige "objeto"` | — |
| `o_motion_exportado_leva_so_o_grafo_do_dono` | documento com 2 objectos Motion, macro de um | `so_do_dono` não existe (o ficheiro levaria os dois) |
| `exportar_recusa_o_que_nao_toca_ate_ao_fim` | trilha com passo estragado | `confere` não existe |
| `a_biblioteca_guarda_lista_renomeia_apaga` | pasta temporária (`PH2D_MACROS_DIR`) | — |
| `aplicar_novo_faz_nascer_a_imagem_do_tamanho_gravado` | gravação de uma tela 96×64, aplicada noutro mundo | o objecto não nasce |
| `aplicar_novo_mostra_o_fim_da_reproducao` | idem: pixels do editor = os do fim do palco | |
| `por_cima_pinta_no_tamanho_original_onde_o_gizmo_esta` | traço gravado em 64×64, alvo 128×128, Macro em (+32px, 0), escala 1: o traço cai deslocado 32 px, mesma largura | o `Encaixe` não existe (cairia esticado ×2) |
| `por_cima_com_rotacao_e_escala` | rotação 90°, escala 2: o traço roda e a largura do pincel dobra | |
| `mover_o_objeto_macro_muda_onde_a_proxima_reproducao_pinta` | | o encaixe assado ficaria |
| `o_motion_por_cima_junta_os_nos` | objecto Motion com 2 nós + macro de 3 nós | |
| `a_receita_aparece_como_macro_no_indice` | mestre com `Macro` | `AssetKind::Macro` não existe |
| `gravar_dentro_do_objeto_macro_e_recusar_fora` | Macro com imagem filha; imagem fora | |
| `a_porta_mundo_declara_se_recusada` | censo das portas | |
| `every_word_this_panel_shows_comes_from_the_string_table` (painel MACROS, Assets) | as chaves novas | o censo já existe |

Provas de mutação no fecho (agente `mutacao`, 4 controlos): o `Encaixe` (trocar a ordem rotação/escala; tirar
a escala do pincel), `so_do_dono` (deixar passar outro dono), `confere` (não tocar até ao fim).

## 6. A cena de smoke `PH2D_MACRO_SMOKE=7` — números MEDIDOS antes da mensagem ao dono

O caminho inteiro, conduzido pelo app e fotografado em cada passo: (1) Add ▸ Macro; (2) Gravar; (3) nova
imagem dentro dele, duas pinceladas; (4) Parar; (5) Guardar na biblioteca; (6) Exportar para
`$PH2D_MACROS_DIR/../exportada.ph2dmacro`; (7) projecto novo (mundo limpo); (8) Importar; (9) Aplicar ▸ Criar
novo — a imagem nasce e as pinceladas tocam; (10) uma imagem maior e Aplicar ▸ Por cima, com o objecto Macro
rodado — as pinceladas entram nela no tamanho gravado, rodadas. Os números (tamanhos, texels pintados em cada
aplicação, a posição do traço no alvo) entram aqui pela sonda headless ANTES do texto do smoke.

## 7. Ordem de execução (série só onde um passo muda o seguinte)

1. Núcleo: `Track::{inicio, objeto}`, `ficheiro`, gates de formato. (Base de tudo.)
2. Em paralelo depois de 1: biblioteca no disco · `so_do_dono` + `remapeia` do Motion · `Encaixe` do Painter.
3. `aplicar` (novo e por cima) sobre 2.
4. Objecto Macro: Add ▸ Macro, `quem_grava` com descendentes, filhos nascidos a gravar.
5. UI: secção Biblioteca, Exportar/Importar, `AssetKind::Macro` + arrasto.
6. Cena `=7`, sonda, fotografias. ADR 0190, handoff.
7. Fecho W17 + W18 juntas (um portão batched, as mutações das duas).

## ⛔ Recusas MEDIDAS / decididas

| Recusa | Porquê |
|---|---|
| marcador `MacroObjeto` | duas verdades para «é macro?» |
| `w×h` no cabeçalho da macro | o tamanho já está no estado inicial |
| loja própria de assets de macro | o índice é derivado do mundo; a receita (`MasterRoot`) já viaja no projecto |
| exportar o `initial` do Motion como está | leva o grafo de todos os objectos do projecto |
| esticar a pintura à imagem de baixo | decisão 5 do dono: tamanho original, onde o gizmo está |
| a família mundo na W18 | a reprodução de um objecto que ainda não nasceu precisa do seu plano (W19); a porta declara-se já |
