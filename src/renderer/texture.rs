use std::{sync::OnceLock, thread::sleep, time::Duration};

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
    palettes: Vec<[[u8; 4]; 256]>,
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
    pub fn new(palettes: Vec<[[u8; 4]; 256]>, device: &wgpu::Device, queue: &wgpu::Queue) -> Self {
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
            bytemuck::cast_slice(&palettes[0]),
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
}

pub const PALETTE_INDEX_BIND_GROUP_LAYOUT_DESC: wgpu::BindGroupLayoutDescriptor =
    wgpu::BindGroupLayoutDescriptor {
        label: Some("Palette Index Layout"),
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
pub static PALETTE_INDEX_BIND_GROUP_LAYOUT: OnceLock<wgpu::BindGroupLayout> = OnceLock::new();

pub struct PalettizedTexture {
    texture: wgpu::Texture,
    view: wgpu::TextureView,
    pub bind_group: wgpu::BindGroup,
}

impl PalettizedTexture {
    pub fn new(
        name: &str,
        palette_indices: Vec<u8>,
        width: u32,
        height: u32,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> Self {
        let size = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some(name),
            size,
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
            bytemuck::cast_slice(&palette_indices[..]),
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(width),
                rows_per_image: Some(height),
            },
            size,
        );

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some(&format!("{} Texture Bind Group", name)),
            layout: PALETTE_INDEX_BIND_GROUP_LAYOUT.get_or_init(|| {
                device.create_bind_group_layout(&PALETTE_INDEX_BIND_GROUP_LAYOUT_DESC)
            }),
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&view),
            }],
        });

        Self {
            texture,
            view,
            bind_group,
        }
    }
}

pub struct DepthTexture {
    texture: wgpu::Texture,
    pub view: wgpu::TextureView,
}

impl DepthTexture {
    pub fn new(device: &wgpu::Device, width: u32, height: u32) -> Self {
        let size = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Depth Texture"),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        Self { texture, view }
    }
}

pub fn parse_picture(lump: &[u8]) -> Vec<u8> {
    let width: u16 = bytemuck::cast_slice(&lump[0..2])[0];
    let height: u16 = bytemuck::cast_slice(&lump[2..4])[0];

    let column_offsets: &[u32] = bytemuck::cast_slice(&lump[8..(8 + 4 * width) as usize]);
    let mut columns = Vec::<Vec<u8>>::with_capacity(width as usize);

    for column_offset in column_offsets {
        columns.push(parse_column(lump, (*column_offset) as usize, height));
    }

    let mut indices = Vec::<u8>::with_capacity((width * height) as usize);
    for row in 0..height {
        for column in 0..width {
            indices.push(columns[column as usize][row as usize]);
        }
    }

    indices
}

fn parse_column(lump: &[u8], offset: usize, height: u16) -> Vec<u8> {
    let mut indices = vec![0u8; height as usize];
    let mut offset = offset;

    let mut top_delta: u8 = lump[offset];
    while top_delta != 0xFF {
        offset += 1;
        let post_length = lump[offset] as usize;
        offset += 2;

        for (pixel_offset, index) in lump[offset..offset + post_length].iter().enumerate() {
            indices[top_delta as usize + pixel_offset] = *index;
        }

        offset += (post_length - 1) + 2;

        top_delta = lump[offset];
    }

    indices
}
