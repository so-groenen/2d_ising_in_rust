use macroquad::prelude::*;

// This method grabs the last allocated render_target (framebuffer) directly in the GPU
// using unsafe openGL functions and finds the corresponding render_target.texture, if present.
// This allows to transfer texture managed by macroquad to be drawn by egui.
// Otherwise, you need to modify the macroquad API so that it can give you the "raw" openGL texture, as explained below.

#[cfg(not(target_arch = "wasm32"))]
pub fn get_raw_opengl_texture_id_from_framebuffer(_render_target: &RenderTarget) -> Option<u32>   //Needs to be called right after allocating a render texture!
{
    use low_level_render_target_handler::{GlTextHandler, GlContext};

    let _new_context = GlContext::default();
    let mut handler = GlTextHandler::default();
    handler.grab_last_render_texture();
    handler.get_texture_id()
}

// The method below also works for non wasm, but as explained requires modification of the original macroquad source file.
// Works for macroquad as of macroquad version = "0.4.14"
//
// Add this method at line 840 in macroquad / texture.rs in the Texture2D implementation.
// To find the correct path: Use "go to definition" of your text editor by right click on "macroquad::texture::Texture2D::raw_miniquad_id"
// Then add this function, just below the function definition of "raw_miniquad_id()":
//
// pub fn raw_gl_id(&self) -> Option<u32>
// {
//     use miniquad::*;
//     let texture = self.raw_miniquad_id();
//     let ctx = get_quad_context();
//     let params = ctx.texture_params(texture);
//     match unsafe { ctx.texture_raw_id(texture) } {
//         miniquad::RawId::OpenGl(id) => Some(id),
//         _ => None,
//     };
// }
//
// ATTENTION: You may need to use "cargo clean" to clear the cache & reload your editor before changes take effect.
// After that you can use the following function to get the texture. Uncomment the cfg if you want it for non wasm architechture.

#[cfg(target_arch = "wasm32")]
pub fn get_raw_opengl_texture_id_from_framebuffer(render_target: &RenderTarget) -> Option<u32>
{
    render_target.texture.raw_gl_id()
}

  