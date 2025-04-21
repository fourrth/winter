#version 320 es
precision mediump float;

out vec4 outputF;
in vec4 vertexColor;

void main()
{
	outputF = vertexColor;
}