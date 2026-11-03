struct Overlay {
    // Physical-pixel origin and integer glyph scale.
    geometry: vec4<u32>,
    glyphs: array<vec4<u32>, 264>,
}
@group(0) @binding(0) var<uniform> overlay: Overlay;

@vertex
fn vertex(@builtin(vertex_index) index: u32) -> @builtin(position) vec4<f32> {
    let positions = array<vec2<f32>, 3>(vec2(-1.0, -1.0), vec2(3.0, -1.0), vec2(-1.0, 3.0));
    return vec4(positions[index], 0.0, 1.0);
}

@fragment
fn fragment(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {
    let point = (vec2<u32>(position.xy) - overlay.geometry.xy) / overlay.geometry.z;
    let background = vec4(0.015, 0.025, 0.035, 0.94);
    if point.x < 8u || point.y < 6u { return background; }
    let pixel = point - vec2(8u, 6u);
    let cell = pixel / vec2(6u, 9u);
    let offset = pixel % vec2(6u, 9u);
    if cell.x >= 44u || cell.y >= 6u || offset.x >= 5u || offset.y >= 7u { return background; }
    let glyph = overlay.glyphs[cell.y * 44u + cell.x];
    let bit = offset.x * 7u + offset.y;
    let word = select(glyph.x, glyph.y, bit >= 32u);
    if ((word >> (bit % 32u)) & 1u) == 0u { return background; }
    if cell.y == 0u { return vec4(0.2, 0.9, 0.8, 1.0); }
    return vec4(0.92, 0.96, 1.0, 1.0);
}
