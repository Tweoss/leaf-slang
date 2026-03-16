@binding(0) @group(0) var input_0 : texture_2d<f32>;

@binding(1) @group(0) var output_0 : texture_storage_2d<rgba8unorm, write>;

struct GlobalParams_std140_0
{
    @align(16) offset_0 : vec2<u32>,
};

@binding(2) @group(0) var<uniform> globalParams_0 : GlobalParams_std140_0;
fn getDim_0( input_1 : texture_2d<f32>) -> vec2<u32>
{
    var width_0 : u32;
    var height_0 : u32;
    {var dim = textureDimensions((input_1));((width_0)) = dim.x;((height_0)) = dim.y;};
    return vec2<u32>(width_0, height_0);
}

@compute
@workgroup_size(16, 16, 1)
fn imagemain(@builtin(global_invocation_id) dispatchThreadID_0 : vec3<u32>)
{
    var dispatchThreadID_1 : vec2<u32> = dispatchThreadID_0.xy;
    if((any((dispatchThreadID_1 >= (getDim_0(input_0))))))
    {
        return;
    }
    var _S1 : vec3<i32> = vec3<i32>(vec3<u32>(dispatchThreadID_1, u32(0)));
    textureStore((output_0), (dispatchThreadID_1 + globalParams_0.offset_0), ((textureLoad((input_0), ((_S1)).xy, ((_S1)).z))));
    return;
}

