// VERTEX_SHADER
#version 460 core
layout (location = 0) in vec3 a_pos;

uniform mat4 model;
uniform mat4 view;
uniform mat4 projection;

void main() {
    gl_Position = projection * view * model * vec4(a_pos, 1.0);
}

// FRAGMENT_SHADER
#version 460 core

uniform float particle_alpha;
out vec4 FragColor;

void main() {
    vec4 color = vec4(1.0, 1.0, 1.0, particle_alpha);
    FragColor = color;
}
