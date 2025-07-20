#version 330 core

in vec2 vTexCoord;
out vec4 fragColor;

uniform sampler2D uTexture;
uniform float uPixelSize;
uniform float uSharpness;

void main() {
    vec2 uv = vTexCoord;
    
    // Create pixel grid effect
    vec2 pixel = floor(uv * uPixelSize) / uPixelSize;
    vec4 color = texture(uTexture, pixel);
    
    // Apply sharpening filter
    vec4 neighbor1 = texture(uTexture, pixel + vec2(1.0/uPixelSize, 0.0));
    vec4 neighbor2 = texture(uTexture, pixel + vec2(-1.0/uPixelSize, 0.0));
    vec4 neighbor3 = texture(uTexture, pixel + vec2(0.0, 1.0/uPixelSize));
    vec4 neighbor4 = texture(uTexture, pixel + vec2(0.0, -1.0/uPixelSize));
    
    // Unsharp mask
    color.rgb = color.rgb + uSharpness * (4.0 * color.rgb - neighbor1.rgb - neighbor2.rgb - neighbor3.rgb - neighbor4.rgb);
    
    // Clamp to prevent overshoot
    color.rgb = clamp(color.rgb, 0.0, 1.0);
    
    // LCD color correction (slightly cooler)
    color.rgb *= vec3(0.95, 1.0, 1.05);
    
    fragColor = color;
} 