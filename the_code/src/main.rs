

use bytemuck::{ Pod, Zeroable };
//<explain> 
// "POD" stands for Plane Old Data. It makes sure that the data is basic enough that the GPU does not struggle to use it.
// "Zeroable" refers to the ability to store something in the form of bytes all with the value of 0. Basically another thing we use to make sure the data is simple enough for GPU.
//</explain>

use glam::{ Mat4, Vec3, Vec2 };
//<explain>
// These are used to handle all our camera functionality.
// Mat4 - A 4x4 Matrix | Think of it as a grid of 16 values 
//  Matrices allow us to transform our original vertex positions into new ones that might be more helpful.
//  For example: rotating the camera, panning the camera, zooming in
// Vec3 - A 1x3 Vector | Think of it as a list of 3 values
// Vec2 - A 1x2 Vector | Think of it as a list of 2 values
//</explain>

use std::{fs::OpenOptions, sync::Arc};
//<explain>
// "ARC" stands for Atomic Reference Counted.
// Things that are "ARC" can be referenced by multiple things at the same time.
//</explain>

use wgpu::{util::DeviceExt};
//<explain>
// Tools that make it quicker/easier to upload data to the GPU.
//</explain>

use winit::{ // Bunch of stuff we'll use to run the window and get input
    application::ApplicationHandler,
    event::{ DeviceEvent, WindowEvent, DeviceId, MouseButton, ElementState },
    event_loop::{ ActiveEventLoop, EventLoop },
    window::{Window, WindowId}
};

use std::collections::HashMap;


struct Camera {
    //<explain>
    // The "Camera" will be using to store the 4 values below
    //</explain>

    distance: f32, // how far away from the origin (radius)
    yaw: f32, // rotation along the horizontal
    pitch: f32, // rotation along the vertical
    sensitivity: f32 // a multiplier for much to move based on mouse movement

}

impl Camera {
    //<explain>
    // Has 3 methods: 
    // new() - returns a "Camera" with default values
    // update_orientation(&mut self, delta: Vec2) - takes in a delta param and updates the camera accordingly
    // view_matrix(&self)->Mat4 - returns a Mat4 to be used to create an MVP (Model View Projection) Matrix
    //</explain>


    fn new() -> Self {

        Self {

            distance: 25.0,
            yaw: 0.0,
            pitch: 0.0,
            sensitivity: 0.015

        }

    }

    fn update_orientation( &mut self, delta: Vec2 ) -> () {
        //<explain>
        // Takes in a Vec2 composed of the change in x and y coordinates of your mouse
        // and updates values accordingly.
        //</explain>

        self.yaw += delta.x * self.sensitivity;
        self.pitch -= delta.y * self.sensitivity;
        self.pitch = self.pitch.clamp(-1.5, 1.5);

    }

    fn view_matrix( &self ) -> Mat4 {

        // Check out this resource to find out about the specific math: https://learnopengl.com/Getting-started/Camera

        let x = self.distance * self.pitch.cos() * self.yaw.cos();
        let y = self.distance * self.pitch.cos() * self.yaw.sin();
        let z = self.distance * self.pitch.sin();

        let position = Vec3::new( x, y, z );
        //<explain>
        // This is the position that the camera is looking from.
        //</explain>

        Mat4::look_at_rh(position, Vec3::ZERO, Vec3::Z)
        //<explain>
        // Creates a 4x4 right-handed matrix.
        // Right handed means that the positive z direction is outside the front of the display 
        // whereas left handed means the positive z direction is outside the back of the display
        // Check out this resource to explain right vs left handed coordinate systems: https://www.youtube.com/watch?v=BoHQtXpWG2Y
        // 1st param - where to look from
        // 2nd param - where to look at
        // 3rd param - up direction | Vec3::Z just uses a <0.0, 0.0, 1.0> vector indicating the z axis as up
        //</explain>

    }



}


#[repr(C)]
//<explain>
// Represent as C - store the struct like you would in c programming language
// Used to make sure it's simple enough for the GPU to handle
//</explain>
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
struct Vertex {

    pos: [ f32; 3 ]
    //<explain>
    // Every vertex or point that makes up a triangle has 3D coordinates x, y, z and we will store them in this struct
    // We will need 3 vertices/vertex-positions for every triangle
    //</explain>

}

impl Vertex {

