run:
    cargo run --release

build-shader:
    slangc src/compute.slang  -target wgsl -o shader_build/compute.wgsl
