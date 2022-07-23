use std::{error::Error, time::SystemTime, sync::Mutex};

use rand::Rng;

use crate::renderer::{vertex_format::VertexSimple, vulkan::VulkanRenderer};

pub struct Game<'a> {
    last_created: SystemTime,
    renderer: &'a Mutex<&'a mut VulkanRenderer>
}

impl<'a> Game<'a> {
    pub fn new(renderer: &'a Mutex<&'a mut VulkanRenderer>) -> Game<'a> {
        return Game {  
            last_created: SystemTime::now(),
            renderer: renderer
        };
    }

    #[profiling::function]
    pub fn update(&mut self) -> Result<(), Box<dyn Error>> {
        let current_time = SystemTime::now();
        if current_time.duration_since(self.last_created)?.as_millis() > 2000 {
            let start_x = rand::thread_rng().gen_range(-1.0f32..1.0f32);
            let start_y = rand::thread_rng().gen_range(-1.0f32..1.0f32);
            let vertices: Vec<VertexSimple> = vec![
                VertexSimple { position: [start_x, start_y, 0.0f32] },
                VertexSimple { position: [start_x, start_y + 0.05f32, 0.0f32] },
                VertexSimple { position: [start_x + 0.05f32, start_y, 0.0f32] },
            ];
            let indices: Vec<u32> = vec![0, 1, 2];
            self.renderer.lock().expect("Couldn't lock").add_mesh(vertices, indices);
            self.last_created = current_time;
        }

        return Ok(());
    }

    #[profiling::function]
    pub fn synchronize(&mut self) -> Result<(), Box<dyn Error>> {
        return Ok(());
    }
}