    fn scale( &mut self, scale_factor: f32 ) -> Self { // Scales a Vertex by a multiplier


        let new_pos = [ self.pos[0] * scale_factor, self.pos[1] * scale_factor, self.pos[2] * scale_factor ];

        return Vertex { pos: new_pos };

    }

    fn translate( &mut self, x: f32, y: f32, z: f32 ) -> Self { // Traslates/Moves a Vertex 

        let new_pos = [ self.pos[0] + x, self.pos[1] + y, self.pos[2] + z ];

        return Vertex { pos: new_pos };

    }

}

#[repr(C)]
//<explain>
// Represent as C - store the struct like you would in c programming language
// Used to make sure it's simple enough for the GPU to handle
//</explain>
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
struct Uniforms {
    mvp: [ [f32; 4]; 4 ],
    //<explain>
    // "MVP" stands for Model View Projection. It is a 4x4 matrix that corrisponds to the Uniforms struct in the "draw_stl.wgsl" shader.
    // It is our final matrix to apply all the transformations 
    //</explain>
    rgba: [ f32; 4 ]
    //<explain>
    // The base color of the 3d model
    // "RGBA" stands for Red Green Blue Alpha
    // Each value is normalized from 0.0 to 1.0
    // A value of <1.0, 1.0, 0.0, 1.0> would create a yellowish color
    // Whereas a value of <0.0, 0.0, 0.0, 1.0> would create pitch black and <1.0, 1.0, 1.0, 1.0> would create a bright white
    //</explain>
}



struct AppState {
    //<explain>
    // AppState is a struct used to keep track of certain values that stay consistent across renders
    //</explain>


    // what each thing is will be covered in the "new" method when implemented
    window: Arc<Window>, // Atomically Reference Counted Window. Means it can be borrowed by multiple things at same time
    surface: wgpu::Surface<'static>, // <'static> is a lifetime designation. Means that the Surface lives for the entire duration of the script.
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    render_pipeline: wgpu::RenderPipeline,
    index_count: u32,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    uniform_buffer: wgpu::Buffer,
    uniform_bind_group: wgpu::BindGroup,
    camera: Camera,
    is_dragging: bool,
    depth_texture_view: wgpu::TextureView

}

impl AppState {

    fn create_depth_texture( device: &wgpu::Device, config: &wgpu::SurfaceConfiguration ) -> wgpu::TextureView {
        //<explain>
        // The GPU does not automatically know which triangles to put on top of which unless you give it the ability to
        // A depth texture is a Texture that will be used later in a Depth Stencil to keep track of where each triangle/vertex
        // is so that the gpu can draw the correct triangle on top.
        // If you did not do this then the drawing order would be purely based on the indices meaning that the bottom of the object could
        // be drawn on top making the object difficult to interperet
        //</explain>

        let size = wgpu::Extent3d {
            //<explain>
            // WebGPU enforces that we use a 3-D texture representation even though it's really a 2-D texture
            // So we just wrap it in a "Extent3d" and give it a depth of 1
            //</explain>
            width: config.width,
            height: config.height,
            depth_or_array_layers: 1
        };

        let desc = wgpu::TextureDescriptor {
            label: Some("Depth Texture Descriptor"),
            size,
            mip_level_count: 1, // How many mip-maps
            //<explain>
            // Mip maps are smaller down-scaled versions of textures used to smooth them out
            // Example: a mip map count of 4 would store a 1, 1/2, 1/4, and 1/8 resolution representation
            // and the GPU would use the smaller, less detailed, version the further away 
            // We dont want down-scaling/smoothing because we want the true values
            //</explain>
            sample_count: 1, // MSAA
            //<explain>
            // Multisampling - how many pieces of depth data to store per pixel
            // Example: a sample_count of 4 would store 4 sub-locations and reference
            // them to smooth out the pixels. 
            //</explain>
            dimension: wgpu::TextureDimension::D2, // D2 indicates using 2 dimensions for the math calculations in the background
            format: wgpu::TextureFormat::Depth32Float, // Use a Depth32Float format. 
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            //<explain>
            // wgpu::TextureUsages::RENDER_ATTACHMENT - tells the GPU to use "Tiled" memory, which is optimized for writing pixels in local blocks.
            // wgpu::TextureUsages::TEXTURE_BINDING - tells the GPU to keep the data in a format that the Shader Units can read.
            //</explain>
            view_formats: &[] // Allows you to create an alias of the type. Not used in the script.

        };

        let texture = device.create_texture(&desc); // Creates the space in memory for the GPU to write to the texture
        texture.create_view(&wgpu::TextureViewDescriptor::default()) // Creates a way to view the texture & returns it
        //<explain>
        // GPU's do not let you touch textures during render passes so instead you look at a "view"
        //</explain>

    }


