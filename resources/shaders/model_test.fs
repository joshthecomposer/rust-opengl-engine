#version 330 core
out vec4 FragColor;

in vec3 FragPos;
in vec3 Normal;
in vec2 TexCoords;

uniform sampler2D texture_diffuse1;

void main() {    
    // Hardcoded directional light properties
    vec3 lightDir = normalize(vec3(10.0, 0.0, 10.0)); // Light coming from above and slightly to the side
    vec3 lightColor = vec3(1.0, 1.0, 1.0);  // White light

  // Normalize the normal
    vec3 norm = normalize(Normal);

    // Ambient lighting (uniform base lighting)
    vec3 ambientColor = vec3(0.2, 0.2, 0.2); // Adjust this for more/less ambient light

    // Diffuse shading (Lambertian reflectance)
    float diff = max(dot(norm, lightDir), 0.0);
    vec3 diffuse = diff * lightColor;

    // Combine ambient and diffuse lighting
    vec3 textureColor = texture(texture_diffuse1, TexCoords).rgb;
    vec3 finalColor = textureColor * (ambientColor + diffuse);

    FragColor = vec4(finalColor, 1.0);
	// FragColor = texture(texture_diffuse1, TexCoords);
}
