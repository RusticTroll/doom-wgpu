pub struct Patch {
    pub width: u32,
    pub height: u32,
    pub indices: Vec<u16>,
}

impl Patch {
    pub fn new(lump: &[u8]) -> Self {
        let width: u16 = bytemuck::cast_slice(&lump[0..2])[0];
        let height: u16 = bytemuck::cast_slice(&lump[2..4])[0];

        let column_offsets: &[u32] = bytemuck::cast_slice(&lump[8..(8 + 4 * width) as usize]);
        let mut columns = Vec::with_capacity(width as usize);

        for column_offset in column_offsets {
            columns.push(parse_column(lump, (*column_offset) as usize, height));
        }

        let mut indices = Vec::with_capacity((width * height) as usize);
        for row in 0..height {
            for column in 0..width {
                indices.push(columns[column as usize][row as usize]);
            }
        }

        Self {
            width: width as u32,
            height: height as u32,
            indices,
        }
    }
}

fn parse_column(lump: &[u8], offset: usize, height: u16) -> Vec<u16> {
    let mut indices = vec![256u16; height as usize];
    let mut offset = offset;

    let mut top_delta: u8 = lump[offset];
    while top_delta != 0xFF {
        offset += 1;
        let post_length = lump[offset] as usize;
        offset += 2;

        for (pixel_offset, index) in lump[offset..offset + post_length].iter().enumerate() {
            indices[top_delta as usize + pixel_offset] = *index as u16;
        }

        offset += (post_length - 1) + 2;

        top_delta = lump[offset];
    }

    indices
}