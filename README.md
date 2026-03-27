

# WGPU Rust Tutorial: STL Renderer + Camera + Lighting


## Table of Contents
- [Summary](#summary)
- [Author](#author)
- [Part 1: WGSL](#part-1-wgsl)
- [Part 2: CPU Side Data](#part-2-cpu-side-data)
- [Part 3: AppState](#part-3-appstate)
- [Part 4: App](#part-4-app)
- [Part 5: main](#part-5-main)
- [Part 6: Testing & Future Improvements](#part-6-testing--future-improvements)

## Author

This tutorial was written by me, Raziel Moesch (Raz for short) because I really enjoy
teaching and genuinely believe that WGPU is a great platform to get started with
graphics programming. I started out with WebGPU but wanted to learn the rust version
of the library, and could not find any tutorials that were less than 5 years old. I scraped together whatever resources I could and made what I think is a great start to see enough of WGPU to get started with still a lot more to be excited to learn about.
If you have any questions, comments, or even mistakes you think I made please reach out.

**My Email is: TheRazielMoesch@gmail.com** 

**Linkedin: https://www.linkedin.com/in/raziel-moesch-61474b21b/**


## Summary
In this tutorial you will learn most of what WGPU Rust offers and will be able to 
use this as a base for future projects. You will learn how to create room on the 
GPU for data and upload that data to the GPU from the CPU. You will learn how to 
manipulate that data so that you can account for a camera, 3-D, lighting, etc.
This tutorial is better geared for those who have a basic understanding of
how graphics programming works. If you have no experience you can still try to 
follow along and replicate what you see, but you will probably feel more 
comfortable if you go through the Learn WebGPU tutorials first.

WGPU is the Rust version of the JavaScript WebGPU library. They are very similar,
but WGPU has some key advantages. 
- 1. WGPU can run on any device since it will auto-compile to whatever shader language
     the device is using. So if you're on MacOS: Metal, Windows: DirectX, 
     Linux: Vulkan, etc. WebGPU only works in the browser with a canvas element.
     Note: You can use WGPU for the web too, but that will not be covered in this 
     tutorial.
- 2. WGPU is type-safe since it is written in Rust.
- 3. If you look at WGSL it looks very similar to Rust which makes it easy to think
     about how you should store data on the CPU side to send to the GPU.

The tutorial will start by going over the wgsl. This is because we want to define what our GPU is expected to do b4 we do anything on the CPU side so we could properly 
prepare. 

The tutorial will then go over creating CPU-side representations of the GPU-side
data that way we can upload matching data.

The tutorial will then go over our AppState struct. This struct will hold anything
that stays consitent across renders/frames. This includes our buffers ( places in GPU memory ), our render pipelines, our camera position, etc.

The tutorial will then go over our App struct. App will be a impl of ApplicationHandler
which basically means that we will define what happens when the user starts the app,
closes the app, etc.

The tutorial will then show you what your expected result should look like, and go
over improvements that could be made. For instance this STL renderer does not account
for STL files where the starting coordinates of the 3-D model are not (0, 0, 0) meaning that the model will be rendered at a offset location and be potentially 
imperceptible. 




## Part 1: WGSL

### Create a file called "draw_stl.wgsl" in your src folder.

### a. Uniform Variables

What is a uniform variable?

A uniform variable is a piece of data that exists across all GPU processes.
It is great for storing things that stay the same during the entire render. In our case we will use it to hold the color of the 3D model & the MVP (Model View Projection)
matrix.

First create a struct. Let's call it Uniforms:
```wgsl
struct Uniforms {

};
```
then let's add a parameter to the struct called "mvp". This will be a 
4x4 matrix where each value is a 32 bit floating point number. 

```wgsl
struct Uniforms {
    mvp: mat4x4<f32>
};
```
Finally we will add a paramter caled "rgba". This will be a 1x4 vector where each value is a 32 bit floating point number. <red, blue, green, alpha> all normalized from 0-1.
```wgsl
struct Uniforms {
    mvp: mat4x4<f32>,
    rgba: vec4<f32>
};
```

**For each part from now on I will try to explain WHY we are doing stuff instead of just HOW**

All we are doing here is creating a structure that we will use in the next step to instanciate a variable to hold our uniform data. We do that in a struct for clarity on the expected data input, conciseness, and the ease that it will provide when we create a matching struct on the CPU side.

In fact let's create that instance now:

```wgsl
@group(0) @binding(0)
var<uniform> uniforms: Uniforms;
```
#### This creates a uniform variable at group 0, binding 0 of type Uniforms named uniforms.
- `@group(0)` - says that the uniform data is expected to be at group 0
- `@binding(0)` - says that the uniform data is expected to be at binding 0
- `var<uniform>` - creates a variable of special type uniform

Put this uniforms variable declaration below the Uniforms struct declaration.

### b. Vertex Shader

#### b4 we define what we want to happen in our vertex shader let's create structs that define what's expected in the input and output of the vertex shader

Let's start with the Vertex Shader Input
```wgsl
    struct VertexInput {
        @location(0) position: vec3<f32>
    };
```
This creates a struct called VertexInput that grabs the data from @location(0). We will upload the data to @location(0) on the CPU side in part 2.

Next let's define a Vertex Shader Output struct
```wgsl
    struct VertexOutput {
    
    };
```

The first parameter we'll add will be to hold our transformed vertices. These 
vertices are transformed based on the camera calcluations done on the CPU side (part 2) that will be applied in our vertex shader function. Because they are a special piece of data we will store them in the @builtin(position) spot in memory so the GPU knows to use them as the points to draw our triangles.
```wgsl
    struct VertexOutput {
        @builtin(position) transformed_position: vec4<f32>
    };
```

The second parameter we'll add is just to pass the color data from the uniform buffer to the fragment shader. In this script the uniforms are only accessible from the vertex shader which is why we are doing this, but there is nothing stopping you from making it accessible in the fragment shader too and not having to pass it through. You will see where in part 2.

```wgsl
    struct VertexOutput {
        @builtin(position) transformed_position: vec4<f32>,
        @location(0) color: vec4<f32> 
    };
```

The third and last parameter we'll add holds what is called our world position coordinates. This is simply the vertices/points b4 any transformations are applied to them. We will use them to calculate our lighting later.

```wgsl
    struct VertexOutput {
        @builtin(position) transformed_position: vec4<f32>,
        @location(0) color: vec4<f32>,
        @location(1) world_position: vec3<f32>
    };
```
This is our final VertexOutput struct

Now we can finally get to the vertex shader function itself

```wgsl
@vertex
fn vs_main(in: VertexInput) -> VertexOutput {


}
```
This defines a function called vs_main. The @vertex attribute tells the GPU that this is a vertex shader function. It takes in a VertexInput and outputs a VertexOutput

There are 5 things we will do in this function:
1. instanciate a VertexOutput
2. set the world_position
3. use the uniforms.mvp to transform the position and set it
4. set the color
5. return the output struct
 
```wgsl
@vertex
fn vs_main(in: VertexInput) -> VertexOutput {

    var out: VertexOutput; // instanciate out

    out.world_position = in.position; // set world position as the unchanged input position
    out.transformed_position = uniforms.mvp * vec4<f32>(in.position, 1.0);  // apply mvp matrix by using matrix multiplication
    out.color = uniforms.rgba; // set color

    return out; // return

}
```

Note: For more info about why position is a vec4 just ask an llm since I don't hv the space. Long story short: it's useful for the math to use 4x4 matrices instead of 3x3 to make translation possible.

### c. Fragment Shader

There are 2 things we will do in this function:
1. Calculate lighting
2. return the color

Let's start with the lighting as it's a little mathy/complex.
Perhaps a theoretical approach will make more sense before the code.
Let's say the light comes from a light source at point ( 1.0, 1.0, 2.0 ) in space
we are gonna normalize the point. Normalization means turning a vector into a unit vector meaning it has a lenght of 1.0 . The length of the vector is currently 2.45
so the gpu will change the values to make it 1.0 while retaining the direction which
is what we really care about. 

Next we want to get the "normal" to the face we are talking about. A normal is a
unit vector that points perpindicular to the surface. Basically it's just which way
the face is facing. To get this we get the partial derivative with respect to x 
of the world position and take the cross product with the partial derivative with
respect to y. For more info just paste this into an llm and it will explain.

Finally we want to compare how closely the direction of the face aligns with the
light source. To do this we take the dot product between the face normal and the negative light normal. The light normal is negative because the rays of light point away from the light source. Then because we don't want the surface to be too bright
cap it at a multiplier of 0.1.

Then we return the colors times the intensity of the light. The final code looks like this:

```wgsl
@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {

    let face_normal = normalize(cross(dpdx(in.world_position), dpdy(in.world_position)));

    let light_dir = normalize(vec3<f32>(1.0, 1.0, 2.0)); // creates a normalized point from where light comes from


    let dot_product = dot(face_normal, -light_dir); // uses dot product to find how much the light contributes to that surface
    
    let intensity = max(dot_product, 0.1); // caps the amount of light possible


    return vec4<f32>(in.color.x * intensity, in.color.y * intensity, in.color.z * intensity, in.color.w ); // return 

}
```

The math may be a little tricky for those unfamiliar with linear algebra. Simply ask an llm to explain, or if you are really interested in how it works take up a linear algebra course. Definitely worth investing the time into.

That is everything for our wgsl shader. Store it in your src folder. I called it "draw_stl.wgsl" so that is what it will be referenced as in "main.rs".

## Part 2: CPU Side Data

### a. Imports

b4 we start the actual code we need to go over what libraries we are using.
1. glam "0.32.1" - handles our mvp matrix math
2. bytemuck "1.25.0" - used to create u8 representations of our struct to give to the GPU
3. pollster "0.4.0" - used to stop the current thread to wait for async functions to finsih
4. wgpu "28.0.0" - wgpu, contains all the tools we need
5. winit "0.30.13" - used to create windows and take input
6. stl "0.2.1" - used to easily process .stl files.

We will also create specific imports in the main.rs file to shorten code:

```rust
use bytemuck::{ Pod, Zeroable }; 
use glam::{ Mat4, Vec3, Vec2 };
use std::{ fs::OpenOptions, sync::Arc };
use wgpu::{ util::DeviceExt };
use winit::{
    application::ApplicationHandler,
    event::{ DeviceEvent, WindowEvent, DeviceId, MouseButton, ElementState },
    event_loop::{ ActiveEventLoop, EventLoop },
    window::{ Window, WindowId }
};
```
`Pod` stands for Plane Old Data. It makes sure that the data is basic enough that the GPU does not struggle to use it.

`Zeroable` refers to the ability to store something in the form of bytes all with the value of 0. Basically another thing we use to make sure the data is simple enough for GPU.

`Mat4` - A 4x4 Matrix | Think of it as a grid of 16 values 
Matrices allow us to transform our original vertex positions into new ones that might be more helpful.
For example: rotating the camera, panning the camera, zooming in

`Vec3` - A 1x3 Vector | Think of it as a list of 3 values

`Vec2` - A 1x2 Vector | Think of it as a list of 2 values

`Arc` stands for Atomic Reference Counted.
Things that are "ARC" can be referenced by multiple things at the same time.

`fs::OpenOptions` is used to get our stl file.

`util::DeviceExt` gives us shortcuts for uploading data to the GPU with less steps.

`winit` - everything imported from here is used to run the window

### b. Camera

Our Camera struct needs to store 4 different values to calculate our camera projection.

In this script the camera will calculate its x,y,z position using spherical trigonometry. This means the camera will rotate around a single point, and have the following values:

1. distance - Think of this as the radius of the sphere
2. yaw - Think of this as the rotation around the horizontal
3. pitch - Think of this as the rotation around the vertical
4. sensitivity - A multiplier we will use to make our mouse movement feel right

Let's create the struct under our imports, and make every value a f32

```rust
struct Camera {
    distance: f32,
    yaw: f32,
    pitch: f32, 
    sensitivity: f32 
}
```

Next let's look at our implementation for our Camera struct. We want 3 methods:

1. new() -> Self - returns a Camera struct with some default values
2. update_orientation( &mut self, delta: Vec2 ) -> () - uses our mouse movement to update our pitch and yaw
3. view_matrix( &self ) -> Mat4 - uses spherical trigonometry to calculate the x,y,z coordinates then uses glam to create a "look at" matrix which will make the camera look at a particular point from a particular point

First create the impl:

```rust
impl Camera {

}
```

Then put the following methods in:

**new**
```rust
    fn new() -> Self {
        Self {
            distance: 25.0,
            yaw: 0.0,
            pitch: 0.0,
            sensitivity: 0.015
        }
    }
```

**update orientation**
```rust
    fn update_orientation( &mut self, delta: Vec2 ) -> () {
        self.yaw += delta.x * self.sensitivity;
        self.pitch -= delta.y * self.sensitivity;
        self.pitch = self.pitch.clamp(-1.5, 1.5);
    }
```

**view matrix**
```rust
    fn view_matrix( &self ) -> Mat4 {

        let x = self.distance * self.pitch.cos() * self.yaw.cos();
        let y = self.distance * self.pitch.cos() * self.yaw.sin();
        let z = self.distance * self.pitch.sin();

        let position = Vec3::new( x, y, z );

        Mat4::look_at_rh( position, Vec3::ZERO, Vec3::Z )

    }
```
Check out this resource to find out about the specific math: https://learnopengl.com/Getting-started/Camera

**What is look_at_rh?**

Creates a 4x4 right-handed matrix that looks at a specific point ( in this case the origin (0, 0, 0) )

Right handed means that the positive z direction is outside the front of the display 
whereas left handed means the positive z direction is outside the back of the display

Check out this resource to explain right vs left handed coordinate systems: https://www.youtube.com/watch?v=BoHQtXpWG2Y

1st param - where to look from

2nd param - where to look at

3rd param - up direction | Vec3::Z just uses a <0.0, 0.0, 1.0> vector indicating the Z axis as up ( we use upward z-axis bc that's what STL files expect )


b. Vertex

Next we will create a Vertex struct that will hold each of our vertices.
All the struct will need to hold is 3 f32 values one for x, y, and z. 

Below our Camera implementation place this:

```rust
#[repr(C)]
#[derive( Copy, Clone, Debug, Pod, Zeroable )]
struct Vertex {
    pos: [f32; 3]
}
```

- `repr(C)` tells rust to store the struct as if it were in the C programming language.
Used because it helps guarantee that the data won't change, and will be simple enough to upload to the GPU.
- `derive( Pod )` tells rust to store the struct as "Plain Old Data". Another step to help guarantee the data is not too complex for the GPU.
- `derive( Zeroable )` makes sure that the data is simple enough that if it were stored as a list of bytes it could theoretically be stored as a list of only zeros. Another step to help guarantee the data is simple enough to upload to the GPU.

Next let's implement some methods for our Vertex struct:

1. `scale( &mut self, scale_factor: f32 ) -> Self` - returns a new instance of Vertex with a scaled position value
2. `translate( &mut self, x: f32, y: f32, z: f32 ) -> Self` - returns a new instance of Vertex with a translated position value.

```rust
impl Vertex {

    fn scale( &mut self, scale_factor: f32 ) -> Self {

        let new_pos = [ self.pos[0] * scale_factor, self.pos[1] * scale_factor, self.pos[2] * scale_factor ];

        return Vertex { pos: new_pos };

    }

    fn translate( &mut self, x: f32, y: f32, z: f32 ) -> Self {

        let new_pos = [ self.pos[0] + x, self.pos[1] + y, self.pos[2] + z ];

        return Vertex { pos: new_pos };

    }

}
```

Next we will make a struct to match our Uniforms struct in our WGSL.

```rust
#[repr(C)]
#[derive( Copy, Clone, Debug, Pod, Zeroable )]
struct Uniforms {
    mvp: [ [f32; 4]; 4 ],
    rgba: [ f32; 4 ]
}
```

Same stuff as b4.


## Part 3: AppState

AppState will be a struct that holds all the data that will stay the same in between renders or that will need to be parsed in between renders. 

Here is the definition of the struct. It will just seem like a list of things for now (bc it is). As we go through the implementation of the struct things will become clearer. What is important to know is that every thing in this struct is a piece of data that will be used for every render that you don't want to recreate.

```rust
struct AppState {

    window: Arc<Window>,
    surface: wgpu::Surface<'static>, // <'static> just means that the liftime of the variable is for the entire script
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
```

Now let's go through the implementation of AppState

We are gonna give AppState 3 methods:
1. create_depth_texture( device: &wgpu::Device, config: &wgpu::SurfaceConfiguration ) -> wgpu::TextureView - This method is a helper to create a depth texture view. Will go more into depth during the actual code
2. new( window: Arc<Window> ) -> Self - Initializes all the buffers and stores returns a AppState struct with all the values.
3. render( &mut self ) -> Result<(), wgpu::SurfaceError> - Uses everything that was created in "new" method to render to the surface.

a. create_depth_texture

**What is a depth texture view?**

To understand what a depth texture view is we first have to understand what a depth texture is. A depth texture is a texture the size of the surface you're drawing on that is specifically used by the GPU to keep track of which vertices are closer to the camera. This way the GPU will draw the things closer to the camera on top. A view to a depth texture is a sort of window that you use to look at the texture. This is because we can't access it directly mid-render.

So, let's start by first creating a size variable. WGPU expects the size to be a 3d texture so well store it as a 1 x width x height texture.

```rust
impl AppState {
    fn create_depth_texture( device: &wgpu::Device, config: &wgpu::SurfaceConfiguration ) -> wgpu::TextureView {
        let size = wgpu::Extent3d {
            width: config.width,
            height: config.height,
            depth_or_array_layers: 1
        };
    }
}
```

Then we have to create a descriptor for our texture:

```rust
impl AppState {
    fn create_depth_texture( device: &wgpu::Device, config: &wgpu::SurfaceConfiguration ) -> wgpu::TextureView {
        let size = wgpu::Extent3d {
            width: config.width,
            height: config.height,
            depth_or_array_layers: 1
        };

        let desc = wgpu::TextureDescriptor {
            label: Some("Depth Texture Descriptor"),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[]
        };
    }
}
```

If you want an explanation for each param in the descriptor here you go:
- label - a label (c'mon)
- size - the size that we created b4
- mip_level_count - Mip maps are smaller down-scaled versions of textured used at different distances for effecieny. A mip-map of 4 would store a version of the texture at 1, 1/2, 1/4, and 1/8 level of detail.
- sample_count - Multisampling, how many pieces of data to store/reference at every pixel location
- dimension - which dimension for the texture to work in
- format - what format, here we use a Depth32Float
- usage - what the texture will be used for
- view_formats - allows you to create an alias for different types


Finally let's actually create our texture

```rust
impl AppState {
    fn create_depth_texture( device: &wgpu::Device, config: &wgpu::SurfaceConfiguration ) -> wgpu::TextureView {
        let size = wgpu::Extent3d {
            width: config.width,
            height: config.height,
            depth_or_array_layers: 1
        };

        let desc = wgpu::TextureDescriptor {
            label: Some("Depth Texture Descriptor"),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[]
        };

        let texture = device.create_texture(&desc);
        texture.create_view(&wgpu::TextureViewDescriptor::default())
    }
}
```

All I added was a `texture` variable that holds the texture, and created a view which is returned.

That's it for the `create_depth_texture` method. Next let's go over the `new` method

The goal for the new method is to create every piece of the AppState struct. First let's go over what each part is, how it corresponds to the GPU, and why we are keeping it in the struct.

- `window: Arc<Window>` - this we do not have to create in the new function since it is passed through in the main function (we'll create later). We will be storing 

- `surface: wgpu::Surface<'static>` think of a surface as the actual thing we are painting on. Like a painting. We ask the GPU to draw a bunch of stuff and then we put it on the painting to display it every frame. `<'static>` refers the lifetime of the variable. It means that it will live for the entirety of the script.

- `device: wgpu::Device` - what we will use to create buffers and bindgroups.
- `queue: wgpu::Queue` - what we will use to actually write information to the GPU
- `config: wgpu::SurfaceConfiguration` - the configuration of the surface. Will be used later to handle changes in size of the window.
- `render_pipeline: wgpu::RenderPipeline`- this stays the same across frames so we want to store it. Think about it as an explanation of the steps the GPU should take.
- `index_count: u32` - how many indices we have
- `vertex_buffer: wgpu::Buffer` - a buffer that stores our vertices
- `index_buffer: wgpu::Buffer` - a buffer that stores our indices
- `uniform_buffer: wgpu::Buffer` - a buffer that stores our Uniforms struct
- `uniform_bind_group: wgpu::BindGroup` - describes which bindgroup the uniform buffer will be in. If you look at the WGSL we said it lies in @binding(0) so we will match that in our `uniform_bind_group`
- `camera: Camera` - an instance of Camera that keeps track of our mouse movement.
- `is_dragging` - a boolean that will be used when we check for changes/inputs later.
- `depth_texture_view` - the thing that we return in `create_depth_texture` method

That's everything that we will need to create in our `new` method. 
I'd like to note that since this is going to be a long method and I want to break it up into parts it won't have hv the previous lines in it. So assume it all as being in the `new` method unless otherwise not, and reference the code in the `the_code` folder (assuming you're looking at this on github) if you want to see the complete product. Otherwise let's start:

The first step is to actually gather our data that we will use. The two pieces are:
1. Vertices
2. Indices

To do this we will first get our vertices and then create indices based off them.

a. Get the file

```rust
async fn new(window: Arc<Window>) -> Self {

    let mut file = OpenOptions::new()
        .read(true)
        .open("3dbenchy.stl")
        .expect("Failed to open STL");
    
}
```

b. read the file using the `stl` library we imported earlier

```rust
async fn new(window: Arc<Window>) -> Self {

    let stl_data = stl::read_stl(&mut file).expect("Failed to parse STL file");

}
```

c. We are going to iterate over our vertices to create indices then store them in their own vectors. 

So first let's create the Vectors to hold the data

```rust
async fn new(window: Arc<Window>) -> Self {

    let mut vertices: Vec<Vertex> = Vec::new();
    let mut indices: Vec<u32> = Vec::new();
    let mut vertex_to_index = HashMap::new();

}
```
You'll notice that there is a 3rd variable there called `vertex_to_index`. This will be used to keep track of which vertices we have already seen b4. This is an effeciency thing so that we dont store the same vertex multiple times and instead can just use indices to tell it to use the same vertex. Image a square made up of 2 triangles. If you were to store all the vertices that would be 6 vertices. Instead what you could do is use 4 vertices and tell the GPU (using indices) to use the same vertices twice.

Now let's actually iterate over the triangles from our STL file and write to our vectors

```rust

async fn new( window: Arc<Window> ) -> Self {

    for triangle in stl_data.triangles {

        for vertex_pos in [ triangle.v1, triangle.v2, triangle.v3 ] {

            let hash_key = (
                vertex_pos[0].to_bits(), // store as u32 over f32 for accurate == checks
                vertex_pos[1].to_bits(),
                vertex_pos[2].to_bits()
            );

            if let Some(&index) = vertex_to_index.get(&hash_key) {

                indices.push(index);

            }
            else {
                let index = vertices.len() as u32;
                let new_vertex = Vertex { pos: vertex_pos }.scale(0.3).translate(0.0, 0.0, -5.0);
                vertices.push(new_vertex);
                vertex_to_index.insert(hash_key, index);
                indices.push(index);
            }

        }

    }

}

```

b. Create our basic stuff

We are still in the same `new()` method for the `AppState` impl

Now we are going to create what I categorized as the basics. 

```rust
async fn new(window: Arc<Window>) -> Self {

    let size = window.inner_size();
    let instance = wgpu::Instance::default();
    let surface = instance.create_surface(window.clone()).unwrap();
    let adapter = instance.request_adapter(
        &wgpu::RequestAdapterOptions {
            compatible_surface: Some(&surface),
            ..Default::default()
        }
    ).await.unwrap();
    let ( device, queue ) = adapter.request_device(
        &wgpu::DeviceDescriptor::default()
    ).await.unwrap();
    let config = surface.get_default_config(
        &adapter,
        size.width,
        size.height
    ).unwrap();
    surface.configure(&device, &config);
    
}
```

It probably feels like I just plumped a bunch of code on your screen with no explanation for anything. And that's because I did. There is nothing logically complex to any of this snippet. Instead it's just the boilerplate for getting these different things that we are going to use. I feel it would be a waste of time to explain why we are getting any of these things when we're about to use them in the next steps, so instead just keep going.

c. Buffers

Let's start by creating our vertex buffer. We are going to use a helper function to create a buffer and write to it at the same time called `.create_buffer_init()`. This takes in 3 params that we need to worry about:
1. label - a label
2. contents - a byte version of our data
3. usage - how the GPU will use it

```rust
async fn new(window: Arc<Window>) -> Self {
    let vertex_buffer = device.create_buffer_init(
        &wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX
        }
    );
}
```
This is where `bytemuck` comes in handy. It will automatically convert our slice into a byte representation which WGPU expects. 

Creating our index buffer is pretty identicale except for our buffer usage. Instead we will tell the GPU to use it as INDEX data.

```rust
async fn new(window: Arc<Window>) -> Self {
    let index_buffer = device.create_buffer_init(
        &wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(&indices),
            usage: wgpu::BufferUsages::INDEX
        }
    )
}
```

**What are VERTEX & INDEX buffer usages?**
- VERTEX - special buffer type that the GPU will know to use as vertices. The GPU stores it in it's own special section which is why in the WGSL we use @location(0) instead of bind groups like we do with uniform variables. 
- INDEX - special buffer type that the GPU will know to use as the order for the vertices. 

Next we will make handle our uniform data. Uniform data will be handled differently in 3 steps:
1. Create the buffer (space in GPU memory)
2. Create a layout for the bind group
3. Create the bind group

Let's start with creating the buffer. Notice we are using `.create_buffer()` instead of `.create_buffer_init()` which means we have different params. This only reserves room in memory, but does not write to anything.

```rust
async fn new(window: Arc<Window>) -> Self {
    let uniform_buffer = device.create_buffer(
        &wgpu::BufferDescriptor {
            label: Some("Uniform Buffer"),
            size: std::mem::size_of::<Uniforms>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false
        }
    );
}
```

Let's go over each param for clarity
- label - a label
- size - the size of the data we are storing. Measured in bytes and stored as a u64
- usage - will go over the different usages here in a sec
- mapped_at_creation - not used here. If used it would create a temporary space in CPU memory connected to the GPU buffer that you could write data to.

**What are UNIFORM & COPY_DST buffer usages?**
- UNIFORM - special type that tells the GPU to store as a uniform meaning it stays the same across all processes per render.
- COPY_DST - tells the GPU that you can upload/change the data there at a later point

Next we have to create a layout for our bind group. 

```rust
async fn new(window: Arc<Window>) -> Self {
    let uniform_bind_group_layout = device.create_bind_group_layout(
        &wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    min_binding_size: None,
                    has_dynamic_offset: false
                },
                count: None
            ]
        }
    );
}
```

Let's go over each of the params in entries. 
- binding - In our WGSL we access the buffer by location with `@group(0) @binding(0)` so we have to make it correspond on the CPU side by setting this to 0
- visibility - Can choose to make it so that the data can only be accessed during a specific stage of the shader. Here we say you can only access it during the vertex shader. 
- ty - specify what type of bind group this is
- count - if true would allow multiple different values to exist in the same buffer

`ty` params:
- ty - what type. We say Uniform
- min_binding_size - not used here, but would set a minimum amount of data required
- has_dynamic_offset - if used would allow for an offset in bytes

Now that we have our bind group layout we can use it to create our bind group:

```rust
async fn new(window: Arc<Window>) -> Self {
    let uniform_bind_group = device.create_bind_group(
        &wgpu::BindGroupDescriptor {
            layout: &uniform_bind_group_layout, // uses our layout
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0, // specifies @binding(0) 
                    resource: uniform_buffer.as_entire_binding(), //bind to "uniform_buffer"
                }
            ],
            label: None // label
        }
    );
}
```
I decided to make the comments just exist in the code for this snippet for simplicity

d. render pipeline

to create our render pipeline we will need to do a few things:
1. create a shader from our WGSL 
2. create a pipeline layout
3. create a depth texture view
4. create the render pipeline

Once we do this we will hv everything we need and can return an AppState with everything filled out

creating a shader is quite simple:

```rust
async fn new(window: Arc<Window>) -> Self {
    let shader = device.create_shader_module(
        wgpu::include_wgsl!("draw_stl.wgsl")
    );
}
```

Now let's create a pipeline layout:

```rust
async fn new(window: Arc<Window>) -> Self {
    let pipeline_layout = device.create_pipeline_layout(
        &wgpu::PipelineLayoutDescriptor {
            bind_group_layouts: &[
                &uniform_bind_group_layout,
            ],
            ..Default::default()
        }
    );
}
```

I do not feel as though anything in the layout creation needs clarification.

Next let's create the depth texture view by simply using the `create_depth_texture` method we created b4. See, things we made b4 r actually coming together!

```rust
async fn new(window: Arc<Window>) -> Self {
    let depth_texture_view = Self::create_depth_texture(&device, &config);
}
```

Next we will create our render pipeline. I feel as though it's worth going over what the pipeline is supposed to establish. 
- What happens in the vertex shader
    - which shader module to use
    - which function in the shader to use as an entry point
    - which buffers to use & how to process it
- What happens in the fragment shader
    - which shader module to use
    - which function in the shader to use as an entry point
    - how to draw the colors
- How the GPU should connect & color things
    - ex. use triangles

This is also the place where we tell the GPU that we have a depth texture we want it to use.

```rust
async fn new(window: Arc<Window>) -> Self {
            let render_pipeline = device.create_render_pipeline( // Creates a render pipeline
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
                                    blend: Some(wgpu::BlendState::REPLACE), // replace the old color with the new color
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
                // cull_mode back makes it so the gpu only renders the back of each triangle so that you don't see the triangle face from all directions
                multisample: wgpu::MultisampleState::default(),
                multiview_mask: None,
                cache: None,

            }
        );
}
```

I hope that the embedded comments are satisfactory here.

e. 

Now that we have everything we need let's return a `Self`

```rust
async fn new(window: Arc<Window>) -> Self {

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
```

We have one last thing to do which is implement a render method. the point of render is to run the shader on the GPU and write the info to the surface.

a. create `render`

```rust
impl AppState {
    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {

    }
}
```

Everything for the rest of the section will exist in this method.

Next we get the surface's current texture and create a view for it

```rust
fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
    let output = self.surface.get_current_texture()?;
    let view = output.texture.create_view(
        &wgpu::TextureViewDescriptor::default()
    );
}
```
Just like for our depth texture we can't directly access it so we go through a view instead. 

Next let's create a command encoder. This is gonna record the instructions we want to send to the GPU.

```rust
fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
    let mut encoder = self.device.create_command_encoder(
        &wgpu::CommandEncoderDescriptor::default()
    );
}
```

Next we are gonna create something called a projection matrix. This is automatically handled for us by `glam` if we give it the correct info. In short a projection matrix handles asymmetric aspect ratios, field of view, min render distance, and max render distance. 

To create one we need the aspect ratio of the surface, our desired fov, and our render min/max 's

```rust
fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
    let aspect = self.config.width as f32 / self.config.height as f32;
    let proj = Mat4::perspective_rh(45.0_f32.to_radians(), aspect, 1e-3, 1e4);
}
```

Next we want to create the mvp (model view projection) matrix that we are gonna upload to the GPU. To do this we will matmul (matrix multiply) our projection matrix by our camera's view matrix.

```rust
fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
    let mvp = proj * self.camera.view_matrix();
}
```

Then we want to write our data to the uniform buffer

We are gonna use our mvp which we just created and then we are going to use a new array for the color of our STL file. We will store this in a Uniforms struct since that's the shape that `uniform_buffer` expects.

```rust
fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
    self.queue.write_buffer(
        &self.uniform_buffer, // upload to `uniform buffer`
        0, // offset in bytes
        bytemuck::cast_slice(
            &[
                Uniforms {
                    mvp: mvp.to_cols_array_2d(), // a [ [f32; 4]; 4 ] representation
                    rgba: [ 0.3, 0.3, 0.3, 1.0 ] // a neutral color
                }
            ]
        )
    );
}
```

Next we are going to record a render pass on the command encoder. A render pass when created needs some information to understand what to do. Then you use it to set the following:
- render pipeline
- bind groups
- vertex buffer
- index buffer
- draw command
We want the instance of our render pass to only exist for a short time span so we are gonna wrap it in curly braces for its lifetime. 

First let's create the render pass itself 

```rust
fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
    { 
    let mut rpass = encoder.begin_render_pass(
                &wgpu::RenderPassDescriptor {
                    color_attachments: &[Some( // describes where we are drawing the colors
                        wgpu::RenderPassColorAttachment {
                            view: &view,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Clear( // clear the screen b4 drawing
                                    wgpu::Color::BLACK // clear to black
                                ),
                                store: wgpu::StoreOp::Store // store the value so we can see it
                            },
                            depth_slice: None
                        }
                    )],
                    depth_stencil_attachment: Some( // You need a depth_stencil_attachment to utilize the depth texture view 
                        wgpu::RenderPassDepthStencilAttachment {
                            view: &self.depth_texture_view, 
                            depth_ops: Some(
                                wgpu::Operations {
                                    load: wgpu::LoadOp::Clear(1.0),  // clear to maximum depth (1.0)
                                    store: wgpu::StoreOp::Store // store the value after
                                }
                            ),
                            stencil_ops: None
                        }
                    ),
                    ..Default::default()
                }
            );
    }
}
```

Please note to keep the following `rpass` stuff in these curly braces. I will explicitely say when we can stop. 

Now **IN THE SAME BRACES**

```rust
fn render(&mut self) -> Result<(), wgpu::SurfaceError> {

    { // same braces as b4

        // *rpass declaration* // 


        rpass.set_pipeline(&self.render_pipeline);
        rpass.set_bind_group(0, &self.uniform_bind_group, &[]);
        rpass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        rpass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
        rpass.draw_indexed(0..self.index_count as u32, 0, 0..1);

    } // same braces as b4

}
```

Now the following things are **after** the curly braces 

We hv 3 things left to do in `render`
1. submut the command encoder
2. put the result on the `output`
3. return an Ok status

```rust
fn render(&mut self) -> Result<(), wgpu::SurfaceError> {

    self.queue.submit(std::iter::once(encoder.finish()));
    output.present();
    Ok(())

}
```


That is it for our AppState. Yay


## Part 4: App

We are almost finished, so let's get right into it:

a. create `App` struct

App will be an impl of ApplicationHandler from winit. We are impl 3 methods:
1. resumed - what to do whenever the app start or unpauses
2. device_event - what to do when you get a device event. Here we use it to keep track of the mouse if the user is clicking
3. window_event- what to do for a window event. anything that happens in the window: resizing, closing, mouse clicks, redraw requests


```rust
struct App {
    state: Option<AppState>
}
```

b. impl ApplicaionHandler

In the `the_code` dir you will see that the code actually starts with the `resumed` method but for the sake of clarity we are gonna start with the `window_event` method.

```rust
impl ApplicationHandler for App {
    fn window_event(&mut self, event_loop: &ActiveEventLoop, _: WindowId, event: WindowEvent) {
        match event {

        }
    }
}
```

The rest of the stuff in the `window_event` method will exist in the `match event` section. This is because all we are doing is checking if we care about the event and telling the window what to do.

We care about the following:
- CloseRequested - exit the event loop
- MouseInput - update "is_dragging" in `AppState`
- RedrawRequested - call AppState's `.render` method 
- Resized - Reconfigure the surface and depth texture, then request a redraw 

Close Request:

```rust
match event {
    WindowEvent::CloseRequested => {
        event_loop.exit();
    }
}
```

Mouse Input:
if left clicking then set `is_dragging` to true. Then when we impl `device_event` it will read the mouse movement whenever `is_dragging` is true.
```rust
match event {

    // *prev* //

    WindowEvent::MouseInput {state, button: MouseButton::Left, ..} => {
        if let Some(app_state) = self.state.as_mut() {
            app_state.is_dragging = state == ElementState::Pressed;
        }
    }
}
```
Redraw Request:
```rust
    match event {

        // *prev* //

        WindowEvent::RedrawRequested => {
            if let Some(state) = self.state.as_mut() {
                state.render().unwrap();
            }
        }
    }
```

Resize:
```rust
    match event {

        // *prev* //

        WindowEvent::Resized(new_size) => {
            if let Some(state) = self.state.as_mut() {
                state.config.width = new_size.width; // set new width
                state.config.height = new_size.height; // set new height
                state.surface.configure(&state.device, &state.config); // reconfig
                state.depth_texture_view = AppState::create_depth_texture(&state.device, &state.config); // remake depth texture
                state.window.request_redraw(); // request redraw for new size
            }
        }

        _ => () // do nothing for everything else
        
    }
```

Next let's take care of the mouse input

```rust
impl ApplicationHandler for App {
    fn device_event(&mut self, _: ActiveEventLoop, _: DeviceId, event: DeviceEvent) {
        if let Some(state) = self.state.as_mut() {
            if state.is_dragging {
                if let DeviceEvent::MouseMotion{ delta } = event {
                    state.camera.update_orientation(Vec2::new(delta.0 as f32, delta.1 as f32)); // update according to mouse
                    state.window.request_redraw(); // redraw
                }
            }
        }
    }
}
```

Lastly we will handle the `resumed` method:

All we are going to do is:
- create a window + give settings
- create an AppState
- request a draw

```rust
impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {

        let window_attributes = Window::default_attributes()
            .with_title("WGPU STL Renderer Tutorial") // give title
            .with_inner_size(winit::dpi::LogicalSize::new(800.0, 600.0)); // set dims

        let window = Arc::new( // create an Atomically Reference Counted Window
            event_loop.create_window(window_attributes).unwrap()
        );

        window.set_visible(true); // make it visible
        window.focus_window(); // make it appear on top 

        let state = pollster::block_on(AppState::new(window.clone())); // wait till `new` is done
        self.state = Some(state); // set state
        window.request_redraw(); // request draw
    }
}
```

## Part 5: main

We are at the end

create a fn called `main`

```rust
fn main() {

}
```

inside we are gonna create an event_loop, an instance of App, and finally run the event loop / app

```rust
fn main() {
    let event_loop = EventLoop::new().unwrap();
    let mut app = App { state: None };
    event_loop.run_app(&mut app).unwrap();
}
```
That's it for main. 

If you feel like you don't understand why we did certain things, or feel like " how am I supposed to know that I'll need to do that " I suggest you look at the WGSL and think about what the GPU needs to receive from the CPU. If there is something that you think could use an improvement in this guide please just reach out so I can make it better for future learners. Otherwise go ahead and run the script. If you don't know how, or are curious about potential improvements you could make, read part 6.

Oh also if it does not work please make sure you actually have an STL file to render. Feel free the use the benchy I use in this repo.

## Part 6: Testing & Future Improvements

Open a terminal in the directory of your project. 



run: `cd [project_folder_name]`

then do: `cargo run`

you should see a window pop up. 

Improvements you should try to implement:

- Auto-aligned 3D models
    - In this script if the 3D model is not in the origin then you need to manually adjust the default camera settings. Instead you could automatically scale and translate the vertices using our `.scale` & `.translate` methods for `Vertex` to align it.
- More camera functionality
    - In this script you can only rotate the camera around a single point. Try to add panning and/or zooming
- Rendering multiple 3D models
    - This script only renders a single 3D model. Try to make it so it process multiple, or if you want to step it up even more add per-object transforms ( for example make it so that two object are moving independantly from eachother )

I will probably release versions of these with these improvements in the future if there is any demand for it. 

Anyway thanks for using this tutorial. Again please reach out if you need anything:

Email: TheRazielMoesch@gmail.com

LinkedIn: https://www.linkedin.com/in/raziel-moesch-61474b21b/
