// VERTEX_SHADER
#version 460 core
layout (location = 0) in vec3 aPos;
layout (location = 1) in vec2 aTexCoord;

out vec2 TexCoord;

void main() {
    gl_Position = vec4(aPos, 1.0);
    TexCoord = aTexCoord;
}

// FRAGMENT_SHADER
#version 460 core
out vec4 FragColor;

in vec2 TexCoord;

uniform sampler2D textTexture;

void main() {
    float alpha = texture(textTexture, TexCoord).r;
    FragColor = vec4(1.0, 1.0, 1.0, alpha);
}
