#version 400 core

out vec4 FragColor;

in vec2 TexCoords;
in vec4 FragPosLightSpace;
in vec3 Normal;
in vec3 FragPos;
in vec4 Color;

uniform sampler2D texture_diffuse1;
uniform sampler2D shadowMaps[5];

struct AmbientLight {
    vec3 color;
    float intensity;
};
uniform AmbientLight ambientLight;

struct LightSource {
    vec3 position;
    vec3 color;
    float intensity;
};
uniform LightSource lightSources[5];

struct Fog {
    vec3 color;
    float density;
};
uniform Fog fog;

float ShadowCalculation(vec4 fragPosLightSpace, int lightIndex) {
    vec3 projCoords = fragPosLightSpace.xyz / fragPosLightSpace.w;
    projCoords = projCoords * 0.5 + 0.5;
    float closestDepth = texture(shadowMaps[lightIndex], projCoords.xy).r;
    float currentDepth = projCoords.z;
    float shadow = currentDepth > closestDepth + 0.005 ? 1.0 : 0.0;
    return shadow;
}

void main() {
    vec3 ambient = ambientLight.color * ambientLight.intensity;

    vec3 norm = normalize(Normal);
    vec3 lighting = ambient;

    for (int i = 0; i < 5; ++i) {
        vec3 lightDir = normalize(lightSources[i].position - FragPos);
        float diff = max(dot(norm, lightDir), 0.0);
        vec3 diffuse = lightSources[i].color * diff * lightSources[i].intensity;
        float shadow = ShadowCalculation(FragPosLightSpace, i);
        lighting += (1.0 - shadow) * diffuse;
    }

	vec3 color = Color.rgb;
    if (texture(texture_diffuse1, TexCoords).rgb != vec3(1.0, 1.0, 1.0)) {
        color *= texture(texture_diffuse1, TexCoords).rgb;
    }

	color *= lighting;



    // Apply fog
    float distance = length(FragPos);
    float fogFactor = exp(-fog.density * distance);
    fogFactor = clamp(fogFactor, 0.0, 1.0);
    vec3 finalColor = mix(fog.color, color, fogFactor);

    FragColor = vec4(finalColor, Color.a);
}
