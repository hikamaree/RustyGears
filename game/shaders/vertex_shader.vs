#version 400 core

layout(location = 0) in vec3 aPos;
layout(location = 1) in vec3 aNormal;
layout(location = 2) in vec2 aTexCoords;

out vec2 TexCoords;
out vec3 FragPos;
out vec3 Normal;
out vec4 FragPosLightSpace[4];  // Max 4 svetla

uniform mat4 model;
uniform mat4 view;
uniform mat4 projection;
uniform mat4 lightSpaceMatrices[4];

void main()
{
    FragPos = vec3(model * vec4(aPos, 1.0));
    Normal = mat3(transpose(inverse(model))) * aNormal;
    TexCoords = aTexCoords;

    for (int i = 0; i < 4; ++i)
    {
        FragPosLightSpace[i] = lightSpaceMatrices[i] * vec4(FragPos, 1.0);
    }

    gl_Position = projection * view * vec4(FragPos, 1.0);
}
