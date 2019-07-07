use glfw::{Context, Glfw, Window, WindowEvent, WindowMode};
use std::ffi::CString;
use std::ops::Deref;
use std::sync::mpsc::Receiver;
use std::sync::{Arc, RwLock};
use std::{error, fmt, ptr};
use vulkano::instance::{Instance, InstanceExtensions, QueueFamily, RawInstanceExtensions};
use vulkano::swapchain::Surface;
use vulkano::VulkanObject;

pub struct WrappedWindow {
    window: Arc<RwLock<Window>>,
    attributes: Arc<RwLock<WindowedAttributes>>
}

pub struct WindowedAttributes {
    size: (i32, i32),
    position: (i32, i32)
}

impl WrappedWindow {
    pub fn is_fullscreen(&self) -> bool {
        let window = self.window.read().unwrap();

        window.with_window_mode(|wmode| {
            match wmode {
                WindowMode::FullScreen(_) => true,
                _ => false,
            }
        })
    }

    pub fn enter_fullscreen(&self) {
        let mut glfw = {
            self.window.read().unwrap().glfw
        };

        glfw.with_primary_monitor(|_: &mut _, m: Option<&glfw::Monitor>| {
            let monitor = m.unwrap();
            let mode: glfw::VidMode = monitor.get_video_mode().unwrap();

            if !self.is_fullscreen() {
                self.save_attributes();
                let mut window = self.window.write().unwrap();

                window.set_monitor(
                    glfw::WindowMode::FullScreen(&monitor),
                    0, 0,
                    mode.width, mode.height,
                    Some(mode.refresh_rate),
                );
            }
        });
    }

    pub fn exit_fullscreen(&self) {
        let mut glfw = {
            self.window.read().unwrap().glfw
        };

        glfw.with_primary_monitor(|_: &mut _, _: _| {
            if self.is_fullscreen() {
                let mut window = self.write().unwrap();
                let (w, h) = self.attributes_size();
                let (x, y) = self.attributes_pos();

                window.set_monitor(
                    glfw::WindowMode::Windowed,
                    x, y,
                    w as u32, h as u32,
                    None,
                );
            }
        });
    }

    pub fn save_attributes(&self) {
        let window = self.window.read().unwrap();
        let mut attributes = self.attributes.write().unwrap();
        attributes.size = window.get_size();
        attributes.position = window.get_pos();
    }

    pub fn load_attributes(&self) {
        let mut window = self.write().unwrap();
        let (w, h) = self.attributes_size();
        let (x, y) = self.attributes_pos();

        window.set_size(w, h);
        window.set_pos(x, y);
    }

    pub fn attributes_size(&self) -> (i32, i32) {
        let attributes = self.attributes.read().unwrap();
        attributes.size
    }

    pub fn attributes_pos(&self) -> (i32, i32) {
        let attributes = self.attributes.read().unwrap();
        attributes.position
    }

    pub fn poll_events(&self) {
        self.window.write().unwrap().glfw.poll_events();
    }
}

impl From<Arc<RwLock<Window>>> for WrappedWindow {
    fn from(window: Arc<RwLock<Window>>) -> Self {

        let attributes = {
            let window = window.read().unwrap();

            Arc::new(RwLock::new(
                WindowedAttributes {
                    size: window.get_size(),
                    position: window.get_pos(),
                }
            ))
        };
        WrappedWindow { window, attributes }
    }
}

impl Deref for WrappedWindow {
    type Target = Arc<RwLock<Window>>;

    fn deref(&self) -> &Arc<RwLock<Window>> {
        &self.window
    }
}

unsafe impl Send for WrappedWindow {}
unsafe impl Sync for WrappedWindow {}

unsafe impl Send for WindowedAttributes {}
unsafe impl Sync for WindowedAttributes {}

pub fn create_window(
    glfw: Glfw,
    width: u32,
    height: u32,
    title: &str,
    mode: WindowMode,
) -> (WrappedWindow, Receiver<(f64, WindowEvent)>) {
    let (window, events) = glfw.create_window(width, height, title, mode).unwrap();
    (WrappedWindow::from(Arc::new(RwLock::new(window))), events)
}

/// error while creating a GLFW-based surface
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum VulkanoGlfwError {
    /// General GLFW error
    GlfwError {
        code: u32,
    },
    NoExtensions,
}

impl error::Error for VulkanoGlfwError {
    #[inline]
    fn description(&self) -> &str {
        match *self {
            VulkanoGlfwError::GlfwError { .. } => "Genral Vulkan GLFW error",
            VulkanoGlfwError::NoExtensions => "Could not load required extensions",
        }
    }

    #[inline]
    fn cause(&self) -> Option<&dyn error::Error> {
        match *self {
            _ => None,
        }
    }
}

impl fmt::Display for VulkanoGlfwError {
    #[inline]
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(fmt, "{}", error::Error::description(self))
    }
}

/// Create a surface from a GLFW window
pub fn create_window_surface(
    instance: Arc<Instance>,
    window: WrappedWindow,
) -> Result<Arc<Surface<WrappedWindow>>, VulkanoGlfwError> {
    let internal_instance = instance.as_ref().internal_object();
    let internal_window = { window.window.write().unwrap().window_ptr() };

    let mut internal_surface: vk_sys::SurfaceKHR = 0;
    let result = unsafe {
        glfw::ffi::glfwCreateWindowSurface(
            internal_instance,
            internal_window,
            ptr::null(),
            &mut internal_surface as *mut u64,
        )
    };
    if result != vk_sys::SUCCESS {
        return Err(VulkanoGlfwError::GlfwError { code: result });
    }
    Ok(Arc::new(unsafe {
        Surface::from_raw_surface(instance, internal_surface, window)
    }))
}
/// create InstanceExtensions from required GLFW extensions
pub fn get_required_instance_extensions(
    glfw: Glfw,
) -> Result<InstanceExtensions, VulkanoGlfwError> {
    get_required_raw_instance_extensions(glfw).and_then(|rie| Ok(InstanceExtensions::from(&rie)))
}

/// create RawInstanceExtensions from required GLFW extensions
pub fn get_required_raw_instance_extensions(
    glfw: Glfw,
) -> Result<RawInstanceExtensions, VulkanoGlfwError> {
    let exts = glfw.get_required_instance_extensions();
    if exts.is_none() {
        return Err(VulkanoGlfwError::NoExtensions);
    }

    let iter = exts.unwrap().into_iter().map(|s| {
        let new_c_string = CString::new(s);
        new_c_string.unwrap()
    });

    Ok(RawInstanceExtensions::new(iter))
}

/// This function returns whether the specified queue family of the specified physical device supports presentation to the platform GLFW was built for.
pub fn get_physical_device_presentation_support(glfw: &Glfw, family: &QueueFamily) -> bool {
    let device = family.physical_device();
    let internal_device = device.internal_object();
    let instance = device.instance();
    let internal_instance = instance.as_ref().internal_object();
    glfw.get_physical_device_presentation_support_raw(
        internal_instance,
        internal_device,
        family.id(),
    )
}