    async fn new( window: Arc<Window> ) -> Self {
        //<explain>
        // Creates a new AppState struct.
        // Reads the vertices from the STL file
        // Creates indices
        // Creates and writes to the vertex and index buffers
        // Creates a buffer and bind group for uniform variables
        // Creates a render pipeline
        // Returns the new AppState struct with all the values
        //</explain>

        let mut file = OpenOptions::new() // Gets the file
            .read(true)
            .open("3dbenchy.stl")
            .expect("Failed to open STL");

        let stl_data= stl::read_stl(&mut file).expect("Failed to parse STL"); // Reads the stl and converts to vertices

        let mut vertices: Vec<Vertex> = Vec::new(); // a variable that will hold our new vertices
        let mut indices: Vec<u32> = Vec::new(); // a variable that will hold our calculated indices
        let mut vertex_to_index = HashMap::new(); // a Hashmap helper to make it easier to check for duplicate vertices
        //<explain>
        // If I were to draw a square without deduplicating it would require 6 vertices
        // because you would store vertices that you are using twice. If you get rid of 
        // duplicates then you only need 4 and proper indexing.
        // This might seem negligible when talking about a square but when you get to STL
        // files with millions of vertices it adds up.
        //</explain>

        for triangle in stl_data.triangles { // Triangles is a [[[f32; 3]; 3]; n] where n is how many triangles there are
            for vertex_pos in [triangle.v1, triangle.v2, triangle.v3] { // loop over the 3 vertices in the triangle

                let hash_key = ( // a unique key to compare against
                    //<explain>
                    // You cannot store a unique key as an f32 because a Nan != Nan in f32
                    // so instead we store it in u32 form using .to_bits()
                    //</explain>
                    vertex_pos[0].to_bits(),
                    vertex_pos[1].to_bits(),
                    vertex_pos[2].to_bits(),
                );

                if let Some(&index) = vertex_to_index.get(&hash_key) { // if same vertex/point then only push the index
                    indices.push(index);
                } else { // otherwise upload the new vertex and new index
                    let index = vertices.len() as u32;
                    let new_vertex = Vertex { pos: vertex_pos }.scale(0.3).translate(0.0, 0.0, -5.0);
                    //<explain>
                    // Using the .scale() and .translate() methods we implemented earlier
                    // These specific values are just what looked best for my benchy model so feel free to change.
                    //</explain> 
                    vertices.push(new_vertex);
                    vertex_to_index.insert(hash_key, index);
                    indices.push(index);
                }
            }
        }


        let size = window.inner_size(); // get physical window size
        let instance = wgpu::Instance::default(); // Get an "Instance" - basically the tool we call to get our other stuff
        let surface = instance.create_surface(window.clone()).unwrap();
        //<explain>
        // Surfaces are like the painting we draw onto that then gets put onto the window.
        // We use window.clone() here. Since window is ARC ( Atomically Reference Counted )
        // this does not actually create a clone/copy it instead creates a new reference and
        // count. 
        // Again, this is done so that multiple things can reference window at the same time.
        //</explain>
        let adapter = instance.request_adapter( // adapter is simply used to get a device & setup stuff
            &wgpu::RequestAdapterOptions {
                compatible_surface: Some(&surface),
                ..Default::default()
            }
        ).await.unwrap();

        let ( device, queue ) = adapter.request_device(
            //<explain>
            // device - represent a connection to GPU | allocates memory, creates buffers, etc
            // queue - a kind of conveyor belt for the GPU | writes to buffers, actually calls commands, etc
            //</explain>
            &wgpu::DeviceDescriptor::default()
        ).await.unwrap();

        let config = surface.get_default_config( // uses the surfaces default configuration
            &adapter, 
            size.width, 
            size.height
        ).unwrap();

        surface.configure(&device, &config);

        // What is a buffer? What are the buffer usages?
        //<explain>
        // A buffer is a piece of memory that you reserve on the GPU side.
        // Typically the process is: Reserve Memory (Create Buffer) -> Write to Memory (write to buffer)
        // There are different types of buffer usages 
        // In our script we use 4 different types:
        // VERTEX - A special buffer usage that the gpu will use to draw triangles from
        // INDEX - A special buffer usage that the gpu will use to know which order to draw triangle in
        // UNIFORM - A special buffer usage that the gpu will know to keep the same across all processes
        // COPY_DST - Short for Copy desintation | Specifies that the buffer is allowed to be the destination 
        // of a data transfer. Needed to change data mid-program (not mid-render).
        //</explain>

        let vertex_buffer = device.create_buffer_init( // Creates a Vertex Buffer & writes to it
            &wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: bytemuck::cast_slice(&vertices), // sends a byte version of the vertices
                usage: wgpu::BufferUsages::VERTEX // specifies the usage as VERTEX
            }
        );

