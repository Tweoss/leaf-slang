float4 bilinearSample(Texture2D<float4> input, float2 uv) {
  float width;
  float height;
  input.GetDimensions(width, height);
  float2 size = float2(width, height);

  // Center onto the center of each uv cell by subtracting .5
  // float2 coord = (uv.mul(float2(2000, 2000)));
  float2 coord = max(uv.mul(size) - float2(0.5), float2(0));

  uint2 m = uint2(floor(coord));
  float2 w = coord - m;
  float2 wi = float2(1.0) - w;
  float4 v00 = input[m];
  float4 v01 = input[min(m + uint2(0, 1), uint2(size) - uint2(1))];
  float4 v10 = input[min(m + uint2(1, 0), uint2(size) - uint2(1))];
  float4 v11 = input[min(m + uint2(1, 1), uint2(size) - uint2(1))];
  
  // return v00;
  return (v11.scale(w.x) + v01.scale(wi.x)).scale(w.y) + (v10.scale(w.x) + v00.scale(wi.x)).scale(wi.y);
}

