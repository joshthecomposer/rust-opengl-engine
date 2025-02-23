#version 330 core
out vec4 FragColor;

in vec2 TexCoord;

uniform sampler2D textTexture;

void main() {
    float alpha = texture(textTexture, TexCoord).r;
    FragColor = vec4(1.0, 1.0, 1.0, alpha);
}
