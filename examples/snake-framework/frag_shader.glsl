#version 320 es

precision lowp float;
in vec4 vertexColor;
out vec4 fragColor;
void main()
{
	fragColor = vertexColor;
}