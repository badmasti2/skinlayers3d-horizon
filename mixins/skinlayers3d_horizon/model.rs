// Original Horizon implementation of pixel-depth skin overlays. MIT licensed.
// Does not contain or translate code from tr7zw's Java mod.
use super::{EntityModelDocument, Part, Polygon};

pub(super) fn model(document: &EntityModelDocument, key: &str) -> Option<Part> {
    let mut part = document.0.get(key).map(|m| Part::parse("root", m))?;
    if matches!(key, "minecraft:player#main" | "minecraft:player_slim#main" | "minecraft:player_head#main") {
        voxelize(&mut part);
    }
    Some(part)
}

fn voxelize(part: &mut Part) {
    if matches!(part.name.as_str(), "hat" | "jacket" | "left_sleeve" | "right_sleeve" | "left_pants" | "right_pants") {
        part.polygons = part.polygons.iter().flat_map(pixel_prisms).collect();
    }
    for child in &mut part.children {
        voxelize(child);
    }
}

// Every quad retains its model-space pose. Constant texel-center UVs on the
// prism's six faces let Horizon's existing alpha test discard transparent
// pixels on its roof AND sides, including skins downloaded after model load.
// Build once at model load, with no skin I/O or per-frame mesh generation.
fn pixel_prisms(poly: &Polygon) -> Vec<Polygon> {
    fn sub(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
        std::array::from_fn(|i| a[i] - b[i])
    }
    fn cross(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
        [a[1]*b[2]-a[2]*b[1], a[2]*b[0]-a[0]*b[2], a[0]*b[1]-a[1]*b[0]]
    }
    let v = &poly.vertices;
    let steps = |a: usize, b: usize| {
        (((v[a][3]-v[b][3]).abs().max((v[a][4]-v[b][4]).abs()) * 64.0).round() as usize).clamp(1, 64)
    };
    let nx = steps(1, 0);
    let ny = steps(3, 0);
    let n = poly.normal;
    // Reject non-finite custom geometry or invalid normals.
    if v.iter().flatten().any(|x| !x.is_finite()) || n.iter().any(|x| !x.is_finite())
        || (n.iter().map(|x| x*x).sum::<f32>() - 1.0).abs() > 0.001 {
        return vec![poly.clone()];
    }
    let at = |x: f32, y: f32| -> [f32; 5] {
        std::array::from_fn(|i| v[0][i] + (v[1][i]-v[0][i])*x + (v[3][i]-v[0][i])*y)
    };
    let mut out = Vec::with_capacity(nx*ny*6);
    for y in 0..ny {
        for x in 0..nx {
            let x0 = x as f32 / nx as f32;
            let x1 = (x+1) as f32 / nx as f32;
            let y0 = y as f32 / ny as f32;
            let y1 = (y+1) as f32 / ny as f32;
            let center = at((x0+x1)*0.5, (y0+y1)*0.5);
            let mut front = [at(x0,y0), at(x1,y0), at(x1,y1), at(x0,y1)];
            for p in &mut front { p[3] = center[3]; p[4] = center[4]; }
            let mut back = front;
            for p in &mut back {
                for i in 0..3 { p[i] -= n[i]*0.75; }
            }
            out.push(Polygon { normal: n, vertices: front });
            out.push(Polygon { normal: n.map(|x| -x), vertices: [back[3],back[2],back[1],back[0]] });
            let position = |p: [f32;5]| [p[0],p[1],p[2]];
            let orientation = cross(sub(position(front[1]),position(front[0])), sub(position(front[3]),position(front[0])))
                .iter().zip(n).map(|(a,b)| a*b).sum::<f32>().signum();
            for i in 0..4 {
                let j = (i+1)%4;
                let edge = sub(position(front[j]),position(front[i]));
                let mut normal = cross(edge,n);
                let length = normal.iter().map(|x| x*x).sum::<f32>().sqrt();
                if length <= 0.000001 { continue; }
                for a in &mut normal { *a = *a / length * orientation; }
                out.push(Polygon { normal, vertices: [front[i],back[i],back[j],front[j]] });
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn face(width: f32, height: f32) -> Polygon {
        Polygon { normal: [0.0, 0.0, 1.0], vertices: [
            [0.0,0.0,0.5,0.5,0.0],
            [width,0.0,0.5,0.5+width/64.0,0.0],
            [width,height,0.5,0.5+width/64.0,height/64.0],
            [0.0,height,0.5,0.5,height/64.0],
        ] }
    }

    #[test]
    fn classic_and_slim_faces_have_one_closed_prism_per_texel() {
        for (w,h) in [(8.0,8.0),(4.0,12.0),(3.0,12.0)] {
            let mesh = pixel_prisms(&face(w,h));
            assert_eq!(mesh.len(), (w*h) as usize * 6);
            for cell in mesh.chunks_exact(6) {
                let uv = [cell[0].vertices[0][3],cell[0].vertices[0][4]];
                // All surfaces sample the same texel, so transparent pixels
                // cannot leave opaque side walls when the GPU alpha-tests.
                for q in cell { for v in q.vertices { assert_eq!([v[3],v[4]],uv); } }
                assert_eq!(cell[0].vertices[0][2], 0.5);
                assert_eq!(cell[1].vertices[0][2], -0.25);
            }
        }
    }

    #[test]
    fn six_sides_have_outward_normals_and_consistent_winding() {
        for reverse in [false,true] {
            let mut source = face(1.0,1.0);
            if reverse { source.vertices.reverse(); }
            for q in pixel_prisms(&source) {
                let a=q.vertices[0]; let b=q.vertices[1]; let c=q.vertices[2];
                let ab=[b[0]-a[0],b[1]-a[1],b[2]-a[2]];
                let ac=[c[0]-a[0],c[1]-a[1],c[2]-a[2]];
                let cross=[ab[1]*ac[2]-ab[2]*ac[1],ab[2]*ac[0]-ab[0]*ac[2],ab[0]*ac[1]-ab[1]*ac[0]];
                let dot=cross.iter().zip(q.normal).map(|(a,b)|a*b).sum::<f32>();
                assert!(if reverse { dot < 0.0 } else { dot > 0.0 });
                let center: [f32;3]=std::array::from_fn(|i|q.vertices.iter().map(|v|v[i]).sum::<f32>()/4.0);
                let outward=[center[0]-0.5,center[1]-0.5,center[2]-0.125];
                assert!(outward.iter().zip(q.normal).map(|(a,b)|a*b).sum::<f32>()>0.0);
            }
        }
    }

    #[test]
    fn pixel_uvs_stay_at_texel_centers_including_mirrored_faces() {
        let mut poly=face(8.0,8.0);
        poly.vertices.reverse();
        for q in pixel_prisms(&poly) { for v in q.vertices {
            for uv in &v[3..] { assert!((uv*64.0-uv.mul_add(64.0,0.0).floor()-0.5).abs()<0.00001); }
        } }
    }
}
