use crate::Float;
use glmath::vector::Vector3;

#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Color(pub Vector3<Float>);

impl Color {
    pub fn from_rgb(r: u8, g: u8, b: u8) -> Self {
        Self(Vector3(
            [r, g, b].map(|val| val as Float / (u8::MAX as Float)),
        ))
    }
}
#[repr(transparent)]
pub struct VertColorp(pub (Vector3<Float>, Color));
impl VertColorp {
    pub fn from(value: [Float; 6]) -> Self {
        // six Floats in an array is exactly
        // what a VertColorp is
        Self((
            Vector3::from([value[0], value[1], value[2]]),
            Color(Vector3::from([value[3], value[4], value[5]])),
        ))
    }
    /// None if not possible
    pub fn try_from_slice(value: &[Float]) -> Option<Self> {
        if value.len() < 6 {
            None
        } else {
            unsafe {
                Some(Self((
                    Vector3::from([
                        *value.get_unchecked(0),
                        *value.get_unchecked(1),
                        *value.get_unchecked(2),
                    ]),
                    Color(Vector3::from([
                        *value.get_unchecked(3),
                        *value.get_unchecked(4),
                        *value.get_unchecked(5),
                    ])),
                )))
            }
        }
    }
    /// Same as [`Self::try_from_slice`] except we panic
    /// on index out of bounds
    pub fn from_slice(value: &[Float]) -> Self {
        Self((
            Vector3::from([value[0], value[1], value[2]]),
            Color(Vector3::from([value[3], value[4], value[5]])),
        ))
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ComponentKind {
    Triangle,
    Rectangle,
    RectangularPrism,
}

// use this responsibly please
// example: Box::from(float_data_to_u8_slice!(&data, 9))
// where Data: ([Vector3<Float>; 3], [Color; 3])
// don't do some stupid UB for the love of god
macro_rules! float_data_to_u8_slice {
    ($data:expr, $count: expr) => {
        bytemuck::must_cast_slice::<Float, u8>(
            ($data as *const _ as *const [Float; $count])
                .as_ref()
                .unwrap_unchecked(),
        )
    };
}

pub mod triangle {
    use crate::vao::default::Component;

    use super::*;

    pub type Data = ([Vector3<Float>; 3], [Color; 3]);

    pub const KIND: ComponentKind = ComponentKind::Triangle;
    pub const FLOAT_AMT: usize = 18;

    pub fn export(data: Data) -> Component {
        Component::new(KIND, unsafe { float_data_to_u8_slice!(&data, FLOAT_AMT) })
    }

    fn new0_no_indices(data: &[VertColorp]) -> Result<Vec<Data>, String> {
        // xyz rgb xyz rgb xyz rgb
        if data.len() % 3 != 0 {
            return Err(format!(
                "data has an invalid length (length %3 != 0) : length = {}",
                data.len()
            ));
        }

        //TODO: with_capacity optimization
        let mut r: Vec<Data> = vec![];
        let mut t: Data = unsafe { *([0.; FLOAT_AMT].as_ptr() as *const Data) };
        for chunk in data.chunks_exact(3) {
            for i in 0..3 {
                t.0[i] = chunk[i].0 .0;
                t.1[i] = chunk[i].0 .1;
            }
            r.push(t);
        }
        Ok(r)
    }
    /// Create many triangles at once
    /// data is your mixed triangle data
    /// right now this means x,y,z,r,g,b,
    /// (though in the future this may change)
    /// indices is optional, where you can supply
    /// them if necessary (if None, then no indices)
    #[inline(always)]
    pub fn new0(data: &[VertColorp], indices: Option<&[u32]>) -> Result<Vec<Data>, String> {
        if let Some(indices) = indices {
            if indices.len() % 3 != 0 {
                return Err(format!(
                    "indices has an invalid length (length %3 != 0) : length = {}",
                    indices.len()
                ));
            }
            let mut t: Data = unsafe { *([0.; FLOAT_AMT].as_ptr() as *const Data) };
            //TODO: with_capacity optimization
            let mut r: Vec<Data> = vec![];
            for index in indices.chunks_exact(3) {
                for i in 0..3 {
                    if let Some(val) = data.get(index[i] as usize) {
                        t.0[i] = val.0 .0;
                        t.1[i] = val.0 .1;
                    } else {
                        return Err(format!(
                            "index given reached out of bound of vertices : index = {}",
                            index[i]
                        ));
                    }
                }
                r.push(t);
            }
            Ok(r)
        } else {
            new0_no_indices(data)
        }
    }
    /// Create a triangle using 3 points and 3 colors
    pub fn new1(
        p1: Vector3<Float>,
        p2: Vector3<Float>,
        p3: Vector3<Float>,
        c1: Color,
        c2: Color,
        c3: Color,
    ) -> Data {
        ([p1, p2, p3], [c1, c2, c3])
    }
}

pub mod rectangle {
    use crate::vao::default::Component;

    use super::*;

    pub type Data = [triangle::Data; 2];
    pub const KIND: ComponentKind = ComponentKind::Rectangle;
    pub const FLOAT_AMT: usize = triangle::FLOAT_AMT * 2;

    pub fn export(data: Data) -> Component {
        Component::new(KIND, unsafe { float_data_to_u8_slice!(&data, FLOAT_AMT) })
    }

    /// Create Rectangle using 4 points and 4 colors
    pub fn new1(
        p1: Vector3<Float>,
        p2: Vector3<Float>,
        p3: Vector3<Float>,
        p4: Vector3<Float>,
        c1: Color,
        c2: Color,
        c3: Color,
        c4: Color,
    ) -> Data {
        [
            triangle::new1(p1, p2, p3, c1, c2, c3),
            triangle::new1(p1, p3, p4, c1, c3, c4),
        ]
    }
    /// Create Rectangle using 4 points and 1 color for everything
    pub fn new2(
        p1: Vector3<Float>,
        p2: Vector3<Float>,
        p3: Vector3<Float>,
        p4: Vector3<Float>,
        c: Color,
    ) -> Data {
        rectangle::new1(p1, p2, p3, p4, c, c, c, c)
    }
}

pub mod rectangular_prism {
    use crate::vao::default::Component;

    use super::*;

    pub type Data = [rectangle::Data; 6];
    pub const KIND: ComponentKind = ComponentKind::RectangularPrism;
    pub const FLOAT_AMT: usize = rectangle::FLOAT_AMT * 6;

    pub fn export(data: Data) -> Component {
        Component::new(KIND, unsafe { float_data_to_u8_slice!(&data, FLOAT_AMT) })
    }

    /// Create Rectangular Prism using 8 points and 6 colors
    /// The points are oriented:
    /// bottom left, bottom right, top right, top left,
    /// then same but back one (but of course they are just points,
    /// so if you use the same color it really doesn't matter)
    /// speaking of colors, it goes:
    /// front, back, top, bottom, left, right
    pub fn new1(
        p1: Vector3<Float>,
        p2: Vector3<Float>,
        p3: Vector3<Float>,
        p4: Vector3<Float>,
        p5: Vector3<Float>,
        p6: Vector3<Float>,
        p7: Vector3<Float>,
        p8: Vector3<Float>,
        c1: Color,
        c2: Color,
        c3: Color,
        c4: Color,
        c5: Color,
        c6: Color,
    ) -> Data {
        let r1 = rectangle::new2(p1, p2, p3, p4, c1); // front
        let r2 = rectangle::new2(p6, p5, p8, p7, c2); // back

        let r3 = rectangle::new2(p5, p1, p4, p8, c3); // left
        let r4 = rectangle::new2(p2, p6, p7, p3, c4); // right

        let r5 = rectangle::new2(p4, p3, p7, p8, c5); // up
        let r6 = rectangle::new2(p5, p6, p2, p1, c6); // down
        [r1, r2, r3, r4, r5, r6]
    }
}
