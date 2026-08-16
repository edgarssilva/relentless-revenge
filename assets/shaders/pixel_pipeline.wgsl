#import bevy_pbr::utils
#import bevy_core_pipeline::fullscreen_vertex_shader::FullscreenVertexOutput

// Bindings automatically injected by your Bevy Post-Process Render Node
@group(0) @binding(0) var screen_texture: texture_2d<f32>;
@group(0) @binding(1) var texture_sampler: sampler;

// Bindings injected if you added DepthPrepass and NormalPrepass to your camera
@group(0) @binding(2) var depth_texture: texture_depth_2d;
@group(0) @binding(3) var normal_texture: texture_2d<f32>;

// ==========================================
// 1. MODULE: QUANTIZATION HELPER
// ==========================================
fn apply_color_quantization(color: vec3<f32>) -> vec3<f32> {
    let bands = 4.0;

    // Smoothly posterize the color channels
    var quantized = floor(color * bands) / bands;

    // (Optional) You can inject retro palette restrictions here!
    return quantized;
}

// ==========================================
// 2. MODULE: SOBEL OUTLINE HELPER
// ==========================================
fn calculate_outline(uv: vec2<f32>, texel_size: vec2<f32>) -> f32 {
    // Sample depth from neighbors (Left, Right, Top, Bottom)
    let depth_center = textureSampleLevel(depth_texture, texture_sampler, uv, 0.0);
    let depth_left = textureSampleLevel(depth_texture, texture_sampler, uv + vec2<f32>(-texel_size.x, 0.0), 0.0);
    let depth_right = textureSampleLevel(depth_texture, texture_sampler, uv + vec2<f32>(texel_size.x, 0.0), 0.0);
    let depth_top = textureSampleLevel(depth_texture, texture_sampler, uv + vec2<f32>(0.0, texel_size.y), 0.0);
    let depth_bottom = textureSampleLevel(depth_texture, texture_sampler, uv + vec2<f32>(0.0, -texel_size.y), 0.0);

    // Sobel edge check math
    let depth_edge = abs(depth_left - depth_right) + abs(depth_top - depth_bottom);

    // Threshold to flag an edge (lower number = more sensitive lines)
    if depth_edge > 0.005 {
        return 0.0; // Draw black outline
    }
    return 1.0; // No outline
}

// ==========================================
// 3. MAIN FRAGMENT ENTRYPOINT
// ==========================================
@fragment
fn fragment(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
    let uv = in.uv;

    // Get screen dimensions to calculate precise pixel neighbors
    let texture_dims = vec2<f32>(textureDimensions(screen_texture));
    let texel_size = vec2<f32>(1.0 / texture_dims.x, 1.0 / texture_dims.y);

    // Step A: Grab the smooth raw 3D color rendered by Bevy
    let raw_color = textureSample(screen_texture, texture_sampler, uv).rgb;

    // Step B: Run it through your Quantization algorithm
    let quantized_color = apply_color_quantization(raw_color);

    // Step C: Check if this pixel lies on an object boundary edge
    let outline_factor = calculate_outline(uv, texel_size);

    // Step D: Mix them together! Multiplying by 0.0 blacks out the pixel for outlines
    let final_rgb = quantized_color * outline_factor;

    return vec4<f32>(final_rgb, 1.0);
}