        let index_buffer = device.create_buffer_init( // creates an Index Buffer & writes to it
            &wgpu::util::BufferInitDescriptor {

                label: Some("Index Buffer"),
                contents: bytemuck::cast_slice(&indices), // sends a byte version of the indices
                //<explain>
                // Why a byte version?
                // "contents" expects a value of [u8] 
                //</explain>
                usage: wgpu::BufferUsages::INDEX // specificies the usage as INDEX

            }
        );

        let uniform_buffer = device.create_buffer( // creates a uniform buffer but does not write to it.
            // using .create_buffer instead of .create_buffer_init because it only creates a buffer and does not write to it
            &wgpu::BufferDescriptor {
                label: Some("Uniform Buffer"),
                size: std::mem::size_of::<Uniforms>() as u64, // this is used instead of contents because we don't have data to send yet.
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST, // specifies the usage as both UNIFORM and COPY_DST
                mapped_at_creation: false // if true this would create a temp space in cpu memory connected to the buffer in gpu memory that you could write to.
            }
        );

        // What are bind groups? (and bind group layouts)
        //<explain>
        // Bind groups tell the gpu which binding to connect a buffer to.
        // This is why in "draw_stl.wgsl" there's @group(0) @binding(0) before the uniforms variable
        // It's telling the GPU: get the value of uniforms from group 0 binding 0 where our uniform buffer is
        // Our Vertex & Index buffers do not need these because they already have special reserved bindings
        // Bind group layouts simply are a blueprint to define the data format but not the data itself
        //</explain>

        let uniform_bind_group_layout = device.create_bind_group_layout( // creates a bind group layout for our uniform buffer
            &wgpu::BindGroupLayoutDescriptor { 

                    label: None,
                    entries: &[
                        wgpu::BindGroupLayoutEntry {

                            binding: 0, // bind it to binding 0
                            visibility: wgpu::ShaderStages::VERTEX, // make it so that the uniform buffer can only be accessed from the vertex shader 
                            ty: wgpu::BindingType::Buffer { // specify binding type
                                ty: wgpu::BufferBindingType::Uniform, // specify buffer binding type
                                min_binding_size: None, // minimum binding size
                                has_dynamic_offset: false // not used here but would be used to only look at a specific part of the uniform buffer based on an offset
                            },
                            count: None, // if true would allow multiple different values to exist in the same buffer sort of like an array

                        }
                    ],

                }
        );

