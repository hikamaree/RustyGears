use crate::InstanceRaw;
use cgmath::InnerSpace;
use cgmath::Vector3;
use cgmath::Matrix4;
use std::sync::Arc;
use crate::Texture;
use crate::Camera;
use std::ops::Range;


#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct LightUniform {
    pub position: [f32; 3],
    pub _padding: u32,
    pub color: [f32; 3],
    pub _padding2: u32,
}


pub trait Vertex {
    fn desc() -> wgpu::VertexBufferLayout<'static>;
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ModelVertex {
    pub position: [f32; 3],
    pub tex_coords: [f32; 2],
    pub normal: [f32; 3],
    pub tangent: [f32; 3],
    pub bitangent: [f32; 3],
}

impl Vertex for ModelVertex {
    fn desc() -> wgpu::VertexBufferLayout<'static> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<ModelVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 5]>() as wgpu::BufferAddress,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 8]>() as wgpu::BufferAddress,
                    shader_location: 3,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 11]>() as wgpu::BufferAddress,
                    shader_location: 4,
                    format: wgpu::VertexFormat::Float32x3,
                },
            ],
        }
    }
}

#[derive(Clone)]
pub struct Material {
    pub name: String,
    pub diffuse_texture: Arc<Texture>,
    pub normal_texture: Arc<Texture>,
    pub bind_group: wgpu::BindGroup,
}

impl Material {
    pub fn new(
        device: &wgpu::Device,
        name: &str,
        diffuse_texture: Texture,
        normal_texture: Texture,
        layout: &wgpu::BindGroupLayout,
    ) -> Self {
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&diffuse_texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&diffuse_texture.sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(&normal_texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::Sampler(&normal_texture.sampler),
                },
            ],
            label: Some(name),
        });

        Self {
            name: String::from(name),
            diffuse_texture: Arc::new(diffuse_texture),
            normal_texture: Arc::new(normal_texture),
            bind_group,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct BoundingSphere {
    pub center: cgmath::Vector3<f32>,
    pub radius: f32,
}

#[derive(Clone)]
pub struct Mesh {
    pub name: String,
    pub vertex_buffer: Arc<wgpu::Buffer>,
    pub index_buffer: Arc<wgpu::Buffer>,
    pub num_elements: u32,
    pub material: usize,
    pub bounding_sphere: BoundingSphere,
}

#[derive(Clone)]
pub struct Model {
    pub meshes: Vec<Mesh>,
    pub materials: Vec<Material>,
}

pub trait DrawModel<'a> {
    #[allow(unused)]
    fn draw_mesh(
        &mut self,
        mesh: &'a Mesh,
        material: &'a Material,
        camera_bind_group: &'a wgpu::BindGroup,
        light_bind_group: &'a wgpu::BindGroup,
        camera: &'a Camera,
        instances: &Vec<InstanceRaw>,
    );
    fn draw_mesh_instanced(
        &mut self,
        mesh: &'a Mesh,
        material: &'a Material,
        camera_bind_group: &'a wgpu::BindGroup,
        light_bind_group: &'a wgpu::BindGroup,
        camera: &'a Camera,
        instances: &Vec<InstanceRaw>,
    ) -> u32;

    #[allow(unused)]
    fn draw_model(
        &mut self,
        model: &'a Model,
        camera_bind_group: &'a wgpu::BindGroup,
        light_bind_group: &'a wgpu::BindGroup,
        camera: &'a Camera,
        instances: &Vec<InstanceRaw>,
    );
    fn draw_model_instanced(
        &mut self,
        model: &'a Model,
        camera_bind_group: &'a wgpu::BindGroup,
        light_bind_group: &'a wgpu::BindGroup,
        camera: &'a Camera,
        instances: &Vec<InstanceRaw>,
    ) -> u32;
    #[allow(unused)]
    fn draw_model_instanced_with_material(
        &mut self,
        model: &'a Model,
        material: &'a Material,
        camera_bind_group: &'a wgpu::BindGroup,
        light_bind_group: &'a wgpu::BindGroup,
        camera: &'a Camera,
        instances: &Vec<InstanceRaw>,
    );
}

