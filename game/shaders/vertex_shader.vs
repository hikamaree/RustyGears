#version 400 core

layout (location = 0) in vec3 aPos;
layout (location = 1) in vec3 aNormal;
layout (location = 2) in vec2 aTexCoords;
layout (location = 5) in vec4 aColor;

out vec2 TexCoords;
out vec4 FragPosLightSpace;
out vec3 Normal;
out vec3 FragPos;
out vec4 Color;

uniform mat4 model;
uniform mat4 view;
uniform mat4 projection;
uniform mat4 lightSpaceMatrices[5];

void main()
{
    TexCoords = aTexCoords;
    FragPos = vec3(model * vec4(aPos, 1.0));
    Normal = mat3(transpose(inverse(model))) * aNormal;  
    gl_Position = projection * view * vec4(FragPos, 1.0);

    for (int i = 0; i < 5; ++i) {
        FragPosLightSpace = lightSpaceMatrices[i] * vec4(FragPos, 1.0);
    }
	Color = aColor;
}
