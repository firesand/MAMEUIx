#version 330 core

in vec2 vTexCoord;
out vec4 fragColor;

uniform sampler2D uTexture;
uniform float uCurvature;
uniform float uScanlines;
uniform float uPhosphor;

void main() {
    // Apply CRT curvature
    vec2 uv = vTexCoord - 0.5;
    float dist = length(uv);
    uv = uv * (1.0 + dist * dist * uCurvature);
    uv = uv + 0.5;
    
    // Clamp to prevent sampling outside texture
    uv = clamp(uv, 0.0, 1.0);
    
    // Sample texture
    vec4 color = texture(uTexture, uv);
    
    // Apply scanlines
    float scanline = sin(uv.y * 1000.0) * 0.5 + 0.5;
    color.rgb *= 1.0 - uScanlines * (1.0 - scanline);
    
    // Phosphor effect (persistence)
    color.rgb = mix(color.rgb, color.rgb * 0.8, uPhosphor);
    
    // Apply gamma correction for CRT
    color.rgb = pow(color.rgb, vec3(2.2));
    
    fragColor = color;
} 