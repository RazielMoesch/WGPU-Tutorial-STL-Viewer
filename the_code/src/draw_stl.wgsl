



struct Uniforms {

    mvp: mat4x4<f32>, // Model View Projecton Matrix
    //<explain>
    // A 4x4 matrix that holds 32 bit floating point numbers
    //</explain>
    rgba: vec4<f32> // Base color of the STL file
    //<explain>
    // each value is normalized from 0-1
    // holds a vector of 4 values each a 32 bit floating point value
    //</explain>


};

@group(0) @binding(0) // specifies that the variable will be at group 0 binding 0 
var<uniform> uniforms: Uniforms; // reserves room in memory for a uniform variable of size Uniforms

// What is a uniform variable?
//<explain>
// Uniform variables are pieces of data that are the same across all processes running on the GPU
// They are useful for things like a color that u want the whole object to be or any piece of data
// that u want to be the same.
//</explain>

struct VertexInput {

    @location(0) position: vec3<f32> // the GPU will look at location 0 to get the info to put into the position 


};


struct VertexOutput {

    @builtin(position) transformed_position: vec4<f32>, // stores the transformed positions in a special "position" slot which the GPU uses to draw the vertices/points
    @location(0) color: vec4<f32>, // store the color in location 0
    @location(1) world_position: vec3<f32> // store the world position in location 1

};


@vertex // specifies the function as a vertex shader function
fn vs_main(in: VertexInput) -> VertexOutput { // takes in a VertexInput and outputs a VertexOutput

    var out: VertexOutput; // instanciate a VertexOutput struct

    out.world_position = in.position; // set world position as the position
    out.transformed_position = uniforms.mvp * vec4<f32>(in.position, 1.0); // set the transformed position according to the mvp 
    out.color = uniforms.rgba; // set the colors according to the color chosen CPU side

    return out; // return out

}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {

    // How does the info from the VertexOutput get to the fragment shader?
    //<explain>
    // Because the VertexOutput stores the info at specific locations if there is already data there 
    // the GPU will use the data already stored there until it changes.
    //</explain>

    let face_normal = normalize(cross(dpdx(in.world_position), dpdy(in.world_position)));

    // How does this face_normal math work?
    //<explain>
    // dpdx and dpdy are special built in operations to measure how much world space changes per pixel in each direction
    // We use this to create lines to use to calculate our normals
    // We then take the cross product of those numbers. A cross product gives us a direction perpindicular to the two
    // lines which will then be normalized meaning it becomes a value of 0-1 in the direciton so a vector might look like
    // <0, 0, 1> meaning the normal is in the positive z direction
    //</explain>

    let light_dir = normalize(vec3<f32>(1.0, 1.0, 2.0)); // creates a normalized point from where light comes from


    let dot_product = dot(face_normal, -light_dir); // uses dot product to find how much the light contributes to that surface
    
    // How does the dot product work here?
    //<explain>
    // If you want a mathematical explanation refer to this resource: https://en.wikipedia.org/wiki/Dot_product
    // For now if you want a non-mathy way of thinking about it just think about it like this:
    // The dot product will measure how closely the direction of the light matches with the direction of the face
    // a more similar line will be a larger value which means more light 
    //</explain>

    let intensity = max(dot_product, 0.1); // maxes the amount of light possible


    return vec4<f32>(in.color.x * intensity, in.color.y * intensity, in.color.z * intensity, in.color.w ); // return 

}

