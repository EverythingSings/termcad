use crate::scene::Camera as SceneCamera;

pub struct Camera {
    pub position: [f32; 3],
    pub target: [f32; 3],
    pub fov: f32,
    pub aspect: f32,
    pub near: f32,
    pub far: f32,
}

impl Camera {
    pub fn from_scene(camera: &SceneCamera, width: u32, height: u32) -> Self {
        Self {
            position: camera.position,
            target: camera.target,
            fov: camera.fov,
            aspect: width as f32 / height as f32,
            near: 0.1,
            far: 1000.0,
        }
    }

    pub fn view_matrix(&self) -> [[f32; 4]; 4] {
        look_at(self.position, self.target, [0.0, 1.0, 0.0])
    }

    pub fn projection_matrix(&self) -> [[f32; 4]; 4] {
        perspective(self.fov.to_radians(), self.aspect, self.near, self.far)
    }

    pub fn view_projection_matrix(&self) -> [[f32; 4]; 4] {
        let view = self.view_matrix();
        let proj = self.projection_matrix();
        // Multiply and transpose for WGSL column-major layout
        transpose(multiply_matrices(proj, view))
    }
}

fn look_at(eye: [f32; 3], target: [f32; 3], up: [f32; 3]) -> [[f32; 4]; 4] {
    let f = normalize(subtract(target, eye));
    let s = normalize(cross(f, up));
    let u = cross(s, f);

    // Row-major: each row contains coefficients for that output component
    [
        [s[0], s[1], s[2], -dot(s, eye)],
        [u[0], u[1], u[2], -dot(u, eye)],
        [-f[0], -f[1], -f[2], dot(f, eye)],
        [0.0, 0.0, 0.0, 1.0],
    ]
}

fn perspective(fov: f32, aspect: f32, near: f32, far: f32) -> [[f32; 4]; 4] {
    // wgpu uses depth range 0-1
    let f = 1.0 / (fov / 2.0).tan();

    // Row-major: row 3 has the -1 for perspective divide
    [
        [f / aspect, 0.0, 0.0, 0.0],
        [0.0, f, 0.0, 0.0],
        [0.0, 0.0, far / (near - far), (near * far) / (near - far)],
        [0.0, 0.0, -1.0, 0.0],
    ]
}

fn multiply_matrices(a: [[f32; 4]; 4], b: [[f32; 4]; 4]) -> [[f32; 4]; 4] {
    let mut result = [[0.0; 4]; 4];
    for i in 0..4 {
        for j in 0..4 {
            for k in 0..4 {
                result[i][j] += a[i][k] * b[k][j];
            }
        }
    }
    result
}

fn transpose(m: [[f32; 4]; 4]) -> [[f32; 4]; 4] {
    [
        [m[0][0], m[1][0], m[2][0], m[3][0]],
        [m[0][1], m[1][1], m[2][1], m[3][1]],
        [m[0][2], m[1][2], m[2][2], m[3][2]],
        [m[0][3], m[1][3], m[2][3], m[3][3]],
    ]
}

fn subtract(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn cross(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn normalize(v: [f32; 3]) -> [f32; 3] {
    let len = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    if len > 0.0 {
        [v[0] / len, v[1] / len, v[2] / len]
    } else {
        [0.0, 0.0, 0.0]
    }
}
