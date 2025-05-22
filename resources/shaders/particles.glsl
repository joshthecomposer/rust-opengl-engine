// VERTEX_SHADER
#version 460 core
layout (location = 0) in vec3 a_pos;
layout (location = 1) in vec2 a_tex_coords;

uniform mat4 model;
uniform mat4 view;
uniform mat4 projection;

out vec2 tex_coords;

void main() {
    gl_Position = projection * view * model * vec4(a_pos, 1.0);
	tex_coords = a_tex_coords;
}

// FRAGMENT_SHADER
#version 460 core

in vec2 tex_coords;

uniform float particle_alpha;
uniform sampler2D texture1;
uniform bool has_tex;

out vec4 FragColor;

void main() {
	vec4 color = vec4(1.0, 1.0, 1.0, particle_alpha);

	if (has_tex) {
		color = texture(texture1, tex_coords);
	}

    FragColor = vec4(color.rgb * vec3(2.0, 2.0, 2.0), color.a * particle_alpha);
}
