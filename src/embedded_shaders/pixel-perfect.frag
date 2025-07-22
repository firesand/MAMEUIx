#version 330
precision mediump float;

in vec2 v_texcoord;
out vec4 fragColor;

uniform sampler2D u_texture;
uniform vec2 u_resolution;

// Pixel-perfect scaling with integer scaling
void main() {
    vec2 uv = v_texcoord;
    vec2 pixel = 1.0 / u_resolution;
    
    // Round to nearest pixel for perfect scaling
    vec2 texel = floor(uv * u_resolution) / u_resolution;
    vec4 color = texture(u_texture, texel);
    
    fragColor = color;
}
