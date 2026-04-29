use crate::profiles::GeneratorKind;
use std::collections::BTreeMap;

pub struct VoxelModel {
    pub size: [u8; 3],
    pub voxels: BTreeMap<[u8; 3], u8>,
}

pub fn placeholder_model(size: [u8; 3], kind: GeneratorKind) -> VoxelModel {
    let mut m = VoxelModel {
        size,
        voxels: BTreeMap::new(),
    };
    let sx = size[0] as i32;
    let sy = size[1] as i32;
    let sz = size[2] as i32;
    match kind {
        GeneratorKind::CharacterBase => {
            ellipsoid(&mut m, sx / 2, sy / 2, sz - 18, sx / 5, sy / 4, sz / 8, 1);
            ellipsoid(&mut m, sx / 2, sy / 2, sz / 2, sx / 4, sy / 4, sz / 7, 1);
            cuboid(
                &mut m,
                sx / 2 - 3,
                sx / 2 + 3,
                sy / 2 - 3,
                sy / 2 + 3,
                sz / 2 - 24,
                sz / 2 - 4,
                1,
            );
            cuboid(
                &mut m,
                sx / 2 - 14,
                sx / 2 - 8,
                sy / 2 - 3,
                sy / 2 + 3,
                7,
                sz / 2 - 20,
                1,
            );
            cuboid(
                &mut m,
                sx / 2 + 8,
                sx / 2 + 14,
                sy / 2 - 3,
                sy / 2 + 3,
                7,
                sz / 2 - 20,
                1,
            );
            cuboid(
                &mut m,
                sx / 2 - 20,
                sx / 2 - 15,
                sy / 2 - 3,
                sy / 2 + 3,
                sz / 2 - 16,
                sz / 2 + 14,
                1,
            );
            cuboid(
                &mut m,
                sx / 2 + 15,
                sx / 2 + 20,
                sy / 2 - 3,
                sy / 2 + 3,
                sz / 2 - 16,
                sz / 2 + 14,
                1,
            );
            marker(&mut m, sx / 2 + 20, sy / 2 + 5, sz / 2 - 14);
            marker(&mut m, sx / 2 - 20, sy / 2 + 5, sz / 2 - 14);
        }
        GeneratorKind::Tool => {
            cuboid(
                &mut m,
                4,
                sx - 10,
                sy / 2 - 1,
                sy / 2 + 1,
                sz / 2 - 1,
                sz / 2 + 1,
                5,
            );
            cuboid(
                &mut m,
                sx - 20,
                sx - 6,
                sy / 2 - 6,
                sy / 2 + 6,
                sz / 2 - 4,
                sz / 2 + 4,
                6,
            );
            marker(&mut m, 12, sy / 2, sz / 2);
        }
    }
    m
}

fn cuboid(m: &mut VoxelModel, x0: i32, x1: i32, y0: i32, y1: i32, z0: i32, z1: i32, c: u8) {
    for x in x0..=x1 {
        for y in y0..=y1 {
            for z in z0..=z1 {
                put(m, x, y, z, c);
            }
        }
    }
}
fn marker(m: &mut VoxelModel, x: i32, y: i32, z: i32) {
    cuboid(m, x - 1, x + 1, y - 1, y + 1, z - 1, z + 1, 9);
}
fn put(m: &mut VoxelModel, x: i32, y: i32, z: i32, c: u8) {
    if x >= 0
        && y >= 0
        && z >= 0
        && x < m.size[0] as i32
        && y < m.size[1] as i32
        && z < m.size[2] as i32
    {
        m.voxels.insert([x as u8, y as u8, z as u8], c);
    }
}
fn ellipsoid(m: &mut VoxelModel, cx: i32, cy: i32, cz: i32, rx: i32, ry: i32, rz: i32, c: u8) {
    for x in cx - rx..=cx + rx {
        for y in cy - ry..=cy + ry {
            for z in cz - rz..=cz + rz {
                let dx = (x - cx) as f32 / rx.max(1) as f32;
                let dy = (y - cy) as f32 / ry.max(1) as f32;
                let dz = (z - cz) as f32 / rz.max(1) as f32;
                if dx * dx + dy * dy + dz * dz <= 1.0 {
                    put(m, x, y, z, c);
                }
            }
        }
    }
}

fn chunk(id: &[u8; 4], content: &[u8], children: &[u8]) -> Vec<u8> {
    let mut out = Vec::new();
    out.extend_from_slice(id);
    out.extend_from_slice(&(content.len() as u32).to_le_bytes());
    out.extend_from_slice(&(children.len() as u32).to_le_bytes());
    out.extend_from_slice(content);
    out.extend_from_slice(children);
    out
}

pub fn write_vox(model: &VoxelModel) -> anyhow::Result<Vec<u8>> {
    let mut size = Vec::new();
    for v in model.size {
        size.extend_from_slice(&(v as i32).to_le_bytes());
    }
    let mut xyzi = Vec::new();
    xyzi.extend_from_slice(&(model.voxels.len() as u32).to_le_bytes());
    for (p, c) in &model.voxels {
        xyzi.extend_from_slice(&[p[0], p[1], p[2], *c]);
    }
    let mut rgba = vec![0u8; 1024];
    let colors = [
        (1, [189, 118, 82, 255]),
        (5, [124, 77, 49, 255]),
        (6, [92, 104, 115, 255]),
        (9, [45, 119, 186, 255]),
    ];
    for (i, c) in colors {
        rgba[i * 4..i * 4 + 4].copy_from_slice(&c);
    }
    let children = [
        chunk(b"SIZE", &size, &[]),
        chunk(b"XYZI", &xyzi, &[]),
        chunk(b"RGBA", &rgba, &[]),
    ]
    .concat();
    let mut out = b"VOX ".to_vec();
    out.extend_from_slice(&150u32.to_le_bytes());
    out.extend_from_slice(&chunk(b"MAIN", &[], &children));
    Ok(out)
}
