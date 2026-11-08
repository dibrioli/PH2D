# Fixtures P12 — user data no `.ase`

- `p12_todos_os_donos.ase`: escrito por `ph2d_pixel_io::ase::escrever` (teste `o_ase_com_user_data_em_todos_os_donos_vai_e_volta`;
  regrava-se com `PH2D_GRAVA_FIXTURE=1`). User data (cor + texto + mapa do artista + mapa da extensão `ph2d.p12`) em sprite,
  camada, cel, tag e tileset.
- `p12_todos_os_donos.trial.json`: o que o **Aseprite Trial 1.3.18.6-x64** (EULA, só corrido; `HOME` limpo) escreveu ao ler a
  fixture, em 2026-11-08:

  ```sh
  HOME=<pasta vazia> timeout 60 ~/.local/opt/aseprite-trial/Aseprite-trial.AppImage -b p12_todos_os_donos.ase \
      --list-layers --list-tags --list-slices --data out.json --sheet s.png
  ```

  exit 0; stdout só com a recusa de gravar a folha (Trial). Lido: `data`, `color` e `properties` da camada, do cel e da tag,
  com o mapa da extensão aninhado pelo nome. O JSON do Trial escreve `"[unsupported type]"` para `Rect` e `Vector` (limite do
  exportador de JSON, não da leitura) e não traz o user data do sprite nem do tileset (nenhum campo para eles).
  Antes da cura do escritor (um `0x2020` vazio por tile depois do do tileset) o Trial avisava
  `Unexpected chunk type 8196 when reading tileset index 0`.
