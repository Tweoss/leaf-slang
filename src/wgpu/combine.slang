import sample;

[format("rgba8")]
layout(binding = 0) Texture2D input;
[format("rgba8")]
layout(binding = 1) WTexture2D output;

uniform uint2 offset;

[shader("compute")]
[numthreads(16, 16, 1)]
void imagemain(uint2 dispatchThreadID: SV_DispatchThreadID)
{
  uint2 inputDim = getDim(input);
  if (any(dispatchThreadID >= inputDim)) {
  	return;
  }
  output.Store(dispatchThreadID + offset, input[dispatchThreadID]);
}