        let uniform_bind_group = device.create_bind_group( // creates a bind group for our uniforms
            &wgpu::BindGroupDescriptor {
                layout: &uniform_bind_group_layout, // uses our layout
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0, // specifies binding(0) again
                        resource: uniform_buffer.as_entire_binding(), // bind to uniform buffer
                    }
                ],
                label: None
            }
        );

        let shader = device.create_shader_module(
            wgpu::include_wgsl!("draw_stl.wgsl")
        );

        let pipeline_layout = device.create_pipeline_layout( // a pipeline layout is a template for a pipelie
            &wgpu::PipelineLayoutDescriptor {
                bind_group_layouts: &[
                    &uniform_bind_group_layout,
                   
                ],
                 ..Default::default()
            }
        );

        let depth_texture_view= Self::create_depth_texture(&device, &config); // This gets our depth texture view we made earlier

        let render_pipeline = device.create_render_pipeline( // Creates a render pipeline
            //<explain>
            // There are 2 type of wgpu pipeline
            // Render Pipelines - drawing things to the screen
            // Comptue Pipelines - using the gpu to compute many things in parallel
            //</explain>
            &wgpu::RenderPipelineDescriptor {
                label: None,
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState { // describe what happens in the vertex shader
                    module: &shader,
                    entry_point: Some("vs_main"), // use the "vs_main" function
                    buffers: &[
                        wgpu::VertexBufferLayout {
                            array_stride: std::mem::size_of::<Vertex>() as u64, // tells the gpu how large each piece of vertex data is so it knows to hv a move up in memory the size of a vertex every time
                            step_mode: wgpu::VertexStepMode::Vertex, 
                            attributes: &wgpu::vertex_attr_array![0 => Float32x3] // Float32x3 corrisponds to a vec3<f32> that we hv in "draw_stl.wgsl"
                        }
                    ],
                    compilation_options: Default::default()
                },
                fragment: Some( // describe what happens in the fragment shader
                    wgpu::FragmentState {
                        module: &shader,
                        entry_point: Some("fs_main"), // use the "fs_main"
                        targets: &[ 
                            Some(
                                wgpu::ColorTargetState {
                                    format: config.format,
                                    blend: Some(wgpu::BlendState::REPLACE), // tells the gpu to interpolate the colors in between
                                    //<explain>
                                    // If you had a red point and a green point the gpu would blend the 2 together in the pixels between them
                                    // so that at the center you would hv yellow but close to red you'd have red and closer to green you'd hv
                                    // green
                                    //</explain>
                                    write_mask: wgpu::ColorWrites::ALL
                                }
                            )
                        ],
                        compilation_options: Default::default()
                    }
                ),
                depth_stencil: Some( // Depth stencils are whats used to make sure that the stuff that's on top would be on top and the stuff on bottom would be on bottom
                    wgpu::DepthStencilState {
                        format: wgpu::TextureFormat::Depth32Float,
                        depth_write_enabled: true,
                        depth_compare: wgpu::CompareFunction::Less,
                        stencil: wgpu::StencilState::default(),
                        bias: wgpu::DepthBiasState::default()
                    }
                ),
                primitive: wgpu::PrimitiveState { cull_mode: Some( wgpu::Face::Back ), ..Default::default() },
                //<explain>
                // cull_mode back makes it so the gpu only renders the back of each triangle so that you don't see the triangle face from all directions
                //</explain>
                multisample: wgpu::MultisampleState::default(),
                multiview_mask: None,
                cache: None,

            }
        );



        Self {

            window,
            surface,
            device,
            queue,
            config,
            render_pipeline,
            vertex_buffer,
            index_buffer,
            uniform_buffer,
            uniform_bind_group,
            camera: Camera::new(),
            is_dragging: false,
            index_count: indices.len() as u32,
            depth_texture_view: depth_texture_view

        }

    }

    
    fn render( &mut self ) -> Result<(), wgpu::SurfaceError> { // method that renders a frame on the gpu and puts it on the surface

        let output = self.surface.get_current_texture()?;
        //<explain>
        // the surface's texture is the object that the gpu will be drawing to
        //</explain>
        let view = output.texture.create_view(
        //<explain>
        // the texture cant be directly accessed so we use a view to look at it
        //</explain
            &wgpu::TextureViewDescriptor::default()
        );
        let mut encoder = self.device.create_command_encoder( 
            &wgpu::CommandEncoderDescriptor::default()
        );

        let aspect = self.config.width as f32 / self.config.height as f32; 
        let proj = Mat4::perspective_rh(45.0_f32.to_radians(), aspect, 1e-3, 1e4);
        //<explain>
        // creates a projection matrix (right handed) with an fov of 45 degrees
        // 1e-3 refers to the z_near or the minimum distance required to be away from the camera to render
        // 1e4 refers to the z_far or the maximum distance that things will render at
        //</explain>
        let mvp = proj * self.camera.view_matrix();
        //<explain>
        // by multiplying the projection and camera's view matrix we get a final matrix to apply to our vertices
        //</explain>

        self.queue.write_buffer( // writes data to the uniform buffer we created earlier
            &self.uniform_buffer, // specifies the buffer we want to write to
            0, // offset in bytes
            bytemuck::cast_slice( // turns it into a &[u8] since thats what wgpu expects
                &[
                    Uniforms {
                        mvp: mvp.to_cols_array_2d(), 
                        rgba: [0.3, 0.3, 0.3, 1.0]
                    }
                ]
            )
        );

        { // These braces are here to limit the lifetime of rpass to a certain region

            let mut rpass = encoder.begin_render_pass(
                &wgpu::RenderPassDescriptor {
                    color_attachments: &[Some( // tells wgpu what to do when a starting a new frame
                        wgpu::RenderPassColorAttachment {
                            view: &view,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Clear( // make the screen black
                                    wgpu::Color::BLACK
                                ),
                                store: wgpu::StoreOp::Store // store
                            },
                            depth_slice: None
                        }
                    )],
                    depth_stencil_attachment: Some(
                        wgpu::RenderPassDepthStencilAttachment {
                            view: &self.depth_texture_view, // use the depth texture view we made earlier
                            depth_ops: Some(
                                wgpu::Operations {
                                    load: wgpu::LoadOp::Clear(1.0), 
                                    store: wgpu::StoreOp::Store
                                }
                            ),
                            stencil_ops: None
                        }
                    ),
                    ..Default::default()
                }
            );

            rpass.set_pipeline(&self.render_pipeline); // set pipeline
            rpass.set_bind_group(0, &self.uniform_bind_group, &[]); // set bind group
            rpass.set_vertex_buffer(0, self.vertex_buffer.slice(..)); // set vertex buffer
            rpass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint32); // set index buffer
            rpass.draw_indexed(0..self.index_count as u32, 0, 0..1); // create draw points command

        }

        self.queue.submit( std::iter::once( encoder.finish() )); // submit the commands to the GPU
        output.present(); // put it on the surface
        Ok(()) // return Ok

    }

}

