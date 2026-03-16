run:
    just build-shader
    cargo run --release -- data

build-shader:
    slangc src/compute.slang  -target wgsl -o shader_build/compute.wgsl
    slangc src/wgpu/warp.slang  -target wgsl -o shader_build/warp.wgsl
    slangc src/wgpu/opacity.slang  -target wgsl -o shader_build/opacity.wgsl
