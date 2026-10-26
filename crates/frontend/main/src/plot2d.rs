use winter_core::opengl::{GLIndexType, GLVertexType};
use winter_simple::glmath::{vector::Vector3, Element};
use winter_simple::{constructs, shapes, Drawable, IntoDrawable, Mesh};

pub struct Plot2D<V: GLVertexType + Element> {
    pub primitives: Vec<shapes::Rectangle<V>>,
}

impl<V: GLVertexType + Element> Plot2D<V> {
    /// Plot `f` over `domain`, then place the resulting local 2D graph
    /// (which lives in `[-1, 1]²` with y auto-fit to the sampled range)
    /// onto the plane spanned by `right` and `up`, centered at `center`.
    ///
    /// - `center`: world-space center of the graph.
    /// - `right`:  world vector for the graph's local +X axis. Its length is
    ///             the half-width of the graph in world space.
    /// - `up`:     world vector for the graph's local +Y axis. Its length is
    ///             the half-height of the graph in world space.
    /// - `thickness`: ribbon thickness in *local* units (`[-1, 1]` span), so
    ///             `0.02` is 2% of half the graph's size.
    pub fn new<F>(
        f: F,
        domain: (V, V),
        samples: usize,
        thickness: V,
        center: Vector3<V>,
        right: Vector3<V>,
        up: Vector3<V>,
    ) -> Self
    where
        F: Fn(V) -> V,
    {
        assert!(samples >= 1, "need at least one segment");
        let (x0, x1) = domain;
        assert!(x1 != x0, "domain must have non-zero width");

        // Sample f across the domain.
        let mut pts: Vec<(V, V)> = Vec::with_capacity(samples + 1);
        let two = V::one() + V::one();
        let half = thickness / two;
        for i in 0..=samples {
            let t = V::from_usize(i) / V::from_usize(samples);
            let x = x0 + (x1 - x0) * t;
            pts.push((x, f(x)));
        }

        // Y range from finite samples; fall back for flat/NaN.
        let mut ymin = V::infinity();
        let mut ymax = V::neg_infinity();
        for &(_, y) in &pts {
            if y.is_finite() {
                ymin = ymin.min(y);
                ymax = ymax.max(y);
            }
        }
        if !ymin.is_finite() || !ymax.is_finite() || (ymax - ymin).abs() < V::epsilon() {
            ymin = -V::one();
            ymax = V::one();
        }

        let x_span = x1 - x0;
        let y_span = ymax - ymin;
        let map_x = |x: V| ((x - x0) / x_span) * two - V::one();
        let map_y = |y: V| ((y - ymin) / y_span) * two - V::one();

        let to_world = |lx: V, ly: V| center.add(right.mul_scalar(lx)).add(up.mul_scalar(ly));

        let mut primitives: Vec<shapes::Rectangle<V>> = Vec::with_capacity(samples);
        for w in pts.windows(2) {
            let (ax, ay) = w[0];
            let (bx, by) = w[1];
            if !ay.is_finite() || !by.is_finite() {
                continue; // skip discontinuities
            }

            let p0x = map_x(ax);
            let p0y = map_y(ay);
            let p1x = map_x(bx);
            let p1y = map_y(by);

            // Local 2D segment direction.
            let dx = p1x - p0x;
            let dy = p1y - p0y;
            let seg = Vector3::from([dx, dy, V::zero()]);
            let len = seg.len();
            if len < V::epsilon() {
                continue; // degenerate segment
            }

            let nx = -dy / len * half;
            let ny = dx / len * half;

            let c0 = to_world(p0x - nx, p0y - ny);
            let c1 = to_world(p0x + nx, p0y + ny);
            let c2 = to_world(p1x + nx, p1y + ny);
            let c3 = to_world(p1x - nx, p1y - ny);

            primitives.push(shapes::Rectangle::new(c0, c1, c2, c3));
        }

        Self { primitives }
    }

    pub fn to_mesh_colored<I: GLIndexType, C: GLVertexType + Element>(
        self,
        color: Vector3<C>,
    ) -> Mesh {
        if let Some(&first) = self.primitives.get(0) {
            let mut base =
                constructs::RectangleSolidColor::<V, C>::new1(first, color).into_drawable();

            for rect in self
                .primitives
                .into_iter()
                .skip(1)
                .map(|r| constructs::RectangleSolidColor::<V, C>::new1(r, color))
            {
                // terrible because this allocates each rect just to be merged into base,
                // but we just need a different merge trait or something to not do the allocation
                base = base.merge(rect.into_drawable());
            }
            base.to_mesh()
        } else {
            Mesh::empty()
        }
    }
}
