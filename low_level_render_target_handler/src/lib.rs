use gl::types::{GLuint,GLint};
const NUMBERS_OF_FRAMEBUFFERS_TO_CHECK: usize = 512;

// Low-level OpenGL based methods to directly interact with the render-targets & render-target-texture in the GPU.
// Goal: Pass on render-targets created/drawn on by macroquad directly to eGui. 

// 1) Use the GlContext::default() FIRST to create an openGL context (ie, load the correct functions)
// Some openGL functions are missing in the miniquad implementation, since they seem to not be used for miniquad.
// The methods outlined below uses the gl & gl_load crate. 

// 2) After you allocate a render target in the GPU using Macroquad, you can use the "GlTextHandler::grab_last_render_texture()"
// to grab the last allocated render-target (ie, Framebuffer) directly from your GPU.
// Once then you can extract from there, the corresponding texture, using the "get_texture_id" methode.
// The resulting raw openGL texture-id (which is a OpenGL Integer) can be used with eGui as egui::TextureId::User(u64) by first casting 
// the raw id to a u64

pub struct GlContext;

impl Default for GlContext 
{
    fn default() -> Self
    {
        println!("OpenGL context: init");
        gl_loader::init_gl();
        gl::load_with(|symbol| gl_loader::get_proc_address(symbol) as *const _);
        Self {}
    }
}
impl Drop for GlContext 
{
    fn drop(&mut self)
    {
        println!("OpenGL context: release");
        gl_loader::end_gl();    
    }    
}

#[derive(Debug, Clone, Copy)]
pub enum OpenGLAttachement
{
    FrameBufferDefault,
    Texture,
    RenderBuffer,    
}
#[derive(Debug)]
pub struct GlTextHandler
{
    last_frame_buffer: Option<GLuint>,
    is_init: bool,
    num_of_frame_buffers_to_check: usize
}
impl Default for GlTextHandler {
    fn default() -> Self 
    {
        Self
        { 
            last_frame_buffer: Default::default(),
            is_init: Default::default(),
            num_of_frame_buffers_to_check: NUMBERS_OF_FRAMEBUFFERS_TO_CHECK 
        }
    }
}

impl GlTextHandler
{
    pub fn set_num_of_framebuffers_to_check(&mut self, num: usize)
    {
        self.num_of_frame_buffers_to_check = num;
    }
    pub fn reset(&mut self)
    {
        *self = GlTextHandler::default();
    }
    pub fn grab_last_render_texture(&mut self) -> bool
    {
        let fbs    = GlTextHandler::_get_valid_frame_buffers(self.num_of_frame_buffers_to_check);
        self.last_frame_buffer = fbs.last();
        self.is_init = true;  
        return self.is_init
    }    
    pub fn get_last_frame_buffer_id(&self) -> Option<GLuint>
    {
        return self.last_frame_buffer;
    }
    pub fn get_texture_id(&self) -> Option<GLuint>
    {
        if !self.is_init
        {
            return None;
        }
        let attachement = self.get_attachement_type()?;
        match attachement
        {
            OpenGLAttachement::Texture => self._get_texture_from_render_target(),
            _                          => None
        }
    }
    pub fn get_attachement_type(&self) -> Option<OpenGLAttachement>
    {
        if !self.is_init
        {
            return None;    
        }
        let frame_buffer = self.last_frame_buffer?;
        let mut obj_type: GLint = 0;
        unsafe 
        {
            let obj_type_ptr: *mut GLint = &mut obj_type;
            gl::GetNamedFramebufferAttachmentParameteriv(
                frame_buffer,
                gl::COLOR_ATTACHMENT0,
                gl::FRAMEBUFFER_ATTACHMENT_OBJECT_TYPE,
                obj_type_ptr);
        }
        match obj_type as gl::types::GLenum
        {
            gl::FRAMEBUFFER_DEFAULT => Some(OpenGLAttachement::FrameBufferDefault),
            gl::TEXTURE             => Some(OpenGLAttachement::Texture),
            gl::RENDERBUFFER        => Some(OpenGLAttachement::RenderBuffer),
            _                       => None,
        }
    }
    pub fn get_type(param: i32) -> Option<OpenGLAttachement>
    {
        match param as gl::types::GLenum
        {
            gl::FRAMEBUFFER_DEFAULT => Some(OpenGLAttachement::FrameBufferDefault),
            gl::TEXTURE             => Some(OpenGLAttachement::Texture),
            gl::RENDERBUFFER        => Some(OpenGLAttachement::RenderBuffer),
            _                       => None,
        }
    }
    fn _get_valid_frame_buffers(n_frame_buffer: usize) -> std::ops::Range<GLuint>
    {
        let mut last: GLuint = 0;
        unsafe 
        {
            for id in 1..n_frame_buffer
            {
                gl::BindFramebuffer(gl::FRAMEBUFFER, id as GLuint); // set...
                let err = gl::GetError();                          //  test ...
                gl::BindFramebuffer(gl::FRAMEBUFFER, 0 as GLuint); // and reset...
                last = id as GLuint;
                if err != gl::NO_ERROR
                {
                    break;
                }
            }
        }
        1..last
    }
    fn _get_texture_from_render_target(&self) -> Option<GLuint>
    {
        if !self.is_init
        {
            return None;
        }
        let frame_buffer = self.last_frame_buffer?;
        let mut texture: GLint = 0;
        unsafe 
        {
            let texture_ptr: *mut GLint = &mut texture;
            gl::GetNamedFramebufferAttachmentParameteriv
            (
            frame_buffer,
            gl::COLOR_ATTACHMENT0,
            gl::FRAMEBUFFER_ATTACHMENT_OBJECT_NAME,
            texture_ptr);
        }
        match texture
        {
            0 => None,
            _ => Some(texture as GLuint)
        }
    }

    // Use this function for debug: If you grab the correct render target, this method will "black out" the render target
    pub fn clear_frame_buffer(&mut self) -> bool
    {
        let Some(framebuffer) = self.last_frame_buffer else 
        {
            return false;
        };
        unsafe 
        {
            println!("Unsafe: openGL clear Framebuffer id: {framebuffer}.");
            gl::BindFramebuffer(gl::FRAMEBUFFER, framebuffer);
            gl::ClearColor(0.0, 0.0, 0.0, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);
            gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
        }
        true
    }
    // !!! Deletes the render target !!!! 
    // Normally not needed since macroquad or egui can manage their own textures
    // Double delete/freeing is OK for textures. Deleting an already deleted framebuffer is a "non-operation".
    pub fn delete_framebuffer(&mut self) -> bool
    {
        if !self.clear_frame_buffer()
        {
            return false;
        }
        let frame_buffer = self.last_frame_buffer.unwrap() as GLuint;
        unsafe 
        {
            println!("Unsafe: openGL delete Framebuffer id:{frame_buffer}.");
            let frame_buffer_ptr: *const GLuint = &frame_buffer; 
            gl::DeleteFramebuffers(1, frame_buffer_ptr);
        }
        true
    }
}
 