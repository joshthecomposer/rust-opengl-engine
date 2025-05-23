// VERTEX_SHADER
#version 460 core

layout (location = 0) in vec3 aPos;
layout (location = 1) in vec2 aTexCoords;

out vec2 TexCoords;

void main()
{
    TexCoords = aTexCoords;
    gl_Position = vec4(aPos, 1.0);
}

// FRAGMENT_SHADER
#version 460 core

in vec2 TexCoords;
out vec4 FragColor;

uniform sampler2D screenTexture; // This must remain a uniform (the input image)
uniform vec2 resolution;        // This must remain a uniform (screen dimensions)

// Chromatic Aberration Intensity (Hardcoded)
// Adjust this value:
// 0.000 (no aberration)
// 0.001 (very subtle)
// 0.003 (subtle, often used)
// 0.005 (a bit more noticeable)
// Higher values will make it very obvious and potentially distracting.
const float CA_INTENSITY = 0.0001;

// FXAA Parameters (Hardcoded)
// These values are generally a good balance for FXAA 3.11
// You can play with these, but often the defaults are preferred.
//
// To make FXAA more aggressive (more smoothing, potential blurring/artifacts):
//   - Reduce FXAA_REDUCE_MIN (e.g., 0.0001)
//   - Reduce FXAA_REDUCE_MUL (e.g., 0.05)
//   - Increase FXAA_SPAN_MAX (e.g., 16.0 or 32.0)
//
// To make it less aggressive (less smoothing, fewer artifacts):
//   - Increase FXAA_REDUCE_MIN (e.g., 0.01)
//   - Increase FXAA_REDUCE_MUL (e.g., 0.2)
//   - Decrease FXAA_SPAN_MAX (e.g., 4.0)
const float FXAA_REDUCE_MIN = 0.01; // Equivalent to 0.0078125
const float FXAA_REDUCE_MUL = 0.05; // Equivalent to 0.125
const float FXAA_SPAN_MAX   = 16.0;

void main() {
    vec2 texel = 1.0 / resolution;
    vec2 currentTexCoords = TexCoords;

    vec3 rgbNW = texture(screenTexture, currentTexCoords + vec2(-1.0, -1.0) * texel).rgb;
    vec3 rgbNE = texture(screenTexture, currentTexCoords + vec2(1.0, -1.0) * texel).rgb;
    vec3 rgbSW = texture(screenTexture, currentTexCoords + vec2(-1.0, 1.0) * texel).rgb;
    vec3 rgbSE = texture(screenTexture, currentTexCoords + vec2(1.0, 1.0) * texel).rgb;
    vec3 rgbM  = texture(screenTexture, currentTexCoords).rgb; // Central sample (original color)

    vec3 luma = vec3(0.299, 0.587, 0.114); // Standard NTSC luma weights
    float lumaNW = dot(rgbNW, luma);
    float lumaNE = dot(rgbNE, luma);
    float lumaSW = dot(rgbSW, luma);
    float lumaSE = dot(rgbSE, luma);
    float lumaM  = dot(rgbM,  luma);

    float lumaMin = min(lumaM, min(min(lumaNW, lumaNE), min(lumaSW, lumaSE)));
    float lumaMax = max(lumaM, max(max(lumaNW, lumaNE), max(lumaSW, lumaSE)));

    vec2 dir;
    dir.x = -((lumaNW + lumaNE) - (lumaSW + lumaSE)); // Horizontal edge detection
    dir.y =  ((lumaNW + lumaSW) - (lumaNE + lumaSE)); // Vertical edge detection

    float dirReduce = max((lumaNW + lumaNE + lumaSW + lumaSE) * FXAA_REDUCE_MUL, FXAA_REDUCE_MIN);
    float rcpDirMin = 1.0 / (min(abs(dir.x), abs(dir.y)) + dirReduce);

    dir = clamp(dir * rcpDirMin, -FXAA_SPAN_MAX, FXAA_SPAN_MAX) * texel;

    // Sample along the detected edge direction
    vec3 result1 = 0.5 * (
        texture(screenTexture, currentTexCoords + dir * (1.0 / 3.0 - 0.5)).rgb +
        texture(screenTexture, currentTexCoords + dir * (2.0 / 3.0 - 0.5)).rgb);
    vec3 result2 = result1 * 0.5 + 0.25 * (
        texture(screenTexture, currentTexCoords + dir * -0.5).rgb +
        texture(screenTexture, currentTexCoords + dir * 0.5).rgb);

    float lumaResult2 = dot(result2, luma);

    vec3 final_fxaa_color = (lumaResult2 < lumaMin || lumaResult2 > lumaMax) ? result1 : result2;

    FragColor = vec4(final_fxaa_color, 1.0); // Adjust mix factor (0.0 to 1.0) to control FXAA strength
}
