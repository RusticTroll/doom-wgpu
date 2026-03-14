use std::sync::OnceLock;

// 2D Texture, 
pub const COLORMAP_BIND_GROUP_LAYOUT_DESC: wgpu::BindGroupLayoutDescriptor =
    wgpu::BindGroupLayoutDescriptor {
        label: Some("Color Map Layout"),
        entries: &[wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Texture {
                sample_type: wgpu::TextureSampleType::Uint,
                view_dimension: wgpu::TextureViewDimension::D2,
                multisampled: false,
            },
            count: None,
        }],
    };
pub static COLORMAP_BIND_GROUP_LAYOUT: OnceLock<wgpu::BindGroupLayout> = OnceLock::new();

pub struct ColorMap {
    color_maps: Vec<[u8; 256]>,
    texture: wgpu::Texture,
    view: wgpu::TextureView,
    pub bind_group: wgpu::BindGroup,
}

const COLORMAP_SIZE: wgpu::Extent3d = wgpu::Extent3d {
    width: 256,
    height: 34,
    depth_or_array_layers: 1,
};

impl ColorMap {
    pub fn new(color_maps: Vec<[u8; 256]>, device: &wgpu::Device, queue: &wgpu::Queue) -> Self {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Color map Texture"),
            size: COLORMAP_SIZE,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::R8Uint,
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
            &bytemuck::cast_slice(&color_maps[..]),
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(256),
                rows_per_image: Some(34),
            },
            COLORMAP_SIZE,
        );

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Color Map Bind Group"),
            layout: COLORMAP_BIND_GROUP_LAYOUT
                .get_or_init(|| device.create_bind_group_layout(&COLORMAP_BIND_GROUP_LAYOUT_DESC)),
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&view),
            }],
        });

        Self {
            color_maps,
            texture,
            view,
            bind_group,
        }
    }
}