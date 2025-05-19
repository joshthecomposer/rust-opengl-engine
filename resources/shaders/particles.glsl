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
out vec4 FragColor;

void main() {
    vec2 frag = gl_FragCoord.xy;

    // Blood-like dark red variation
    float r = 0.5; // 0.3 to 0.5
    float g = 0.03; // 0.0 to 0.04
    float b = 0.02; // 0.0 to 0.03

    FragColor = vec4(r, g, b, 0.5);
}
