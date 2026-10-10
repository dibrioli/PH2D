// A saída das cascatas e da grade fina (C3) em `Rgba16Float`.
@group(0) @binding(4) var saida: texture_storage_2d<rgba16float, write>;
@group(2) @binding(4) var fina_saida: texture_storage_2d<rgba16float, write>;
// ⭐ W5c: a c0 da cascata holográfica (o nível Ultra).
@group(3) @binding(11) var c0_saida: texture_storage_2d<rgba16float, write>;
