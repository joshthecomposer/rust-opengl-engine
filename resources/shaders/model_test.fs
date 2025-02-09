#version 330 core
out vec4 FragColor;

in vec3 FragPos;
in vec3 Normal;
in vec2 TexCoords;
in vec4 FragPosLightSpace;

uniform sampler2D texture_diffuse1;

struct DirLight {
 	vec3 direction;
	vec3 view_pos;

	vec3 ambient;
	vec3 diffuse;
	vec3 specular;
};
uniform DirLight dir_light;

vec3 calculate_directional_light() {

// Hardcoded directional light properties
    // vec3 lightDir = normalize(vec3(0.0, 0.07071, 0.07071)); // Light coming from above and slightly to the side
    vec3 lightDir = normalize(dir_light.direction);
    vec3 lightColor = vec3(1.0, 1.0, 1.0);  // White light

	// Ambient
    vec3 ambientColor = vec3(dir_light.ambient); // Adjust this for more/less ambient light

  // Normalize the normal
    vec3 norm = normalize(Normal);

    // Diffuse shading (Lambertian reflectance)
    float diff = max(dot(norm, lightDir), 0.0);
    vec3 diffuse = diff * lightColor;

    // Combine ambient and diffuse lighting
    vec3 textureColor = texture(texture_diffuse1, TexCoords).rgb;
    vec3 finalColor = textureColor * (ambientColor + diffuse);

	// float shadow = ShadowCalculation(FragPosLightSpace);

	// return (ambient + (1.0 - shadow) * (diffuse)) * texture(texture_diffuse1, TexCoords).rgb;
	return finalColor;
}

void main() {    
	vec3 result = vec3(0.0f, 0.0f, 0.0f); 

	result += calculate_directional_light();
    // vec3 n = normalize(Normal);
    // FragColor = vec4(n * 0.5 + 0.5, 1.0); 
	// FragColor = vec4(0.0, 0.0, 1.0, 1.0);
    FragColor = vec4(result, 1.0);
	// FragColor = texture(texture_diffuse1, TexCoords);
}
