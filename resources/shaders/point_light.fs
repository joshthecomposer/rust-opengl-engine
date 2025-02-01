#version 330 core

in vec3 light_color;
out vec4 FragColor;

void main()
{
    FragColor = vec4(light_color, 1.0);
}
