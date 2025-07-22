#version 330
precision mediump float;

in vec2 v_texcoord;
out vec4 fragColor;

uniform sampler2D u_texture;
uniform vec2 u_resolution;

// NTSC color space simulation
void main() {
    vec2 uv = v_texcoord;
    vec4 color = texture(u_texture, uv);
    
    // NTSC color matrix (simplified)
    vec3 ntsc = vec3(
        color.r * 0.299 + color.g * 0.587 + color.b * 0.114,
        color.r * -0.147 + color.g * -0.289 + color.b * 0.436,
        color.r * 0.615 + color.g * -0.515 + color.b * -0.100
    );
    
    // Convert back to RGB
    vec3 rgb = vec3(
        ntsc.x + ntsc.y * 1.140 + ntsc.z * 0.000,
        ntsc.x + ntsc.y * -0.581 + ntsc.z * -0.395,
        ntsc.x + ntsc.y * 0.000 + ntsc.z * 2.032
    );
    
    fragColor = vec4(rgb, color.a);
}