impl<'a, 'b> DrawModel<'b> for wgpu::RenderPass<'a>
where 'b: 'a {
    fn draw_mesh(
        &mut self,
        mesh: &'b Mesh,
        material: &'b Material,
        camera_bind_group: &'b wgpu::BindGroup,
        light_bind_group: &'b wgpu::BindGroup,
        camera: &'a Camera,
        instances: &Vec<InstanceRaw>,
    ) {
        self.draw_mesh_instanced(mesh, material, camera_bind_group, light_bind_group, &camera, instances);
    }

    fn draw_mesh_instanced(
        &mut self,
        mesh: &'b Mesh,
        material: &'b Material,
        camera_bind_group: &'b wgpu::BindGroup,
        light_bind_group: &'b wgpu::BindGroup,
        camera: &'a Camera,
        insts: &Vec<InstanceRaw>,
    ) -> u32 {
        let visible_indices: Vec<u32> = insts
            .iter()
            .enumerate()
            .filter_map(|(i, raw)| {
                let model = Matrix4::from(raw.model);
                let center_local = mesh.bounding_sphere.center.extend(1.0);
                let center_world = model * center_local;
                let center = center_world.truncate();

                let scale_x = Vector3::new(model.x.x, model.x.y, model.x.z).magnitude();
                let scale_y = Vector3::new(model.y.x, model.y.y, model.y.z).magnitude();
                let scale_z = Vector3::new(model.z.x, model.z.y, model.z.z).magnitude();
                let max_scale = scale_x.max(scale_y).max(scale_z);

                let radius = mesh.bounding_sphere.radius * max_scale;

                if camera.can_see(center, radius) {
                    Some(i as u32)
                } else {
                    None
                }
            })
        .collect();

        if visible_indices.is_empty() {
            return 0;
        }

        fn group_contiguous_ranges(indices: &[u32]) -> Vec<Range<u32>> {
            let mut ranges = Vec::new();
            let mut start = indices[0];
            let mut prev = start;

            for &i in &indices[1..] {
                if i == prev + 1 {
                    prev = i;
                } else {
                    ranges.push(start..prev + 1);
                    start = i;
                    prev = i;
                }
            }

            ranges.push(start..prev + 1);
            ranges
        }

        let instance_ranges = group_contiguous_ranges(&visible_indices);

        self.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
        self.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
        self.set_bind_group(0, &material.bind_group, &[]);
        self.set_bind_group(1, camera_bind_group, &[]);
        self.set_bind_group(2, light_bind_group, &[]);

        for range in instance_ranges {
            self.draw_indexed(0..mesh.num_elements, 0, range);
        }

        let triangle_count_per_instance = mesh.num_elements / 3;
        let total = triangle_count_per_instance * visible_indices.len() as u32;
        total
    }

    fn draw_model(
        &mut self,
        model: &'b Model,
        camera_bind_group: &'b wgpu::BindGroup,
        light_bind_group: &'b wgpu::BindGroup,
        camera: &'a Camera,
        instances: &Vec<InstanceRaw>,
    ) {
        self.draw_model_instanced(model, camera_bind_group, light_bind_group, &camera, instances);
    }

    fn draw_model_instanced(
        &mut self,
        model: &'b Model,
        camera_bind_group: &'b wgpu::BindGroup,
        light_bind_group: &'b wgpu::BindGroup,
        camera: &'a Camera,
        insts: &Vec<InstanceRaw>,
    ) -> u32 {
        let mut total_triangles = 0;
        for mesh in &model.meshes {
            let material = &model.materials[mesh.material];
            total_triangles += self.draw_mesh_instanced(
                mesh,
                material,
                camera_bind_group,
                light_bind_group,
                &camera,
                insts,
            );
        }
        total_triangles
    }

    fn draw_model_instanced_with_material(
        &mut self,
        model: &'b Model,
        material: &'b Material,
        camera_bind_group: &'b wgpu::BindGroup,
        light_bind_group: &'b wgpu::BindGroup,
        camera: &'a Camera,
        insts: &Vec<InstanceRaw>,
    ) {
        for mesh in &model.meshes {
            self.draw_mesh_instanced(
                mesh,
                material,
                camera_bind_group,
                light_bind_group,
                &camera,
                insts,
            );
        }
    }
}
