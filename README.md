

# WGPU Rust Tutorial: STL Renderer + Camera + Lighting


## Table of Contents
- [Summary](#summary)
- [Author](#author)
- [Part 1: WGSL](#part-1-wgsl)
- [Part 2: CPU Side Data](#part-2-cpu-side-data)
- [Part 3: AppState](#part-3-appstate)
- [Part 4: App](#part-4-app)
- [Part 5: Testing & Future Improvements](#part-5-testing--future-improvements)

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

    var out: VertexOuput; // instanciate out

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
        let z = self.distance & self.pitch.sin();

        let position = Vec3::new( x, y, z );

        mat4::look_at_rh( position, Vec3::ZERO, Vec3::Z )

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




## Part 3: AppState

## Part 4: App

## Part 5: Testing & Future Improvements