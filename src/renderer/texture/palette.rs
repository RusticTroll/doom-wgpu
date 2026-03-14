use std::sync::OnceLock;

pub const PALETTE_BIND_GROUP_LAYOUT_DESC: wgpu::BindGroupLayoutDescriptor =
    wgpu::BindGroupLayoutDescriptor {
        label: Some("Palette Layout"),
        entries: &[wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Texture {
                sample_type: wgpu::TextureSampleType::Float { filterable: false },
                view_dimension: wgpu::TextureViewDimension::D1,
                multisampled: false,
            },
            count: None,
        }],
    };
pub static PALETTE_BIND_GROUP_LAYOUT: OnceLock<wgpu::BindGroupLayout> = OnceLock::new();

pub struct Palette {
    palettes: Vec<[[u8; 3]; 256]>,
    texture: wgpu::Texture,
    view: wgpu::TextureView,
    pub bind_group: wgpu::BindGroup,
}

const PALETTE_SIZE: wgpu::Extent3d = wgpu::Extent3d {
    width: 256,
    height: 1,
    depth_or_array_layers: 1,
};

impl Palette {
    pub fn new(palettes: Vec<[[u8; 3]; 256]>, device: &wgpu::Device, queue: &wgpu::Queue) -> Self {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Palette Texture"),
            size: PALETTE_SIZE,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D1,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        queue.write_texture(
            wgpu::TexelCopyTextureInfoBase {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &rgb_to_rgba(bytemuck::cast_slice(&palettes[0])),
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4 * 256),
                rows_per_image: None,
            },
            PALETTE_SIZE,
        );

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Palette Bind Group"),
            layout: PALETTE_BIND_GROUP_LAYOUT
                .get_or_init(|| device.create_bind_group_layout(&PALETTE_BIND_GROUP_LAYOUT_DESC)),
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&view),
            }],
        });

        Self {
            palettes,
            texture,
            view,
            bind_group,
        }
    }

    pub fn set_palette_index(&mut self, index: usize, queue: &wgpu::Queue) {
        queue.write_texture(
            wgpu::TexelCopyTextureInfoBase {
                texture: &self.texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &rgb_to_rgba(bytemuck::cast_slice(&self.palettes[index])),
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4 * 256),
                rows_per_image: None,
            },
            PALETTE_SIZE,
        );
    }
}

fn rgb_to_rgba(rgb_bytes: &[u8]) -> Vec<u8> {
    // The RGBA buffer will be 4/3 the size of the RGB buffer.
    let mut rgba_bytes = Vec::with_capacity(rgb_bytes.len() * 4 / 3);

    for chunk in rgb_bytes.chunks_exact(3) {
        // Push the Red, Green, and Blue bytes.
        rgba_bytes.push(chunk[0]); // Red
        rgba_bytes.push(chunk[1]); // Green
        rgba_bytes.push(chunk[2]); // Blue
        // Push the Alpha channel as opaque (255).
        rgba_bytes.push(255); // Alpha
    }

    rgba_bytes
}