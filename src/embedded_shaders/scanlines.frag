#version 330 core

in vec2 vTexCoord;
out vec4 fragColor;

uniform sampler2D uTexture;
uniform float uIntensity;
uniform float uFrequency;

void main() {
    vec4 color = texture(uTexture, vTexCoord);
    
    // Create scanline effect
    float scanline = sin(vTexCoord.y * uFrequency * 100.0) * 0.5 + 0.5;
    color.rgb *= 1.0 - uIntensity * (1.0 - scanline);
    
    // Add subtle darkening to simulate CRT
    color.rgb *= 0.95;
    
    fragColor = color;
} 