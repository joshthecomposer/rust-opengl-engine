// VERTEX_SHADER
#version 460 core
layout (location = 0) in vec3 aPos;
layout (location = 1) in vec4 a_color;

out vec4 color;

void main() {
    gl_Position = vec4(aPos, 1.0);
    color = a_color;
}

// FRAGMENT_SHADER
#version 460 core
out vec4 FragColor;

in vec4 color;

void main() {
    FragColor = vec4(color);
}