struct App {
    state: Option<AppState> // hold an AppState that we created b4
}

impl ApplicationHandler for App { // impl ApplicationHandler from winit to handle window functions

    fn resumed(&mut self, event_loop: &ActiveEventLoop) { // runs when the app starts or resumes after a pause
        
        let window_attributes = Window::default_attributes() 
            .with_title("WGPU STL Renderer Tutorial") // make it titled
            .with_inner_size(winit::dpi::LogicalSize::new(800.0, 600.0)); // make it 800x600

        let window = Arc::new(
            event_loop.create_window(window_attributes).unwrap()
        );

        window.set_visible(true);
        window.focus_window(); // make it appear on top

        let state = pollster::block_on(AppState::new(window.clone())); 
        //<explain>
        // pollster::block_on is used to stop the main thread while the AppState::new method is running
        // we need this since AppState::new is an async method
        //</explain>
        self.state = Some(state);

        window.request_redraw();

    }

    fn device_event( // handle device events
            &mut self,
            _: &ActiveEventLoop,
            _: DeviceId,
            event: DeviceEvent,
        ) {

            if let Some(state) = self.state.as_mut() {

                if state.is_dragging {

                    if let DeviceEvent::MouseMotion{ delta } = event {
                        state.camera.update_orientation(Vec2::new(delta.0 as f32, delta.1 as f32));
                        state.window.request_redraw();
                    }

                }

            }
        
    }

    fn window_event( // handles window event
            &mut self,
            event_loop: &ActiveEventLoop,
            _: WindowId,
            event: WindowEvent,
        ) {

            match event {

                WindowEvent::CloseRequested => {
                    event_loop.exit();
                }

                WindowEvent::MouseInput {state, button: MouseButton::Left, ..} => {
                    if let Some(app_state) = self.state.as_mut() {
                        app_state.is_dragging = state == ElementState::Pressed;
                    }
                }

                WindowEvent::RedrawRequested => {

                    if let Some(state) = self.state.as_mut() {
                        state.render().unwrap();
                    }

                }

                WindowEvent::Resized(new_size) => {

                    if let Some(state) = self.state.as_mut() {

                        state.config.width = new_size.width;
                        state.config.height = new_size.height;
                        state.surface.configure(&state.device, &state.config);
                        state.depth_texture_view = AppState::create_depth_texture(&state.device, &state.config);
                        state.window.request_redraw();

                    }

                }

                _ => ()

            }
        
    }

}




fn main() {

    let event_loop = EventLoop::new().unwrap(); // create event loop
    let mut app = App { state: None }; // create App instance
    event_loop.run_app(&mut app).unwrap(); // run

}

