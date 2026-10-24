# 11 — Um script renomeia um objecto a correr (24/10/2026)

> Decisão do dono 24/10, pergunta (a) de 22/10 ([08](08_scripts_tocam_propriedades.md) §«Perguntas»): **sim**.

## 1. A lei

- `ph2d.set(e, "name", "Porta Aberta")` / `ph2d.get(e, "name")` — o `node.name = …` do Godot, o
  `gameObject.name` do Unity e o `part.Name` do Roblox: o nome é uma **propriedade**, não um verbo à parte.
  `"name"` entra em `api_table::RUNTIME_PROPERTIES` (ao lado de `sprite.image`), por isso a completação, o
  `ph2d.properties` e o lint do editor já o conhecem.
- É um **override da corrida**, `ph2d_ecs::RunName` (`name.rs`), lido por `name_in_run`:
  - não é registado nem serializado — não entra no `Ctrl+Z`, no gravar, nem no `FieldDrive`;
  - o `Name` do documento fica intocado: ele é a IDENTIDADE (`stable_name_id`, ligações da timeline, juntas,
    `stable_id_named`) — mudá-lo a correr partiria essas ligações;
  - o rebobinar (`script_bridge::rewind` → `script_name::rewind`) devolve o nome do artista;
  - escrever o nome do artista tira o override; `""` e um não-texto são erro com o porquê.
- **Quem vê o nome novo:** `ph2d.object` e `ph2d.find_by_name` (a procura da pista, `lane.rs::find`, a partir do
  tique seguinte — a escrita pousa no fim do tique, como todo `ph2d.set`). O nome velho deixa de achar
  (o `get_node` do Godot também falha depois de um rename).
- **Quem NÃO vê:** a Hierarquia — ela mostra o DOCUMENTO (lei escrita em `build_hierarchy_snapshot`; o Godot
  mostra a corrida numa árvore remota à parte); as referências autoradas por nome (`camera_follow.target` …)
  seguem o objecto pelo documento, não se perdem.

## 2. Provas

- `ph2d-app-components` `script_name::tests` (2): renomear, achar pelo nome novo e não pelo velho, documento
  intocado, fora do livro e do registo, rebobinar; voltar ao nome do artista; `""` e `3` com o porquê.
- **RED medido:** com a procura a ler só o `Name`, o 1.º teste reprova («achado pelo nome novo»).

## 3. Smoke (24/10)

- **Cena 8** (`PH2D_CODE_SMOKE=8`) ganhou o passo: ao abrir, o `aventura.luau` faz
  `ph2d.set(self.door, "name", "Door Open")` e escreve-o no console; a câmera procura a porta por
  `ph2d.find_by_name("Door Open")` e só aproxima quando a acha — renomear partido = câmera parada (falha visível).
  A Hierarquia continua «Door» (a lei do §1).
- Gate: `code_smoke_props::tests::a_porta_abre_pelo_nome_…` (nome da corrida «Door Open», documento «Door»,
  câmera a 5 m, rebobinar devolve «Door»). Mutação «renomear para outro nome» → vermelho.
- Fotos: [`conduz_propriedades/`](ferramentas/conduz_propriedades/) (01–06 refeitas;
  `foto_07_console_door_open.png` = o console com a frase).
