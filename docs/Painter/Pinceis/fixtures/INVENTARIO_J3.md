Raiz: docs/Painter/Pinceis/fixtures/ (prefixo = <oraculo-dir> do exame.sh). Rebelle tem A/B; Krita/MyPaint uma foto por cena (A=B); ArtRage só A.
A-1 Hard Round: rebelle_bateria/PENCIL/digital_round_hard · krita_bateria/b__Basic-1 · mypaint_bateria/deevad/basic_digital_brush
A-2 Soft Round: rebelle_bateria/PENCIL/digital_round_soft · krita_bateria/b__Basic-3_Flow
A-3 Round Pressure: krita_bateria/b__Basic-5_Size_Opacity · mypaint_bateria/deevad/brush
A-4 Flat Block-in: rebelle_bateria/PENCIL/digital_flat · mypaint_bateria/deevad/basic_digital_knife
A-5 Pixel 1px: krita_bateria/u__Pixel_Art · mypaint_bateria/experimental/1pixel
A-6 Glaze Round: krita_bateria/f__Bristles-4_Glaze · mypaint_bateria/deevad/detail_brush_large_glazing
B-1 Graphite HB: rebelle_bateria/PENCIL/graphite_pencil_hb · krita_bateria/c__Pencil-2
B-2 Graphite 2H: rebelle_bateria/PENCIL/graphite_pencil_2h · mypaint_bateria/deevad/4H_pencil
B-3 Graphite 2B: rebelle_bateria/PENCIL/graphite_pencil_2b · mypaint_bateria/deevad/2B_pencil · mypaint_bateria/tanda/pencil-2b
B-4 Graphite 8B: rebelle_bateria/PENCIL/graphite_pencil_8b · mypaint_bateria/Dieterle/8B_Pencil_1 · mypaint_bateria/tanda/pencil-8b
B-5 Mechanical 0.5: rebelle_bateria/PENCIL/graphite_pencil_pentell
B-6 Tilted Shade: rebelle_bateria/PENCIL/digital_tilted_hard · krita_bateria/c__Pencil-6_Quick_Shade · krita_bateria/c__Pencil-5_Tilted (penciltilt † fora da bateria: NewFeaturesWetPaint/fixtures/rebelle/penciltilt.png; o Rebelle 0–63° em fixtures/rebelle_tilt/)
B-7 Rough Sketch: krita_bateria/c__Pencil-4_Soft · mypaint_bateria/ramon/Sketch_1
B-8 Colored 4B: rebelle_bateria/PENCIL/colored_pencil_4b_color
B-9 Craft Flat: rebelle_bateria/PENCIL/graphite_pencil_craft_flat
B-10 Scratch: rebelle_bateria/PENCIL/scratches_scratch
B-11 Erodible Pencil: sem oráculo (doc. PS)
C-1 Dip Pen: rebelle_bateria/INK_PEN/pen_dip_pen · mypaint_bateria/deevad/soft-dip-pen
C-2 Ink Bullet: rebelle_bateria/INK_PEN/pen_bullet
C-3 Ink Brush: rebelle_bateria/INK_PEN/brush_brush
C-4 Studio Pen: krita_bateria/d__Ink-3_Gpen · mypaint_bateria/deevad/pen
C-5 Mapping Pen: krita_bateria/d__Ink-1_Precision
C-6 Technical Pen: krita_bateria/d__Ink-2_Fineliner · mypaint_bateria/deevad/liner
C-7 Ballpoint: mypaint_bateria/deevad/ballpen
C-8 Fountain Pen: rebelle_bateria/INK_PEN/pen_fountain_round · mypaint_bateria/Dieterle/Fountain_SF_1
C-9 Reed Pen: rebelle_bateria/INK_PEN/pen_reed_pen
C-10 Brush Pen: krita_bateria/d__Ink-7_Brush_Rough
C-11 Dry Ink: krita_bateria/d__Ink-4_Pen_Rough · mypaint_bateria/classic/textured_ink
C-12 Gloop Pen: sem oráculo (doc. AR)
C-13 Ink Bleed: sem oráculo (doc. Pc; ArtRage Watercolor/HeavyBleed só A, fraco)
C-14 Gesinski Ink: sem oráculo (doc. Pc)
C-15 Wood Stick: rebelle_bateria/INK_PEN/pen_wood
D-1 Monoline: sem oráculo (doc. Pc; ArtRage InkPen/TechnicalPen só A)
D-2 Script: sem oráculo (doc. Pc; ArtRage InkPen/WritingChisel só A)
D-3 Broad Nib: rebelle_bateria/INK_PEN/pen_fountain_square · mypaint_bateria/classic/calligraphy
D-4 Shale: sem oráculo (doc. Pc)
E-1 Marker Bullet: rebelle_bateria/MARKER/marker_bullet · mypaint_bateria/classic/marker_fat
E-2 Marker Liner: rebelle_bateria/MARKER/marker_liner · mypaint_bateria/classic/marker_small
E-3 Marker Chisel: rebelle_bateria/MARKER/marker_chisel · krita_bateria/e__Marker_Chisel_Smooth
E-4 Dry Marker: rebelle_bateria/MARKER/marker_dry · krita_bateria/e__Marker_Dry
J-1 Airbrush: rebelle_bateria/AIRBRUSH/default_spray_2 · krita_bateria/b__Airbrush_Soft · mypaint_bateria/deevad/airbrush
J-2 Spray Can: rebelle_bateria/AIRBRUSH/default_spray_3 · krita_bateria/y__Texture_Spray · mypaint_bateria/deevad/spray
J-3 Splatter: mypaint_bateria/deevad/spray2 · mypaint_bateria/tanda/splatter-04
J-4 Spray Drips: sem oráculo (doc. Pc; Rebelle AIRBRUSH default_spray_4/5 por verificar)
J-5 Stipple: krita_bateria/v__Texture_Pointillism · mypaint_bateria/classic/puantilism
J-6 Grainy Shader: sem oráculo (doc. Pc)

Fotos NOVAS (da raiz da árvore):
- Rebelle (EXCLUSIVO, tela virtual + placa): linha `prefixo|TOOL|Grupo/Nome||gestos tamanhos sobre` (formato de ferramentas/lista_281.txt) → bash docs/Painter/Pinceis/ferramentas/lote_bateria.sh <lista> docs/Painter/Pinceis/fixtures/rebelle_bateria ; depois python3 -I docs/Painter/Pinceis/ferramentas/indice_bateria.py … > indice.csv. Trial expira ~06/11.
- Krita: python3 -I docs/Painter/Pinceis/ferramentas/krita_bateria.py --presets "c) Pencil-2,d) Ink-2 Fineliner" (Xvfb próprio; perfil partilhado ⇒ trate como exclusivo)
- MyPaint: python3 -I docs/Painter/Pinceis/ferramentas/mypaint_bateria.py --pincel /usr/share/mypaint-data/2.0/brushes/<pasta>/<ficheiro>.myb (livre)
- ArtRage (EXCLUSIVO, :71): bash ~/Apps/artrage/inicia.sh ; python3 -I docs/Painter/Pinceis/ferramentas/fotos_artrage.py docs/Painter/Pinceis/ferramentas/lista_artrage.tsv docs/Painter/Pinceis/fixtures/artrage_bateria --so <nome> (só A)
