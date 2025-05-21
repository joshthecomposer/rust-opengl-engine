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
// #version 460 core
// // Output to 2 color attachments: 0 = main scene, 1 = bloom
// layout (location = 0) out vec4 FragColor;
// layout (location = 1) out vec4 BrightColor;
// 
// void main() {
//     vec4 color = vec4(1.0, 1.0, 1.0, 1.0); // white glowing particles
// 
//     FragColor = color;
// 
//     // Compute brightness using luminance formula
//     float brightness = dot(color.rgb, vec3(0.2126, 0.7152, 0.0722));
// 
//     // Only output to bloom if it's "bright enough"
//     if (brightness > 1.0) {
//         BrightColor = color;
//     } else {
//         BrightColor = vec4(0.0);
//     }
// }
#version 460 core
layout (location = 0) out vec4 FragColor;
layout (location = 1) out vec4 BrightColor;

void main() {
    vec4 color = vec4(3.0, 3.0, 3.0, 1.0); // HDR white
    FragColor = color;
    BrightColor = color; // always emit
}
