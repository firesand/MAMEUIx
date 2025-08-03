#version 330
precision mediump float;

in vec2 v_texcoord;
out vec4 fragColor;

uniform sampler2D u_texture;
uniform vec2 u_resolution;
uniform float u_time;

// CRT Royale - Advanced CRT simulation
void main() {
    vec2 uv = v_texcoord;
    vec2 pixel = 1.0 / u_resolution;
    
    // Sample the texture with nearest neighbor for pixel-perfect scaling
    vec2 texel = floor(uv * u_resolution) / u_resolution;
    vec4 color = texture(u_texture, texel);
    
    // Apply scanlines
    float scanline = sin(uv.y * u_resolution.y * 3.14159) * 0.5 + 0.5;
    scanline = scanline * 0.3 + 0.7;
    color.rgb *= scanline;
    
    // Apply slight bloom/glow
    vec4 bloom = texture(u_texture, texel + pixel * 0.5) * 0.3;
    color.rgb += bloom.rgb * 0.2;
    
    fragColor = color;
}
