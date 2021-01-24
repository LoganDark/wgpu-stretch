#version 450

layout(std140, binding = 0)
uniform Globals {
	vec2 vp_size;
};

const vec2 vert[4] = vec2[4](
	vec2(1, 1),  // top right
	vec2(-1, 1), // top left
	vec2(1, -1), // bottom right
	vec2(-1, -1) // bottom left
);

void main() {
	// align 20x20 square in the top left of the window
	gl_Position = vec4(vec2(-1, 1) + vec2(10, -10) / vp_size + vert[gl_VertexIndex] / vp_size * 20, 0, 1);
}
